//! Whole-repo digest builder: the shared foundation for the doc-sync and
//! code-sweep tiers of repo currency.
//!
//! A digest is built from, in order:
//!   * `git ls-files`             -- the tracked file tree (respects `.gitignore`)
//!   * per-file one-line summary  -- `interpretations.summary` for files
//!     Librarian has already read; a zero-cost local extraction otherwise
//!     (first heading / doc-comment / function-or-type signature)
//!   * `git log --oneline -N`     -- recent history, for "what changed lately"
//!   * doc file content           -- README*, CONTRIBUTING*, CHANGELOG*,
//!     ARCHITECTURE*, any `*.md`/`*.rst` under `docs/` or the repo root
//!
//! Large repos are summarised per top-level directory first and only
//! collapsed to a per-directory file count once the combined text would be
//! too large for a single prompt -- see `build_directory_digest`.

use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Cap on how many characters of hierarchical digest text go into a single
/// LLM prompt. Past this, per-directory blocks collapse to a file count plus
/// a handful of representative summaries instead of a full listing.
pub const MAX_DIGEST_CHARS: usize = 24_000;

/// Cap on how much of a single doc file's content is embedded verbatim.
pub const MAX_DOC_CHARS: usize = 16_000;

/// Above this many tracked files, a repo is treated as large/complex enough
/// to escalate straight to the bigger model rather than the default one.
pub const LARGE_REPO_FILE_THRESHOLD: usize = 300;

#[derive(Debug, Clone)]
pub struct DocFile {
    /// Path relative to the repo root, forward-slash separated.
    pub rel_path: String,
    pub abs_path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct RepoDigest {
    pub repo_path: PathBuf,
    pub repo_name: String,
    /// Every path `git ls-files` reports, relative to the repo root.
    pub file_tree: Vec<String>,
    /// `git log --oneline -20`, newest first.
    pub commit_log: Vec<String>,
    pub doc_files: Vec<DocFile>,
    /// Hierarchical, size-bounded text describing every tracked file grouped
    /// by top-level directory -- the shared evidence base for both the
    /// docsync and code-sweep prompts.
    pub directory_digest: String,
}

/// True when a repo's tracked-file count or digest size warrants the bigger
/// model, mirroring `interpret.rs`'s escalation shape.
pub fn is_large_or_complex(digest: &RepoDigest) -> bool {
    digest.file_tree.len() > LARGE_REPO_FILE_THRESHOLD
        || digest.directory_digest.len() >= MAX_DIGEST_CHARS
}

fn git_output(repo_path: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .with_context(|| format!("running git {args:?} in {}", repo_path.display()))?;
    if !out.status.success() {
        return Err(anyhow!(
            "git {args:?} failed in {}: {}",
            repo_path.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn git_lines(repo_path: &Path, args: &[&str]) -> Result<Vec<String>> {
    Ok(git_output(repo_path, args)?
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect())
}

/// Every path `git ls-files` reports, relative to the repo root.
pub fn file_tree(repo_path: &Path) -> Result<Vec<String>> {
    git_lines(repo_path, &["ls-files"])
}

/// Recent history, newest first. Repos with zero commits yield an empty list
/// rather than an error, since a fresh checkout is a normal state to digest.
pub fn commit_log(repo_path: &Path, n: usize) -> Vec<String> {
    git_lines(repo_path, &["log", "--oneline", &format!("-{n}")]).unwrap_or_default()
}

pub(crate) fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max).collect()
}

/// True when `rel_path` (repo-relative, forward-slash separated) is
/// doc-like material: a README/CONTRIBUTING/CHANGELOG/ARCHITECTURE file
/// wherever it lives, or any markdown/reStructuredText at the repo root or
/// under a top-level `docs/`/`doc/` directory.
pub fn is_doc_file(rel_path: &str) -> bool {
    let lower = rel_path.to_lowercase();
    let file_name = lower.rsplit('/').next().unwrap_or(&lower);

    const NAMED_PREFIXES: &[&str] = &["readme", "contributing", "changelog", "architecture"];
    if NAMED_PREFIXES.iter().any(|p| file_name.starts_with(p)) {
        return true;
    }

    let top = lower.split('/').next().unwrap_or("");
    let under_docs = top == "docs" || top == "doc";
    let at_root = !lower.contains('/');
    (under_docs || at_root) && (file_name.ends_with(".md") || file_name.ends_with(".rst"))
}

/// Read every doc-like tracked file's current working-tree content.
///
/// Unreadable files (binary, permissions, vanished mid-scan) are skipped
/// rather than failing the whole digest -- a single bad file must not block
/// every other doc from being read.
pub fn doc_files(repo_path: &Path, tree: &[String]) -> Vec<DocFile> {
    let mut out = Vec::new();
    for rel in tree {
        if !is_doc_file(rel) {
            continue;
        }
        let abs = repo_path.join(rel);
        let content = match std::fs::read_to_string(&abs) {
            Ok(c) => c,
            Err(_) => continue,
        };
        out.push(DocFile {
            rel_path: rel.clone(),
            abs_path: abs,
            content: truncate_chars(&content, MAX_DOC_CHARS),
        });
    }
    out
}

/// Extensions and file names worth scanning for environment-drift markers:
/// scripts, packaging/config, and CI files, plus doc files (which routinely
/// contain install instructions). Deliberately narrow -- source-code bodies
/// rarely name a package manager, and scanning everything would be both slow
/// and noisy.
const ENV_SCAN_EXTENSIONS: &[&str] = &[
    "sh", "bash", "zsh", "yml", "yaml", "toml", "cfg", "conf", "service", "desktop", "py",
];
const ENV_SCAN_NAMES: &[&str] = &["dockerfile", "makefile", "install.sh", "setup.sh"];

/// Files worth checking for superseded-environment references: doc files
/// plus scripts/packaging/CI files, capped per-file to avoid reading huge
/// generated artefacts that happen to match an extension.
pub fn scannable_environment_files(repo_path: &Path, tree: &[String]) -> Vec<(String, String)> {
    const MAX_FILE_BYTES: u64 = 1 << 20; // 1 MiB
    let mut out = Vec::new();
    for rel in tree {
        let lower = rel.to_lowercase();
        let file_name = lower.rsplit('/').next().unwrap_or(&lower);
        let ext = file_name.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
        let matches = is_doc_file(rel)
            || ENV_SCAN_EXTENSIONS.contains(&ext)
            || ENV_SCAN_NAMES.iter().any(|n| file_name == *n || file_name.starts_with(n));
        if !matches {
            continue;
        }
        let abs = repo_path.join(rel);
        let Ok(md) = std::fs::metadata(&abs) else { continue };
        if md.len() > MAX_FILE_BYTES {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&abs) {
            out.push((rel.clone(), content));
        }
    }
    out
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().to_string()
}

fn is_signature_line(t: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "pub fn ", "pub async fn ", "fn ", "async fn ", "def ", "function ", "class ",
        "struct ", "pub struct ", "impl ", "interface ", "type ", "export function ",
        "export const ", "export class ", "export default ",
    ];
    KEYWORDS.iter().any(|k| t.starts_with(k))
}

/// A summary with no model call: the first markdown heading, doc-comment, or
/// function/type signature found in the first ~60 lines. Used only for files
/// Librarian has not yet interpreted, so every tracked file still gets a
/// one-line description in the digest.
fn lightweight_summary(path: &Path) -> String {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    for line in content.lines().take(60) {
        let t = line.trim();
        if t.is_empty() || t.starts_with("#!") {
            continue;
        }
        if let Some(h) = t.strip_prefix("# ") {
            if !h.trim().is_empty() {
                return h.trim().to_string();
            }
        }
        if let Some(h) = t.strip_prefix("//!").or_else(|| t.strip_prefix("///")) {
            let h = h.trim();
            if !h.is_empty() {
                return h.to_string();
            }
            continue;
        }
        if t.starts_with("\"\"\"") || t.starts_with("'''") {
            let h = t.trim_start_matches(['"', '\'']).trim();
            if !h.is_empty() {
                return h.chars().take(120).collect();
            }
            continue;
        }
        if is_signature_line(t) {
            return t.chars().take(120).collect();
        }
    }
    String::new()
}

/// One-line summary per tracked file: `interpretations.summary` (falling
/// back to `purpose` then `title`) where a row exists for that path, else a
/// zero-cost local extraction.
fn file_summaries(conn: &Connection, repo_path: &Path, tree: &[String]) -> Result<Vec<(String, String)>> {
    let mut q = conn.prepare(
        "SELECT COALESCE(NULLIF(i.summary, ''), NULLIF(i.purpose, ''), i.title)
           FROM files f JOIN interpretations i ON i.file_id = f.id
          WHERE f.path = ?1",
    )?;
    let mut out = Vec::with_capacity(tree.len());
    for rel in tree {
        let abs = repo_path.join(rel).to_string_lossy().to_string();
        let stored: Option<String> = q.query_row(rusqlite::params![abs], |r| r.get(0)).ok();
        let line = match stored {
            Some(s) if !s.trim().is_empty() => first_line(&s),
            _ => lightweight_summary(&repo_path.join(rel)),
        };
        out.push((rel.clone(), line));
    }
    Ok(out)
}

fn top_level_dir(rel: &str) -> &str {
    match rel.split_once('/') {
        Some((d, _)) => d,
        None => "",
    }
}

/// Hierarchical, size-bounded digest text: per top-level directory, a bullet
/// list of `path: summary`. Repos too large for the full listing collapse
/// each directory to a file count plus a handful of representative entries
/// instead of truncating the text mid-line.
fn build_directory_digest(summaries: &[(String, String)]) -> String {
    let mut groups: BTreeMap<&str, Vec<&(String, String)>> = BTreeMap::new();
    for pair in summaries {
        groups.entry(top_level_dir(&pair.0)).or_default().push(pair);
    }

    let mut full = String::new();
    for (dir, files) in &groups {
        let label = if dir.is_empty() { "(repo root)" } else { dir };
        full.push_str(&format!("## {label}\n"));
        for (path, summary) in files.iter() {
            if summary.is_empty() {
                full.push_str(&format!("- {path}\n"));
            } else {
                full.push_str(&format!("- {path}: {summary}\n"));
            }
        }
        full.push('\n');
    }

    if full.chars().count() <= MAX_DIGEST_CHARS {
        return full;
    }

    let mut collapsed = String::new();
    for (dir, files) in &groups {
        let label = if dir.is_empty() { "(repo root)" } else { dir };
        collapsed.push_str(&format!("## {label} ({} files)\n", files.len()));
        for (path, summary) in files.iter().take(8) {
            if summary.is_empty() {
                collapsed.push_str(&format!("- {path}\n"));
            } else {
                collapsed.push_str(&format!("- {path}: {summary}\n"));
            }
        }
        if files.len() > 8 {
            collapsed.push_str(&format!("- ... and {} more\n", files.len() - 8));
        }
        collapsed.push('\n');
    }
    truncate_chars(&collapsed, MAX_DIGEST_CHARS)
}

/// Build a full digest for one repository.
pub fn build(conn: &Connection, repo_path: &Path) -> Result<RepoDigest> {
    if !repo_path.join(".git").exists() {
        return Err(anyhow!("not a git repository: {}", repo_path.display()));
    }
    let repo_name = repo_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| repo_path.to_string_lossy().to_string());

    let tree = file_tree(repo_path)?;
    let commit_log = commit_log(repo_path, 20);
    let docs = doc_files(repo_path, &tree);
    let summaries = file_summaries(conn, repo_path, &tree)?;
    let directory_digest = build_directory_digest(&summaries);

    Ok(RepoDigest {
        repo_path: repo_path.to_path_buf(),
        repo_name,
        file_tree: tree,
        commit_log,
        doc_files: docs,
        directory_digest,
    })
}

/// Every repo Librarian already knows about (path, name), for tiers that
/// scan across every repo rather than one at a time.
pub fn known_repos(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut q = conn.prepare("SELECT path, name FROM repos WHERE kind = 'repo' ORDER BY name")?;
    let rows = q.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[cfg(test)]
pub(crate) mod test_support {
    //! Shared fixture helper for tests in this module and sibling
    //! repo-currency modules that need a real, on-disk git repository
    //! (`repo_digest` shells out to `git ls-files`/`git log`, so a fake
    //! `.git` directory like `repos.rs`'s tests use is not enough here).
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NONCE: AtomicU64 = AtomicU64::new(0);

    pub fn tmp_repo_dir(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "librarian-currency-test-{tag}-{}-{}",
            std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn git(dir: &Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .expect("git must be installed to run repo-currency tests");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// Initialise a real git repo at `dir`, write `files` (relative path ->
    /// content), and commit them all so `git ls-files`/`git log` have real
    /// answers.
    pub fn init_repo_with_commit(dir: &Path, files: &[(&str, &str)]) {
        git(dir, &["init", "-q"]);
        git(dir, &["config", "user.email", "test@example.com"]);
        git(dir, &["config", "user.name", "Librarian Test"]);
        git(dir, &["config", "commit.gpgsign", "false"]);
        for (rel, content) in files {
            let path = dir.join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(&path, content).unwrap();
        }
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "-m", "initial"]);
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{init_repo_with_commit, tmp_repo_dir};
    use super::{build, doc_files, file_tree, is_doc_file};
    use crate::db;

    #[test]
    fn doc_named_files_are_recognised_wherever_they_are() {
        assert!(is_doc_file("README.md"));
        assert!(is_doc_file("readme"));
        assert!(is_doc_file("CONTRIBUTING.md"));
        assert!(is_doc_file("CHANGELOG.rst"));
        assert!(is_doc_file("ARCHITECTURE.md"));
        assert!(is_doc_file("packages/sub/README.md"));
    }

    #[test]
    fn generic_markdown_only_counts_at_root_or_under_docs() {
        assert!(is_doc_file("docs/guide.md"));
        assert!(is_doc_file("notes.md"));
        assert!(!is_doc_file("src/notes.md"));
        assert!(!is_doc_file("nested/docs/guide.md"));
    }

    #[test]
    fn non_doc_files_are_rejected() {
        assert!(!is_doc_file("src/main.rs"));
        assert!(!is_doc_file("Cargo.toml"));
        assert!(!is_doc_file("docs/logo.png"));
    }

    #[test]
    fn doc_files_reads_content_of_matching_tracked_files() {
        let dir = tmp_repo_dir("docfiles");
        init_repo_with_commit(
            &dir,
            &[
                ("README.md", "# Hello\nworld"),
                ("src/main.rs", "fn main() {}"),
                ("docs/guide.md", "# Guide"),
            ],
        );
        let tree = file_tree(&dir).unwrap();
        assert!(tree.contains(&"README.md".to_string()));
        assert!(tree.contains(&"src/main.rs".to_string()));

        let docs = doc_files(&dir, &tree);
        let rels: Vec<&str> = docs.iter().map(|d| d.rel_path.as_str()).collect();
        assert!(rels.contains(&"README.md"));
        assert!(rels.contains(&"docs/guide.md"));
        assert!(!rels.contains(&"src/main.rs"));

        let readme = docs.iter().find(|d| d.rel_path == "README.md").unwrap();
        assert!(readme.content.contains("Hello"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn build_produces_a_directory_digest_covering_every_tracked_file() {
        let dir = tmp_repo_dir("digest");
        init_repo_with_commit(
            &dir,
            &[
                ("README.md", "# Proj\nA project."),
                ("src/lib.rs", "//! Library entry point.\npub fn go() {}"),
                ("src/util.rs", "pub fn helper() {}"),
            ],
        );

        let db = db::open(&dir.join("catalog-test.db")).unwrap();
        let conn = db.lock().unwrap();
        let digest = build(&conn, &dir).unwrap();
        drop(conn);

        assert_eq!(digest.repo_name, dir.file_name().unwrap().to_string_lossy());
        assert!(digest.file_tree.contains(&"src/lib.rs".to_string()));
        assert_eq!(digest.doc_files.len(), 1);
        assert!(digest.directory_digest.contains("src/lib.rs"));
        assert!(digest.directory_digest.contains("Library entry point."));
        assert!(!digest.commit_log.is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn build_rejects_a_directory_that_is_not_a_git_repository() {
        let dir = tmp_repo_dir("notgit");
        let db = db::open(&dir.join("catalog-test.db")).unwrap();
        let conn = db.lock().unwrap();
        assert!(build(&conn, &dir).is_err());
        drop(conn);
        std::fs::remove_dir_all(&dir).ok();
    }
}
