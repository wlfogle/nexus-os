//! Relevance, labelling and supersession.
//!
//! Tier 3 tells us what a file *is*. This decides what it *means* in the
//! context of the repositories that exist, by combining the model's judgement
//! with structural signals it cannot see:
//!
//!   * semantic distance to each repo's centroid (which project is this about?)
//!   * that repo's commit recency (is that project still alive?)
//!   * lexical overlap with the repo's TF-IDF fingerprint
//!   * whether identical bytes already live inside a repo
//!   * how long since the file itself was touched
//!
//! It also builds the supersession graph: for every loose file, the
//! repo-owned file that has replaced it. That is the direct answer to
//! "I keep referencing old information" -- the stale copy now points at the
//! canonical one rather than merely being flagged old.

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;

use crate::config::Config;
use crate::db;

/// Cosine above which two files are considered to be about the same thing.
///
/// Deliberately high: a false "superseded by" is worse than a missed one,
/// because it sends you to the wrong file. Raised from 0.86 after a live run
/// put more than half of all edges in the 0.86-0.90 band, which on inspection
/// was noise.
const SUPERSEDE_MIN_SIMILARITY: f32 = 0.92;

/// Minimum extracted characters before a file may take part in supersession.
///
/// Short strings embed into nearly the same vector regardless of meaning. A
/// live run paired a screenshot whose OCR produced 17 characters
/// ("FI Firnuare Setup") with an HTML file containing 19 ("AI Coding
/// Assistant") at cosine 1.0. Neither had enough signal to compare, so both
/// are now excluded rather than trusted.
const SUPERSEDE_MIN_TEXT: i64 = 200;

/// Semantic score above which a file counts as belonging to a repo.
const RELEVANT_MIN: f32 = 0.45;

/// Terms kept per repository fingerprint.
const TOPICS_PER_REPO: usize = 40;

/// May a file of class `a` be superseded by one of class `b`?
///
/// Supersession means "a newer version of the same thing", which implies the
/// same kind of thing. Prose may move between plain text and a rendered
/// document, but a screenshot is never a newer version of an HTML page.
pub fn classes_compatible(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let prose = |c: &str| matches!(c, "doc" | "document" | "web");
    let codeish = |c: &str| matches!(c, "code" | "script");
    (prose(a) && prose(b)) || (codeish(a) && codeish(b))
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ClassifyReport {
    pub classified: usize,
    pub topics: usize,
    pub supersedes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Supersession {
    pub old_id: i64,
    pub old_path: String,
    pub new_id: i64,
    pub new_path: String,
    pub new_repo: Option<String>,
    pub similarity: f32,
    pub reason: String,
}

// ------------------------------------------------------------- tokenising --

/// Lowercase alphanumeric words of length >= 3. Deliberately crude: the IDF
/// term below removes the words that appear everywhere, which is a better
/// stopword list than any fixed one.
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.len() >= 3 && w.len() <= 40)
        .filter(|w| !w.chars().all(|c| c.is_numeric()))
        .map(|w| w.to_lowercase())
        .collect()
}

/// Build a TF-IDF fingerprint per repository from the text of the files it owns.
pub fn rebuild_repo_topics(conn: &mut Connection) -> Result<usize> {
    // term -> repo -> count
    let mut per_repo: HashMap<i64, HashMap<String, f64>> = HashMap::new();
    let mut doc_freq: HashMap<String, usize> = HashMap::new();

    {
        let mut q = conn.prepare(
            "SELECT f.repo_id, t.body
               FROM files f JOIN file_text t ON t.file_id = f.id
              WHERE f.present = 1 AND f.repo_id IS NOT NULL",
        )?;
        let mut rows = q.query([])?;
        while let Some(row) = rows.next()? {
            let repo_id: i64 = row.get(0)?;
            let body: String = row.get(1)?;
            let counts = per_repo.entry(repo_id).or_default();
            let mut seen_here: HashMap<String, bool> = HashMap::new();
            for term in tokenize(&body) {
                *counts.entry(term.clone()).or_insert(0.0) += 1.0;
                seen_here.entry(term).or_insert(true);
            }
            for term in seen_here.keys() {
                *doc_freq.entry(term.clone()).or_insert(0) += 1;
            }
        }
    }

    let n_docs = doc_freq.values().copied().max().unwrap_or(1).max(1) as f64;

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM repo_topics", [])?;
    let mut total = 0usize;
    {
        let mut ins = tx.prepare(
            "INSERT INTO repo_topics(repo_id, term, weight) VALUES (?1, ?2, ?3)",
        )?;
        for (repo_id, counts) in &per_repo {
            let mut scored: Vec<(String, f64)> = counts
                .iter()
                .map(|(term, tf)| {
                    let df = *doc_freq.get(term).unwrap_or(&1) as f64;
                    // Classic TF-IDF: frequent here, rare elsewhere.
                    (term.clone(), tf.ln_1p() * (n_docs / df).ln().max(0.0))
                })
                .filter(|(_, w)| *w > 0.0)
                .collect();
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            for (term, weight) in scored.into_iter().take(TOPICS_PER_REPO) {
                ins.execute(params![repo_id, term, weight])?;
                total += 1;
            }
        }
    }
    tx.commit()?;
    Ok(total)
}

// --------------------------------------------------------------- vectors ---

/// Mean chunk vector per file, in one pass instead of a query per file.
pub fn load_file_vectors(conn: &Connection) -> Result<HashMap<i64, Vec<f32>>> {
    let mut acc: HashMap<i64, (Vec<f32>, usize)> = HashMap::new();
    let mut q = conn.prepare(
        "SELECT c.file_id, c.embedding
           FROM chunks c JOIN files f ON f.id = c.file_id
          WHERE c.embedding IS NOT NULL AND f.present = 1",
    )?;
    let mut rows = q.query([])?;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        let v = db::blob_to_vec(&blob);
        if v.is_empty() {
            continue;
        }
        let e = acc.entry(id).or_insert_with(|| (vec![0.0; v.len()], 0));
        if e.0.len() != v.len() {
            continue;
        }
        for (a, x) in e.0.iter_mut().zip(v.iter()) {
            *a += x;
        }
        e.1 += 1;
    }

    let mut out = HashMap::with_capacity(acc.len());
    for (id, (mut v, n)) in acc {
        if n == 0 {
            continue;
        }
        for x in v.iter_mut() {
            *x /= n as f32;
        }
        db::normalize(&mut v);
        out.insert(id, v);
    }
    Ok(out)
}

fn load_centroids(conn: &Connection) -> Result<Vec<(i64, Vec<f32>)>> {
    let mut q = conn.prepare("SELECT repo_id, embedding FROM repo_centroids")?;
    let mut rows = q.query([])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        let v = db::blob_to_vec(&blob);
        if !v.is_empty() {
            out.push((id, v));
        }
    }
    Ok(out)
}

/// Decide a label from the combined signals.
///
/// Kept as a free function so the decision table is directly testable without
/// a database.
pub fn decide_label(
    kind: &str,
    status: &str,
    semantic: f32,
    age_days: i64,
    stale_days: i64,
    duplicate_of_repo_file: bool,
    has_text: bool,
) -> &'static str {
    // Credentials outrank everything: they must surface for review regardless
    // of how relevant or fresh they look.
    if kind == "secret" {
        return "SECRET_RISK";
    }
    if !has_text {
        return "UNKNOWN";
    }
    // Identical bytes already inside a repo: the repo copy is authoritative.
    if duplicate_of_repo_file {
        return "DUPLICATE";
    }
    let relevant = semantic >= RELEVANT_MIN;
    match status {
        "junk" => "UNRELATED",
        "superseded" | "stale" if relevant => "RELEVANT_STALE",
        "current" if relevant => {
            if age_days > stale_days {
                "RELEVANT_STALE"
            } else {
                "CURRENT"
            }
        }
        _ if relevant => {
            if age_days > stale_days {
                "RELEVANT_STALE"
            } else {
                "CURRENT"
            }
        }
        _ => "UNRELATED",
    }
}

/// Classify every interpreted file and rebuild the supersession graph.
pub fn run(conn: &mut Connection, cfg: &Config) -> Result<ClassifyReport> {
    let mut report = ClassifyReport::default();
    report.topics = rebuild_repo_topics(conn)?;

    let vectors = load_file_vectors(conn)?;
    let centroids = load_centroids(conn)?;
    let now = db::now();

    // Hashes that occur inside at least one repo-owned file.
    let mut repo_hashes: HashMap<String, i64> = HashMap::new();
    {
        let mut q = conn.prepare(
            "SELECT sha256, MIN(id) FROM files
              WHERE present = 1 AND repo_id IS NOT NULL AND sha256 IS NOT NULL
              GROUP BY sha256",
        )?;
        let mut rows = q.query([])?;
        while let Some(row) = rows.next()? {
            repo_hashes.insert(row.get(0)?, row.get(1)?);
        }
    }

    struct Row {
        id: i64,
        mtime: f64,
        sha: Option<String>,
        repo_id: Option<i64>,
        kind: String,
        status: String,
        has_text: bool,
    }

    let rows: Vec<Row> = {
        let mut q = conn.prepare(
            "SELECT f.id, f.mtime, f.sha256, f.repo_id,
                    COALESCE(i.kind,''), COALESCE(i.status,''),
                    (t.file_id IS NOT NULL)
               FROM files f
               JOIN interpretations i ON i.file_id = f.id
               LEFT JOIN file_text t  ON t.file_id = f.id
              WHERE f.present = 1",
        )?;
        let mapped = q.query_map([], |r| {
            Ok(Row {
                id: r.get(0)?,
                mtime: r.get(1)?,
                sha: r.get(2)?,
                repo_id: r.get(3)?,
                kind: r.get(4)?,
                status: r.get(5)?,
                has_text: r.get::<_, i64>(6)? != 0,
            })
        })?;
        let mut v = Vec::new();
        for m in mapped {
            v.push(m?);
        }
        v
    };

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM classifications", [])?;
    {
        let mut ins = tx.prepare(
            "INSERT INTO classifications(file_id, label, score, best_repo, signals, decided_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for r in &rows {
            let vec = vectors.get(&r.id);
            let mut best_repo: Option<i64> = None;
            let mut semantic = 0.0f32;
            if let Some(v) = vec {
                for (repo_id, c) in &centroids {
                    if c.len() != v.len() {
                        continue;
                    }
                    let s = db::dot(v, c);
                    if s > semantic {
                        semantic = s;
                        best_repo = Some(*repo_id);
                    }
                }
            }

            let age_days = (((now - r.mtime) / 86_400.0).max(0.0)) as i64;
            let dup = r
                .sha
                .as_ref()
                .and_then(|s| repo_hashes.get(s))
                .map(|owner| r.repo_id.is_none() && *owner != r.id)
                .unwrap_or(false);

            let label = decide_label(
                &r.kind,
                &r.status,
                semantic,
                age_days,
                cfg.stale_days,
                dup,
                r.has_text,
            );

            let signals = serde_json::json!({
                "semantic": semantic,
                "age_days": age_days,
                "interpretation_status": r.status,
                "kind": r.kind,
                "duplicate_of_repo_file": dup,
                "repo_owned": r.repo_id.is_some(),
            });

            ins.execute(params![
                r.id,
                label,
                semantic,
                best_repo,
                signals.to_string(),
                now
            ])?;
            report.classified += 1;
        }
    }
    tx.commit()?;

    report.supersedes = build_supersedes(conn, &vectors)?;
    Ok(report)
}

/// For each loose file, find the repo-owned file that has replaced it.
///
/// Only loose-versus-owned pairs are considered. Comparing everything against
/// everything is quadratic and, more importantly, meaningless: two files inside
/// the same repo being similar is normal, not evidence of staleness.
fn build_supersedes(
    conn: &mut Connection,
    vectors: &HashMap<i64, Vec<f32>>,
) -> Result<usize> {
    struct Cand {
        id: i64,
        mtime: f64,
        class: String,
    }
    let mut loose: Vec<Cand> = Vec::new();
    let mut owned: Vec<Cand> = Vec::new();
    {
        // Only files with enough extracted text are eligible: a comparison
        // between two near-empty documents is meaningless however high the
        // cosine comes out.
        let mut q = conn.prepare(
            "SELECT f.id, f.mtime, f.class, (f.repo_id IS NOT NULL)
               FROM files f JOIN file_text t ON t.file_id = f.id
              WHERE f.present = 1 AND LENGTH(t.body) >= ?1",
        )?;
        let mut rows = q.query(params![SUPERSEDE_MIN_TEXT])?;
        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            if !vectors.contains_key(&id) {
                continue;
            }
            let c = Cand {
                id,
                mtime: row.get(1)?,
                class: row.get(2)?,
            };
            if row.get::<_, i64>(3)? != 0 {
                owned.push(c);
            } else {
                loose.push(c);
            }
        }
    }

    let mut edges: Vec<(i64, i64, f32)> = Vec::new();
    for l in &loose {
        let lv = &vectors[&l.id];
        let mut best: Option<(i64, f32)> = None;
        for o in &owned {
            // A newer version of a thing is the same kind of thing.
            if !classes_compatible(&l.class, &o.class) {
                continue;
            }
            let ov = &vectors[&o.id];
            if ov.len() != lv.len() {
                continue;
            }
            // Only a *newer* canonical file supersedes an older loose one.
            if o.mtime <= l.mtime {
                continue;
            }
            let s = db::dot(lv, ov);
            if s >= SUPERSEDE_MIN_SIMILARITY && best.map(|(_, b)| s > b).unwrap_or(true) {
                best = Some((o.id, s));
            }
        }
        if let Some((new_id, sim)) = best {
            edges.push((l.id, new_id, sim));
        }
    }

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM supersedes", [])?;
    {
        let mut ins = tx.prepare(
            "INSERT OR REPLACE INTO supersedes(old_id, new_id, similarity, reason)
             VALUES (?1, ?2, ?3, ?4)",
        )?;
        for (old, new, sim) in &edges {
            ins.execute(params![
                old,
                new,
                sim,
                format!("a newer repo-owned file covers the same content ({:.0}% similar)", sim * 100.0)
            ])?;
        }
    }
    tx.commit()?;
    Ok(edges.len())
}

/// Supersession edges for display, newest evidence first.
pub fn list_supersessions(conn: &Connection, limit: i64) -> Result<Vec<Supersession>> {
    let mut q = conn.prepare(
        "SELECT s.old_id, fo.path, s.new_id, fn.path, r.name, s.similarity, s.reason
           FROM supersedes s
           JOIN files fo ON fo.id = s.old_id
           JOIN files fn ON fn.id = s.new_id
           LEFT JOIN repos r ON r.id = fn.repo_id
          WHERE fo.present = 1 AND fn.present = 1
          ORDER BY s.similarity DESC
          LIMIT ?1",
    )?;
    let rows = q.query_map(params![limit], |r| {
        Ok(Supersession {
            old_id: r.get(0)?,
            old_path: r.get(1)?,
            new_id: r.get(2)?,
            new_path: r.get(3)?,
            new_repo: r.get(4)?,
            similarity: r.get(5)?,
            reason: r.get(6)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Per-repo topic fingerprint, for display.
pub fn repo_topics(conn: &Connection, repo: &str, limit: i64) -> Result<Vec<(String, f64)>> {
    let mut q = conn.prepare(
        "SELECT t.term, t.weight
           FROM repo_topics t JOIN repos r ON r.id = t.repo_id
          WHERE r.name = ?1
          ORDER BY t.weight DESC LIMIT ?2",
    )?;
    let rows = q.query_map(params![repo, limit], |r| Ok((r.get(0)?, r.get(1)?)))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{decide_label, tokenize};

    #[test]
    fn tokenizer_keeps_words_and_drops_noise() {
        let t = tokenize("Hello, world! ab 12345 snake_case FOO-bar");
        assert!(t.contains(&"hello".to_string()));
        assert!(t.contains(&"world".to_string()));
        assert!(t.contains(&"snake_case".to_string()));
        assert!(t.contains(&"foo".to_string()));
        // too short, and pure digits
        assert!(!t.contains(&"ab".to_string()));
        assert!(!t.contains(&"12345".to_string()));
    }

    #[test]
    fn secrets_outrank_every_other_signal() {
        // Even a perfectly current, highly relevant file must surface.
        assert_eq!(
            decide_label("secret", "current", 0.99, 0, 180, false, true),
            "SECRET_RISK"
        );
    }

    #[test]
    fn missing_text_is_unknown_not_unrelated() {
        assert_eq!(
            decide_label("artifact", "", 0.0, 0, 180, false, false),
            "UNKNOWN"
        );
    }

    #[test]
    fn identical_bytes_in_a_repo_make_it_a_duplicate() {
        assert_eq!(
            decide_label("doc", "current", 0.9, 1, 180, true, true),
            "DUPLICATE"
        );
    }

    #[test]
    fn relevant_and_fresh_is_current() {
        assert_eq!(
            decide_label("doc", "current", 0.8, 10, 180, false, true),
            "CURRENT"
        );
    }

    #[test]
    fn relevant_but_old_is_stale_even_if_the_model_said_current() {
        assert_eq!(
            decide_label("doc", "current", 0.8, 400, 180, false, true),
            "RELEVANT_STALE"
        );
    }

    #[test]
    fn model_saying_stale_wins_when_relevant() {
        assert_eq!(
            decide_label("doc", "stale", 0.8, 1, 180, false, true),
            "RELEVANT_STALE"
        );
    }

    #[test]
    fn low_semantic_score_is_unrelated() {
        assert_eq!(
            decide_label("doc", "current", 0.1, 1, 180, false, true),
            "UNRELATED"
        );
    }

    #[test]
    fn junk_is_unrelated_regardless_of_similarity() {
        assert_eq!(
            decide_label("doc", "junk", 0.95, 1, 180, false, true),
            "UNRELATED"
        );
    }

    #[test]
    fn an_image_can_never_supersede_a_web_page() {
        // The exact false positive seen on a live run: a screenshot whose OCR
        // produced 17 characters matched an HTML file at cosine 1.0.
        assert!(!super::classes_compatible("image", "web"));
        assert!(!super::classes_compatible("image", "code"));
        assert!(!super::classes_compatible("archive", "doc"));
    }

    #[test]
    fn prose_formats_are_interchangeable() {
        assert!(super::classes_compatible("doc", "document"));
        assert!(super::classes_compatible("doc", "web"));
        assert!(super::classes_compatible("document", "web"));
    }

    #[test]
    fn code_and_scripts_are_interchangeable_but_not_with_config() {
        assert!(super::classes_compatible("code", "script"));
        assert!(!super::classes_compatible("code", "config"));
    }

    #[test]
    fn identical_classes_always_match() {
        for c in ["doc", "code", "config", "image", "data"] {
            assert!(super::classes_compatible(c, c));
        }
    }
}
