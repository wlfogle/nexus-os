use std::process::Command;

use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::scanner::doc_type_for_extension;
use crate::types::DocType;

/// Maximum remote file size we are willing to download for analysis (5 MB).
const MAX_REMOTE_FILE_BYTES: u64 = 5 * 1024 * 1024;

/// A GitHub repository owned by the configured user.
#[derive(Debug, Clone)]
pub struct GithubRepo {
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub default_branch: String,
}

/// A documentation file discovered inside a GitHub repository.
#[derive(Debug, Clone)]
pub struct GithubDoc {
    pub path: String,
    pub download_url: String,
    pub doc_type: DocType,
    pub size_bytes: u64,
}

/// Attempt to obtain a GitHub token from the `gh` CLI (`gh auth token`).
///
/// Returns `None` if `gh` is not installed, not authenticated, or returns an empty
/// token.
pub fn get_token() -> Option<String> {
    let output = Command::new("gh").args(["auth", "token"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

// ─── REST response shapes ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RepoApiItem {
    name: String,
    full_name: String,
    html_url: String,
    default_branch: String,
}

#[derive(Debug, Deserialize)]
struct TreeResponse {
    tree: Vec<TreeEntry>,
}

#[derive(Debug, Deserialize)]
struct TreeEntry {
    path: String,
    #[serde(rename = "type")]
    entry_type: String,
    #[serde(default)]
    size: Option<u64>,
}

/// Build a configured reqwest client with the GitHub headers applied.
fn client(token: &str) -> Result<reqwest::Client> {
    use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("nexus-codex"));
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    if !token.is_empty() {
        let value = HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|e| anyhow!("invalid token header: {e}"))?;
        headers.insert(AUTHORIZATION, value);
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| anyhow!("failed to build HTTP client: {e}"))
}

/// List up to 100 owned repositories for `username`.
pub async fn list_repos(username: &str, token: &str) -> Result<Vec<GithubRepo>> {
    let client = client(token)?;
    let url = format!(
        "https://api.github.com/users/{username}/repos?per_page=100&type=owner"
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("GitHub repos request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("GitHub repos request returned {status}: {body}"));
    }

    let items: Vec<RepoApiItem> = resp
        .json()
        .await
        .map_err(|e| anyhow!("failed to parse repos response: {e}"))?;

    Ok(items
        .into_iter()
        .map(|r| GithubRepo {
            name: r.name,
            full_name: r.full_name,
            html_url: r.html_url,
            default_branch: r.default_branch,
        })
        .collect())
}

/// List all documentation files within `repo` using the Git Trees API.
///
/// Files larger than 5 MB are skipped. The `download_url` is the raw content URL
/// for the file on the repo's default branch.
pub async fn fetch_repo_docs(repo: &GithubRepo, token: &str) -> Result<Vec<GithubDoc>> {
    let client = client(token)?;
    let url = format!(
        "https://api.github.com/repos/{}/git/trees/{}?recursive=1",
        repo.full_name, repo.default_branch
    );

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("GitHub tree request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("GitHub tree request returned {status}: {body}"));
    }

    let tree: TreeResponse = resp
        .json()
        .await
        .map_err(|e| anyhow!("failed to parse tree response: {e}"))?;

    let mut docs = Vec::new();
    for entry in tree.tree {
        if entry.entry_type != "blob" {
            continue;
        }

        let ext = match std::path::Path::new(&entry.path)
            .extension()
            .and_then(|e| e.to_str())
        {
            Some(e) => e.to_string(),
            None => continue,
        };

        let doc_type = match doc_type_for_extension(&ext) {
            Some(dt) => dt,
            None => continue,
        };

        let size_bytes = entry.size.unwrap_or(0);
        if size_bytes > MAX_REMOTE_FILE_BYTES {
            continue;
        }

        let download_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}",
            repo.full_name, repo.default_branch, entry.path
        );

        docs.push(GithubDoc {
            path: entry.path,
            download_url,
            doc_type,
            size_bytes,
        });
    }

    Ok(docs)
}

/// Download the raw text content of a documentation file from its `download_url`.
pub async fn download_content(download_url: &str, token: &str) -> Result<String> {
    let client = client(token)?;
    let resp = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| anyhow!("GitHub content request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(anyhow!("content request returned {status}"));
    }

    resp.text()
        .await
        .map_err(|e| anyhow!("failed to read content body: {e}"))
}
