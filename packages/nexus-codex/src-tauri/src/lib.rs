mod commands;
mod config;
mod analyzer;
mod github;
mod ollama;
mod pdf;
mod report;
mod repo;
mod scanner;
mod state;
mod types;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Load persisted config (falls back to defaults if absent)
            let cfg = config::load_config().unwrap_or_default();
            app.manage(std::sync::Arc::new(std::sync::Mutex::new(cfg)));
            app.manage(state::new_registry());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_ollama_models,
            commands::get_config,
            commands::save_config,
            commands::start_scan,
            commands::cancel_scan,
            commands::get_scan_status,
            commands::get_report,
            commands::export_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running nexus-codex");
}
