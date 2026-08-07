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
pub mod commands;
pub mod config;
pub mod db;
pub mod embed;
pub mod engine;
pub mod extract;
pub mod interpret;
pub mod ollama;
pub mod repos;
pub mod scan;
pub mod search;

use std::sync::Arc;

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Librarian");
}
