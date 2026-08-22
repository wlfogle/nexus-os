//! Markdown notes with wikilinks and backlinks.
//!
//! Notes are plain `.md` files under `<library>/Notes/`, so they stay readable
//! and editable by anything else. The database is an index over them, never the
//! source of truth -- deleting the row and re-scanning reproduces it exactly.
//!
//! `[[Target]]` links are parsed out and resolved by title, which gives
//! backlinks: for any note, everything that points at it. Unresolved links are
//! kept too, so a link to a note you have not written yet still shows up as an
//! intention rather than vanishing.

use anyhow::{anyhow, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub created_at: f64,
    pub updated_at: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoteLink {
    /// Link text as written inside the double brackets.
    pub target: String,
    /// Resolved note id, when a note with that title exists.
    pub dst_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoteDetail {
    pub note: Note,
    /// Links this note makes.
    pub links: Vec<NoteLink>,
    /// Notes that link *to* this one.
    pub backlinks: Vec<(i64, String)>,
}

/// Turn a title into a filesystem-safe file name.
pub fn slugify(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut prev_dash = false;
    for c in title.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("untitled");
    }
    out.truncate(80);
    out
}

/// Extract `[[wikilink]]` targets in document order, without duplicates.
pub fn parse_links(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i + 3 < bytes.len() {
        if bytes[i] == b'[' && bytes[i + 1] == b'[' {
            if let Some(rel) = body[i + 2..].find("]]") {
                let raw = &body[i + 2..i + 2 + rel];
                // Support "[[target|display]]" by keeping only the target.
                let target = raw.split('|').next().unwrap_or(raw).trim();
                if !target.is_empty() && !out.iter().any(|t| t == target) {
                    out.push(target.to_string());
                }
                i += rel + 4;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Extract `#tags` from the body.
pub fn parse_tags(body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for word in body.split_whitespace() {
        // Skip markdown headings ("## Heading") -- a tag needs a word character
        // straight after the hash.
        if let Some(rest) = word.strip_prefix('#') {
            if rest.is_empty() || rest.starts_with('#') {
                continue;
            }
            let tag: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '/')
                .collect();
            if tag.len() >= 2 && !tag.chars().all(|c| c.is_numeric()) {
                let tag = tag.to_lowercase();
                if !out.contains(&tag) {
                    out.push(tag);
                }
            }
        }
    }
    out
}

/// First markdown H1, else the first non-empty line, else the file stem.
pub fn derive_title(body: &str, path: &Path) -> String {
    for line in body.lines().take(40) {
        let t = line.trim();
        if let Some(h) = t.strip_prefix("# ") {
            if !h.trim().is_empty() {
                return h.trim().to_string();
            }
        }
    }
    for line in body.lines().take(40) {
        let t = line.trim();
        if !t.is_empty() && !t.starts_with('#') {
            return t.chars().take(80).collect();
        }
    }
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "untitled".into())
}

fn notes_dir(cfg: &Config) -> PathBuf {
    cfg.library.join("Notes")
}

// ------------------------------------------------------------------ write --

/// Create or overwrite a note, on disk and in the index.
pub fn save(conn: &mut Connection, cfg: &Config, title: &str, body: &str) -> Result<i64> {
    let dir = notes_dir(cfg);
    std::fs::create_dir_all(&dir)?;

    let title = if title.trim().is_empty() {
        derive_title(body, Path::new("untitled.md"))
    } else {
        title.trim().to_string()
    };
    let path = dir.join(format!("{}.md", slugify(&title)));

    // Keep the H1 in sync so the file is self-describing outside the app.
    let content = if body.trim_start().starts_with("# ") {
        body.to_string()
    } else {
        format!("# {title}\n\n{body}")
    };
    std::fs::write(&path, &content)?;

    index_one(conn, &path, &content)
}

/// Index a single note file, returning its id.
pub fn index_one(conn: &mut Connection, path: &Path, content: &str) -> Result<i64> {
    let path_s = path.to_string_lossy().to_string();
    let title = derive_title(content, path);
    let tags = parse_tags(content);
    let now = db::now();

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO notes(path, title, body, tags, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)
         ON CONFLICT(path) DO UPDATE SET
            title = excluded.title,
            body = excluded.body,
            tags = excluded.tags,
            updated_at = excluded.updated_at",
        params![
            path_s,
            title,
            content,
            serde_json::to_string(&tags)?,
            now
        ],
    )?;
    let id: i64 = tx.query_row(
        "SELECT id FROM notes WHERE path = ?1",
        params![path_s],
        |r| r.get(0),
    )?;

    tx.execute("DELETE FROM note_links WHERE src_id = ?1", params![id])?;
    {
        let mut ins = tx.prepare(
            "INSERT OR IGNORE INTO note_links(src_id, target, dst_id) VALUES (?1, ?2, ?3)",
        )?;
        for target in parse_links(content) {
            // Resolve by title now; unresolved links are re-resolved by
            // `resolve_links` once the target note exists.
            let dst: Option<i64> = tx
                .query_row(
                    "SELECT id FROM notes WHERE title = ?1 COLLATE NOCASE",
                    params![target],
                    |r| r.get(0),
                )
                .ok();
            ins.execute(params![id, target, dst])?;
        }
    }
    tx.commit()?;
    Ok(id)
}

/// Re-resolve links that had no target when they were written.
pub fn resolve_links(conn: &Connection) -> Result<usize> {
    let n = conn.execute(
        "UPDATE note_links
            SET dst_id = (SELECT n.id FROM notes n
                           WHERE n.title = note_links.target COLLATE NOCASE)
          WHERE dst_id IS NULL
            AND EXISTS (SELECT 1 FROM notes n
                         WHERE n.title = note_links.target COLLATE NOCASE)",
        [],
    )?;
    Ok(n)
}

/// Index every `.md` file in the notes directory. Rows for files that no longer
/// exist are dropped, since the filesystem is authoritative.
pub fn reindex(conn: &mut Connection, cfg: &Config) -> Result<usize> {
    let dir = notes_dir(cfg);
    std::fs::create_dir_all(&dir)?;

    let mut seen: Vec<String> = Vec::new();
    let entries = std::fs::read_dir(&dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        index_one(conn, &path, &content)?;
        seen.push(path.to_string_lossy().to_string());
    }

    // Drop notes whose file is gone.
    let existing: Vec<(i64, String)> = {
        let mut q = conn.prepare("SELECT id, path FROM notes")?;
        let mapped = q.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        let mut v = Vec::new();
        for m in mapped {
            v.push(m?);
        }
        v
    };
    for (id, path) in existing {
        if !Path::new(&path).is_file() {
            conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        }
    }

    resolve_links(conn)?;
    Ok(seen.len())
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let path: String = conn
        .query_row("SELECT path FROM notes WHERE id = ?1", params![id], |r| {
            r.get(0)
        })
        .map_err(|_| anyhow!("no note with id {id}"))?;
    // Remove the file first: if that fails the row stays and the state is still
    // consistent, which is the safer direction.
    if Path::new(&path).is_file() {
        std::fs::remove_file(&path)?;
    }
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    Ok(())
}

// ------------------------------------------------------------------- read --

fn row_to_note(r: &rusqlite::Row) -> rusqlite::Result<Note> {
    let tags: String = r.get(4)?;
    Ok(Note {
        id: r.get(0)?,
        path: r.get(1)?,
        title: r.get(2)?,
        body: r.get(3)?,
        tags: serde_json::from_str(&tags).unwrap_or_default(),
        created_at: r.get(5)?,
        updated_at: r.get(6)?,
    })
}

pub fn list(conn: &Connection, limit: i64) -> Result<Vec<Note>> {
    let mut q = conn.prepare(
        "SELECT id, path, title, body, tags, created_at, updated_at
           FROM notes ORDER BY updated_at DESC LIMIT ?1",
    )?;
    let rows = q.query_map(params![limit], |r| row_to_note(r))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: i64) -> Result<NoteDetail> {
    let note = conn.query_row(
        "SELECT id, path, title, body, tags, created_at, updated_at
           FROM notes WHERE id = ?1",
        params![id],
        |r| row_to_note(r),
    )?;

    let mut links = Vec::new();
    {
        let mut q = conn.prepare(
            "SELECT target, dst_id FROM note_links WHERE src_id = ?1 ORDER BY target",
        )?;
        let rows = q.query_map(params![id], |r| {
            Ok(NoteLink {
                target: r.get(0)?,
                dst_id: r.get(1)?,
            })
        })?;
        for row in rows {
            links.push(row?);
        }
    }

    let mut backlinks = Vec::new();
    {
        let mut q = conn.prepare(
            "SELECT n.id, n.title
               FROM note_links l JOIN notes n ON n.id = l.src_id
              WHERE l.dst_id = ?1 ORDER BY n.title",
        )?;
        let rows = q.query_map(params![id], |r| Ok((r.get(0)?, r.get(1)?)))?;
        for row in rows {
            backlinks.push(row?);
        }
    }

    Ok(NoteDetail {
        note,
        links,
        backlinks,
    })
}

#[cfg(test)]
mod tests {
    use super::{derive_title, parse_links, parse_tags, slugify};
    use std::path::Path;

    #[test]
    fn slug_is_filesystem_safe() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("  spaced   out  "), "spaced-out");
        assert_eq!(slugify("///"), "untitled");
        assert_eq!(slugify("Café Notes"), "café-notes");
    }

    #[test]
    fn wikilinks_are_extracted_in_order_without_duplicates() {
        let body = "see [[Alpha]] and [[Beta]], also [[Alpha]] again";
        assert_eq!(parse_links(body), vec!["Alpha", "Beta"]);
    }

    #[test]
    fn piped_wikilinks_keep_only_the_target() {
        assert_eq!(parse_links("[[real-target|shown text]]"), vec!["real-target"]);
    }

    #[test]
    fn unterminated_wikilink_is_ignored() {
        assert!(parse_links("broken [[oops").is_empty());
    }

    #[test]
    fn tags_are_parsed_but_headings_are_not() {
        let body = "# Heading\n\nbody #alpha and #beta-two\n## Another";
        let tags = parse_tags(body);
        assert!(tags.contains(&"alpha".to_string()));
        assert!(tags.contains(&"beta-two".to_string()));
        assert!(!tags.contains(&"heading".to_string()));
        assert!(!tags.iter().any(|t| t.starts_with('#')));
    }

    #[test]
    fn title_prefers_the_h1() {
        assert_eq!(
            derive_title("# Real Title\n\nbody", Path::new("x.md")),
            "Real Title"
        );
    }

    #[test]
    fn title_falls_back_to_first_line_then_filename() {
        assert_eq!(derive_title("just text\nmore", Path::new("x.md")), "just text");
        assert_eq!(derive_title("", Path::new("my-note.md")), "my-note");
    }
}
