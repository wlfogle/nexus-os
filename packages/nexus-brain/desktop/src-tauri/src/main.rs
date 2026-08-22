// Suppress the extra console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // `--headless` runs the same pipeline the window drives, printing progress
    // instead of emitting events. Useful from cron, and it makes the engine
    // verifiable without a display.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |flags: &[&str]| args.iter().any(|a| flags.contains(&a.as_str()));

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

    if has(&["--headless", "--run", "-H"]) {
        std::process::exit(librarian_lib::run_headless());
    }

    librarian_lib::run()
}
