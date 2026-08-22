//! Librarian: a repo-aware ecosystem librarian.
//!
//! Pipeline tiers, in order:
//!   0 `scan`      inventory the filesystem (stat only)
//!   1 `extract`   read bytes, hash, render to text
//!   2 `embed`     chunk and embed with a local model
//!   3 `interpret` have a local LLM read each file and judge it
//!
//! `repos` supplies ownership, `search` answers questions over the result, and
//! `actions` turns judgements into reversible filesystem changes.

pub mod actions;
pub mod classify;
pub mod commands;
pub mod config;
pub mod db;
pub mod embed;
pub mod engine;
pub mod extract;
pub mod interpret;
pub mod notes;
pub mod ollama;
pub mod repos;
pub mod scan;
pub mod search;

use std::sync::Arc;

/// Run the pipeline to completion with no window, printing progress.
///
/// This is the same `run_pipeline` the window drives, differing only in the
/// progress sink. Having it means the pipeline can be exercised end to end
/// without a display -- useful for cron, and the only honest way to verify the
/// engine does not deadlock.
pub fn run_headless() -> i32 {
    let state = match engine::AppState::new() {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("librarian: failed to initialise: {e}");
            return 1;
        }
    };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("librarian: cannot start runtime: {e}");
            return 1;
        }
    };

    let sink: Arc<dyn engine::ProgressSink> = Arc::new(engine::StdoutSink);
    match runtime.block_on(engine::run_pipeline(sink, state)) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("librarian: pipeline failed: {e}");
            1
        }
    }
}

/// Rebuild repo fingerprints, labels, supersession edges and the notes index
/// from whatever is already in the catalog.
///
/// The full pipeline only reaches this stage once every file has been
/// interpreted, which can take hours. These stages depend on nothing but
/// existing rows, so running them alone makes the results observable
/// immediately and lets the scoring thresholds be tuned without paying for
/// inference again.
pub fn run_classify() -> i32 {
    let state = match engine::AppState::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("librarian: failed to initialise: {e}");
            return 1;
        }
    };
    let cfg = state.config();
    let mut conn = match state.db.lock() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("librarian: database is poisoned: {e}");
            return 1;
        }
    };

    match embed::rebuild_centroids(&mut conn) {
        Ok(n) => println!("centroids : {n} repo(s)"),
        Err(e) => {
            eprintln!("centroids failed: {e}");
            return 1;
        }
    }
    match classify::run(&mut conn, &cfg) {
        Ok(r) => println!(
            "classify  : {} labelled, {} repo topics, {} superseded",
            r.classified, r.topics, r.supersedes
        ),
        Err(e) => {
            eprintln!("classify failed: {e}");
            return 1;
        }
    }
    match notes::reindex(&mut conn, &cfg) {
        Ok(n) => println!("notes     : {n} indexed"),
        Err(e) => {
            eprintln!("notes reindex failed: {e}");
            return 1;
        }
    }
    0
}

/// Read-only: discover every git repository under the configured roots (plus
/// the vault and monorepo) and print them. Touches no database and reads no
/// file content -- purely for verifying `repos::discover` in isolation.
pub fn run_list_repos() -> i32 {
    let cfg = match config::Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("librarian: failed to load config: {e}");
            return 1;
        }
    };
    let found = repos::discover(&cfg);
    for path in &found {
        println!("{}", path.display());
    }
    println!("---");
    println!("{} repositories found", found.len());
    0
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = match engine::AppState::new() {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("librarian: failed to initialise: {e}");
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_stats,
            commands::get_config,
            commands::save_config,
            commands::health,
            commands::list_models,
            commands::start_pipeline,
            commands::stop_pipeline,
            commands::list_catalog,
            commands::search_catalog,
            commands::list_stale,
            commands::list_repos,
            commands::list_duplicates,
            commands::list_review,
            commands::list_history,
            commands::decide_action,
            commands::apply_approved,
            commands::undo_plan,
            commands::similar_files,
            commands::read_file_text,
            commands::list_supersessions,
            commands::repo_topics,
            commands::list_notes,
            commands::get_note,
            commands::save_note,
            commands::delete_note,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Librarian");
}
