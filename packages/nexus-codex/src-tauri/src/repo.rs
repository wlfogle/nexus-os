use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{DateTime, Utc};

/// Read `remote.origin.url` from a `.git/config` file, if present.
fn read_origin_url(git_dir: &Path) -> Option<String> {
    let config_path = git_dir.join("config");
    let data = std::fs::read_to_string(&config_path).ok()?;

    let mut in_origin = false;
    for line in data.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            // Section header, e.g. [remote "origin"]
            in_origin = trimmed.replace(' ', "") == "[remote\"origin\"]";
            continue;
        }
        if in_origin {
            if let Some(rest) = trimmed.strip_prefix("url") {
                let rest = rest.trim_start();
                if let Some(eq) = rest.strip_prefix('=') {
                    return Some(eq.trim().to_string());
                }
            }
        }
    }
    None
}

/// Walk up the directory tree from `path` until a `.git` directory is found.
///
/// Returns `(repo_name, repo_root)` where `repo_name` is taken from the
/// `remote.origin.url` if present, otherwise the directory name of the repo root.
pub fn find_repo_for_path(path: &Path) -> Option<(String, String)> {
    let mut current: Option<&Path> = if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    };

    while let Some(dir) = current {
        let git_dir = dir.join(".git");
        if git_dir.exists() {
            let repo_root = dir.to_string_lossy().to_string();
            let origin = read_origin_url(&git_dir);

            let repo_name = match &origin {
                Some(url) => repo_name_from_url(url),
                None => dir
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| repo_root.clone()),
            };

            return Some((repo_name, repo_root));
        }
        current = dir.parent();
    }

    None
}

/// Return the full remote origin URL for the repository containing `path`, if any.
pub fn find_repo_url(path: &Path) -> Option<String> {
    let mut current: Option<&Path> = if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    };

    while let Some(dir) = current {
        let git_dir = dir.join(".git");
        if git_dir.exists() {
            return read_origin_url(&git_dir);
        }
        current = dir.parent();
    }
    None
}

/// Derive a short repository name from a remote URL.
fn repo_name_from_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    let stripped = trimmed.strip_suffix(".git").unwrap_or(trimmed);
    stripped
        .rsplit(['/', ':'])
        .next()
        .unwrap_or(stripped)
        .to_string()
}

/// Return the ISO-8601 date of the last commit that touched `path`.
///
/// Uses `git log --follow --format=%aI -1 -- <file>`.
pub fn get_file_git_age(path: &Path) -> Option<String> {
    let dir = path.parent()?;
    let output = Command::new("git")
        .current_dir(dir)
        .args(["log", "--follow", "--format=%aI", "-1", "--"])
        .arg(path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let date = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if date.is_empty() {
        None
    } else {
        Some(date)
    }
}

/// Return how many days have elapsed since the most recent commit touching `dir`.
///
/// Uses `git log --format=%aI -1 -- <dir>` and compares to the current time.
pub fn get_dir_latest_code_age(dir: &Path) -> Option<i64> {
    let work_dir: PathBuf = if dir.is_dir() {
        dir.to_path_buf()
    } else {
        dir.parent()?.to_path_buf()
    };

    let output = Command::new("git")
        .current_dir(&work_dir)
        .args(["log", "--format=%aI", "-1", "--"])
        .arg(dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let date_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if date_str.is_empty() {
        return None;
    }

    let committed = DateTime::parse_from_rfc3339(&date_str).ok()?;
    let now = Utc::now();
    let age = now.signed_duration_since(committed.with_timezone(&Utc));
    Some(age.num_days())
}
