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

/// Bump when the built-in prune lists change, so an existing `config.json`
/// picks up the new entries instead of silently pinning the old ones.
pub const CONFIG_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Schema version of the persisted file. Absent in v1 files, hence `default`.
    #[serde(default)]
    pub version: u32,
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
            version: CONFIG_VERSION,
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
                // Per-user application state. `.local/share` alone contributes
                // six figures of icon themes, shell completions, flatpak
                // appstream data and shader caches -- none of it authored by
                // the user, all of it noise in a catalogue of ideas.
                ".local", ".config", ".icons", ".themes", ".fonts",
                ".thumbnails", ".pulse", ".dbus", ".gvfs", ".java",
                // Credentials. Never walked, never read, never sent anywhere.
                ".ssh", ".gnupg", ".pki", ".aws", ".kube", ".docker",
                ".password-store", ".secrets", ".netrc",
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
        let mut cfg: Self = serde_json::from_str(&raw).unwrap_or_default();
        if cfg.migrate() {
            cfg.save()?;
        }
        Ok(cfg)
    }

    /// Fold newly shipped defaults into an older config file.
    ///
    /// Prune lists are additive: entries the user added are kept, and entries
    /// added to the built-in defaults since the file was written are merged in.
    /// Without this, changing a default has no effect on any machine that has
    /// already run the app once -- which is exactly how `.local` and `.config`
    /// kept getting walked after being added to the deny list.
    ///
    /// Returns true when something changed and the file should be rewritten.
    fn migrate(&mut self) -> bool {
        if self.version >= CONFIG_VERSION {
            return false;
        }
        let defaults = Self::default();

        let merge = |current: &mut Vec<String>, incoming: &[String]| {
            for item in incoming {
                if !current.iter().any(|c| c == item) {
                    current.push(item.clone());
                }
            }
        };
        merge(&mut self.prune_names, &defaults.prune_names);
        merge(&mut self.prune_fragments, &defaults.prune_fragments);

        self.version = CONFIG_VERSION;
        true
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

#[cfg(test)]
mod tests {
    use super::{Config, CONFIG_VERSION};
    use std::path::PathBuf;

    #[test]
    fn migration_adds_new_defaults_and_keeps_user_entries() {
        // A v1 file: no version field, short prune list, one custom entry.
        let mut old = Config {
            version: 0,
            prune_names: vec!["node_modules".into(), "my-own-junk-dir".into()],
            prune_fragments: vec!["/custom/fragment/".into()],
            ..Config::default()
        };

        assert!(old.migrate(), "a v1 config must report that it changed");
        assert_eq!(old.version, CONFIG_VERSION);

        // The entries that were silently missing before are now present.
        for expected in [".local", ".config", ".ssh", ".icons"] {
            assert!(
                old.prune_names.iter().any(|p| p == expected),
                "{expected} should have been merged in"
            );
        }
        // The user's own additions survive.
        assert!(old.prune_names.iter().any(|p| p == "my-own-junk-dir"));
        assert!(old.prune_fragments.iter().any(|p| p == "/custom/fragment/"));
    }

    #[test]
    fn migration_is_idempotent_and_does_not_duplicate() {
        let mut c = Config::default();
        assert!(!c.migrate(), "a current config needs no migration");
        let before = c.prune_names.len();
        c.version = 0;
        c.migrate();
        assert_eq!(
            c.prune_names.len(),
            before,
            "re-migrating must not duplicate entries"
        );
    }

    #[test]
    fn credential_and_state_dirs_are_pruned() {
        let c = Config::default();
        for dir in [".ssh", ".gnupg", ".aws", ".local", ".config"] {
            let p = super::home().join(dir);
            assert!(c.is_pruned(&p), "{dir} must be pruned");
        }
    }

    #[test]
    fn ordinary_project_dirs_are_not_pruned() {
        let c = Config::default();
        for dir in ["nexus-os", "Documents", "scripts"] {
            let p = super::home().join(dir);
            assert!(!c.is_pruned(&p), "{dir} must not be pruned");
        }
    }

    #[test]
    fn path_fragments_are_pruned_anywhere() {
        let c = Config::default();
        assert!(c.is_pruned(&PathBuf::from("/home/x/go/pkg/mod/foo")));
    }
}
