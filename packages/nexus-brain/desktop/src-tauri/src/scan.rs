//! Tier 0: inventory.
//!
//! Walks the configured roots recording only what `stat` can tell us. No file
//! contents are read here, so a full pass over a large home directory finishes
//! in seconds and can be re-run cheaply.
//!
//! Incremental by construction: a file whose size and mtime are unchanged keeps
//! its existing pipeline `stage`, so extraction, embedding and interpretation
//! are never redone for untouched files. Files that have disappeared are marked
//! `present = 0` rather than deleted, which preserves their interpretation in
//! case they come back (or moved, and are re-attached by hash later).

use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::db;

/// Broad content class, decided by extension.
///
/// This is only a routing hint: the interpretation tier decides what a file
/// really is by reading it.
pub fn classify_ext(ext: &str) -> &'static str {
    match ext {
        "sh" | "bash" | "zsh" | "fish" | "py" | "pl" | "rb" | "ps1" | "bat" | "cmd"
        | "expect" | "awk" | "tcl" => "script",

        "rs" | "go" | "c" | "h" | "cpp" | "hpp" | "cc" | "java" | "kt" | "swift" | "js"
        | "jsx" | "ts" | "tsx" | "vue" | "svelte" | "php" | "cs" | "scala" | "clj" | "ex"
        | "exs" | "lua" | "r" | "m" | "mm" | "s" | "asm" | "zig" | "dart" | "qml" => "code",

        "md" | "txt" | "rst" | "org" | "adoc" | "tex" => "doc",
        "pdf" | "docx" | "odt" | "rtf" | "epub" => "document",

        "json" | "yaml" | "yml" | "toml" | "ini" | "conf" | "cfg" | "env" | "properties"
        | "service" | "desktop" | "nix" | "tf" => "config",

        "csv" | "tsv" | "sql" | "xml" | "m3u" | "plist" | "log" => "data",
        "html" | "htm" | "css" | "scss" => "web",
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "tiff" => "image",

        "zip" | "tar" | "gz" | "xz" | "bz2" | "zst" | "7z" | "rar" | "deb" | "rpm"
        | "appimage" | "iso" | "img" | "qcow2" | "vdi" | "vmdk" => "archive",

        "mp4" | "mkv" | "avi" | "mov" | "mp3" | "flac" | "wav" | "m4a" => "media",

        "" => "unknown",
        _ => "other",
    }
}

/// Classes whose bytes are worth reading and interpreting.
pub fn is_readable_class(class: &str) -> bool {
    matches!(
        class,
        "script" | "code" | "doc" | "document" | "config" | "data" | "web" | "image"
    )
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct ScanReport {
    pub seen: usize,
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub vanished: usize,
    pub dirs_visited: usize,
}

struct Entry {
    path: String,
    parent: String,
    name: String,
    ext: String,
    class: String,
    size: i64,
    mtime: f64,
}

fn collect(cfg: &Config, report: &mut ScanReport) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut visited: HashSet<PathBuf> = HashSet::new();

    for root in &cfg.roots {
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            // Guard against symlink loops and the vault being linked into home.
            let real = std::fs::canonicalize(&dir).unwrap_or_else(|_| dir.clone());
            if !visited.insert(real) {
                continue;
            }
            report.dirs_visited += 1;

            let rd = match std::fs::read_dir(&dir) {
                Ok(r) => r,
                Err(_) => continue,
            };
            for entry in rd.flatten() {
                let path = entry.path();
                let ft = match entry.file_type() {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                if ft.is_symlink() {
                    // Catalogue the link target through its canonical path only.
                    continue;
                }
                if ft.is_dir() {
                    if !cfg.is_pruned(&path) {
                        stack.push(path);
                    }
                    continue;
                }
                if !ft.is_file() {
                    continue;
                }

                let md = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let name = entry.file_name().to_string_lossy().to_string();
                let ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let class = classify_ext(&ext).to_string();
                let mtime = md
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(0.0);

                out.push(Entry {
                    path: path.to_string_lossy().to_string(),
                    parent: path
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    name,
                    ext,
                    class,
                    size: md.len() as i64,
                    mtime,
                });
            }
        }
    }
    out
}

/// Run a full inventory pass and reconcile it with the catalog.
pub fn run(conn: &mut Connection, cfg: &Config) -> Result<ScanReport> {
    let mut report = ScanReport::default();
    let entries = collect(cfg, &mut report);
    report.seen = entries.len();

    let now = db::now();
    let tx = conn.transaction()?;

    // Everything currently believed present; anything not re-seen is gone.
    tx.execute("UPDATE files SET present = 0 WHERE present = 1", [])?;

    {
        let mut existing = tx.prepare(
            "SELECT id, size, mtime, stage FROM files WHERE path = ?1",
        )?;
        let mut insert = tx.prepare(
            "INSERT INTO files
               (path,parent,name,ext,class,size,mtime,stage,present,first_seen,last_seen)
             VALUES (?1,?2,?3,?4,?5,?6,?7,0,1,?8,?8)",
        )?;
        let mut touch = tx.prepare(
            "UPDATE files SET present = 1, last_seen = ?2 WHERE id = ?1",
        )?;
        // Content changed: reset the pipeline so it is re-read and re-judged.
        let mut changed = tx.prepare(
            "UPDATE files
               SET present = 1, last_seen = ?2, size = ?3, mtime = ?4,
                   stage = 0, sha256 = NULL, text_len = 0, extract_err = NULL
             WHERE id = ?1",
        )?;

        for e in &entries {
            let found: Option<(i64, i64, f64, i64)> = existing
                .query_row(params![e.path], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
                })
                .ok();

            match found {
                Some((id, size, mtime, _stage)) => {
                    // 1s tolerance: some filesystems round mtime.
                    if size == e.size && (mtime - e.mtime).abs() < 1.0 {
                        touch.execute(params![id, now])?;
                        report.unchanged += 1;
                    } else {
                        changed.execute(params![id, now, e.size, e.mtime])?;
                        // stale derived rows for this file
                        tx.execute("DELETE FROM chunks WHERE file_id = ?1", params![id])?;
                        tx.execute("DELETE FROM file_text WHERE file_id = ?1", params![id])?;
                        report.updated += 1;
                    }
                }
                None => {
                    insert.execute(params![
                        e.path, e.parent, e.name, e.ext, e.class, e.size, e.mtime, now
                    ])?;
                    report.added += 1;
                }
            }
        }
    }

    report.vanished = tx.query_row(
        "SELECT COUNT(*) FROM files WHERE present = 0",
        [],
        |r| r.get::<_, i64>(0),
    )? as usize;

    tx.commit()?;
    Ok(report)
}

/// Files that still need their contents read.
pub fn pending_extract(conn: &Connection, cfg: &Config, limit: i64) -> Result<Vec<(i64, String, String, i64)>> {
    let mut q = conn.prepare(
        "SELECT id, path, class, size FROM files
         WHERE present = 1 AND stage = 0 AND size > 0 AND size <= ?1
         ORDER BY mtime DESC LIMIT ?2",
    )?;
    let rows = q.query_map(params![cfg.max_read_bytes as i64, limit], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn path_exists(p: &str) -> bool {
    Path::new(p).is_file()
}

#[cfg(test)]
mod tests {
    use super::{classify_ext, is_readable_class};

    #[test]
    fn classifies_by_extension() {
        assert_eq!(classify_ext("sh"), "script");
        assert_eq!(classify_ext("rs"), "code");
        assert_eq!(classify_ext("md"), "doc");
        assert_eq!(classify_ext("pdf"), "document");
        assert_eq!(classify_ext("toml"), "config");
        assert_eq!(classify_ext("png"), "image");
        assert_eq!(classify_ext("qcow2"), "archive");
        assert_eq!(classify_ext("mkv"), "media");
        assert_eq!(classify_ext(""), "unknown");
        assert_eq!(classify_ext("xyzzy"), "other");
    }

    #[test]
    fn only_meaningful_classes_are_read() {
        assert!(is_readable_class("doc"));
        assert!(is_readable_class("code"));
        assert!(!is_readable_class("media"));
        assert!(!is_readable_class("archive"));
    }
}
