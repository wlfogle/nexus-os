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

use clevo_hw::flexikey::{FlexikeyEngine, Profile, ProfilesIndex};
use clevo_hw::{Color, PowerReader, RgbController, SensorReader};
use serde::Serialize;

/// Shared application state, held once and handed to every command via
/// Tauri's managed-state mechanism.
pub struct AppState {
    pub rgb: Arc<RgbController>,
    pub sensors: Arc<SensorReader>,
    pub power: Arc<PowerReader>,
    pub flexikey: Arc<FlexikeyEngine>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            rgb: Arc::new(RgbController::new()),
            sensors: Arc::new(SensorReader::new()),
            power: Arc::new(PowerReader::new()),
            flexikey: Arc::new(FlexikeyEngine::new()),
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

// ---------------------------------------------------------------------------
// Flexikey commands (flexikey-agent). Thin wrappers over `clevo_hw::flexikey`
// - profile I/O and keyboard grabbing are genuine blocking device/filesystem
// work, so every command runs inside `spawn_blocking` per the same pattern
// as the commands above.
// ---------------------------------------------------------------------------

#[tauri::command]
async fn list_flexikey_profiles() -> Result<ProfilesIndex, String> {
    tokio::task::spawn_blocking(clevo_hw::flexikey::load_profiles_index)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_flexikey_profile(name: String) -> Result<Profile, String> {
    tokio::task::spawn_blocking(move || clevo_hw::flexikey::load_profile(&name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_flexikey_profile(profile: Profile) -> Result<(), String> {
    tokio::task::spawn_blocking(move || clevo_hw::flexikey::save_profile(&profile))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_flexikey_profile(name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || clevo_hw::flexikey::delete_profile(&name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_active_flexikey_profile(name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || clevo_hw::flexikey::set_active_profile(&name))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn capture_next_key() -> Result<String, String> {
    tokio::task::spawn_blocking(clevo_hw::flexikey::capture_next_key)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_flexikey_engine(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let engine = state.flexikey.clone();
    tokio::task::spawn_blocking(move || engine.start())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_flexikey_engine(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let engine = state.flexikey.clone();
    tokio::task::spawn_blocking(move || engine.stop())
        .await
        .map_err(|e| e.to_string())?
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
            list_flexikey_profiles,
            get_flexikey_profile,
            save_flexikey_profile,
            delete_flexikey_profile,
            set_active_flexikey_profile,
            capture_next_key,
            start_flexikey_engine,
            stop_flexikey_engine,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the OriginPC Control Center application");
}
