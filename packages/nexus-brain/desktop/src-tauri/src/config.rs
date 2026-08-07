//! Runtime configuration.
//!
//! Defaults are derived from the machine this is built for: Ollama serving a
//! large local model library from an external drive, repos consolidated under
//! a vault on the Data partition, and a home directory that must be crawled
//! without wandering into caches, game data or VM images.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

pub fn state_dir() -> PathBuf {
    home().join(".local/state/librarian")
}

pub fn library_dir() -> PathBuf {
    home().join("Library/Librarian")
}

/// Which local model handles which kind of work.
///
/// Ollama keeps a model resident for `OLLAMA_KEEP_ALIVE`, and the GPU here has
/// 12 GB, so loading a different model per file would thrash constantly. The
/// engine therefore groups the work queue *by model* and drains one model at a
/// time. Cheap models do the bulk; the escalation ladder is only walked for
/// files the previous tier was not confident about.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRouting {
    /// 768-dim embeddings for semantic search and repo matching.
    pub embed: String,
    /// Fast first pass over ordinary text.
    pub triage: String,
    /// Source code, build files, shell scripts.
    pub code: String,
    /// Prose: notes, plans, documentation.
    pub docs: String,
    /// Screenshots and images (cheap).
    pub vision: String,
    /// Screenshots that the cheap vision model could not read.
    pub vision_escalate: String,
    /// Anything the routed model scored below `escalate_below`.
    pub escalate: String,
    /// Last resort for genuinely ambiguous items.
    pub escalate_max: String,
    /// Confidence under which a result is re-run on the next tier up.
    pub escalate_below: f32,
}

impl Default for ModelRouting {
    fn default() -> Self {
        Self {
            embed: "nomic-embed-text:latest".into(),
            triage: "llama3.2:3b".into(),
            code: "qwen2.5-coder:7b".into(),
            docs: "qwen2.5:7b".into(),
            vision: "moondream:latest".into(),
            vision_escalate: "llama3.2-vision:11b".into(),
            escalate: "codestral:22b".into(),
            escalate_max: "phi4:latest".into(),
            escalate_below: 0.55,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Directories crawled for content.
    pub roots: Vec<PathBuf>,
    /// Directory *names* pruned anywhere in the tree.
    pub prune_names: Vec<String>,
    /// Absolute path fragments pruned wherever they appear.
    pub prune_fragments: Vec<String>,
    /// Where standalone repos live, flat: `<vault>/<repo>`.
    pub vault: PathBuf,
    /// Monorepo that absorbs most projects as packages.
    pub monorepo: PathBuf,
    /// Managed library root (Inbox / Archive / Quarantine / Notes).
    pub library: PathBuf,
    /// Base URL of the Ollama server.
    pub ollama_url: String,
    pub models: ModelRouting,
    /// Files larger than this are catalogued but never read for content.
    pub max_read_bytes: u64,
    /// Actions at or above this confidence are applied without asking.
    pub auto_apply_above: f32,
    /// Age in days past which unreferenced material is considered stale.
    pub stale_days: i64,
    /// Parallel interpretation requests in flight against Ollama.
    pub interpret_concurrency: usize,
}

impl Default for Config {
    fn default() -> Self {
        let h = home();
        Self {
            roots: vec![h.clone()],
            prune_names: [
                // dependency and tool caches
                ".cache", ".npm", ".gradle", ".nvm", ".bun", ".yarn", ".m2",
                ".ivy2", ".rustup", ".cargo", ".pub-cache", "node_modules",
                "site-packages", "dist-packages", "__pycache__", ".venv",
                "venv", ".tox", ".mypy_cache", ".pytest_cache", ".ruff_cache",
                // build output
                "target", "build", "dist", "_internal", ".next",
                ".parcel-cache", "vendor",
                // large opaque application state
                ".steam", ".var", ".wine", ".wineprefixes", ".PlayOnLinux",
                ".Genymobile", ".android", "Android", "snap",
                "VirtualBox VMs", "redroid-data", ".redroid-data", ".skiko",
                ".qt", ".vnc", ".rpmdb", ".visicut", ".insomniac", "Faugus",
                // browser and editor profiles
                ".mozilla", ".waterfox", ".thunderbird", ".tor-browser",
                ".vscode", ".vscode-shared", ".vscode-react-native",
                // our own state, and git internals
                ".warp", ".xmltv", ".git",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            prune_fragments: ["/go/pkg/mod/", "/.nexus-consolidate-trash/"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            vault: PathBuf::from("/media/loufogle/Data/Repos"),
            monorepo: h.join("nexus-os"),
            library: h.join("Library/Librarian"),
            ollama_url: "http://127.0.0.1:11434".into(),
            models: ModelRouting::default(),
            max_read_bytes: 8 << 20,
            auto_apply_above: 0.85,
            stale_days: 180,
            interpret_concurrency: 4,
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        state_dir().join("config.json")
    }

    pub fn load() -> Result<Self> {
        let p = Self::path();
        if !p.exists() {
            let c = Self::default();
            c.save()?;
            return Ok(c);
        }
        let raw = std::fs::read_to_string(&p)
            .with_context(|| format!("reading {}", p.display()))?;
        Ok(serde_json::from_str(&raw).unwrap_or_default())
    }

    pub fn save(&self) -> Result<()> {
        let p = Self::path();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&p, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }

    /// True when this directory must not be descended into.
    pub fn is_pruned(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            if self.prune_names.iter().any(|p| p == name) {
                return true;
            }
        }
        let s = format!("{}/", path.to_string_lossy());
        self.prune_fragments.iter().any(|f| s.contains(f.as_str()))
    }

    /// Sub-directories of the managed library.
    pub fn library_buckets(&self) -> [(&'static str, PathBuf); 6] {
        [
            ("Inbox", self.library.join("Inbox")),
            ("Archive", self.library.join("Archive")),
            ("Quarantine", self.library.join("Quarantine")),
            ("Notes", self.library.join("Notes")),
            ("RepoRefs", self.library.join("RepoRefs")),
            ("Backups", self.library.join("Backups")),
        ]
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(state_dir())?;
        for (_, p) in self.library_buckets() {
            std::fs::create_dir_all(&p)?;
        }
        Ok(())
    }
}
