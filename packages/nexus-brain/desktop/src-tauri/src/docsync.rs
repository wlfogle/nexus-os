//! Docs tier of repo currency: keep a repo's own documentation honest against
//! its real, current file tree and history.
//!
//! Writes go straight to the working-tree file, exactly like a manual edit --
//! never staged, never committed, never pushed. Every write is journalled
//! through the same `actions`/`journal` tables the loose-file mover uses, under
//! a new `doc_sync` kind, so it is visible in History and carries a full
//! before/after payload for a future revert.

use anyhow::{anyhow, Context, Result};
use rusqlite::params;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tauri::State;

use crate::config::Config;
use crate::db::{self, Db};
use crate::engine::AppState;
use crate::environment_drift;
use crate::ollama::Ollama;
use crate::repo_digest;

const DOC_SYSTEM: &str = "\
You are a meticulous technical writer maintaining documentation for a real, \
evolving codebase. You are shown the current content of one documentation \
file plus a digest of the repository's actual current file tree, per-file \
summaries, recent commits, and specific evidence of environment/tooling \
drift. Reply with a single JSON object and nothing else.";

const NEW_DOC_SYSTEM: &str = "\
You are a meticulous technical writer deciding whether a repository has a \
major component with no documentation coverage at all. You are conservative: \
you only propose a new file for something substantial, never for a single \
minor file. Reply with a single JSON object and nothing else.";

/// Commits since a doc file last changed before it is considered a candidate
/// purely on staleness grounds. High enough that ordinary iteration does not
/// flood the candidate list.
const STALE_DOC_COMMIT_THRESHOLD: usize = 15;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocSyncCandidate {
    pub repo_path: String,
    pub repo_name: String,
    pub doc_files: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocSyncResult {
    pub repo_path: String,
    pub updated_files: Vec<String>,
    pub diff_summary: String,
}

struct DocProposal {
    changed: bool,
    content: String,
}

fn coerce_doc_proposal(v: &serde_json::Value) -> DocProposal {
    DocProposal {
        changed: v.get("changed").and_then(|x| x.as_bool()).unwrap_or(false),
        content: v
            .get("updated_content")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
    }
}

fn commits_since_doc(repo_path: &Path, rel_path: &str) -> Option<usize> {
    let last = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["log", "-1", "--format=%H", "--", rel_path])
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .ok()?;
    if !last.status.success() {
        return None;
    }
    let hash = String::from_utf8_lossy(&last.stdout).trim().to_string();
    if hash.is_empty() {
        return None;
    }
    let count = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["rev-list", "--count", &format!("{hash}..HEAD")])
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .ok()?;
    if !count.status.success() {
        return None;
    }
    String::from_utf8_lossy(&count.stdout).trim().parse::<usize>().ok()
}

/// Repos with documentation-tier findings worth a `run_docsync` pass.
///
/// A repo is a candidate when any of: it has tracked files but zero
/// documentation at all; a doc file contains likely environment-drift
/// evidence; or a doc file has not changed across a burst of unrelated
/// commit activity.
pub fn list_candidates(conn: &rusqlite::Connection) -> Result<Vec<DocSyncCandidate>> {
    let repos = repo_digest::known_repos(conn)?;
    let mut out = Vec::new();

    for (path, name) in repos {
        let repo_path = Path::new(&path);
        if !repo_path.is_dir() {
            continue;
        }
        let tree = match repo_digest::file_tree(repo_path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if tree.is_empty() {
            continue;
        }
        let docs = repo_digest::doc_files(repo_path, &tree);
        let doc_rel_paths: Vec<String> = docs.iter().map(|d| d.rel_path.clone()).collect();

        let mut reasons: Vec<String> = Vec::new();

        if docs.is_empty() {
            reasons.push(format!(
                "no documentation files found across {} tracked files",
                tree.len()
            ));
        }

        let doc_text: String = docs.iter().map(|d| d.content.as_str()).collect::<Vec<_>>().join("\n");
        let env_files = repo_digest::scannable_environment_files(repo_path, &tree);
        let drift_hits = environment_drift::scan_repo(&name, &doc_text, &env_files)
            .into_iter()
            .filter(|d| d.likely_drift)
            .count();
        if drift_hits > 0 {
            reasons.push(format!(
                "{drift_hits} possible superseded-environment reference(s) (pacman/AUR/Garuda-era tooling)"
            ));
        }

        for doc in &docs {
            if let Some(n) = commits_since_doc(repo_path, &doc.rel_path) {
                if n >= STALE_DOC_COMMIT_THRESHOLD {
                    reasons.push(format!(
                        "{} has not changed across the last {n} commits touching this repo",
                        doc.rel_path
                    ));
                }
            }
        }

        if reasons.is_empty() {
            continue;
        }

        out.push(DocSyncCandidate {
            repo_path: path,
            repo_name: name,
            doc_files: doc_rel_paths,
            reason: reasons.join("; "),
        });
    }

    Ok(out)
}

fn build_doc_prompt(digest: &repo_digest::RepoDigest, doc: &repo_digest::DocFile, drift: &[String]) -> String {
    format!(
        r#"REPO: {name}
DOC FILE: {rel}

CURRENT CONTENT OF THIS DOC FILE:
{content}

REPO FILE TREE BY DIRECTORY (with one-line summaries):
{digest}

RECENT COMMITS (newest first):
{commits}

ENVIRONMENT-DRIFT EVIDENCE (superseded tooling references found in this repo, may be empty):
{drift}

Update this documentation file to be strictly accurate against the repo
digest above:
- Preserve sections that are still correct.
- Correct stale claims, but ONLY when the ENVIRONMENT-DRIFT EVIDENCE above
  actually names that exact reference. If a command or tool is not listed as
  drift evidence, leave it exactly as written even if a more common or more
  generic alternative exists -- e.g. do not replace `nala` with `apt` just
  because `apt` is more widely known. Both are legitimate on Debian/Ubuntu
  systems; swapping a correct, intentional tool choice for a generic one is
  itself a regression, not a fix. This applies universally, not just to this
  one repo or machine: never assume a single package manager exists on the
  reader's system. If you are documenting or writing install instructions
  and are unsure which is available, prefer a detect-then-fallback form (for
  example `command -v nala >/dev/null 2>&1 && sudo nala install <pkg> ||
  sudo apt install <pkg>`) over hard-coding one tool.
- Add brief sections for major components shown in the digest that this doc
  does not mention at all.
- If you are not confident a claim is still true, mark it with a "⚠" note
  rather than inventing detail or silently deleting it.
- Do not fabricate file paths, commands, or components not shown above.
- Never claim the document "previously" said something -- you are only shown
  its current content, not its history. Describe what you changed and why
  using only the evidence above; do not invent a backstory for the change.

Reply with a single JSON object:
{{"changed": true|false, "updated_content": "the full updated file content, markdown, only if changed is true"}}
Set "changed" to false and omit "updated_content" if the file needs no changes."#,
        name = digest.repo_name,
        rel = doc.rel_path,
        content = doc.content,
        digest = digest.directory_digest,
        commits = digest.commit_log.join("\n"),
        drift = if drift.is_empty() { "(none found)".to_string() } else { drift.join("\n") },
    )
}

async fn ask_for_doc_update(client: &Ollama, model: &str, prompt: &str) -> Result<DocProposal> {
    let value = client.generate_json(model, Some(DOC_SYSTEM), prompt, 8192).await?;
    Ok(coerce_doc_proposal(&value))
}

async fn maybe_propose_new_doc(
    client: &Ollama,
    model: &str,
    digest: &repo_digest::RepoDigest,
    drift: &[String],
) -> Result<Option<(String, String)>> {
    let existing_docs: Vec<&str> = digest.doc_files.iter().map(|d| d.rel_path.as_str()).collect();
    let prompt = format!(
        r#"REPO: {name}
EXISTING DOCUMENTATION FILES: {existing:?}

REPO FILE TREE BY DIRECTORY (with one-line summaries):
{digest}

RECENT COMMITS (newest first):
{commits}

ENVIRONMENT-DRIFT EVIDENCE (may be empty):
{drift}

Is there a major component (a top-level directory or subsystem) with no
documentation coverage at all among the existing documentation files listed
above? Only propose a new file for a genuinely undocumented MAJOR component,
never for a single minor file.

Reply with a single JSON object:
{{"needed": true|false, "path": "docs/<name>.md, relative to the repo root, only if needed", "content": "full markdown content for the new file, only if needed"}}"#,
        name = digest.repo_name,
        existing = existing_docs,
        digest = digest.directory_digest,
        commits = digest.commit_log.join("\n"),
        drift = if drift.is_empty() { "(none found)".to_string() } else { drift.join("\n") },
    );

    let value = client.generate_json(model, Some(NEW_DOC_SYSTEM), &prompt, 8192).await?;
    let needed = value.get("needed").and_then(|x| x.as_bool()).unwrap_or(false);
    if !needed {
        return Ok(None);
    }
    let path = value
        .get("path")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .trim_start_matches('/')
        .to_string();
    let content = value.get("content").and_then(|x| x.as_str()).unwrap_or("").to_string();
    if path.is_empty() || path.contains("..") || content.trim().is_empty() {
        return Ok(None);
    }
    if digest.file_tree.iter().any(|f| f == &path) {
        return Ok(None); // never silently overwrite an existing tracked file
    }
    Ok(Some((path, content)))
}

/// Journal a doc-content write under the `doc_sync` kind, storing the full
/// before/after content so a future revert can restore it exactly.
async fn journal_doc_sync(dbh: &Db, repo_path: &Path, rel_path: &str, before: &str, after: &str) -> Result<()> {
    let dbh2 = dbh.clone();
    let abs = repo_path.join(rel_path).to_string_lossy().to_string();
    let repo_display = repo_path.display().to_string();
    let before = before.to_string();
    let after = after.to_string();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = dbh2.lock().unwrap();
        let now = db::now();
        conn.execute(
            "INSERT INTO plans(created_at, note, status) VALUES (?1, ?2, 'applied')",
            params![now, format!("doc_sync: {repo_display}")],
        )?;
        let plan_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO actions
               (plan_id,file_id,kind,src,dest,category,reason,confidence,state,applied_at)
             VALUES (?1,NULL,'doc_sync',?2,?2,'doc_sync','content refreshed by repo-currency doc sync',1.0,'applied',?3)",
            params![plan_id, abs, now],
        )?;
        let action_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO journal(action_id, op, payload, ts)
             VALUES (?1, 'doc_content_update', ?2, ?3)",
            params![
                action_id,
                serde_json::json!({ "path": abs, "before": before, "after": after }).to_string(),
                now,
            ],
        )?;
        Ok(())
    })
    .await?
}

/// Revert a single `doc_sync` journal entry, restoring the file to its
/// pre-sync content. Exposed for a future History "revert" action; not wired
/// to a Tauri command by this change since the command surface for repo
/// currency is fixed to the four commands in the contract.
#[allow(dead_code)]
pub fn undo_doc_sync(conn: &rusqlite::Connection, action_id: i64) -> Result<()> {
    let payload: String = conn.query_row(
        "SELECT payload FROM journal WHERE action_id = ?1 AND op = 'doc_content_update'",
        params![action_id],
        |r| r.get(0),
    )?;
    let v: serde_json::Value = serde_json::from_str(&payload)?;
    let path = v["path"].as_str().ok_or_else(|| anyhow!("journal payload missing path"))?;
    let before = v["before"].as_str().unwrap_or("");
    std::fs::write(path, before)?;
    conn.execute(
        "UPDATE journal SET undone = 1 WHERE action_id = ?1 AND op = 'doc_content_update'",
        params![action_id],
    )?;
    conn.execute(
        "UPDATE actions SET state = 'pending' WHERE id = ?1",
        params![action_id],
    )?;
    Ok(())
}

/// Run the docs tier against one repository, writing accurate content
/// straight into the working tree.
pub async fn run(dbh: &Db, cfg: &Config, client: &Ollama, repo_path_str: &str) -> Result<DocSyncResult> {
    let repo_path = PathBuf::from(repo_path_str);
    if !repo_path.join(".git").exists() {
        return Err(anyhow!("not a git repository: {}", repo_path.display()));
    }

    let dbh2 = dbh.clone();
    let rp = repo_path.clone();
    let digest = tokio::task::spawn_blocking(move || -> Result<repo_digest::RepoDigest> {
        let conn = dbh2.lock().unwrap();
        repo_digest::build(&conn, &rp)
    })
    .await??;

    let doc_text: String = digest.doc_files.iter().map(|d| d.content.as_str()).collect::<Vec<_>>().join("\n");
    let rp2 = repo_path.clone();
    let tree2 = digest.file_tree.clone();
    let name2 = digest.repo_name.clone();
    let drift_evidence: Vec<String> = tokio::task::spawn_blocking(move || {
        let env_files = repo_digest::scannable_environment_files(&rp2, &tree2);
        environment_drift::scan_repo(&name2, &doc_text, &env_files)
    })
    .await
    .unwrap_or_default()
    .into_iter()
    .filter(|d| d.likely_drift)
    .map(|d| format!("{}:{} `{}`", d.file_path, d.line_number, d.marker))
    .take(30)
    .collect();

    let escalate = repo_digest::is_large_or_complex(&digest);
    let model = if escalate { cfg.models.escalate.clone() } else { cfg.models.docs.clone() };

    let mut updated_files = Vec::new();
    let mut diff_lines = Vec::new();

    for doc in &digest.doc_files {
        let prompt = build_doc_prompt(&digest, doc, &drift_evidence);
        let proposal = match ask_for_doc_update(client, &model, &prompt).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("librarian: docsync {} failed on {model}: {e}", doc.rel_path);
                if model == cfg.models.escalate {
                    continue;
                }
                match ask_for_doc_update(client, &cfg.models.escalate, &prompt).await {
                    Ok(p) => p,
                    Err(e2) => {
                        eprintln!("librarian: docsync escalation also failed for {}: {e2}", doc.rel_path);
                        continue;
                    }
                }
            }
        };

        if !proposal.changed || proposal.content.trim().is_empty() || proposal.content == doc.content {
            continue;
        }

        std::fs::write(&doc.abs_path, &proposal.content)
            .with_context(|| format!("writing {}", doc.abs_path.display()))?;
        journal_doc_sync(dbh, &repo_path, &doc.rel_path, &doc.content, &proposal.content).await?;
        diff_lines.push(format!(
            "{}: {} -> {} bytes",
            doc.rel_path,
            doc.content.len(),
            proposal.content.len()
        ));
        updated_files.push(doc.rel_path.clone());
    }

    match maybe_propose_new_doc(client, &model, &digest, &drift_evidence).await {
        Ok(Some((rel, content))) => {
            let abs = repo_path.join(&rel);
            if let Some(parent) = abs.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&abs, &content)?;
            journal_doc_sync(dbh, &repo_path, &rel, "", &content).await?;
            diff_lines.push(format!("{rel}: new file ({} bytes)", content.len()));
            updated_files.push(rel);
        }
        Ok(None) => {}
        Err(e) => eprintln!("librarian: docsync new-doc proposal failed for {}: {e}", digest.repo_name),
    }

    let diff_summary = if diff_lines.is_empty() {
        "no changes needed".to_string()
    } else {
        diff_lines.join("\n")
    };

    Ok(DocSyncResult {
        repo_path: repo_path_str.to_string(),
        updated_files,
        diff_summary,
    })
}

// -------------------------------------------------------------- commands --

// Plain (non-async) command: async Tauri commands that take a reference
// (`State<'_, ..>`) must return `Result`, but the repo-currency contract
// fixes this command's return type to a bare `Vec`. A sync command has no
// such restriction and still runs off the webview's event loop, so the
// blocking git/database work below never freezes the window.
#[tauri::command]
pub fn list_docsync_candidates(state: State<'_, Arc<AppState>>) -> Vec<DocSyncCandidate> {
    let conn = state.db.lock().unwrap();
    list_candidates(&conn).unwrap_or_else(|e| {
        eprintln!("librarian: list_docsync_candidates failed: {e}");
        Vec::new()
    })
}

#[tauri::command]
pub async fn run_docsync(state: State<'_, Arc<AppState>>, repo_path: String) -> Result<DocSyncResult, String> {
    let cfg = state.config();
    let client = state.client();
    let dbh = state.db.clone();
    run(&dbh, &cfg, &client, &repo_path).await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{coerce_doc_proposal, commits_since_doc};
    use crate::repo_digest::test_support::{init_repo_with_commit, tmp_repo_dir};

    #[test]
    fn coerce_reads_a_changed_proposal() {
        let v = serde_json::json!({"changed": true, "updated_content": "# New\n"});
        let p = coerce_doc_proposal(&v);
        assert!(p.changed);
        assert_eq!(p.content, "# New\n");
    }

    #[test]
    fn coerce_defaults_to_unchanged_when_fields_are_missing() {
        let p = coerce_doc_proposal(&serde_json::json!({}));
        assert!(!p.changed);
        assert_eq!(p.content, "");
    }

    #[test]
    fn commits_since_doc_counts_commits_after_the_docs_last_touch() {
        let dir = tmp_repo_dir("commits-since");
        init_repo_with_commit(&dir, &[("README.md", "# A"), ("src/a.rs", "fn a() {}")]);

        // Two further commits that never touch README.md.
        for i in 0..2 {
            std::fs::write(dir.join(format!("src/b{i}.rs")), "fn b() {}").unwrap();
            let out = std::process::Command::new("git")
                .arg("-C")
                .arg(&dir)
                .args(["add", "-A"])
                .output()
                .unwrap();
            assert!(out.status.success());
            let out = std::process::Command::new("git")
                .arg("-C")
                .arg(&dir)
                .args(["commit", "-q", "-m", &format!("commit {i}")])
                .output()
                .unwrap();
            assert!(out.status.success());
        }

        let n = commits_since_doc(&dir, "README.md").unwrap();
        assert_eq!(n, 2);

        std::fs::remove_dir_all(&dir).ok();
    }
}
