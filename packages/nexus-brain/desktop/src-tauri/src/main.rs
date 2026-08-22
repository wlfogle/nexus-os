// Suppress the extra console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // `--headless` runs the same pipeline the window drives, printing progress
    // instead of emitting events. Useful from cron, and it makes the engine
    // verifiable without a display.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |flags: &[&str]| args.iter().any(|a| flags.contains(&a.as_str()));
    let arg_value = |flag: &str| -> Option<String> {
        args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
    };

    // Re-derive labels, repo fingerprints, supersession edges and the notes
    // index from existing rows. No inference, so it finishes in seconds.
    if has(&["--classify", "-C"]) {
        std::process::exit(librarian_lib::run_classify());
    }

    // Read-only: walk the configured roots (plus the vault and monorepo) and
    // print every git working tree found, without touching the database or
    // reading any file content. Exists to verify repo discovery in isolation
    // from the rest of the pipeline, which can otherwise take hours.
    if has(&["--list-repos"]) {
        std::process::exit(librarian_lib::run_list_repos());
    }

    // Repo currency, docs tier: read every repo's real file tree/history and
    // write accurate documentation straight into the working tree (never
    // staged, committed, or pushed). `--repo <path>` scopes to one repo;
    // `--all` runs every repo Librarian already knows about.
    if has(&["--docsync"]) {
        let repo = arg_value("--repo");
        if repo.is_none() && !has(&["--all"]) {
            eprintln!("librarian: --docsync requires --repo <path> or --all");
            std::process::exit(2);
        }
        std::process::exit(librarian_lib::run_docsync_cli(repo));
    }

    // Repo currency, code tier: read-only. Prints environment-drift,
    // unreferenced-file, and docs-contradiction findings; never moves or
    // edits anything (that only happens via the `run_code_relocation`
    // command from the window UI).
    if has(&["--code-sweep"]) {
        let repo = arg_value("--repo");
        if repo.is_none() && !has(&["--all"]) {
            eprintln!("librarian: --code-sweep requires --repo <path> or --all");
            std::process::exit(2);
        }
        std::process::exit(librarian_lib::run_code_sweep_cli(repo));
    }

    if has(&["--headless", "--run", "-H"]) {
        std::process::exit(librarian_lib::run_headless());
    }

    librarian_lib::run()
}
