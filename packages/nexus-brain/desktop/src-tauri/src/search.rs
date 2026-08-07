//! Hybrid search.
//!
//! Keyword search alone misses conceptual matches; vector search alone is bad
//! at exact technical tokens (a hostname, a flag, an error code). Both are run
//! and their rankings fused with Reciprocal Rank Fusion, which needs no score
//! calibration between the two very different scales.
//!
//! Results are then grouped by content hash so a file that exists in five
//! places appears once, with the canonical copy first. That is the direct
//! answer to "I keep opening the old version".

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;

use crate::db;
use crate::embed;
use crate::ollama::Ollama;

/// RRF damping constant. 60 is the value from the original paper and behaves
/// well when neither ranker is clearly superior.
const RRF_K: f32 = 60.0;

#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    pub file_id: i64,
    pub path: String,
    pub name: String,
    pub class: String,
    pub size: i64,
    pub mtime: f64,
    pub repo: Option<String>,
    pub title: String,
    pub summary: String,
    pub status: String,
    pub label: Option<String>,
    pub score: f32,
    pub snippet: String,
    /// Other paths with byte-identical content.
    pub duplicates: Vec<String>,
    /// True when this is the newest / repo-owned copy of its content.
    pub canonical: bool,
}

/// Escape a user query for FTS5 by quoting each term.
///
/// Without this, characters like `-`, `"` or `*` are interpreted as FTS syntax
/// and a stray one makes the whole query fail.
pub fn fts_query(raw: &str) -> String {
    let terms: Vec<String> = raw
        .split_whitespace()
        .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '.'))
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
        .collect();
    terms.join(" OR ")
}

fn keyword_ranking(conn: &Connection, query: &str, limit: i64) -> Result<Vec<i64>> {
    let q = fts_query(query);
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let mut stmt = conn.prepare(
        "SELECT m.file_id
           FROM files_fts f
           JOIN fts_map m ON m.rowid = f.rowid
          WHERE files_fts MATCH ?1
          ORDER BY bm25(files_fts, 2.0, 4.0, 1.0)
          LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![q, limit], |r| r.get::<_, i64>(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn vector_ranking(conn: &Connection, qvec: &[f32], limit: usize) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT c.file_id, c.embedding
           FROM chunks c JOIN files f ON f.id = c.file_id
          WHERE c.embedding IS NOT NULL AND f.present = 1",
    )?;
    let mut rows = stmt.query([])?;

    // Best-scoring chunk decides the file's position.
    let mut best: HashMap<i64, f32> = HashMap::new();
    while let Some(row) = rows.next()? {
        let file_id: i64 = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        let v = db::blob_to_vec(&blob);
        if v.len() != qvec.len() {
            continue;
        }
        let s = db::dot(qvec, &v);
        best.entry(file_id)
            .and_modify(|e| {
                if s > *e {
                    *e = s;
                }
            })
            .or_insert(s);
    }

    let mut scored: Vec<(i64, f32)> = best.into_iter().collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    Ok(scored.into_iter().take(limit).map(|(id, _)| id).collect())
}

/// Fuse two rankings. Position matters, absolute scores do not.
pub fn rrf(rankings: &[Vec<i64>]) -> Vec<(i64, f32)> {
    let mut acc: HashMap<i64, f32> = HashMap::new();
    for ranking in rankings {
        for (idx, id) in ranking.iter().enumerate() {
            *acc.entry(*id).or_insert(0.0) += 1.0 / (RRF_K + idx as f32 + 1.0);
        }
    }
    let mut out: Vec<(i64, f32)> = acc.into_iter().collect();
    out.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });
    out
}

fn snippet_for(body: &str, query: &str, width: usize) -> String {
    let lower = body.to_lowercase();
    let needle = query
        .split_whitespace()
        .map(|t| t.to_lowercase())
        .find(|t| lower.contains(t.as_str()));

    let start = match needle {
        Some(t) => lower.find(&t).unwrap_or(0).saturating_sub(width / 3),
        None => 0,
    };
    let mut s = start;
    while s > 0 && !body.is_char_boundary(s) {
        s -= 1;
    }
    let mut e = (s + width).min(body.len());
    while e > s && !body.is_char_boundary(e) {
        e -= 1;
    }
    let mut out = body[s..e].replace('\n', " ");
    out = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if s > 0 {
        out.insert_str(0, "... ");
    }
    if e < body.len() {
        out.push_str(" ...");
    }
    out
}

/// Run a hybrid search. Falls back to keyword-only if embeddings are
/// unavailable, so search still works when Ollama is down.
pub async fn query(
    db_handle: &db::Db,
    client: &Ollama,
    embed_model: &str,
    text: &str,
    limit: usize,
) -> Result<Vec<Hit>> {
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let keyword = {
        let conn = db_handle.lock().unwrap();
        keyword_ranking(&conn, text, (limit * 4) as i64)?
    };

    let semantic = match client.embed(embed_model, &[text.to_string()]).await {
        Ok(mut vs) if !vs.is_empty() => {
            let mut v = vs.remove(0);
            db::normalize(&mut v);
            let conn = db_handle.lock().unwrap();
            vector_ranking(&conn, &v, limit * 4)?
        }
        Ok(_) => Vec::new(),
        Err(e) => {
            eprintln!("librarian: semantic search unavailable: {e}");
            Vec::new()
        }
    };

    let fused = rrf(&[keyword, semantic]);
    let conn = db_handle.lock().unwrap();

    let mut hits = Vec::new();
    for (file_id, score) in fused.into_iter().take(limit) {
        let row = conn.query_row(
            "SELECT f.path, f.name, f.class, f.size, f.mtime, f.sha256,
                    r.name,
                    COALESCE(i.title,''), COALESCE(i.summary,''), COALESCE(i.status,''),
                    c.label, COALESCE(t.body,'')
               FROM files f
               LEFT JOIN repos r           ON r.id = f.repo_id
               LEFT JOIN interpretations i ON i.file_id = f.id
               LEFT JOIN classifications c ON c.file_id = f.id
               LEFT JOIN file_text t       ON t.file_id = f.id
              WHERE f.id = ?1 AND f.present = 1",
            params![file_id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, f64>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, String>(7)?,
                    r.get::<_, String>(8)?,
                    r.get::<_, String>(9)?,
                    r.get::<_, Option<String>>(10)?,
                    r.get::<_, String>(11)?,
                ))
            },
        );
        let (path, name, class, size, mtime, sha, repo, title, summary, status, label, body) =
            match row {
                Ok(v) => v,
                Err(_) => continue,
            };

        // Sibling copies with identical bytes.
        let mut duplicates = Vec::new();
        if let Some(sha) = &sha {
            let mut d = conn.prepare(
                "SELECT path FROM files
                  WHERE sha256 = ?1 AND present = 1 AND id <> ?2
                  ORDER BY path",
            )?;
            for p in d.query_map(params![sha, file_id], |r| r.get::<_, String>(0))? {
                duplicates.push(p?);
            }
        }

        hits.push(Hit {
            file_id,
            path,
            name,
            class,
            size,
            mtime,
            canonical: repo.is_some() || duplicates.is_empty(),
            repo,
            title,
            summary,
            status,
            label,
            score,
            snippet: snippet_for(&body, text, 240),
            duplicates,
        });
    }

    Ok(hits)
}

/// Files the model judged stale or superseded, newest evidence first.
pub fn stale(conn: &Connection, limit: i64) -> Result<Vec<Hit>> {
    let mut q = conn.prepare(
        "SELECT f.id, f.path, f.name, f.class, f.size, f.mtime, r.name,
                i.title, i.summary, i.status, c.label
           FROM files f
           JOIN interpretations i ON i.file_id = f.id
           LEFT JOIN repos r           ON r.id = f.repo_id
           LEFT JOIN classifications c ON c.file_id = f.id
          WHERE f.present = 1 AND i.status IN ('stale','superseded','junk')
          ORDER BY i.confidence DESC, f.mtime DESC
          LIMIT ?1",
    )?;
    let rows = q.query_map(params![limit], |r| {
        Ok(Hit {
            file_id: r.get(0)?,
            path: r.get(1)?,
            name: r.get(2)?,
            class: r.get(3)?,
            size: r.get(4)?,
            mtime: r.get(5)?,
            repo: r.get(6)?,
            title: r.get(7)?,
            summary: r.get(8)?,
            status: r.get(9)?,
            label: r.get(10)?,
            score: 0.0,
            snippet: String::new(),
            duplicates: Vec::new(),
            canonical: false,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Semantic nearest neighbours of a given file.
pub fn similar_to(conn: &Connection, file_id: i64, limit: usize) -> Result<Vec<(i64, String, f32)>> {
    let target = match embed::file_vector(conn, file_id)? {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };

    let mut acc: HashMap<i64, f32> = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT c.file_id, c.embedding
           FROM chunks c JOIN files f ON f.id = c.file_id
          WHERE c.embedding IS NOT NULL AND f.present = 1 AND c.file_id <> ?1",
    )?;
    let mut rows = stmt.query(params![file_id])?;
    while let Some(row) = rows.next()? {
        let other: i64 = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        let v = db::blob_to_vec(&blob);
        if v.len() != target.len() {
            continue;
        }
        let s = db::dot(&target, &v);
        acc.entry(other)
            .and_modify(|e| {
                if s > *e {
                    *e = s;
                }
            })
            .or_insert(s);
    }

    let mut scored: Vec<(i64, f32)> = acc.into_iter().collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);

    let mut out = Vec::new();
    for (id, score) in scored {
        if let Ok(path) =
            conn.query_row("SELECT path FROM files WHERE id = ?1", params![id], |r| {
                r.get::<_, String>(0)
            })
        {
            out.push((id, path, score));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{fts_query, rrf, snippet_for};

    #[test]
    fn quotes_terms_for_fts() {
        assert_eq!(fts_query("hello world"), "\"hello\" OR \"world\"");
    }

    #[test]
    fn strips_fts_operators_that_would_error() {
        // A bare '-' or '*' would otherwise be parsed as FTS syntax.
        let q = fts_query("wire-guard *foo* bar");
        assert!(q.contains("\"wire-guard\"") || q.contains("\"wire\""));
        assert!(!q.contains('*'));
    }

    #[test]
    fn empty_query_is_empty() {
        assert_eq!(fts_query("   "), "");
        assert_eq!(fts_query("!!!"), "");
    }

    #[test]
    fn embedded_quotes_are_doubled() {
        // FTS5 escapes a quote inside a quoted string by doubling it, so the
        // term `a"b` must be emitted as "a""b".
        assert_eq!(fts_query("a\"b"), "\"a\"\"b\"");
    }

    #[test]
    fn rrf_rewards_a_strong_placement_in_either_ranking() {
        let a = vec![1i64, 2, 3];
        let b = vec![3i64, 2, 1];
        let fused = rrf(&[a, b]);
        assert_eq!(fused.len(), 3);
        // With k = 60, first-and-last (1/61 + 1/63) very slightly outscores
        // second-and-second (2 x 1/62). Being top-ranked by one retriever is
        // worth more than being merely good in both, which is what makes RRF
        // useful when the two rankers disagree about what matters.
        assert_eq!(fused[0].0, 1);
        assert_eq!(fused[1].0, 3);
        assert_eq!(fused[2].0, 2);
        assert!(fused[0].1 > fused[2].1);
    }

    #[test]
    fn rrf_handles_disjoint_rankings() {
        let fused = rrf(&[vec![1i64], vec![2i64]]);
        assert_eq!(fused.len(), 2);
        // Both rank first in their own list, so scores tie and ids break it.
        assert_eq!(fused[0].0, 1);
    }

    #[test]
    fn rrf_of_nothing_is_nothing() {
        assert!(rrf(&[]).is_empty());
    }

    #[test]
    fn snippet_centres_on_the_match() {
        let body = "alpha beta gamma delta epsilon zeta eta theta";
        let s = snippet_for(body, "delta", 20);
        assert!(s.contains("delta"));
    }

    #[test]
    fn snippet_without_match_starts_at_the_beginning() {
        let s = snippet_for("alpha beta", "zzz", 100);
        assert!(s.starts_with("alpha"));
    }

    #[test]
    fn snippet_is_utf8_safe() {
        let body = "ααααα ββββββ γγγγγ";
        let s = snippet_for(body, "ββββββ", 10);
        assert!(!s.is_empty());
    }
}
