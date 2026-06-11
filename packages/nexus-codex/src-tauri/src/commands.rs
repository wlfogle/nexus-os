use std::sync::Arc;
use tauri::State;

use crate::state::{SharedConfig, SharedScanRegistry};
use crate::types::{Config, OllamaModel, Report, ScanStatus};

/// List all Ollama models available locally, with auto-selection scoring applied.
#[tauri::command]
pub async fn get_ollama_models(
    config: State<'_, SharedConfig>,
) -> Result<Vec<OllamaModel>, String> {
    let ollama_url = config
        .lock()
        .map_err(|e| e.to_string())?
        .ollama_url
        .clone();
    crate::ollama::list_models(&ollama_url)
        .await
        .map_err(|e| e.to_string())
}

/// Return the current application configuration.
#[tauri::command]
pub async fn get_config(config: State<'_, SharedConfig>) -> Result<Config, String> {
    Ok(config.lock().map_err(|e| e.to_string())?.clone())
}

/// Persist a new configuration (replaces in-memory state and writes to disk).
#[tauri::command]
pub async fn save_config(
    new_config: Config,
    config: State<'_, SharedConfig>,
) -> Result<(), String> {
    crate::config::save_config(&new_config).map_err(|e| e.to_string())?;
    *config.lock().map_err(|e| e.to_string())? = new_config;
    Ok(())
}

/// Start a full scan using the current configuration.
/// Returns the scan_id that can be used to poll status or retrieve the report.
/// Emits `scan-progress`, `scan-doc`, `scan-complete`, and `scan-error` events.
#[tauri::command]
pub async fn start_scan(
    app: tauri::AppHandle,
    config: State<'_, SharedConfig>,
    registry: State<'_, SharedScanRegistry>,
) -> Result<String, String> {
    let cfg = config.lock().map_err(|e| e.to_string())?.clone();
    let scan_id = uuid::Uuid::new_v4().to_string();
    let registry_arc = Arc::clone(registry.inner());
    crate::analyzer::start_scan(app, scan_id.clone(), cfg, registry_arc)
        .await
        .map_err(|e| e.to_string())?;
    Ok(scan_id)
}

/// Abort a running scan identified by scan_id.
#[tauri::command]
pub async fn cancel_scan(
    scan_id: String,
    registry: State<'_, SharedScanRegistry>,
) -> Result<(), String> {
    let mut reg = registry.lock().map_err(|e| e.to_string())?;
    if let Some(entry) = reg.scans.get_mut(&scan_id) {
        if let Some(handle) = entry.handle.take() {
            handle.abort();
        }
        entry.status.state = crate::types::ScanState::Cancelled;
    }
    Ok(())
}

/// Return the live status for a running or completed scan.
#[tauri::command]
pub async fn get_scan_status(
    scan_id: String,
    registry: State<'_, SharedScanRegistry>,
) -> Result<ScanStatus, String> {
    let reg = registry.lock().map_err(|e| e.to_string())?;
    reg.scans
        .get(&scan_id)
        .map(|e| e.status.clone())
        .ok_or_else(|| format!("scan {} not found", scan_id))
}

/// Retrieve the completed report for a finished scan.
/// Returns an error if the scan is not yet complete.
#[tauri::command]
pub async fn get_report(
    scan_id: String,
    registry: State<'_, SharedScanRegistry>,
) -> Result<Report, String> {
    let reg = registry.lock().map_err(|e| e.to_string())?;
    let entry = reg
        .scans
        .get(&scan_id)
        .ok_or_else(|| format!("scan {} not found", scan_id))?;
    if entry.status.state != crate::types::ScanState::Complete {
        return Err("scan is not complete yet".to_string());
    }
    entry
        .report
        .clone()
        .ok_or_else(|| "report not available".to_string())
}

/// Export an existing report to disk in the requested format ("markdown" or "json").
/// Returns the absolute path of the written file.
#[tauri::command]
pub async fn export_report(
    scan_id: String,
    format: String,
    output_path: String,
    registry: State<'_, SharedScanRegistry>,
) -> Result<String, String> {
    let report = {
        let reg = registry.lock().map_err(|e| e.to_string())?;
        reg.scans
            .get(&scan_id)
            .and_then(|e| e.report.clone())
            .ok_or_else(|| format!("no completed report for scan {}", scan_id))?
    };
    match format.as_str() {
        "markdown" => crate::report::export_markdown(&report, &output_path)
            .map_err(|e| e.to_string()),
        "json" => crate::report::export_json(&report, &output_path)
            .map_err(|e| e.to_string()),
        other => Err(format!("unknown export format: {}", other)),
    }
}
