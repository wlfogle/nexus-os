//! Tier 2: embeddings.
//!
//! Splits each file's text into overlapping chunks and embeds them with the
//! local `nomic-embed-text` model. Vectors are L2-normalised before storage so
//! every later similarity computation is a dot product.
//!
//! Per-repo centroids (the mean of a repo's chunk vectors, re-normalised) give
//! the classifier a cheap first-pass "which project is this about?" signal
//! without comparing every file against every chunk.

use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::HashMap;

use crate::config::Config;
use crate::db;
use crate::ollama::Ollama;

/// Target chunk size in characters. Roughly 250-400 tokens, comfortably inside
/// the embedding model's window with room for the overlap.
const CHUNK_CHARS: usize = 1400;
const OVERLAP_CHARS: usize = 200;
/// Chunks per HTTP round trip.
const EMBED_BATCH: usize = 32;

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct EmbedReport {
    pub files: usize,
    pub chunks: usize,
    pub failed: usize,
}

/// Split text into overlapping chunks, preferring to break at a newline.
pub fn chunk_text(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    if chars.len() <= CHUNK_CHARS {
        return vec![text.to_string()];
    }

    let mut out = Vec::new();
    let mut start = 0usize;
    while start < chars.len() {
        let hard_end = (start + CHUNK_CHARS).min(chars.len());
        // Prefer a newline in the last 20% of the window so chunks align with
        // logical boundaries instead of slicing mid-line.
        let mut end = hard_end;
        if hard_end < chars.len() {
            let floor = start + (CHUNK_CHARS * 4 / 5);
            if let Some(pos) = (floor..hard_end).rev().find(|&i| chars[i] == '\n') {
                end = pos + 1;
            }
        }
        let piece: String = chars[start..end].iter().collect();
        if !piece.trim().is_empty() {
            out.push(piece);
        }
        if end >= chars.len() {
            break;
        }
        start = end.saturating_sub(OVERLAP_CHARS).max(start + 1);
    }
    out
}

/// Files that have text but no vectors yet.
fn pending(conn: &Connection, limit: i64) -> Result<Vec<(i64, String)>> {
    let mut q = conn.prepare(
        "SELECT f.id, t.body
           FROM files f JOIN file_text t ON t.file_id = f.id
          WHERE f.present = 1 AND f.stage = 1 AND LENGTH(t.body) > 0
          ORDER BY f.mtime DESC
          LIMIT ?1",
    )?;
    let rows = q.query_map(params![limit], |r| Ok((r.get(0)?, r.get(1)?)))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Embed one batch of files, advancing them to stage 2.
pub async fn run(
    db_handle: &db::Db,
    cfg: &Config,
    client: &Ollama,
    limit: i64,
) -> Result<EmbedReport> {
    let mut report = EmbedReport::default();

    let work = {
        let conn = db_handle.lock().unwrap();
        pending(&conn, limit)?
    };
    if work.is_empty() {
        return Ok(report);
    }

    // Flatten every file's chunks into one stream so batches stay full.
    let mut flat: Vec<(i64, usize, String)> = Vec::new();
    for (file_id, body) in &work {
        for (i, c) in chunk_text(body).into_iter().enumerate() {
            flat.push((*file_id, i, c));
        }
    }

    let mut vectors: Vec<Option<Vec<f32>>> = Vec::with_capacity(flat.len());
    for group in flat.chunks(EMBED_BATCH) {
        let inputs: Vec<String> = group.iter().map(|(_, _, t)| t.clone()).collect();
        match client.embed(&cfg.models.embed, &inputs).await {
            Ok(vs) => {
                for mut v in vs {
                    db::normalize(&mut v);
                    vectors.push(Some(v));
                }
            }
            Err(e) => {
                eprintln!("librarian: embed batch failed: {e}");
                report.failed += group.len();
                for _ in group {
                    vectors.push(None);
                }
            }
        }
    }

    {
        let mut conn = db_handle.lock().unwrap();
        let tx = conn.transaction()?;
        {
            let mut ins = tx.prepare(
                "INSERT INTO chunks(file_id, ord, text, embedding, dim)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for ((file_id, ord, text), vec) in flat.iter().zip(vectors.iter()) {
                let (blob, dim) = match vec {
                    Some(v) => (Some(db::vec_to_blob(v)), v.len() as i64),
                    None => (None, 0),
                };
                ins.execute(params![file_id, *ord as i64, text, blob, dim])?;
                report.chunks += 1;
            }
        }
        for (file_id, _) in &work {
            tx.execute(
                "UPDATE files SET stage = 2 WHERE id = ?1",
                params![file_id],
            )?;
            report.files += 1;
        }
        tx.commit()?;
    }

    Ok(report)
}

/// Recompute the mean embedding for every repo.
pub fn rebuild_centroids(conn: &mut Connection) -> Result<usize> {
    let mut sums: HashMap<i64, (Vec<f32>, usize)> = HashMap::new();

    {
        let mut q = conn.prepare(
            "SELECT f.repo_id, c.embedding
               FROM chunks c JOIN files f ON f.id = c.file_id
              WHERE f.repo_id IS NOT NULL AND c.embedding IS NOT NULL AND f.present = 1",
        )?;
        let mut rows = q.query([])?;
        while let Some(row) = rows.next()? {
            let repo_id: i64 = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            let v = db::blob_to_vec(&blob);
            if v.is_empty() {
                continue;
            }
            let entry = sums.entry(repo_id).or_insert_with(|| (vec![0.0; v.len()], 0));
            if entry.0.len() != v.len() {
                continue;
            }
            for (acc, x) in entry.0.iter_mut().zip(v.iter()) {
                *acc += x;
            }
            entry.1 += 1;
        }
    }

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM repo_centroids", [])?;
    {
        let mut ins = tx.prepare(
            "INSERT INTO repo_centroids(repo_id, embedding, dim, n_chunks)
             VALUES (?1, ?2, ?3, ?4)",
        )?;
        for (repo_id, (mut acc, n)) in sums.iter().map(|(k, v)| (*k, v.clone())) {
            if n == 0 {
                continue;
            }
            for x in acc.iter_mut() {
                *x /= n as f32;
            }
            db::normalize(&mut acc);
            ins.execute(params![repo_id, db::vec_to_blob(&acc), acc.len() as i64, n as i64])?;
        }
    }
    let count = tx.query_row("SELECT COUNT(*) FROM repo_centroids", [], |r| {
        r.get::<_, i64>(0)
    })?;
    tx.commit()?;
    Ok(count as usize)
}

/// Mean of a file's chunk vectors: its overall semantic position.
pub fn file_vector(conn: &Connection, file_id: i64) -> Result<Option<Vec<f32>>> {
    let mut q = conn.prepare(
        "SELECT embedding FROM chunks WHERE file_id = ?1 AND embedding IS NOT NULL",
    )?;
    let mut rows = q.query(params![file_id])?;
    let mut acc: Vec<f32> = Vec::new();
    let mut n = 0usize;
    while let Some(row) = rows.next()? {
        let blob: Vec<u8> = row.get(0)?;
        let v = db::blob_to_vec(&blob);
        if v.is_empty() {
            continue;
        }
        if acc.is_empty() {
            acc = vec![0.0; v.len()];
        }
        if acc.len() != v.len() {
            continue;
        }
        for (a, x) in acc.iter_mut().zip(v.iter()) {
            *a += x;
        }
        n += 1;
    }
    if n == 0 {
        return Ok(None);
    }
    for x in acc.iter_mut() {
        *x /= n as f32;
    }
    db::normalize(&mut acc);
    Ok(Some(acc))
}

#[cfg(test)]
mod tests {
    use super::{chunk_text, CHUNK_CHARS};
    use crate::db::{blob_to_vec, dot, normalize, vec_to_blob};

    #[test]
    fn short_text_is_one_chunk() {
        let c = chunk_text("hello world");
        assert_eq!(c.len(), 1);
        assert_eq!(c[0], "hello world");
    }

    #[test]
    fn empty_text_yields_nothing() {
        assert!(chunk_text("").is_empty());
    }

    #[test]
    fn long_text_is_split_and_covers_everything() {
        let line = "abcdefghij\n";
        let text = line.repeat(1000); // 11k chars
        let chunks = chunk_text(&text);
        assert!(chunks.len() > 1);
        for c in &chunks {
            assert!(c.chars().count() <= CHUNK_CHARS);
        }
        // Overlapping chunks must still cover the start and the end.
        assert!(text.starts_with(chunks[0].as_str()));
        assert!(text.ends_with(chunks[chunks.len() - 1].as_str()));
    }

    #[test]
    fn chunking_terminates_on_text_without_newlines() {
        let text = "x".repeat(10_000);
        let chunks = chunk_text(&text);
        assert!(chunks.len() > 1);
    }

    #[test]
    fn vector_roundtrip_and_normalisation() {
        let mut v = vec![3.0f32, 4.0];
        normalize(&mut v);
        assert!((v[0] - 0.6).abs() < 1e-6);
        assert!((v[1] - 0.8).abs() < 1e-6);

        let blob = vec_to_blob(&v);
        let back = blob_to_vec(&blob);
        assert_eq!(back.len(), 2);
        assert!((dot(&v, &back) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn normalising_zero_vector_is_safe() {
        let mut v = vec![0.0f32, 0.0];
        normalize(&mut v);
        assert_eq!(v, vec![0.0, 0.0]);
    }
}
