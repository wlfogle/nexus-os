//! Code tier of repo currency: report-only detection, with relocation as the
//! single exception that may write to the working tree.
//!
//! Every finding here is a *signal*, not a verdict: environment-drift
//! markers, files nothing in the repo references by name, and content an LLM
//! judges to contradict the repo's own documentation. None of it edits code
//! content. The only mutation this module ever performs is
//! [`relocate`] / `run_code_relocation`, which moves a file byte-for-byte
//! (via `git mv` when possible) into a repo-local holding directory -- never
//! deleting, never editing.

use anyhow::{anyhow, Result};
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

const CONTRADICTS_DOCS_SYSTEM: &str = "\
You are a meticulous code reviewer cross-checking a repository's actual \
files against its own documentation. You only report a contradiction when \
you are shown concrete evidence for it in the file tree/summaries provided -- \
you never invent files or claims that are not shown. Reply with a single \
JSON object and nothing else.";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeFinding {
    pub file_path: String,
    /// One of "environment_drift" | "unreferenced" | "contradicts_docs".
    pub kind: String,
    pub description: String,
    pub suggested_relocation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSweepCandidate {
    pub repo_path: String,
    pub repo_name: String,
    pub findings: Vec<CodeFinding>,
}

/// Filenames that are expected to have no in-repo referrer -- entry points,
/// manifests, and platform-invoked files. Flagging these as "unreferenced"
/// would be pure noise, not a real drift signal.
const ENTRY_POINT_BASENAMES: &[&str] = &[
    "readme", "readme.md", "readme.rst", "license", "license.md", "changelog", "changelog.md",
    "contributing", "contributing.md", "architecture", "architecture.md",
    "cargo.toml", "cargo.lock", "package.json", "package-lock.json", "pnpm-lock.yaml",
    "yarn.lock", "go.mod", "go.sum", "pyproject.toml", "setup.py", "requirements.txt",
    "dockerfile", "makefile", "justfile", ".gitignore", ".gitattributes",
    "main.rs", "lib.rs", "mod.rs", "build.rs", "index.ts", "index.js", "index.tsx",
    "index.jsx", "__init__.py", "__main__.py", "app.py", "main.py", "main.go",
    "tauri.conf.json", "vite.config.ts", "vite.config.js", "tsconfig.json",
];

fn basename(rel: &str) -> String {
    rel.rsplit('/').next().unwrap_or(rel).to_string()
}

/// The repo-local convention to move superseded material into: an existing
/// `legacy/`/`archive/legacy/`/`archive/` directory if the repo already has
/// one, else `_deprecated/` at the repo root.
fn holding_directory(tree: &[String]) -> String {
    let has_prefix = |prefix: &str| tree.iter().any(|p| p.to_lowercase().starts_with(prefix));
    if has_prefix("archive/legacy/") {
        "archive/legacy".to_string()
    } else if has_prefix("legacy/") {
        "legacy".to_string()
    } else if has_prefix("archive/") {
        "archive".to_string()
    } else {
        "_deprecated".to_string()
    }
}

/// Only suggest a relocation for environment-drift when the file itself
/// looks wholly obsolete (its own name names the superseded environment),
/// not for a file that merely mentions one marker among otherwise-current
/// content -- relocating the latter on a single line of evidence would be
/// too aggressive for a report-only signal.
fn relocation_suggestion(file_path: &str, holding: &str) -> Option<String> {
    let lower = file_path.to_lowercase();
    if lower.contains("garuda") || lower.contains("calamares") {
        Some(format!("{holding}/{}", basename(file_path)))
    } else {
        None
    }
}

/// `git grep`'s answer to "does anything else in this repo mention this
/// file's name?", used as the unreferenced-file signal.
fn is_referenced(repo_path: &Path, rel_self: &str, needle: &str) -> bool {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(["grep", "-l", "-F", "-i", needle])
        .env("GIT_TERMINAL_PROMPT", "0")
        .output();
    let o = match out {
        Ok(o) => o,
        Err(_) => return true, // git grep unavailable: do not flag as unreferenced
    };
    match o.status.code() {
        Some(0) => {
            let hits: Vec<&str> = std::str::from_utf8(&o.stdout).unwrap_or("").lines().collect();
            hits.iter().any(|h| *h != rel_self)
        }
        Some(1) => false, // no tracked file contains this token at all
        _ => true,        // git grep errored for another reason: do not flag
    }
}

/// Tracked files nothing else in the repo references by name, excluding
/// well-known entry points, manifests, and platform-invoked directories
/// (`.github/`, `.gitlab*`) that are never referenced by another tracked
/// file even when they are perfectly current.
fn find_unreferenced(repo_path: &Path, tree: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for rel in tree {
        let base = basename(rel);
        let base_lower = base.to_lowercase();
        if ENTRY_POINT_BASENAMES.contains(&base_lower.as_str()) || base_lower.starts_with('.') {
            continue;
        }
        let lower_path = rel.to_lowercase();
        if lower_path.starts_with(".github/") || lower_path.starts_with(".gitlab") {
            continue;
        }
        let stem = Path::new(&base)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| base.clone());
        if stem.len() < 3 {
            continue; // too short/common a token for git grep to mean anything
        }
        if is_referenced(repo_path, rel, &stem) {
            continue;
        }
        out.push(rel.clone());
    }
    out
}

async fn contradicts_docs_findings(
    client: &Ollama,
    cfg: &Config,
    digest: &repo_digest::RepoDigest,
) -> Result<Vec<CodeFinding>> {
    let model = if repo_digest::is_large_or_complex(digest) {
        cfg.models.escalate.clone()
    } else {
        cfg.models.docs.clone()
    };
    let doc_excerpt: String = digest
        .doc_files
        .iter()
        .map(|d| format!("--- {} ---\n{}", d.rel_path, repo_digest::truncate_chars(&d.content, 4000)))
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        r#"REPO: {name}
CURRENT DOCUMENTATION:
{docs}

REPO FILE TREE BY DIRECTORY (with one-line summaries):
{digest}

RECENT COMMITS (newest first):
{commits}

List up to 5 tracked files (use the EXACT path as it appears in the file tree
above) whose content clearly contradicts a specific claim in the
documentation above (e.g. docs describe a component, tool, or behaviour that
the file's actual content shows has changed or been removed). Do not guess
about files not shown above.

Reply with a single JSON object:
{{"findings": [{{"file_path": "...", "description": "one sentence citing the contradiction"}}]}}
If nothing contradicts, reply {{"findings": []}}."#,
        name = digest.repo_name,
        docs = doc_excerpt,
        digest = digest.directory_digest,
        commits = digest.commit_log.join("\n"),
    );

    let value = client.generate_json(&model, Some(CONTRADICTS_DOCS_SYSTEM), &prompt, 8192).await?;
    let mut out = Vec::new();
    if let Some(arr) = value.get("findings").and_then(|v| v.as_array()) {
        for item in arr.iter().take(5) {
            let file_path = item.get("file_path").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
            let description = item.get("description").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
            if file_path.is_empty() || description.is_empty() {
                continue;
            }
            if !digest.file_tree.iter().any(|f| f == &file_path) {
                continue; // never trust a fabricated path
            }
            out.push(CodeFinding {
                file_path,
                kind: "contradicts_docs".into(),
                description,
                suggested_relocation: None,
            });
        }
    }
    Ok(out)
}

/// Scan every known repo for code-tier findings.
pub async fn list_candidates(dbh: &Db, cfg: &Config, client: &Ollama) -> Result<Vec<CodeSweepCandidate>> {
    let dbh2 = dbh.clone();
    let repos = tokio::task::spawn_blocking(move || -> Result<Vec<(String, String)>> {
        let conn = dbh2.lock().unwrap();
        repo_digest::known_repos(&conn)
    })
    .await??;

    let mut out = Vec::new();
    for (path, _name) in repos {
        let repo_path = PathBuf::from(&path);
        if !repo_path.is_dir() {
            continue;
        }

        let dbh3 = dbh.clone();
        let rp = repo_path.clone();
        let digest = match tokio::task::spawn_blocking(move || -> Result<repo_digest::RepoDigest> {
            let conn = dbh3.lock().unwrap();
            repo_digest::build(&conn, &rp)
        })
        .await
        {
            Ok(Ok(d)) => d,
            _ => continue,
        };
        if digest.file_tree.is_empty() {
            continue;
        }

        let mut findings = Vec::new();
        let holding = holding_directory(&digest.file_tree);

        // Signal 1: environment drift, evaluated over scripts/config/docs.
        let doc_text: String = digest.doc_files.iter().map(|d| d.content.as_str()).collect::<Vec<_>>().join("\n");
        let rp2 = repo_path.clone();
        let tree2 = digest.file_tree.clone();
        let name2 = digest.repo_name.clone();
        let drift_matches = tokio::task::spawn_blocking(move || {
            let env_files = repo_digest::scannable_environment_files(&rp2, &tree2);
            environment_drift::scan_repo(&name2, &doc_text, &env_files)
        })
        .await
        .unwrap_or_default();
        for d in drift_matches.into_iter().filter(|d| d.likely_drift) {
            findings.push(CodeFinding {
                suggested_relocation: relocation_suggestion(&d.file_path, &holding),
                file_path: d.file_path,
                kind: "environment_drift".into(),
                description: format!(
                    "line {}: references superseded tooling `{}` with no current-manager fallback nearby",
                    d.line_number, d.marker
                ),
            });
        }

        // Signal 2: files nothing else in the repo references by name.
        let rp3 = repo_path.clone();
        let tree3 = digest.file_tree.clone();
        let unreferenced = tokio::task::spawn_blocking(move || find_unreferenced(&rp3, &tree3))
            .await
            .unwrap_or_default();
        for f in unreferenced {
            findings.push(CodeFinding {
                suggested_relocation: Some(format!("{holding}/{}", basename(&f))),
                description: "no other tracked file in this repo references this file by name".into(),
                kind: "unreferenced".into(),
                file_path: f,
            });
        }

        // Signal 3: content that contradicts the repo's own docs, judged by
        // an LLM using the digest as evidence -- never a bare grep verdict.
        if !digest.doc_files.is_empty() {
            match contradicts_docs_findings(client, cfg, &digest).await {
                Ok(mut cf) => findings.append(&mut cf),
                Err(e) => eprintln!(
                    "librarian: code-sweep contradicts-docs check failed for {}: {e}",
                    digest.repo_name
                ),
            }
        }

        if findings.is_empty() {
            continue;
        }

        out.push(CodeSweepCandidate {
            repo_path: path,
            repo_name: digest.repo_name,
            findings,
        });
    }

    Ok(out)
}

/// A path is safe to use as a relocation source/destination when it is
/// relative and contains no `..` component -- it must resolve to somewhere
/// inside the repo it names.
fn safe_relative(rel: &str) -> Result<&Path> {
    let p = Path::new(rel);
    if p.is_absolute() {
        return Err(anyhow!("path must be relative to the repo root: {rel}"));
    }
    if p.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err(anyhow!("path must not contain '..': {rel}"));
    }
    Ok(p)
}

async fn journal_relocation(dbh: &Db, src_abs: &Path, dest_abs: &Path) -> Result<()> {
    let dbh2 = dbh.clone();
    let src = src_abs.to_string_lossy().to_string();
    let dest = dest_abs.to_string_lossy().to_string();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let conn = dbh2.lock().unwrap();
        let now = db::now();
        conn.execute(
            "INSERT INTO plans(created_at, note, status) VALUES (?1, ?2, 'applied')",
            params![now, format!("code_relocate: {src} -> {dest}")],
        )?;
        let plan_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO actions
               (plan_id,file_id,kind,src,dest,category,reason,confidence,state,applied_at)
             VALUES (?1,NULL,'code_relocate',?2,?3,'code_relocate','superseded code relocated by repo-currency code sweep',1.0,'applied',?4)",
            params![plan_id, src, dest, now],
        )?;
        let action_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO journal(action_id, op, payload, ts)
             VALUES (?1, 'moved', ?2, ?3)",
            params![
                action_id,
                serde_json::json!({ "from": src, "to": dest }).to_string(),
                now,
            ],
        )?;
        Ok(())
    })
    .await?
}

/// The one code-tier write: relocate a file within its repo, never touching
/// its bytes. Prefers `git mv` so history follows the file; falls back to a
/// plain rename for untracked files or repos without git.
pub async fn relocate(dbh: &Db, repo_path_str: &str, file_path: &str, destination: &str) -> Result<String> {
    let repo_path = PathBuf::from(repo_path_str);
    if !repo_path.is_dir() {
        return Err(anyhow!("repo path does not exist: {}", repo_path.display()));
    }
    let rel_src = safe_relative(file_path)?.to_path_buf();
    let rel_dest = safe_relative(destination)?.to_path_buf();

    let src_abs = repo_path.join(&rel_src);
    let dest_abs = repo_path.join(&rel_dest);
    if !src_abs.is_file() {
        return Err(anyhow!("source file does not exist: {}", src_abs.display()));
    }
    if dest_abs.exists() {
        return Err(anyhow!("destination already exists: {}", dest_abs.display()));
    }

    let repo_path2 = repo_path.clone();
    let src_abs2 = src_abs.clone();
    let dest_abs2 = dest_abs.clone();
    let how = tokio::task::spawn_blocking(move || -> Result<&'static str> {
        if let Some(parent) = dest_abs2.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if repo_path2.join(".git").exists() {
            let out = Command::new("git")
                .arg("-C")
                .arg(&repo_path2)
                .arg("mv")
                .arg(&src_abs2)
                .arg(&dest_abs2)
                .env("GIT_TERMINAL_PROMPT", "0")
                .output()?;
            if out.status.success() {
                return Ok("git mv");
            }
            // `git mv` refuses untracked files; a plain rename is still a
            // safe, content-preserving relocation for those.
            std::fs::rename(&src_abs2, &dest_abs2)?;
            Ok("renamed")
        } else {
            std::fs::rename(&src_abs2, &dest_abs2)?;
            Ok("renamed")
        }
    })
    .await??;

    journal_relocation(dbh, &src_abs, &dest_abs).await?;

    Ok(format!(
        "moved {} to {} ({how})",
        rel_src.display(),
        rel_dest.display()
    ))
}

// -------------------------------------------------------------- commands --

// Plain (non-async) command for the same reason as `list_docsync_candidates`
// in `docsync.rs`: the contract fixes this to a bare `Vec` return, which an
// async command taking `State<'_, ..>` is not allowed to do. The async work
// inside (the contradicts-docs LLM call) runs via `block_on` on Tauri's
// async runtime, same as any other blocking command.
#[tauri::command]
pub fn list_code_sweep_candidates(state: State<'_, Arc<AppState>>) -> Vec<CodeSweepCandidate> {
    let cfg = state.config();
    let client = state.client();
    let dbh = state.db.clone();
    tauri::async_runtime::block_on(async move {
        list_candidates(&dbh, &cfg, &client).await.unwrap_or_else(|e| {
            eprintln!("librarian: list_code_sweep_candidates failed: {e}");
            Vec::new()
        })
    })
}

#[tauri::command]
pub async fn run_code_relocation(
    state: State<'_, Arc<AppState>>,
    repo_path: String,
    file_path: String,
    destination: String,
) -> Result<String, String> {
    relocate(&state.db, &repo_path, &file_path, &destination)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{find_unreferenced, holding_directory, relocate, relocation_suggestion, safe_relative};
    use crate::db;
    use crate::repo_digest::test_support::{init_repo_with_commit, tmp_repo_dir};

    #[test]
    fn holding_directory_reuses_an_existing_convention() {
        let tree = vec!["archive/legacy/old.sh".to_string(), "src/main.rs".to_string()];
        assert_eq!(holding_directory(&tree), "archive/legacy");

        let tree = vec!["legacy/old.sh".to_string()];
        assert_eq!(holding_directory(&tree), "legacy");

        let tree = vec!["archive/notes.md".to_string()];
        assert_eq!(holding_directory(&tree), "archive");
    }

    #[test]
    fn holding_directory_defaults_to_deprecated_when_no_convention_exists() {
        let tree = vec!["src/main.rs".to_string(), "README.md".to_string()];
        assert_eq!(holding_directory(&tree), "_deprecated");
    }

    #[test]
    fn relocation_is_only_suggested_for_files_named_after_the_dead_environment() {
        assert_eq!(
            relocation_suggestion("scripts/garuda-setup.sh", "_deprecated"),
            Some("_deprecated/garuda-setup.sh".to_string())
        );
        assert_eq!(relocation_suggestion("scripts/install.sh", "_deprecated"), None);
    }

    #[test]
    fn safe_relative_rejects_absolute_and_traversal_paths() {
        assert!(safe_relative("src/main.rs").is_ok());
        assert!(safe_relative("/etc/passwd").is_err());
        assert!(safe_relative("../outside").is_err());
        assert!(safe_relative("a/../../outside").is_err());
    }

    #[test]
    fn unreferenced_detects_a_file_nothing_else_mentions() {
        let dir = tmp_repo_dir("unref");
        init_repo_with_commit(
            &dir,
            &[
                ("src/main.rs", "mod used_module;\nfn main() { used_module::go(); }"),
                ("src/used_module.rs", "pub fn go() {}"),
                ("src/orphaned_leftover.rs", "pub fn dead() {}"),
                ("README.md", "# proj"),
            ],
        );
        let tree = crate::repo_digest::file_tree(&dir).unwrap();
        let unreferenced = find_unreferenced(&dir, &tree);
        assert!(unreferenced.contains(&"src/orphaned_leftover.rs".to_string()));
        assert!(!unreferenced.contains(&"src/used_module.rs".to_string()));
        assert!(!unreferenced.contains(&"README.md".to_string()));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn relocate_moves_the_file_and_journals_it() {
        let dir = tmp_repo_dir("relocate");
        init_repo_with_commit(
            &dir,
            &[("scripts/garuda-old.sh", "echo old"), ("README.md", "# proj")],
        );
        let dbh = db::open(&dir.join("catalog-test.db")).unwrap();

        let msg = relocate(
            &dbh,
            &dir.to_string_lossy(),
            "scripts/garuda-old.sh",
            "_deprecated/garuda-old.sh",
        )
        .await
        .unwrap();
        assert!(msg.contains("moved"));

        assert!(!dir.join("scripts/garuda-old.sh").exists());
        assert!(dir.join("_deprecated/garuda-old.sh").exists());
        assert_eq!(
            std::fs::read_to_string(dir.join("_deprecated/garuda-old.sh")).unwrap(),
            "echo old"
        );

        let conn = dbh.lock().unwrap();
        let kind: String = conn
            .query_row("SELECT kind FROM actions ORDER BY id DESC LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(kind, "code_relocate");
        let op: String = conn
            .query_row("SELECT op FROM journal ORDER BY id DESC LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(op, "moved");
        drop(conn);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn relocate_refuses_to_overwrite_an_existing_destination() {
        let dir = tmp_repo_dir("relocate-clobber");
        init_repo_with_commit(&dir, &[("a.txt", "one"), ("b.txt", "two")]);
        let dbh = db::open(&dir.join("catalog-test.db")).unwrap();

        let result = relocate(&dbh, &dir.to_string_lossy(), "a.txt", "b.txt").await;
        assert!(result.is_err());
        assert_eq!(std::fs::read_to_string(dir.join("a.txt")).unwrap(), "one");
        assert_eq!(std::fs::read_to_string(dir.join("b.txt")).unwrap(), "two");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn relocate_rejects_path_traversal() {
        let dir = tmp_repo_dir("relocate-traversal");
        init_repo_with_commit(&dir, &[("a.txt", "one")]);
        let dbh = db::open(&dir.join("catalog-test.db")).unwrap();

        let result = relocate(&dbh, &dir.to_string_lossy(), "a.txt", "../escaped.txt").await;
        assert!(result.is_err());

        std::fs::remove_dir_all(&dir).ok();
    }
}
