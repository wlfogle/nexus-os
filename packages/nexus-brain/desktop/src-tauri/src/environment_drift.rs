//! Environment-drift signal detection.
//!
//! Superseded-environment markers (`pacman`, `paru`, `yay`, `/etc/pacman.conf`,
//! AUR references, Calamares, Garuda-specific paths -- this machine ran
//! Garuda Linux before Pop!_OS) are not automatically wrong: a repo whose
//! entire purpose is Arch/Garuda packaging is *supposed* to mention them, and
//! a script with a multi-distro dispatch table using them as one branch
//! alongside `apt`/`dnf`/`nala` is not drift either. This module surfaces
//! concrete, contextual matches as *evidence* for the docsync/code-sweep LLM
//! prompts -- it never asserts drift as a verdict from grep alone.

const SUPERSEDED_MARKERS: &[&str] = &[
    "pacman", "paru", "yay", "/etc/pacman.conf", "pacman.conf", "aur", "calamares", "garuda",
];

/// Tokens that indicate a currently-supported package manager is present
/// nearby -- evidence of an intentional multi-distro dispatch rather than a
/// leftover single-distro assumption.
const CURRENT_MANAGER_TOKENS: &[&str] = &[
    "apt-get", "apt install", "apt update", "apt-cache", "dnf install", "dnf ", "nala ",
    "zypper", "apk add",
];

/// Lines of context inspected on each side of a match before deciding
/// whether a fallback to a currently-supported manager exists nearby.
const CONTEXT_WINDOW: usize = 8;

/// Repo-level declarations that mean the markers below are the *point* of
/// the project, not a leftover from an abandoned environment.
const ARCH_PACKAGING_DECLARATIONS: &[&str] = &[
    "garuda linux", "arch linux", "archlinux", "aur helper", "aur package",
    "pacman wrapper", "pacman hook", "for garuda", "for arch linux", "garuda-specific",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftMatch {
    pub file_path: String,
    pub line_number: usize,
    pub marker: String,
    pub snippet: String,
    /// True when no other-manager fallback was found nearby and the repo
    /// does not declare itself an Arch/Garuda packaging project -- i.e. this
    /// is real evidence of stale tooling, not an intentional reference.
    pub likely_drift: bool,
}

/// Heuristic: does this repo's own name or documentation declare it IS an
/// Arch/Garuda-focused project, i.e. these markers are the point, not drift?
pub fn repo_is_intentionally_arch_focused(repo_name: &str, doc_text: &str) -> bool {
    let hay = format!("{} {}", repo_name.to_lowercase(), doc_text.to_lowercase());
    ARCH_PACKAGING_DECLARATIONS.iter().any(|d| hay.contains(d))
}

fn line_has_current_manager(line: &str) -> bool {
    let l = line.to_lowercase();
    CURRENT_MANAGER_TOKENS.iter().any(|t| l.contains(t))
}

/// Scan one file's text content for superseded-environment markers.
///
/// `repo_is_packaging_project` should come from
/// `repo_is_intentionally_arch_focused`, computed once per repo.
pub fn scan_file(file_path: &str, content: &str, repo_is_packaging_project: bool) -> Vec<DriftMatch> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        let Some(marker) = SUPERSEDED_MARKERS.iter().find(|m| lower.contains(**m)) else {
            continue;
        };
        let start = i.saturating_sub(CONTEXT_WINDOW);
        let end = (i + CONTEXT_WINDOW + 1).min(lines.len());
        let window = &lines[start..end];
        let has_fallback = window.iter().any(|l| line_has_current_manager(l));
        let likely_drift = !repo_is_packaging_project && !has_fallback;
        out.push(DriftMatch {
            file_path: file_path.to_string(),
            line_number: i + 1,
            marker: (*marker).to_string(),
            snippet: window.join("\n"),
            likely_drift,
        });
    }
    out
}

/// Scan every file in `files` (repo-relative path, content) for drift
/// markers. `doc_text` is the repo's own concatenated documentation, used
/// once to decide whether the repo is intentionally Arch/Garuda-focused.
pub fn scan_repo(repo_name: &str, doc_text: &str, files: &[(String, String)]) -> Vec<DriftMatch> {
    let packaging = repo_is_intentionally_arch_focused(repo_name, doc_text);
    let mut out = Vec::new();
    for (path, content) in files {
        out.extend(scan_file(path, content, packaging));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{repo_is_intentionally_arch_focused, scan_file, scan_repo};

    #[test]
    fn bare_pacman_call_with_no_fallback_is_drift() {
        let content = "\
#!/bin/bash
echo installing dependencies
pacman -S --noconfirm base-devel jellyfin
echo done
";
        let matches = scan_file("install.sh", content, false);
        assert_eq!(matches.len(), 1);
        assert!(matches[0].likely_drift, "bare pacman call must be flagged as drift");
        assert_eq!(matches[0].marker, "pacman");
        assert_eq!(matches[0].line_number, 3);
    }

    #[test]
    fn multi_distro_dispatch_table_is_not_drift() {
        let content = "\
#!/bin/bash
case \"$DISTRO\" in
  arch|garuda)
    pacman -S --noconfirm \"$pkg\"
    ;;
  debian|ubuntu|pop)
    apt-get install -y \"$pkg\"
    ;;
  fedora)
    dnf install -y \"$pkg\"
    ;;
esac
";
        let matches = scan_file("install.sh", content, false);
        // Two markers appear (`garuda` in the case label, `pacman` in the
        // branch body); neither should be flagged since a current-manager
        // fallback sits in the same dispatch table.
        assert_eq!(matches.len(), 2);
        assert!(
            matches.iter().all(|m| !m.likely_drift),
            "a dispatch table with a current-manager fallback nearby must not be flagged: {matches:?}"
        );
    }

    #[test]
    fn a_project_that_is_itself_an_arch_packaging_tool_is_not_drift() {
        let content = "pacman -S --noconfirm base-devel\nmakepkg -si\n";
        let matches = scan_file("build.sh", content, /* repo_is_packaging_project */ true);
        assert_eq!(matches.len(), 1);
        assert!(
            !matches[0].likely_drift,
            "an intentional Arch/Garuda packaging project must not be flagged"
        );
    }

    #[test]
    fn repo_declaring_itself_arch_focused_is_detected() {
        assert!(repo_is_intentionally_arch_focused(
            "garuda-assistant",
            "A settings assistant for Garuda Linux."
        ));
        assert!(repo_is_intentionally_arch_focused(
            "my-tool",
            "This is an AUR helper written in Rust."
        ));
        assert!(!repo_is_intentionally_arch_focused(
            "nexus-os",
            "A from-scratch Rust microkernel for Pop!_OS development."
        ));
    }

    #[test]
    fn scan_repo_aggregates_across_files_and_applies_the_packaging_check_once() {
        let files = vec![
            ("install.sh".to_string(), "pacman -S foo\n".to_string()),
            ("README.md".to_string(), "Some unrelated docs.\n".to_string()),
        ];
        let drifted = scan_repo("some-tool", "not an arch project", &files);
        assert_eq!(drifted.len(), 1);
        assert!(drifted[0].likely_drift);

        let not_drifted = scan_repo("garuda-tool", "Built for Garuda Linux.", &files);
        assert_eq!(not_drifted.len(), 1);
        assert!(!not_drifted[0].likely_drift);
    }

    #[test]
    fn calamares_and_aur_markers_are_also_detected() {
        assert_eq!(scan_file("f.sh", "run calamares installer\n", false).len(), 1);
        assert_eq!(scan_file("f.md", "install it from the AUR\n", false).len(), 1);
    }
}
