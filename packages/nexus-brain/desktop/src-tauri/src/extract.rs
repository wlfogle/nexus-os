//! Tier 1: read the bytes.
//!
//! Produces a SHA-256 (used for duplicate detection and for re-attaching a
//! file's history after it moves) plus a plain-text rendering used by search,
//! embedding and interpretation.
//!
//! External helpers are used opportunistically and never required: `pdftotext`
//! for PDFs, `tesseract` for scans and screenshots. If neither is installed the
//! file is still catalogued, just without text.

use anyhow::Result;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;
use std::process::Command;

use crate::db;

/// Upper bound on stored text per file. Enough for a model to understand the
/// file; keeps the database from ballooning on generated monsters.
const MAX_TEXT: usize = 200_000;

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct ExtractReport {
    pub processed: usize,
    pub with_text: usize,
    pub failed: usize,
}

pub fn sha256_file(path: &Path) -> Option<String> {
    let mut f = std::fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        match f.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buf[..n]),
            Err(_) => return None,
        }
    }
    Some(format!("{:x}", hasher.finalize()))
}

/// Heuristic: does this look like text rather than a binary blob?
///
/// A NUL byte in the first block is decisive; otherwise require that most bytes
/// are printable. This prevents feeding compiled objects to the models.
fn looks_textual(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let sample = &bytes[..bytes.len().min(8192)];
    if sample.contains(&0) {
        return false;
    }
    let printable = sample
        .iter()
        .filter(|&&b| b == b'\n' || b == b'\r' || b == b'\t' || (0x20..0x7f).contains(&b) || b >= 0x80)
        .count();
    printable * 100 / sample.len() >= 85
}

fn have(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {cmd}"))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_capture(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn pdf_text(path: &Path) -> Option<String> {
    if have("pdftotext") {
        if let Some(t) = run_capture("pdftotext", &["-q", "-layout", &path.to_string_lossy(), "-"]) {
            return Some(t);
        }
    }
    // Scanned PDF with no text layer: rasterise-free OCR is not possible, but
    // tesseract can read single-page PDFs directly on many builds.
    if have("tesseract") {
        return run_capture(
            "tesseract",
            &[&path.to_string_lossy(), "stdout", "--psm", "3"],
        );
    }
    None
}

fn image_text(path: &Path) -> Option<String> {
    if !have("tesseract") {
        return None;
    }
    run_capture(
        "tesseract",
        &[&path.to_string_lossy(), "stdout", "--psm", "3"],
    )
}

/// Case-insensitive ASCII prefix test on a byte slice.
fn starts_with_ci(hay: &[u8], needle: &[u8]) -> bool {
    hay.len() >= needle.len()
        && hay[..needle.len()].eq_ignore_ascii_case(needle)
}

/// Find the next case-insensitive occurrence of `needle` in `hay`.
fn find_ci(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    (0..=hay.len() - needle.len()).find(|&i| hay[i..i + needle.len()].eq_ignore_ascii_case(needle))
}

/// Crude tag stripper for saved web pages: keeps the readable prose.
///
/// Operates on bytes throughout. Tag syntax is pure ASCII, so multi-byte UTF-8
/// sequences in the text pass through untouched and are decoded at the end.
fn html_to_text(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut kept: Vec<u8> = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0usize;
    let mut depth = 0usize;

    while i < bytes.len() {
        // Drop the entire contents of script and style elements.
        let rest = &bytes[i..];
        let close: Option<&[u8]> = if starts_with_ci(rest, b"<script") {
            Some(b"</script>")
        } else if starts_with_ci(rest, b"<style") {
            Some(b"</style>")
        } else {
            None
        };
        if let Some(close) = close {
            match find_ci(rest, close) {
                Some(j) => {
                    i += j + close.len();
                    continue;
                }
                None => break,
            }
        }

        match bytes[i] {
            b'<' => depth += 1,
            b'>' => depth = depth.saturating_sub(1),
            c if depth == 0 => kept.push(c),
            _ => {}
        }
        i += 1;
    }

    String::from_utf8_lossy(&kept)
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Read a file and return its text rendering, if it has one.
pub fn text_of(path: &Path, class: &str) -> Result<Option<String>, String> {
    match class {
        "document" => {
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if ext == "pdf" {
                return Ok(pdf_text(path));
            }
            Ok(None)
        }
        "image" => Ok(image_text(path)),
        _ => {
            let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
            if !looks_textual(&bytes) {
                return Ok(None);
            }
            let raw = String::from_utf8_lossy(&bytes).to_string();
            let text = if class == "web" { html_to_text(&raw) } else { raw };
            Ok(Some(text))
        }
    }
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    // Cut on a char boundary.
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

/// Extract a batch of files, advancing them to stage 1.
pub fn run(conn: &mut Connection, batch: &[(i64, String, String, i64)]) -> Result<ExtractReport> {
    let mut report = ExtractReport::default();

    // Do the IO outside the write transaction so the database is not locked
    // while OCR runs.
    struct Done {
        id: i64,
        sha: Option<String>,
        text: Option<String>,
        err: Option<String>,
        name: String,
        path: String,
    }
    let mut results: Vec<Done> = Vec::with_capacity(batch.len());

    for (id, path_s, class, _size) in batch {
        let p = Path::new(path_s);
        if !p.is_file() {
            results.push(Done {
                id: *id,
                sha: None,
                text: None,
                err: Some("vanished".into()),
                name: String::new(),
                path: path_s.clone(),
            });
            continue;
        }
        let sha = sha256_file(p);
        let (text, err) = match text_of(p, class) {
            Ok(t) => (t, None),
            Err(e) => (None, Some(e)),
        };
        let name = p
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        results.push(Done {
            id: *id,
            sha,
            text,
            err,
            name,
            path: path_s.clone(),
        });
    }

    let tx = conn.transaction()?;
    {
        let mut upd = tx.prepare(
            "UPDATE files SET sha256 = ?2, text_len = ?3, extract_err = ?4, stage = 1
             WHERE id = ?1",
        )?;
        let mut put_text = tx.prepare(
            "INSERT INTO file_text(file_id, body) VALUES (?1, ?2)
             ON CONFLICT(file_id) DO UPDATE SET body = excluded.body",
        )?;
        let mut clear_fts_map =
            tx.prepare("DELETE FROM fts_map WHERE file_id = ?1")?;
        let mut put_fts = tx.prepare(
            "INSERT INTO files_fts(rowid, path, name, body) VALUES (?1, ?2, ?3, ?4)",
        )?;
        let mut map_fts =
            tx.prepare("INSERT INTO fts_map(rowid, file_id) VALUES (?1, ?2)")?;

        for d in &results {
            let body = d.text.as_ref().map(|t| truncate_chars(t, MAX_TEXT));
            let len = body.as_ref().map(|b| b.len() as i64).unwrap_or(0);

            upd.execute(params![d.id, d.sha, len, d.err])?;

            if let Some(b) = &body {
                put_text.execute(params![d.id, b])?;
                clear_fts_map.execute(params![d.id])?;
                // Reuse the file id as the FTS rowid: one text row per file.
                put_fts.execute(params![d.id, d.path, d.name, b])?;
                map_fts.execute(params![d.id, d.id])?;
                report.with_text += 1;
            }
            if d.err.is_some() {
                report.failed += 1;
            }
            report.processed += 1;
        }
    }
    tx.commit()?;
    Ok(report)
}

/// Mark oversized or non-textual files as done so the pipeline does not
/// revisit them on every pass.
pub fn skip_unreadable(conn: &Connection, max_bytes: i64) -> Result<usize> {
    let n = conn.execute(
        "UPDATE files SET stage = 3, extract_err = 'skipped: too large or not readable'
         WHERE present = 1 AND stage = 0
           AND (size = 0 OR size > ?1 OR class IN ('media','archive','other','unknown'))",
        params![max_bytes],
    )?;
    Ok(n)
}

/// Groups of files whose bytes are identical.
pub fn duplicate_groups(conn: &Connection, limit: i64) -> Result<Vec<(String, i64, i64, Vec<String>)>> {
    let mut q = conn.prepare(
        "SELECT sha256, COUNT(*) n, MAX(size) sz
           FROM files
          WHERE present = 1 AND sha256 IS NOT NULL AND size > 0
          GROUP BY sha256 HAVING n > 1
          ORDER BY (n - 1) * sz DESC
          LIMIT ?1",
    )?;
    let heads: Vec<(String, i64, i64)> = q
        .query_map(params![limit], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<std::result::Result<_, _>>()?;

    let mut out = Vec::new();
    let mut members = conn.prepare(
        "SELECT path FROM files WHERE present = 1 AND sha256 = ?1 ORDER BY path",
    )?;
    for (sha, n, sz) in heads {
        let paths: Vec<String> = members
            .query_map(params![sha], |r| r.get(0))?
            .collect::<std::result::Result<_, _>>()?;
        out.push((sha, n, sz, paths));
    }
    Ok(out)
}

/// A file that vanished but whose bytes reappeared elsewhere has moved; carry
/// its interpretation across so the work is not redone.
pub fn reattach_moved(conn: &Connection) -> Result<usize> {
    let n = conn.execute(
        "INSERT OR IGNORE INTO interpretations
            (file_id,title,kind,purpose,summary,topics,entities,related_repo,
             status,action,reason,confidence,model,escalated,decided_at)
         SELECT nf.id, i.title, i.kind, i.purpose, i.summary, i.topics, i.entities,
                i.related_repo, i.status, i.action, i.reason, i.confidence,
                i.model, i.escalated, i.decided_at
           FROM files nf
           JOIN files of ON of.sha256 = nf.sha256 AND of.present = 0 AND of.id <> nf.id
           JOIN interpretations i ON i.file_id = of.id
          WHERE nf.present = 1
            AND nf.sha256 IS NOT NULL
            AND nf.id NOT IN (SELECT file_id FROM interpretations)",
        [],
    )?;
    if n > 0 {
        conn.execute(
            "UPDATE files SET stage = 3
              WHERE present = 1 AND stage < 3
                AND id IN (SELECT file_id FROM interpretations)",
            [],
        )?;
    }
    Ok(n)
}

pub fn now() -> f64 {
    db::now()
}

#[cfg(test)]
mod tests {
    use super::{html_to_text, looks_textual, truncate_chars};

    #[test]
    fn binary_is_rejected() {
        assert!(!looks_textual(&[0x7f, 0x45, 0x4c, 0x46, 0x00, 0x01]));
        assert!(!looks_textual(b""));
    }

    #[test]
    fn plain_text_is_accepted() {
        assert!(looks_textual(b"#!/usr/bin/env bash\necho hello\n"));
    }

    #[test]
    fn utf8_text_is_accepted() {
        assert!(looks_textual("# Título\nnotas del proyecto\n".as_bytes()));
    }

    #[test]
    fn html_tags_are_stripped() {
        let html = "<html><head><style>a{color:red}</style></head>\
                    <body><p>Hello</p><script>var x=1;</script><p>World</p></body></html>";
        let text = html_to_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(!text.contains("color"));
        assert!(!text.contains("var x"));
    }

    #[test]
    fn entities_are_decoded() {
        assert_eq!(html_to_text("<p>a &amp; b</p>"), "a & b");
    }

    #[test]
    fn truncation_respects_char_boundaries() {
        let s = "ééééé";
        let t = truncate_chars(s, 5);
        assert!(s.starts_with(&t));
        assert!(t.len() <= 5);
    }
}
