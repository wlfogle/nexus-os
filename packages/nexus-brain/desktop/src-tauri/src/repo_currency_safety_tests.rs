//! Static safety checks for the repo currency feature (docsync + code sweep).
//!
//! Kept in a file of its own, separate from the modules it inspects, because
//! `include_str!`-scanning a file for a forbidden pattern from *within that
//! same file* is self-defeating: the pattern this test searches for would
//! then also match the test's own source defining what to search for.

#[cfg(test)]
mod tests {
    const DOCSYNC: &str = include_str!("docsync.rs");
    const CODE_SWEEP: &str = include_str!("code_sweep.rs");
    const REPO_DIGEST: &str = include_str!("repo_digest.rs");
    const ENVIRONMENT_DRIFT: &str = include_str!("environment_drift.rs");

    /// Every file in this feature follows the codebase-wide convention of
    /// putting `#[cfg(test)] mod tests { .. }` (and, for `repo_digest.rs`,
    /// a `test_support` fixture module) after all production code. Test
    /// fixtures legitimately call `git add`/`git commit` to build a real
    /// on-disk repo to test against -- that is not the production feature
    /// doing it, so the safety scan below only ever looks at the code
    /// *before* the first `#[cfg(test)]`.
    fn production_code(src: &str) -> &str {
        match src.find("#[cfg(test)]") {
            Some(idx) => &src[..idx],
            None => src,
        }
    }

    /// Builds a quoted-argument pattern (e.g. `"commit"`) at runtime so this
    /// file's own source never contains the literal 4-character sequence it
    /// is searching for.
    fn quoted_arg(word: &str) -> String {
        let q = '"';
        format!("{q}{word}{q}")
    }

    #[test]
    fn feature_never_stages_commits_or_pushes() {
        for (name, src) in [
            ("docsync.rs", DOCSYNC),
            ("code_sweep.rs", CODE_SWEEP),
            ("repo_digest.rs", REPO_DIGEST),
            ("environment_drift.rs", ENVIRONMENT_DRIFT),
        ] {
            let prod = production_code(src);
            for word in ["commit", "push", "add"] {
                let pattern = quoted_arg(word);
                assert!(
                    !prod.contains(&pattern),
                    "{name} must never stage, commit, or push -- found {pattern:?}"
                );
            }
        }
    }

    #[test]
    fn only_run_code_relocation_writes_or_moves_code_content() {
        // Content-mutating filesystem calls are not permitted anywhere in
        // the code-sweep module's production logic: findings are
        // report-only, and the one exception (`relocate`, backing
        // `run_code_relocation`) moves a file via `git mv`/`fs::rename`
        // without ever rewriting its bytes.
        let prod = production_code(CODE_SWEEP);
        for forbidden in ["fs::write(", "write_all(", "std::fs::write("] {
            assert!(
                !prod.contains(forbidden),
                "code_sweep.rs must never write code content, found {forbidden:?}"
            );
        }
    }

    #[test]
    fn docsync_is_the_only_module_that_writes_file_content() {
        // Doc content writes are allowed only in docsync.rs (journalled,
        // direct working-tree writes are the documented behaviour for the
        // docs tier). The shared digest/drift modules only ever read.
        for (name, src) in [
            ("repo_digest.rs", REPO_DIGEST),
            ("environment_drift.rs", ENVIRONMENT_DRIFT),
        ] {
            let prod = production_code(src);
            for forbidden in ["fs::write(", "write_all(", "std::fs::write("] {
                assert!(
                    !prod.contains(forbidden),
                    "{name} must never write file content, found {forbidden:?}"
                );
            }
        }
    }
}
