//! Tier 3: read and interpret every file with a local model.
//!
//! Each file is sent to a model chosen by its content class, and the model
//! returns a structured judgement: what the file is, what it is for, which
//! project it belongs to, whether it is still current, and what should happen
//! to it.
//!
//! Two properties matter for this to finish in reasonable time on one GPU:
//!
//! * **Model-grouped batching.** The queue is sorted by target model and each
//!   model drains fully before the next is touched. Ollama keeps a model
//!   resident, so interleaving files across models would evict and reload
//!   multi-gigabyte weights constantly.
//! * **Escalation, not brute force.** Cheap models handle the bulk. Only files
//!   whose confidence lands below the threshold are re-run on a bigger model,
//!   so the expensive weights see a small fraction of the corpus.

use anyhow::Result;
use base64::Engine;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::Config;
use crate::db;
use crate::ollama::Ollama;

const SYSTEM: &str = "\
You are a meticulous file librarian. You are shown one file from a developer's \
computer. Decide what it actually is by reading its contents, not by guessing \
from the name. Reply with a single JSON object and nothing else.";

/// The judgement we ask the model for.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Interpretation {
    pub title: String,
    pub kind: String,
    pub purpose: String,
    pub summary: String,
    pub topics: Vec<String>,
    pub entities: Vec<String>,
    pub related_repo: String,
    pub status: String,
    pub action: String,
    pub reason: String,
    pub confidence: f32,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct InterpretReport {
    pub done: usize,
    pub escalated: usize,
    pub failed: usize,
    pub by_model: BTreeMap<String, usize>,
}

/// A file awaiting interpretation.
#[derive(Debug, Clone)]
pub struct Job {
    pub id: i64,
    pub path: String,
    pub class: String,
    pub size: i64,
    pub body: String,
    pub repo: Option<String>,
}

/// Which model should read this class of file first.
pub fn route(cfg: &Config, class: &str, size: i64) -> String {
    let m = &cfg.models;
    match class {
        "image" => m.vision.clone(),
        "code" | "script" | "config" => m.code.clone(),
        "doc" | "document" | "web" => {
            // Very short prose is not worth a 7B model.
            if size < 2_000 {
                m.triage.clone()
            } else {
                m.docs.clone()
            }
        }
        _ => m.triage.clone(),
    }
}

/// Next model up when confidence is too low.
fn escalation_for(cfg: &Config, class: &str, current: &str) -> Option<String> {
    let m = &cfg.models;
    let next = if class == "image" {
        m.vision_escalate.clone()
    } else if current == m.triage {
        m.docs.clone()
    } else if current == m.docs || current == m.code {
        m.escalate.clone()
    } else if current == m.escalate {
        m.escalate_max.clone()
    } else {
        return None;
    };
    if next == current {
        None
    } else {
        Some(next)
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

fn build_prompt(job: &Job, repos: &[String]) -> String {
    let repo_list = if repos.is_empty() {
        "(none known)".to_string()
    } else {
        repos.join(", ")
    };
    let owned = job
        .repo
        .as_deref()
        .unwrap_or("none - this file sits outside every git repository");

    format!(
        r#"FILE PATH: {path}
CONTENT CLASS: {class}
SIZE: {size} bytes
OWNING GIT REPO: {owned}
KNOWN PROJECTS ON THIS MACHINE: {repo_list}

--- FILE CONTENT (truncated) ---
{body}
--- END CONTENT ---

Return JSON with exactly these keys:
{{
  "title":        short human title for this file,
  "kind":         one of note|script|config|source|doc|data|log|artifact|secret|junk,
  "purpose":      one sentence on what it is for,
  "summary":      2-3 sentences on what it contains,
  "topics":       array of up to 6 lowercase topic keywords,
  "entities":     array of concrete names it references (projects, services, hosts, tools),
  "related_repo": the single most related project from the list above, or "" if none,
  "status":       one of current|stale|superseded|reference|junk,
  "action":       one of keep|file|archive|quarantine|review,
  "reason":       one sentence justifying status and action,
  "confidence":   number between 0 and 1
}}

Guidance:
- "junk" means build output, crash dumps, editor backups, or throwaway scratch.
- "secret" means it contains credentials, tokens, private keys or password exports.
  Always set action to "review" for those, never "archive".
- "stale" means the content is real but superseded or long abandoned.
- Set confidence below 0.5 when the content is too ambiguous to judge."#,
        path = job.path,
        class = job.class,
        size = job.size,
        owned = owned,
        repo_list = repo_list,
        body = truncate(&job.body, 12_000)
    )
}

fn coerce(v: &serde_json::Value) -> Interpretation {
    let s = |k: &str| -> String {
        v.get(k)
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    let arr = |k: &str| -> Vec<String> {
        match v.get(k) {
            Some(serde_json::Value::Array(a)) => a
                .iter()
                .filter_map(|x| x.as_str().map(|s| s.trim().to_lowercase()))
                .filter(|s| !s.is_empty())
                .take(8)
                .collect(),
            // Models sometimes emit a comma-separated string instead.
            Some(serde_json::Value::String(s)) => s
                .split(',')
                .map(|p| p.trim().to_lowercase())
                .filter(|p| !p.is_empty())
                .take(8)
                .collect(),
            _ => Vec::new(),
        }
    };
    let conf = v
        .get("confidence")
        .and_then(|x| x.as_f64().or_else(|| x.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0.0) as f32;

    let mut it = Interpretation {
        title: s("title"),
        kind: s("kind").to_lowercase(),
        purpose: s("purpose"),
        summary: s("summary"),
        topics: arr("topics"),
        entities: arr("entities"),
        related_repo: s("related_repo"),
        status: s("status").to_lowercase(),
        action: s("action").to_lowercase(),
        reason: s("reason"),
        confidence: conf.clamp(0.0, 1.0),
    };

    // A file holding credentials must never be auto-filed away.
    if it.kind == "secret" {
        it.action = "review".into();
    }
    if !matches!(
        it.status.as_str(),
        "current" | "stale" | "superseded" | "reference" | "junk"
    ) {
        it.status = "reference".into();
    }
    if !matches!(
        it.action.as_str(),
        "keep" | "file" | "archive" | "quarantine" | "review"
    ) {
        it.action = "review".into();
    }
    it
}

/// Files ready to be interpreted, newest first.
pub fn pending(conn: &Connection, limit: i64) -> Result<Vec<Job>> {
    let mut q = conn.prepare(
        "SELECT f.id, f.path, f.class, f.size,
                COALESCE(t.body, ''), r.name
           FROM files f
           LEFT JOIN file_text t ON t.file_id = f.id
           LEFT JOIN repos r     ON r.id = f.repo_id
          WHERE f.present = 1 AND f.stage = 2
          ORDER BY (f.repo_id IS NULL) DESC, f.mtime DESC
          LIMIT ?1",
    )?;
    let rows = q.query_map(params![limit], |r| {
        Ok(Job {
            id: r.get(0)?,
            path: r.get(1)?,
            class: r.get(2)?,
            size: r.get(3)?,
            body: r.get(4)?,
            repo: r.get(5)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

fn repo_names(conn: &Connection) -> Result<Vec<String>> {
    let mut q = conn.prepare(
        "SELECT DISTINCT name FROM repos WHERE kind = 'repo' ORDER BY last_commit DESC LIMIT 40",
    )?;
    let rows = q.query_map([], |r| r.get::<_, String>(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

async fn ask(
    client: &Ollama,
    model: &str,
    job: &Job,
    repos: &[String],
) -> Result<Interpretation> {
    let prompt = build_prompt(job, repos);
    let value = if job.class == "image" {
        let bytes = std::fs::read(&job.path)?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
        client
            .generate_json_image(model, Some(SYSTEM), &prompt, b64, 8192)
            .await?
    } else {
        client.generate_json(model, Some(SYSTEM), &prompt, 8192).await?
    };
    Ok(coerce(&value))
}

/// Interpret one batch, grouped by model so weights load once each.
pub async fn run(
    db_handle: &db::Db,
    cfg: &Config,
    client: &Ollama,
    limit: i64,
) -> Result<InterpretReport> {
    let mut report = InterpretReport::default();

    let (jobs, repos) = {
        let conn = db_handle.lock().unwrap();
        (pending(&conn, limit)?, repo_names(&conn)?)
    };
    if jobs.is_empty() {
        return Ok(report);
    }

    // Group by target model; drain one model fully before moving on.
    let mut buckets: BTreeMap<String, Vec<Job>> = BTreeMap::new();
    for job in jobs {
        let model = route(cfg, &job.class, job.size);
        buckets.entry(model).or_default().push(job);
    }

    for (model, group) in buckets {
        for job in group {
            // A file with no readable text cannot be judged on content.
            if job.body.trim().is_empty() && job.class != "image" {
                let it = Interpretation {
                    title: job
                        .path
                        .rsplit('/')
                        .next()
                        .unwrap_or_default()
                        .to_string(),
                    kind: "artifact".into(),
                    purpose: "No readable text could be extracted.".into(),
                    status: "reference".into(),
                    action: "review".into(),
                    reason: "empty or unreadable content".into(),
                    confidence: 0.2,
                    ..Default::default()
                };
                store(db_handle, job.id, &it, "none", false)?;
                report.done += 1;
                continue;
            }

            let mut used = model.clone();
            let mut escalated = false;
            let mut result = match ask(client, &model, &job, &repos).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("librarian: interpret {} failed on {model}: {e}", job.path);
                    report.failed += 1;
                    continue;
                }
            };

            if result.confidence < cfg.models.escalate_below {
                if let Some(bigger) = escalation_for(cfg, &job.class, &model) {
                    match ask(client, &bigger, &job, &repos).await {
                        Ok(better) => {
                            if better.confidence > result.confidence {
                                result = better;
                                used = bigger;
                                escalated = true;
                                report.escalated += 1;
                            }
                        }
                        Err(e) => {
                            eprintln!("librarian: escalation to {bigger} failed: {e}");
                        }
                    }
                }
            }

            store(db_handle, job.id, &result, &used, escalated)?;
            *report.by_model.entry(used).or_insert(0) += 1;
            report.done += 1;
        }
    }

    Ok(report)
}

fn store(
    db_handle: &db::Db,
    file_id: i64,
    it: &Interpretation,
    model: &str,
    escalated: bool,
) -> Result<()> {
    let conn = db_handle.lock().unwrap();
    conn.execute(
        "INSERT INTO interpretations
           (file_id,title,kind,purpose,summary,topics,entities,related_repo,
            status,action,reason,confidence,model,escalated,decided_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)
         ON CONFLICT(file_id) DO UPDATE SET
            title=excluded.title, kind=excluded.kind, purpose=excluded.purpose,
            summary=excluded.summary, topics=excluded.topics,
            entities=excluded.entities, related_repo=excluded.related_repo,
            status=excluded.status, action=excluded.action,
            reason=excluded.reason, confidence=excluded.confidence,
            model=excluded.model, escalated=excluded.escalated,
            decided_at=excluded.decided_at",
        params![
            file_id,
            it.title,
            it.kind,
            it.purpose,
            it.summary,
            serde_json::to_string(&it.topics)?,
            serde_json::to_string(&it.entities)?,
            it.related_repo,
            it.status,
            it.action,
            it.reason,
            it.confidence,
            model,
            i64::from(escalated),
            db::now(),
        ],
    )?;
    conn.execute(
        "UPDATE files SET stage = 3 WHERE id = ?1",
        params![file_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{coerce, escalation_for, route, truncate};
    use crate::config::Config;

    fn cfg() -> Config {
        Config::default()
    }

    #[test]
    fn routes_by_class() {
        let c = cfg();
        assert_eq!(route(&c, "code", 5000), c.models.code);
        assert_eq!(route(&c, "script", 5000), c.models.code);
        assert_eq!(route(&c, "image", 5000), c.models.vision);
        assert_eq!(route(&c, "doc", 50_000), c.models.docs);
        // tiny prose is not worth the bigger model
        assert_eq!(route(&c, "doc", 100), c.models.triage);
    }

    #[test]
    fn escalation_climbs_and_terminates() {
        let c = cfg();
        assert_eq!(
            escalation_for(&c, "doc", &c.models.triage),
            Some(c.models.docs.clone())
        );
        assert_eq!(
            escalation_for(&c, "code", &c.models.code),
            Some(c.models.escalate.clone())
        );
        assert_eq!(
            escalation_for(&c, "doc", &c.models.escalate),
            Some(c.models.escalate_max.clone())
        );
        assert_eq!(escalation_for(&c, "doc", &c.models.escalate_max), None);
    }

    #[test]
    fn image_escalates_to_vision_model() {
        let c = cfg();
        assert_eq!(
            escalation_for(&c, "image", &c.models.vision),
            Some(c.models.vision_escalate.clone())
        );
    }

    #[test]
    fn coerce_reads_a_well_formed_object() {
        let v = serde_json::json!({
            "title": "deploy script",
            "kind": "script",
            "purpose": "deploys the stack",
            "summary": "does things",
            "topics": ["deploy", "Docker"],
            "entities": ["jellyfin"],
            "related_repo": "nexus-os",
            "status": "current",
            "action": "keep",
            "reason": "referenced by the repo",
            "confidence": 0.91
        });
        let it = coerce(&v);
        assert_eq!(it.kind, "script");
        assert_eq!(it.topics, vec!["deploy", "docker"]);
        assert_eq!(it.action, "keep");
        assert!((it.confidence - 0.91).abs() < 1e-6);
    }

    #[test]
    fn coerce_accepts_comma_separated_topics() {
        let v = serde_json::json!({"topics": "alpha, Beta ,gamma"});
        assert_eq!(coerce(&v).topics, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn secrets_are_forced_to_review() {
        let v = serde_json::json!({
            "kind": "secret", "action": "archive", "status": "current", "confidence": 0.99
        });
        assert_eq!(coerce(&v).action, "review");
    }

    #[test]
    fn invalid_enums_fall_back_safely() {
        let v = serde_json::json!({"status": "banana", "action": "explode"});
        let it = coerce(&v);
        assert_eq!(it.status, "reference");
        assert_eq!(it.action, "review");
    }

    #[test]
    fn confidence_is_clamped_and_parses_strings() {
        assert_eq!(coerce(&serde_json::json!({"confidence": 5.0})).confidence, 1.0);
        assert_eq!(coerce(&serde_json::json!({"confidence": -1})).confidence, 0.0);
        assert!((coerce(&serde_json::json!({"confidence": "0.4"})).confidence - 0.4).abs() < 1e-6);
    }

    #[test]
    fn missing_fields_are_tolerated() {
        let it = coerce(&serde_json::json!({}));
        assert_eq!(it.title, "");
        assert_eq!(it.confidence, 0.0);
        assert!(it.topics.is_empty());
    }

    #[test]
    fn truncate_respects_char_boundaries() {
        let s = "aéé";
        assert!(s.starts_with(truncate(s, 2)));
    }
}
