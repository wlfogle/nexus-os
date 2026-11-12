//! Read-only reconciliation. Dates never decide relevance or disposition.
//! Lexical evidence is a lead, not a claim that a feature is implemented.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use walkdir::WalkDir;

use crate::config::Config;

const MAX_INDEX_BYTES: usize = 64 * 1024 * 1024;
const MAX_ENTRIES: usize = 250_000;

#[derive(Default)]
pub struct Session {
    pub running: bool,
    pub progress: Progress,
    pub report: Option<Report>,
    pub error: Option<String>,
    pub cancel: std::sync::Arc<AtomicBool>,
}

#[derive(Serialize)]
pub struct Status {
    pub running: bool,
    pub progress: Progress,
    pub report: Option<Report>,
    pub error: Option<String>,
}

impl Session {
    pub fn finding(&self, index: usize) -> Result<Finding> {
        if self.running {
            bail!("Comparison is still running");
        }
        self.report
            .as_ref()
            .and_then(|r| r.findings.get(index))
            .cloned()
            .context("No such comparison finding")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Options {
    pub roots: Vec<PathBuf>,
    pub vault: PathBuf,
    pub max_files: usize,
    pub max_repo_files: usize,
    pub max_bytes: usize,
}

impl Options {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            roots: cfg.roots.clone(),
            vault: cfg.vault.clone(),
            max_files: 10_000,
            max_repo_files: 30_000,
            max_bytes: 128 * 1024,
        }
    }

    fn validate(&self) -> Result<()> {
        if self.roots.is_empty()
            || self.max_files == 0
            || self.max_repo_files == 0
            || self.max_files > 100_000
            || self.max_repo_files > 100_000
            || !(1..=1024 * 1024).contains(&self.max_bytes)
        {
            bail!("Choose roots and positive limits (at most 100000 files, 1 MiB per file)");
        }
        checked_path(&self.vault, false)?;
        for root in &self.roots {
            if root == Path::new("/") || root.starts_with(&self.vault) {
                bail!(
                    "Choose source folders outside the repository vault, not the filesystem root"
                );
            }
            checked_path(root, false)?;
            if !root.is_dir() {
                bail!("Scan root is not a directory: {}", root.display());
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Progress {
    pub phase: String,
    pub visited: usize,
    pub indexed: usize,
    pub compared: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Citation {
    pub path: PathBuf,
    pub line: usize,
    pub excerpt: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Match {
    pub repo: PathBuf,
    pub path: PathBuf,
    pub sha256: String,
    pub relation: String,
    pub score: f32,
    pub evidence: Vec<String>,
    pub citations: Vec<Citation>,
    #[serde(default)]
    pub repo_role: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Finding {
    pub path: PathBuf,
    pub root: PathBuf,
    pub sha256: String,
    pub modified: f64,
    pub size: usize,
    pub protected_tree: Option<PathBuf>,
    pub kind: String,
    pub disposition: String,
    pub ideas: Vec<String>,
    pub cautions: Vec<String>,
    pub matches: Vec<Match>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Report {
    pub started_at: f64,
    pub finished_at: f64,
    pub roots: Vec<PathBuf>,
    pub vault: PathBuf,
    pub repositories: Vec<PathBuf>,
    pub protected_trees: Vec<PathBuf>,
    pub findings: Vec<Finding>,
    pub skipped: BTreeMap<String, usize>,
    pub warnings: Vec<String>,
    pub cancelled: bool,
    pub limited: bool,
    #[serde(default)]
    pub assessments: BTreeMap<String, crate::recovery::Assessment>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Preview {
    pub source: String,
    pub counterpart: String,
    pub diff: String,
    pub truncated: bool,
}

struct Indexed {
    repo: PathBuf,
    path: PathBuf,
    hash: String,
    body: String,
    terms: HashSet<String>,
}

pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Reject every symlink component, including dangling links and configured roots.
/// All write paths use the same check again immediately before touching disk.
pub fn checked_path(path: &Path, missing: bool) -> Result<()> {
    if !path.is_absolute() {
        bail!("Path must be absolute");
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Normal(_) => current.push(component),
            _ => bail!("Non-normal path component"),
        }
        match std::fs::symlink_metadata(&current) {
            Ok(md) if md.file_type().is_symlink() => {
                bail!("Symlink path refused: {}", current.display())
            }
            Ok(_) => {}
            Err(e) if missing && e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

pub fn protected_tree(path: &Path) -> Option<PathBuf> {
    path.parent()?
        .ancestors()
        .find(|p| std::fs::symlink_metadata(p.join(".git")).is_ok())
        .map(Path::to_path_buf)
}

pub fn secret_path(path: &Path) -> bool {
    path.components()
        .filter_map(|c| c.as_os_str().to_str())
        .any(|name| {
            let n = name.to_ascii_lowercase();
            n == ".env"
                || n.starts_with(".env.")
                || n.starts_with("id_rsa")
                || n.starts_with("id_ed25519")
                || n.ends_with(".pem")
                || n.ends_with(".key")
                || [
                    "password",
                    "credential",
                    "bitwarden",
                    "vaultwarden-export",
                    "secrets",
                ]
                .iter()
                .any(|term| n.contains(term))
        })
}

/// Fail closed for common credential formats. This is not a complete DLP tool.
pub fn secret_text(body: &str) -> bool {
    if body.contains("PRIVATE KEY-----")
        || body.contains("\"encrypted\":false")
        || body.contains("\"encrypted\": false")
    {
        return true;
    }
    body.lines().any(|line| {
        let lower = line.to_ascii_lowercase();
        let key = [
            "api_key",
            "apikey",
            "api-key",
            "access_token",
            "auth_token",
            "password",
            "privatekey",
            "presharedkey",
            "client_secret",
            "authorization:",
        ]
        .iter()
        .any(|k| lower.contains(k));
        if !key {
            return false;
        }
        let value = line
            .split_once('=')
            .or_else(|| line.split_once(':'))
            .map(|(_, v)| v.trim());
        value.is_some_and(|v| {
            v.len() > 4
                && ![
                    "$",
                    "example",
                    "placeholder",
                    "redacted",
                    "getenv",
                    "environ",
                    "none",
                    "null",
                    "{{",
                    "<",
                    "your_",
                    "os.",
                    "self.",
                    "config.",
                    "true",
                    "false",
                ]
                .iter()
                .any(|s| v.to_ascii_lowercase().contains(s))
        })
    })
}

fn candidate(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(
        crate::scan::classify_ext(&ext),
        "script" | "code" | "doc" | "config" | "web"
    ) || matches!(
        path.file_name().and_then(|s| s.to_str()),
        Some("Makefile" | "Dockerfile" | "README")
    )
}

pub fn readable(path: &Path, max_bytes: usize) -> Result<String> {
    checked_path(path, false)?;
    if secret_path(path) {
        bail!("Excluded sensitive filename");
    }
    let md = std::fs::symlink_metadata(path)?;
    if !md.is_file() || md.len() > max_bytes as u64 {
        bail!("Not a regular file within the size limit");
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(max_bytes as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes || bytes.contains(&0) {
        bail!("Oversized or binary content");
    }
    let body = String::from_utf8(bytes).context("Non-UTF8 text")?;
    if secret_text(&body) {
        bail!("Potential credentials: content excluded");
    }
    Ok(body)
}

pub(crate) fn tokens(text: &str) -> HashSet<String> {
    const STOP: &[&str] = &[
        "then", "else", "echo", "true", "false", "none", "null", "self", "this", "that", "with",
        "from", "return", "const", "string", "import", "function", "error", "path", "file",
        "files", "home", "loufogle", "media", "data", "repos", "local", "config", "usr", "bash",
        "python", "print", "script", "scripts", "sudo", "install", "system", "name", "line",
        "result", "Some", "into", "clone",
    ];
    text.split(|c: char| !c.is_alphanumeric() && c != '-')
        .filter(|s| s.len() >= 4 && s.len() < 64)
        .map(str::to_ascii_lowercase)
        .filter(|s| !STOP.contains(&s.as_str()) && !s.chars().all(|c| c.is_numeric()))
        .take(8_000)
        .collect()
}

fn skip(report: &mut Report, why: &str) {
    *report.skipped.entry(why.into()).or_default() += 1;
}

pub fn canonical_guidance(cfg: &Config) -> Result<String> {
    let path = cfg.vault.join("nexus-os/docs/NEXUS-OS-SOURCE-OF-TRUTH.md");
    checked_path(&path, false)?;
    let mut bytes = Vec::new();
    std::fs::File::open(&path)?
        .take(128 * 1024)
        .read_to_end(&mut bytes)?;
    let body = String::from_utf8(bytes)?;
    // Only the documented hierarchy/topology portion, never credential sections.
    let section = body
        .split_once("## 4. Credentials")
        .context("Canonical guidance boundary not found")?
        .0;
    Ok(section
        .lines()
        .map(|line| {
            if secret_text(line) {
                "[sensitive line omitted]"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

fn repo_roles(cfg: &Config) -> HashMap<String, String> {
    let mut roles = HashMap::new();
    if let Some(name) = cfg.monorepo.file_name().and_then(|s| s.to_str()) {
        roles.insert(
            name.to_string(),
            "current umbrella (configured monorepo)".into(),
        );
    }
    if let Ok(body) = canonical_guidance(cfg) {
        let section = body
            .split("## 1. Component hierarchy")
            .nth(1)
            .and_then(|s| s.split("## 2.").next())
            .unwrap_or("");
        for block in section.split("\n- ") {
            let role = if block.contains("**CURRENT**") {
                "current module"
            } else if block.contains("**FORERUNNER**") || block.contains("**Legacy/superseded**") {
                "historical reference"
            } else if block.contains("**Third-party clones**") {
                "third-party; never automatically merge"
            } else {
                continue;
            };
            for name in block.split('`').skip(1).step_by(2) {
                if !name.contains(['/', ' ', '*']) {
                    roles.insert(name.into(), role.into());
                }
            }
            if let Some(name) = block.strip_prefix("**").and_then(|s| s.split("**").next()) {
                if !name.contains(['/', ' ', ':']) {
                    roles.insert(name.into(), role.into());
                }
            }
        }
    }
    roles
}

fn prune(cfg: &Config, path: &Path) -> bool {
    cfg.is_pruned(path)
        || path.file_name().and_then(|s| s.to_str()).is_some_and(|n| {
            n.starts_with('.')
                || matches!(
                    n,
                    "archive"
                        | "Archive"
                        | "Quarantine"
                        | "Backups"
                        | "Games"
                        | "Music"
                        | "Videos"
                        | "Pictures"
                        | "vms"
                        | "logs"
                )
        })
}

pub fn run(
    cfg: &Config,
    opts: &Options,
    cancel: &AtomicBool,
    mut progress: impl FnMut(Progress),
) -> Result<Report> {
    opts.validate()?;
    let mut report = Report {
        started_at: crate::db::now(),
        roots: opts.roots.clone(),
        vault: opts.vault.clone(),
        ..Report::default()
    };
    let mut index = Vec::<Indexed>::new();
    let roles = repo_roles(cfg);
    let mut index_bytes = 0;
    let mut visited = 0;
    let mut dirs = std::fs::read_dir(&opts.vault)?.collect::<std::io::Result<Vec<_>>>()?;
    dirs.sort_by_key(|e| e.path());
    for entry in dirs {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let repo = entry.path();
        if !entry.file_type()?.is_dir() || !repo.join(".git").exists() {
            continue;
        }
        checked_path(&repo, false)?;
        let out = Command::new("git")
            .args(["--no-pager", "-C"])
            .arg(&repo)
            .args(["ls-files", "-z"])
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()?;
        if !out.status.success() {
            report
                .warnings
                .push(format!("Unusable Git metadata: {}", repo.display()));
            continue;
        }
        report.repositories.push(repo.clone());
        for relative in out.stdout.split(|&b| b == 0).filter(|s| !s.is_empty()) {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            if index.len() >= opts.max_repo_files || index_bytes >= MAX_INDEX_BYTES {
                report.limited = true;
                break;
            }
            let Ok(relative) = std::str::from_utf8(relative) else {
                skip(&mut report, "non-UTF8 repo path");
                continue;
            };
            let relative = Path::new(relative);
            if relative
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
            {
                skip(&mut report, "unsafe repo path");
                continue;
            }
            let path = repo.join(relative);
            if relative.ancestors().any(|p| prune(cfg, p))
                || secret_path(&path)
                || !candidate(&path)
            {
                skip(&mut report, "excluded repo file");
                continue;
            }
            match readable(&path, opts.max_bytes) {
                Ok(body) => {
                    index_bytes += body.len();
                    let mut terms = tokens(&body);
                    terms.extend(tokens(&relative.to_string_lossy()));
                    index.push(Indexed {
                        repo: repo.clone(),
                        path,
                        hash: hash(body.as_bytes()),
                        body,
                        terms,
                    });
                }
                Err(_) => skip(
                    &mut report,
                    "unreadable, oversized, binary or sensitive repo file",
                ),
            }
        }
        progress(Progress {
            phase: "index repositories".into(),
            visited,
            indexed: index.len(),
            compared: 0,
        });
    }
    if report.limited {
        report.warnings.push("Repository index limit reached; matches may be incomplete. Increase limits or narrow the vault.".into());
    }
    let mut by_term: HashMap<&str, Vec<usize>> = HashMap::new();
    let mut by_hash: HashMap<&str, Vec<usize>> = HashMap::new();
    let mut by_name: HashMap<&std::ffi::OsStr, Vec<usize>> = HashMap::new();
    for (i, file) in index.iter().enumerate() {
        by_hash.entry(&file.hash).or_default().push(i);
        if let Some(name) = file.path.file_name() {
            by_name.entry(name).or_default().push(i);
        }
        for term in &file.terms {
            by_term.entry(term).or_default().push(i);
        }
    }
    let mut seen = HashSet::new();
    'roots: for root in &opts.roots {
        let walker = WalkDir::new(root)
            .follow_links(false)
            .max_depth(32)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                e.depth() == 0
                    || !e.file_type().is_dir()
                    || (!e.path().starts_with(&opts.vault)
                        && !e.path().starts_with(&cfg.library)
                        && !prune(cfg, e.path()))
            });
        for entry in walker {
            if cancel.load(Ordering::Relaxed) {
                break 'roots;
            }
            visited += 1;
            if visited > MAX_ENTRIES || report.findings.len() >= opts.max_files {
                report.limited = true;
                report.warnings.push(
                    "Source traversal/file limit reached; this is a partial comparison.".into(),
                );
                break 'roots;
            }
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    report.limited = true;
                    report.warnings.push(e.to_string());
                    continue;
                }
            };
            let path = entry.path();
            if entry.file_type().is_symlink() {
                skip(&mut report, "symlink");
                continue;
            }
            if entry.depth() == 32 && entry.file_type().is_dir() {
                report.limited = true;
                report
                    .warnings
                    .push(format!("Depth limit: {}", path.display()));
            }
            if !entry.file_type().is_file() || !seen.insert(path.to_path_buf()) {
                continue;
            }
            if path.starts_with(&opts.vault)
                || path.starts_with(&cfg.library)
                || path
                    .components()
                    .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
                || secret_path(path)
                || !candidate(path)
            {
                skip(&mut report, "excluded source file");
                continue;
            }
            let body = match readable(path, opts.max_bytes) {
                Ok(body) => body,
                Err(_) => {
                    skip(
                        &mut report,
                        "unreadable, oversized, binary or sensitive source file",
                    );
                    continue;
                }
            };
            let sha = hash(body.as_bytes());
            let mut terms = tokens(&body);
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                terms.extend(tokens(&name.replace('_', " ")));
            }
            let protected = protected_tree(path);
            if let Some(tree) = &protected {
                if !report.protected_trees.contains(tree) {
                    report.protected_trees.push(tree.clone());
                }
            }
            let mut shared: HashMap<usize, usize> = HashMap::new();
            for term in &terms {
                if let Some(ids) = by_term.get(term.as_str()) {
                    // Ubiquitous terms carry no useful project evidence.
                    if ids.len() > (index.len() / 8).max(20) {
                        continue;
                    }
                    for &id in ids {
                        *shared.entry(id).or_default() += 1;
                    }
                }
            }
            if let Some(ids) = by_hash.get(sha.as_str()) {
                for &id in ids {
                    shared.entry(id).or_default();
                }
            }
            if let Some(ids) = path.file_name().and_then(|n| by_name.get(n)) {
                for &id in ids {
                    shared.entry(id).or_default();
                }
            }
            let mut candidates: Vec<_> = shared.into_iter().collect();
            candidates.sort_by(|(a, ac), (b, bc)| {
                (index[*b].hash == sha)
                    .cmp(&(index[*a].hash == sha))
                    .then(
                        (path.file_name() == index[*b].path.file_name())
                            .cmp(&(path.file_name() == index[*a].path.file_name())),
                    )
                    .then(bc.cmp(ac))
                    .then(a.cmp(b))
            });
            // Retrieval is bounded independently of corpus size. This is a
            // candidate search, never an exhaustive proof of feature absence.
            candidates.truncate(100);
            let weight = |term: &String| {
                let count = by_term.get(term.as_str()).map_or(0, Vec::len);
                ((index.len() as f32 + 1.0) / (count as f32 + 1.0))
                    .ln()
                    .max(0.1)
            };
            let total_weight: f32 = terms
                .iter()
                .filter(|t| by_term.contains_key(t.as_str()))
                .map(weight)
                .sum();
            let mut matches = Vec::new();
            for (id, count) in candidates {
                let file = &index[id];
                let exact = sha == file.hash;
                let name = path.file_name() == file.path.file_name();
                let similarity = terms.intersection(&file.terms).count() as f32
                    / terms.union(&file.terms).count().max(1) as f32;
                if !exact && !name && count < 3 {
                    continue;
                }
                let shared_weight: f32 = terms.intersection(&file.terms).map(weight).sum();
                let coverage = shared_weight / total_weight.max(0.1);
                let role = roles
                    .get(file.repo.file_name().and_then(|s| s.to_str()).unwrap_or(""))
                    .cloned()
                    .unwrap_or_else(|| "unclassified; review project guidance".into());
                let score = if exact {
                    1.0
                } else {
                    (coverage * 0.65
                        + similarity * 0.15
                        + if name { 0.25 } else { 0.0 }
                        + if role.starts_with("current") {
                            0.04
                        } else {
                            0.0
                        })
                    .min(0.99)
                };
                let mut evidence = vec![format!(
                    "{count} shared discriminative terms; lexical overlap {similarity:.3}"
                )];
                if exact {
                    evidence.push("Identical SHA-256; byte-for-byte counterpart".into());
                }
                if name {
                    evidence.push("Same filename (not proof of equivalent behavior)".into());
                }
                evidence.push(format!(
                    "Rarity-weighted source coverage {coverage:.3}; repository role: {role}"
                ));
                let citations = file
                    .body
                    .lines()
                    .enumerate()
                    .filter(|(_, line)| tokens(line).intersection(&terms).next().is_some())
                    .take(3)
                    .map(|(line, text)| Citation {
                        path: file.path.clone(),
                        line: line + 1,
                        excerpt: text.chars().take(220).collect(),
                    })
                    .collect();
                matches.push(Match {
                    repo: file.repo.clone(),
                    path: file.path.clone(),
                    sha256: file.hash.clone(),
                    relation: if exact {
                        "identical"
                    } else if name {
                        "divergent_copy"
                    } else {
                        "related_evidence"
                    }
                    .into(),
                    score,
                    evidence,
                    citations,
                    repo_role: role,
                });
            }
            matches.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.path.cmp(&b.path)));
            matches.truncate(5);
            let ambiguous = matches.len() > 1
                && matches[0].repo != matches[1].repo
                && (matches[0].score - matches[1].score).abs() < 0.08;
            let kind = if ambiguous {
                "ambiguous"
            } else {
                matches
                    .first()
                    .map(|m| m.relation.as_str())
                    .unwrap_or("unmatched")
            }
            .to_string();
            let mut cautions = vec!["No age cutoff. Similarity is not proof of feature equivalence or current topology.".into()];
            if protected.is_some() {
                cautions.push("Source-tree protected: Git marker found, even if metadata is broken. No individual file moves.".into());
            }
            if body.contains("fuser -k") || body.contains("killall ") || body.contains("rm -rf") {
                cautions.push("Contains process termination or recursive removal; never execute as part of comparison.".into());
            }
            let ideas: Vec<String> = body
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.starts_with("#!") {
                        return None;
                    }
                    line.strip_prefix('#')
                        .or_else(|| line.strip_prefix("//"))
                        .map(str::trim)
                        .filter(|s| s.len() >= 12)
                        .map(|s| s.chars().take(200).collect())
                })
                .take(5)
                .collect();
            let md = match entry.metadata() {
                Ok(md) => md,
                Err(_) => {
                    skip(&mut report, "source changed during comparison");
                    continue;
                }
            };
            report.findings.push(Finding {
                path: path.to_path_buf(),
                root: root.clone(),
                sha256: sha,
                modified: md
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(0.0),
                size: body.len(),
                protected_tree: protected,
                kind,
                disposition: "review purpose and recover ideas before reference archiving".into(),
                ideas,
                cautions,
                matches,
            });
            if report.findings.len() % 50 == 0 {
                progress(Progress {
                    phase: "compare sources".into(),
                    visited,
                    indexed: index.len(),
                    compared: report.findings.len(),
                });
            }
        }
    }
    report.cancelled = cancel.load(Ordering::Relaxed);
    report.finished_at = crate::db::now();
    progress(Progress {
        phase: "complete".into(),
        visited,
        indexed: index.len(),
        compared: report.findings.len(),
    });
    Ok(report)
}

pub fn preview(finding: &Finding, counterpart: Option<usize>) -> Result<Preview> {
    let source = readable(&finding.path, 1024 * 1024)?;
    if hash(source.as_bytes()) != finding.sha256 {
        bail!("Source changed since comparison; rescan");
    }
    let (other, label) = match counterpart {
        Some(i) => {
            let matched = finding.matches.get(i).context("Unknown counterpart")?;
            let body = readable(&matched.path, 1024 * 1024)?;
            if hash(body.as_bytes()) != matched.sha256 {
                bail!("Counterpart changed; rescan");
            }
            (body, matched.path.display().to_string())
        }
        None => (String::new(), "(no counterpart)".into()),
    };
    let a: Vec<_> = source.split_inclusive('\n').collect();
    let b: Vec<_> = other.split_inclusive('\n').collect();
    let prefix = a.iter().zip(&b).take_while(|(a, b)| a == b).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let mut diff = format!("--- {}\n+++ {}\n", finding.path.display(), label);
    let mut truncated = false;
    if a != b {
        diff.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            if a.len() == suffix {
                prefix
            } else {
                prefix + 1
            },
            a.len() - prefix - suffix,
            if b.len() == suffix {
                prefix
            } else {
                prefix + 1
            },
            b.len() - prefix - suffix
        ));
        for (sign, lines) in [
            ('-', &a[prefix..a.len() - suffix]),
            ('+', &b[prefix..b.len() - suffix]),
        ] {
            for line in lines {
                if diff.len() > 40_000 {
                    truncated = true;
                    break;
                }
                diff.push(sign);
                diff.push_str(line);
                if !line.ends_with('\n') {
                    diff.push_str("\n\\ No newline at end of file\n");
                }
            }
        }
    }
    if truncated {
        diff.push_str("\n[Preview truncated; not an applicable patch]\n");
    }
    Ok(Preview {
        source,
        counterpart: other,
        diff,
        truncated,
    })
}

/// Does not initialise AppState or load/migrate the catalog.
pub fn cli(args: Vec<String>) -> i32 {
    let result = (|| -> Result<()> {
        let mut cfg = Config::default();
        let mut opts = Options::from_config(&cfg);
        let mut output = None;
        let mut assess_index: Option<usize> = None;
        let mut roots = Vec::new();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--compare" => {}
                "--root" => roots.push(PathBuf::from(
                    args.next().context("--root needs an absolute path")?,
                )),
                "--vault" => {
                    opts.vault = args
                        .next()
                        .context("--vault needs an absolute path")?
                        .into()
                }
                "--max-files" => {
                    opts.max_files = args.next().context("--max-files needs a count")?.parse()?
                }
                "--max-repo-files" => {
                    opts.max_repo_files = args
                        .next()
                        .context("--max-repo-files needs a count")?
                        .parse()?
                }
                "--output" => {
                    output = Some(PathBuf::from(
                        args.next().context("--output needs an absolute path")?,
                    ))
                }
                "--assess-index" => {
                    assess_index = Some(
                        args.next()
                            .context("--assess-index needs an index")?
                            .parse()?,
                    )
                }
                _ => bail!("Unknown comparison option: {arg}"),
            }
        }
        if !roots.is_empty() {
            opts.roots = roots;
        }
        cfg.vault = opts.vault.clone();
        let mut report = run(&cfg, &opts, &AtomicBool::new(false), |p| {
            eprintln!(
                "{}: {} indexed, {} compared, {} visited",
                p.phase, p.indexed, p.compared, p.visited
            );
        })?;
        if let Some(index) = assess_index {
            let finding = report
                .findings
                .get(index)
                .context("Assessment index is outside the comparison")?
                .clone();
            let path = finding.path.display().to_string();
            let assessment = tokio::runtime::Runtime::new()?
                .block_on(crate::recovery::assess(finding, cfg.clone()))?;
            report.assessments.insert(path, assessment);
        }
        if let Some(path) = output {
            checked_path(&path, true)?;
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)?;
            serde_json::to_writer_pretty(&mut file, &report)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            println!("Private comparison report: {}", path.display());
        }
        let mut counts = BTreeMap::new();
        for finding in &report.findings {
            *counts.entry(finding.kind.as_str()).or_insert(0) += 1;
        }
        println!(
            "{} repositories; {} findings; {} protected source trees; partial={}; cancelled={}",
            report.repositories.len(),
            report.findings.len(),
            report.protected_trees.len(),
            report.limited,
            report.cancelled
        );
        println!("Finding counts: {}", serde_json::to_string(&counts)?);
        println!(
            "Skipped counts: {}",
            serde_json::to_string(&report.skipped)?
        );
        for warning in &report.warnings {
            eprintln!("Coverage warning: {warning}");
        }
        Ok(())
    })();
    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Read-only comparison failed: {e}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    pub fn fixture() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "librarian-compare-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
    fn repo(root: &Path, name: &str, body: &str) -> PathBuf {
        let p = root.join(name);
        std::fs::create_dir_all(&p).unwrap();
        assert!(Command::new("git")
            .arg("init")
            .arg("-q")
            .arg(&p)
            .status()
            .unwrap()
            .success());
        std::fs::write(p.join("launch.sh"), body).unwrap();
        assert!(Command::new("git")
            .arg("-C")
            .arg(&p)
            .args(["add", "launch.sh"])
            .status()
            .unwrap()
            .success());
        p
    }
    #[test]
    fn compares_without_dates_or_mutations_and_preserves_broken_trees() {
        let d = fixture();
        let home = d.join("home");
        let vault = d.join("repos");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&vault).unwrap();
        let body = "# Launch OpenWrt LuCI tunnel\nssh -N -L 8101:10.10.0.1:80 tiamat\n";
        repo(&vault, "current", body);
        std::fs::write(home.join("launch.sh"), body).unwrap();
        let old = std::time::UNIX_EPOCH + std::time::Duration::from_secs(946684800);
        std::fs::File::options()
            .write(true)
            .open(home.join("launch.sh"))
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(old))
            .unwrap();
        let tree = home.join("nexus-os");
        std::fs::create_dir_all(&tree).unwrap();
        std::fs::write(tree.join(".git"), "gitdir: /missing").unwrap();
        std::fs::write(tree.join("launch.sh"), body).unwrap();
        std::fs::write(home.join("App passwords.md"), "do not read").unwrap();
        let cfg = Config {
            roots: vec![home.clone()],
            vault: vault.clone(),
            ..Config::default()
        };
        let r = run(
            &cfg,
            &Options::from_config(&cfg),
            &AtomicBool::new(false),
            |_| {},
        )
        .unwrap();
        assert_eq!(r.findings.len(), 2);
        assert_eq!(r.findings[0].kind, "identical");
        assert!(r
            .findings
            .iter()
            .any(|f| f.protected_tree == Some(tree.clone())));
        assert_eq!(
            std::fs::read_to_string(home.join("launch.sh")).unwrap(),
            body
        );
        let mut finding = r.findings[0].clone();
        finding.modified = 1.0;
        assert_eq!(preview(&finding, Some(0)).unwrap().source, body);
        let no_newline = body.trim_end_matches('\n');
        std::fs::write(&finding.matches[0].path, no_newline).unwrap();
        finding.matches[0].sha256 = hash(no_newline.as_bytes());
        assert!(preview(&finding, Some(0))
            .unwrap()
            .diff
            .contains("\\ No newline at end of file"));
        std::fs::write(&finding.path, "changed").unwrap();
        assert!(preview(&finding, Some(0)).is_err());
        std::fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn refuses_symlinks_and_credentials() {
        let d = fixture();
        let p = d.join("plain.sh");
        std::fs::write(&p, "echo safe").unwrap();
        let link = d.join("link");
        std::os::unix::fs::symlink(&p, &link).unwrap();
        assert!(checked_path(&link, false).is_err());
        assert!(secret_text("API_KEY=not-a-real-test-token"));
        assert!(secret_path(Path::new("/tmp/credentials.md")));
        assert!(!secret_text("echo safe"));
        std::fs::remove_dir_all(d).unwrap();
    }
    #[test]
    fn cancellation_and_limits_are_visible() {
        let d = fixture();
        let vault = d.join("repos");
        std::fs::create_dir_all(&vault).unwrap();
        let cfg = Config {
            roots: vec![d.clone()],
            vault,
            ..Config::default()
        };
        let r = run(
            &cfg,
            &Options::from_config(&cfg),
            &AtomicBool::new(true),
            |_| {},
        )
        .unwrap();
        assert!(r.cancelled);
        assert!(r.findings.is_empty());
        std::fs::remove_dir_all(d).unwrap();
    }

    #[test]
    fn canonical_lineage_is_explicit_and_excludes_the_credential_section() {
        let d = fixture();
        let docs = d.join("nexus-os/docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("NEXUS-OS-SOURCE-OF-TRUTH.md"),
            "# Canonical\n## 1. Component hierarchy\n- **nexus-mediastack** — **CURRENT** media engine.\n- **bulletproof-mediastack** — **FORERUNNER** only.\n## 2. Hosts\nCurrent gateway.\n## 4. Credentials\nDO NOT INCLUDE THIS SECTION\n").unwrap();
        let cfg = Config {
            vault: d.clone(),
            ..Config::default()
        };
        let roles = repo_roles(&cfg);
        assert_eq!(roles["nexus-mediastack"], "current module");
        assert_eq!(roles["bulletproof-mediastack"], "historical reference");
        assert!(!canonical_guidance(&cfg).unwrap().contains("DO NOT INCLUDE"));
        std::fs::remove_dir_all(d).unwrap();
    }
}
