//! OriginPC Control Center Tauri backend.
//!
//! Every hardware-touching command is `async fn` with the actual device
//! I/O inside `tokio::task::spawn_blocking` - this is the structural fix
//! for the freeze bug found in the Python/PyQt5 version, where RGB writes
//! ran synchronously on the Qt GUI thread. A blocking hidraw write here can
//! only ever stall a threadpool worker, never the webview/event loop.
//!
//! See `../CONTRACT.md` for the full frozen command/event contract that
//! this file and the frontend both implement against. Commands beyond the
//! initial set below (effects, key bindings tab, fan control, power
//! profiles) are owned by backend-agent per the migration plan.

use std::sync::Arc;

use clevo_hw::{Color, PowerReader, RgbController, SensorReader};
use serde::Serialize;

mod hotkey_osd;

/// Shared application state, held once and handed to every command via
/// Tauri's managed-state mechanism.
pub struct AppState {
    pub rgb: Arc<RgbController>,
    pub sensors: Arc<SensorReader>,
    pub power: Arc<PowerReader>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            rgb: Arc::new(RgbController::new()),
            sensors: Arc::new(SensorReader::new()),
            power: Arc::new(PowerReader::new()),
        }
    }
}

#[derive(Serialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub device_path: Option<&'static str>,
}

#[tauri::command]
async fn get_connection_status(
    state: tauri::State<'_, AppState>,
) -> Result<ConnectionStatus, String> {
    let rgb = state.rgb.clone();
    tokio::task::spawn_blocking(move || ConnectionStatus {
        connected: rgb.is_connected(),
        device_path: rgb.current_device_path(),
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_key_color(
    state: tauri::State<'_, AppState>,
    key: String,
    r: u8,
    g: u8,
    b: u8,
) -> Result<(), String> {
    let rgb = state.rgb.clone();
    tokio::task::spawn_blocking(move || rgb.set_key_color(&key, Color::new(r, g, b)))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_group_color(
    state: tauri::State<'_, AppState>,
    group: String,
    r: u8,
    g: u8,
    b: u8,
) -> Result<(), String> {
    let rgb = state.rgb.clone();
    tokio::task::spawn_blocking(move || rgb.set_group_color(&group, Color::new(r, g, b)))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_all_keys(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let rgb = state.rgb.clone();
    tokio::task::spawn_blocking(move || rgb.clear_all_keys())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_sensor_snapshot(
    state: tauri::State<'_, AppState>,
) -> Result<clevo_hw::SensorSnapshot, String> {
    let sensors = state.sensors.clone();
    tokio::task::spawn_blocking(move || sensors.snapshot())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_power_info(state: tauri::State<'_, AppState>) -> Result<clevo_hw::PowerInfo, String> {
    let power = state.power.clone();
    tokio::task::spawn_blocking(move || power.snapshot())
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_connection_status,
            set_key_color,
            set_group_color,
            clear_all_keys,
            get_sensor_snapshot,
            get_power_info,
        ])
        .setup(|app| {
            // osd-lidmonitor-agent: background evdev listener that pops up
            // the `osd` window on Fn-hotkey presses. See hotkey_osd.rs and
            // CONTRACT.md's "osd-lidmonitor-agent" section.
            hotkey_osd::spawn(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the OriginPC Control Center application");
}
