//! Git repository intelligence.
//!
//! Shelling out to `git` rather than linking libgit2: the CLI is already
//! installed, always matches the on-disk format, and correctly handles the
//! cases that trip up naive detection here -- `.git` as a *file* (worktrees and
//! absorbed submodules), and repos nested inside other repos.
//!
//! The important derived fact is `recoverable`: a repo whose working tree is
//! clean and whose every local commit already exists on some remote can be
//! deleted and re-cloned losing nothing. That is what makes it safe to reclaim
//! space aggressively.

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::Config;
use crate::db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub path: String,
    pub name: String,
    pub owner: String,
    pub remote: Option<String>,
    pub kind: String,
    pub branch: Option<String>,
    pub last_commit: i64,
    pub dirty: i64,
    pub untracked: i64,
    pub unpushed: i64,
    pub stashes: i64,
    pub recoverable: bool,
    pub size_bytes: i64,
}

/// Run a git command inside `dir`, returning trimmed stdout on success.
fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git_lines(dir: &Path, args: &[&str]) -> Vec<String> {
    git(dir, args)
        .map(|s| {
            s.lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Extract the owner segment from a remote URL.
///
/// Handles `https://host/owner/name.git`, `git@host:owner/name.git` and
/// `ssh://host/owner/name`.
pub fn owner_from_remote(url: &str) -> String {
    let mut s = url.trim().to_string();
    if let Some(stripped) = s.strip_suffix(".git") {
        s = stripped.to_string();
    }
    // strip scheme://host/
    if let Some(idx) = s.find("://") {
        let rest = &s[idx + 3..];
        s = match rest.find('/') {
            Some(i) => rest[i + 1..].to_string(),
            None => return "local".into(),
        };
    } else if let Some(idx) = s.find('@') {
        // git@host:owner/name
        let rest = &s[idx + 1..];
        s = match rest.find(':') {
            Some(i) => rest[i + 1..].to_string(),
            None => return "local".into(),
        };
    }
    let parts: Vec<&str> = s.split('/').filter(|p| !p.is_empty()).collect();
    if parts.len() >= 2 {
        parts[parts.len() - 2].to_string()
    } else {
        "local".into()
    }
}

/// The roots actually walked for repo discovery.
///
/// `cfg.roots` alone is not enough: on this machine (and by convention, any
/// machine following the same layout) real repos live under `cfg.vault` and
/// the `cfg.monorepo` checkout, which are tracked as separate config fields
/// from `roots` and are not guaranteed to be reachable from it (no symlink is
/// assumed, and even if one existed the walk below refuses to follow
/// symlinks). Without this, `roots` defaulting to just the home directory
/// silently means zero repos are ever found. Deduplicated against `cfg.roots`
/// so a user who already lists the vault there does not get it walked twice.
fn effective_roots(cfg: &Config) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    for root in cfg
        .roots
        .iter()
        .chain(std::iter::once(&cfg.vault))
        .chain(std::iter::once(&cfg.monorepo))
    {
        if seen.insert(root.clone()) {
            out.push(root.clone());
        }
    }
    out
}

/// Walk the configured roots (plus the vault and monorepo, see
/// `effective_roots`) and return every git working tree found.
///
/// Detection looks for `.git` as either a directory or a file, so worktrees and
/// absorbed submodules are not missed. Once a repo root is found the walk still
/// descends, because nested repos (submodules, vendored clones) are real and
/// need their own rows -- but they are marked with `parent_id` so callers can
/// tell a top-level project from something that travels with it.
pub fn discover(cfg: &Config) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();

    for root in effective_roots(cfg) {
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            let entries = match std::fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => continue,
            };
            let mut subdirs = Vec::new();
            let mut is_repo = false;

            for entry in entries.flatten() {
                let p = entry.path();
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name == ".git" {
                    is_repo = true;
                    continue;
                }
                let ft = match entry.file_type() {
                    Ok(t) => t,
                    Err(_) => continue,
                };
                // Never follow symlinks: the vault is symlinked back into home
                // and following them would double-count every repo.
                if ft.is_dir() && !ft.is_symlink() && !cfg.is_pruned(&p) {
                    subdirs.push(p);
                }
            }

            if is_repo && seen.insert(dir.clone()) {
                found.push(dir.clone());
            }
            stack.extend(subdirs);
        }
    }

    found.sort();
    found
}

/// Collect metadata for one working tree.
pub fn inspect(path: &Path) -> RepoInfo {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    let remote = git(path, &["remote", "get-url", "origin"]);
    let owner = remote
        .as_deref()
        .map(owner_from_remote)
        .unwrap_or_else(|| "local".into());

    // worktree / submodule detection: `.git` is a file, not a directory
    let dot_git = path.join(".git");
    let kind = if dot_git.is_file() {
        match std::fs::read_to_string(&dot_git) {
            Ok(s) if s.contains("/worktrees/") => "worktree",
            Ok(_) => "submodule",
            Err(_) => "repo",
        }
    } else {
        "repo"
    }
    .to_string();

    let branch = git(path, &["rev-parse", "--abbrev-ref", "HEAD"]);
    let last_commit = git(path, &["log", "-1", "--format=%ct"])
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    let dirty = git_lines(path, &["diff", "--name-only", "HEAD"]).len() as i64;
    let untracked =
        git_lines(path, &["ls-files", "--others", "--exclude-standard"]).len() as i64;
    let stashes = git_lines(path, &["stash", "list"]).len() as i64;

    // Commits reachable from any local ref but present on no remote.
    let unpushed = git(path, &["rev-list", "--count", "--all", "--not", "--remotes"])
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    let has_remote = remote.is_some();
    let recoverable =
        has_remote && unpushed == 0 && dirty == 0 && untracked == 0 && stashes == 0;

    RepoInfo {
        path: path.to_string_lossy().to_string(),
        name,
        owner,
        remote,
        kind,
        branch,
        last_commit,
        dirty,
        untracked,
        unpushed,
        stashes,
        recoverable,
        size_bytes: 0,
    }
}

/// Rescan every repo and replace the `repos` table contents.
pub fn refresh(conn: &mut Connection, cfg: &Config) -> Result<usize> {
    let paths = discover(cfg);
    let infos: Vec<RepoInfo> = paths.iter().map(|p| inspect(p)).collect();

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM repos", [])?;
    {
        let mut ins = tx.prepare(
            "INSERT INTO repos
               (path,name,owner,remote,kind,branch,last_commit,dirty,untracked,
                unpushed,stashes,recoverable,size_bytes,scanned_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        )?;
        let ts = db::now();
        for r in &infos {
            ins.execute(params![
                r.path,
                r.name,
                r.owner,
                r.remote,
                r.kind,
                r.branch,
                r.last_commit,
                r.dirty,
                r.untracked,
                r.unpushed,
                r.stashes,
                i64::from(r.recoverable),
                r.size_bytes,
                ts,
            ])?;
        }
    }

    // Link nested repos to the closest enclosing repo.
    {
        let mut rows: Vec<(i64, String)> = Vec::new();
        {
            let mut q = tx.prepare("SELECT id, path FROM repos ORDER BY LENGTH(path)")?;
            let mapped = q.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            for m in mapped {
                rows.push(m?);
            }
        }
        for (id, path) in &rows {
            let mut best: Option<(i64, usize)> = None;
            for (pid, ppath) in &rows {
                if pid == id {
                    continue;
                }
                let prefix = format!("{ppath}/");
                if path.starts_with(&prefix) {
                    let len = ppath.len();
                    if best.map(|(_, l)| len > l).unwrap_or(true) {
                        best = Some((*pid, len));
                    }
                }
            }
            if let Some((pid, _)) = best {
                tx.execute(
                    "UPDATE repos SET parent_id = ?1 WHERE id = ?2",
                    params![pid, id],
                )?;
            }
        }
    }

    tx.commit()?;
    Ok(infos.len())
}

/// Attach every catalogued file to the innermost repo that contains it.
pub fn assign_files(conn: &mut Connection) -> Result<()> {
    let mut repos: Vec<(i64, String)> = Vec::new();
    {
        let mut q = conn.prepare("SELECT id, path FROM repos")?;
        let mapped = q.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        for m in mapped {
            repos.push(m?);
        }
    }
    // Longest path first so the innermost repo wins.
    repos.sort_by_key(|(_, p)| std::cmp::Reverse(p.len()));

    let tx = conn.transaction()?;
    tx.execute("UPDATE files SET repo_id = NULL", [])?;
    for (id, path) in &repos {
        tx.execute(
            "UPDATE files SET repo_id = ?1
             WHERE repo_id IS NULL AND path LIKE ?2 ESCAPE '\\'",
            params![id, format!("{}/%", escape_like(path))],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Escape `%` and `_` so a literal path can be used in a LIKE pattern.
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

pub fn list(conn: &Connection) -> Result<Vec<RepoInfo>> {
    let mut q = conn.prepare(
        "SELECT path,name,owner,remote,kind,branch,last_commit,dirty,untracked,
                unpushed,stashes,recoverable,size_bytes
         FROM repos ORDER BY last_commit DESC",
    )?;
    let rows = q.query_map([], |r| {
        Ok(RepoInfo {
            path: r.get(0)?,
            name: r.get(1)?,
            owner: r.get(2)?,
            remote: r.get(3)?,
            kind: r.get(4)?,
            branch: r.get(5)?,
            last_commit: r.get(6)?,
            dirty: r.get(7)?,
            untracked: r.get(8)?,
            unpushed: r.get(9)?,
            stashes: r.get(10)?,
            recoverable: r.get::<_, i64>(11)? != 0,
            size_bytes: r.get(12)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{discover, escape_like, owner_from_remote};
    use crate::config::Config;
    use std::fs;

    /// Creates `<tmp>/<name>` and marks it as a git working tree by creating
    /// a `.git` directory inside it (discovery only checks for presence, not
    /// a real repository), returning the path.
    fn make_fake_repo(base: &std::path::Path, name: &str) -> std::path::PathBuf {
        let repo = base.join(name);
        fs::create_dir_all(repo.join(".git")).unwrap();
        repo
    }

    #[test]
    fn vault_and_monorepo_are_always_walked_even_if_not_in_roots() {
        let base = std::env::temp_dir().join(format!(
            "librarian-repos-test-{}-{}",
            std::process::id(),
            db_test_nonce()
        ));
        let roots_root = base.join("only_root");
        let vault = base.join("vault");
        let monorepo = base.join("monorepo");
        fs::create_dir_all(&roots_root).unwrap();
        fs::create_dir_all(&vault).unwrap();
        fs::create_dir_all(&monorepo).unwrap();

        // A repo that lives only under the vault/monorepo, never under `roots`.
        let vault_repo = make_fake_repo(&vault, "some-project");
        let monorepo_repo = monorepo.clone(); // the monorepo checkout is itself a repo
        fs::create_dir_all(monorepo_repo.join(".git")).unwrap();

        let cfg = Config {
            roots: vec![roots_root],
            vault: vault.clone(),
            monorepo: monorepo.clone(),
            ..Config::default()
        };

        let found = discover(&cfg);
        assert!(
            found.contains(&vault_repo),
            "repo under cfg.vault must be discovered even though it is not in cfg.roots: {found:?}"
        );
        assert!(
            found.contains(&monorepo_repo),
            "cfg.monorepo itself must be discovered even though it is not in cfg.roots: {found:?}"
        );

        let _ = fs::remove_dir_all(&base);
    }

    /// A tiny counter so parallel test runs never collide on the same tmp dir.
    fn db_test_nonce() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        N.fetch_add(1, Ordering::Relaxed)
    }

    #[test]
    fn https_remote() {
        assert_eq!(
            owner_from_remote("https://github.com/wlfogle/nexus-os.git"),
            "wlfogle"
        );
    }

    #[test]
    fn https_remote_without_suffix() {
        assert_eq!(
            owner_from_remote("https://github.com/wlfogle/nexus-terminal"),
            "wlfogle"
        );
    }

    #[test]
    fn scp_style_remote() {
        assert_eq!(
            owner_from_remote("git@github.com:45Drives/cockpit-file-sharing.git"),
            "45Drives"
        );
    }

    #[test]
    fn gitlab_remote() {
        assert_eq!(
            owner_from_remote("https://gitlab.com/newbit/rootAVD.git"),
            "newbit"
        );
    }

    #[test]
    fn like_escaping() {
        assert_eq!(escape_like("a_b%c"), "a\\_b\\%c");
    }
}
