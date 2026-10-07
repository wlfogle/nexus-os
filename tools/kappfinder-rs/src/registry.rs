//! An index of the `.desktop` entries already present on the system.
//!
//! This is how the tool answers its central question: "is this installed
//! program already reachable from the application menu?"

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::desktop;

/// `$XDG_DATA_HOME`, defaulting to `~/.local/share`.
pub fn data_home() -> PathBuf {
    if let Some(v) = env::var_os("XDG_DATA_HOME") {
        let p = PathBuf::from(v);
        if p.is_absolute() {
            return p;
        }
    }
    home_dir().join(".local/share")
}

/// `$XDG_CONFIG_HOME`, defaulting to `~/.config`.
pub fn config_home() -> PathBuf {
    if let Some(v) = env::var_os("XDG_CONFIG_HOME") {
        let p = PathBuf::from(v);
        if p.is_absolute() {
            return p;
        }
    }
    home_dir().join(".config")
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// `$XDG_DATA_DIRS`, defaulting to the spec's `/usr/local/share:/usr/share`.
fn data_dirs() -> Vec<PathBuf> {
    match env::var_os("XDG_DATA_DIRS") {
        Some(v) if !v.is_empty() => env::split_paths(&v).collect(),
        _ => vec![
            PathBuf::from("/usr/local/share"),
            PathBuf::from("/usr/share"),
        ],
    }
}

/// The user-writable directory new entries go into by default.
pub fn user_applications_dir() -> PathBuf {
    data_home().join("applications")
}

/// Every directory that may contain menu entries, most-specific first.
pub fn application_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![user_applications_dir()];
    for base in data_dirs() {
        dirs.push(base.join("applications"));
    }
    // Flatpak and Snap exports are usually already on XDG_DATA_DIRS, but add
    // them explicitly so we do not offer duplicates when they are not.
    dirs.push(data_home().join("flatpak/exports/share/applications"));
    dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    dirs.push(PathBuf::from("/var/lib/snapd/desktop/applications"));

    let mut seen = Vec::new();
    for dir in dirs {
        if !seen.contains(&dir) {
            seen.push(dir);
        }
    }
    seen
}

/// Which programs already have a menu entry.
#[derive(Debug, Default)]
pub struct Registry {
    /// program basename -> the entry that launches it
    by_program: HashMap<String, PathBuf>,
    /// desktop file stem -> path, as a fallback match
    by_stem: HashMap<String, PathBuf>,
    /// Entries carrying our own generated marker.
    pub generated: Vec<PathBuf>,
    /// Directories that actually existed and were read.
    pub scanned_dirs: Vec<PathBuf>,
    /// Total `.desktop` files parsed.
    pub total_entries: usize,
}

impl Registry {
    /// Walk every application directory and build the index.
    ///
    /// When `ignore_nodisplay` is true, entries marked `NoDisplay=true` or
    /// `Hidden=true` are treated as absent, so the tool will offer to create
    /// a visible replacement for them.
    pub fn build(extra_dir: Option<&Path>, ignore_nodisplay: bool) -> Self {
        let mut registry = Registry::default();

        let mut dirs = application_dirs();
        if let Some(extra) = extra_dir {
            let extra = extra.to_path_buf();
            if !dirs.contains(&extra) {
                dirs.insert(0, extra);
            }
        }

        for dir in dirs {
            if !dir.is_dir() {
                continue;
            }
            registry.scanned_dirs.push(dir.clone());

            for entry in WalkDir::new(&dir)
                .follow_links(true)
                .max_depth(8)
                .into_iter()
                .filter_map(Result::ok)
            {
                let path = entry.path();
                if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                    continue;
                }
                let Some(parsed) = desktop::parse(path) else {
                    continue;
                };
                registry.total_entries += 1;

                if parsed.generated {
                    registry.generated.push(path.to_path_buf());
                }

                // Only Application entries launch programs. An empty Type is
                // malformed but common enough in the wild to accept.
                if !parsed.entry_type.is_empty() && parsed.entry_type != "Application" {
                    continue;
                }
                if ignore_nodisplay && !parsed.visible() {
                    continue;
                }

                if let Some(program) = parsed.program() {
                    registry
                        .by_program
                        .entry(program)
                        .or_insert_with(|| path.to_path_buf());
                }
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    registry
                        .by_stem
                        .entry(stem.to_string())
                        .or_insert_with(|| path.to_path_buf());
                }
            }
        }

        registry
    }

    /// The entry that already covers `bin`, if any.
    ///
    /// Matching is by launched program first, then by desktop-file stem, which
    /// catches entries whose `Exec` is too convoluted to parse.
    pub fn covers(&self, bin: &str) -> Option<&Path> {
        if let Some(p) = self.by_program.get(bin) {
            return Some(p.as_path());
        }
        self.by_stem.get(bin).map(|p| p.as_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_dirs_include_user_dir() {
        let dirs = application_dirs();
        assert!(dirs.contains(&user_applications_dir()));
    }

    #[test]
    fn application_dirs_are_unique() {
        let dirs = application_dirs();
        let mut sorted = dirs.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), dirs.len(), "duplicate application dirs");
    }

    #[test]
    fn xdg_paths_are_absolute() {
        assert!(data_home().is_absolute());
        assert!(config_home().is_absolute());
    }

    #[test]
    fn building_the_registry_does_not_panic() {
        // The real system index; content varies by machine, so only assert
        // that it completes and reports the directories it read.
        let reg = Registry::build(None, false);
        assert!(reg.total_entries >= reg.generated.len());
    }
}
