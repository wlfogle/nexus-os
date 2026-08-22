//! OriginPC Control Center Tauri backend.
//!
//! Every hardware-touching command is `async fn` with the actual device
//! I/O inside `tokio::task::spawn_blocking` - this is the structural fix
//! for the freeze bug found in the Python/PyQt5 version, where RGB writes
//! ran synchronously on the Qt GUI thread. A blocking hidraw write here can
//! only ever stall a threadpool worker, never the webview/event loop.
//!
//! See `../CONTRACT.md` for the full frozen command/event contract that
//! this file and the frontend both implement against. Lighting effects
//! (`effects.rs`), fan-mode control (`fan.rs`), power profiles, and the
//! periodic `system-stats` push event are implemented here by
//! backend-agent; the Flexikey key-bindings commands are a separate
//! module owned by flexikey-agent per the migration plan.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use clevo_hw::flexikey::{FlexikeyEngine, Profile, ProfilesIndex};
use clevo_hw::{
    Color, PowerInfo, PowerProfile, PowerReader, RgbController, SensorReader, SensorSnapshot,
    SystemUsage, UsageReader,
};
use serde::Serialize;
use tauri::{Emitter, Manager};

mod effects;
mod fan;

mod hotkey_osd;
mod tray;

/// A running lighting-effect task plus its cooperative cancellation flag.
///
/// We deliberately don't rely on `JoinHandle::abort()` alone to stop the
/// task: aborting only takes effect at an `.await` point, so it can't
/// interrupt a frame write already in flight inside `spawn_blocking` on a
/// separate OS thread. `stop_running_effect` sets `cancel` and then awaits
/// `join`, guaranteeing the effect loop has fully stopped (including any
/// in-flight write) before a subsequent `clear_all_keys()` can run - see
/// `effects::run`'s doc comment for the full race this avoids.
struct RunningEffect {
    cancel: Arc<AtomicBool>,
    join: tokio::task::JoinHandle<()>,
}

/// Shared application state, held once and handed to every command via
/// Tauri's managed-state mechanism.
pub struct AppState {
    pub rgb: Arc<RgbController>,
    pub sensors: Arc<SensorReader>,
    pub power: Arc<PowerReader>,
    pub usage: Arc<UsageReader>,
    /// The currently running lighting effect task (if any), so
    /// `start_effect`/`stop_effect` can replace or stop it without leaking
    /// the previous animation loop.
    effect_handle: Mutex<Option<RunningEffect>>,
    pub flexikey: Arc<FlexikeyEngine>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            rgb: Arc::new(RgbController::new()),
            sensors: Arc::new(SensorReader::new()),
            power: Arc::new(PowerReader::new()),
            usage: Arc::new(UsageReader::new()),
            effect_handle: Mutex::new(None),
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
async fn get_power_info(state: tauri::State<'_, AppState>) -> Result<PowerInfo, String> {
    let power = state.power.clone();
    tokio::task::spawn_blocking(move || power.snapshot())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_system_usage(state: tauri::State<'_, AppState>) -> Result<SystemUsage, String> {
    let usage = state.usage.clone();
    tokio::task::spawn_blocking(move || usage.snapshot())
        .await
        .map_err(|e| e.to_string())
}

/// Text output for the "TLP Stats" detail view - see
/// `PowerReader::tlp_stats`'s doc comment for why this deliberately does
/// not reproduce the Python app's `sudo tlp-stat` behavior.
#[tauri::command]
async fn get_tlp_stats() -> Result<String, String> {
    tokio::task::spawn_blocking(PowerReader::tlp_stats)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn start_effect(
    state: tauri::State<'_, AppState>,
    effect: String,
    speed: f64,
) -> Result<(), String> {
    let kind = effects::Effect::parse(&effect)?;
    // Replace any effect already running rather than letting two animation
    // loops race writes to the same device, matching the reference app's
    // `start_effect` always calling `stop_current_effect` first.
    stop_running_effect(&state).await?;

    let rgb = state.rgb.clone();
    let cancel = Arc::new(AtomicBool::new(false));
    let join = tokio::spawn(effects::run(rgb, kind, speed, cancel.clone()));
    *state.effect_handle.lock().unwrap() = Some(RunningEffect { cancel, join });
    Ok(())
}

#[tauri::command]
async fn stop_effect(state: tauri::State<'_, AppState>) -> Result<(), String> {
    stop_running_effect(&state).await
}

/// How long to wait for a running effect to observe cancellation and exit
/// cleanly before falling back to a hard abort. Generous relative to the
/// ~33ms frame period - only matters if a hidraw write is genuinely wedged
/// (e.g. device stuck mid-transfer), in which case we still proceed with
/// the clear rather than hang the `stop_effect` command forever.
const EFFECT_STOP_GRACE_PERIOD: Duration = Duration::from_secs(2);

/// Stops the running effect task (if any) and clears the keyboard, so
/// stopping an effect never leaves stray colors lit - mirrors the Python
/// app's `stop_current_effect`. Waits for the task to actually finish
/// (not just signals it to) before clearing, so the clear can never race
/// a still-in-flight frame write and get silently overwritten by it.
async fn stop_running_effect(state: &tauri::State<'_, AppState>) -> Result<(), String> {
    let running = state.effect_handle.lock().unwrap().take();
    if let Some(RunningEffect { cancel, mut join }) = running {
        cancel.store(true, Ordering::SeqCst);
        if tokio::time::timeout(EFFECT_STOP_GRACE_PERIOD, &mut join)
            .await
            .is_err()
        {
            // The loop didn't exit in time (wedged write) - force it and
            // proceed with the clear anyway rather than hang this command.
            join.abort();
        }
    }
    let rgb = state.rgb.clone();
    tokio::task::spawn_blocking(move || rgb.clear_all_keys())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_power_profile(profile: String) -> Result<(), String> {
    let profile = match profile.as_str() {
        "performance" => PowerProfile::Performance,
        "balanced" => PowerProfile::Balanced,
        "power_save" => PowerProfile::PowerSave,
        other => {
            return Err(format!(
                "unknown power profile '{other}' (expected \"performance\", \"balanced\" or \"power_save\")"
            ))
        }
    };
    tokio::task::spawn_blocking(move || PowerReader::set_profile(profile))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn set_fan_mode(mode: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || fan::set_fan_mode(&mode))
        .await
        .map_err(|e| e.to_string())?
}

/// Payload for the periodic `system-stats` push event - replaces frontend
/// polling of `get_sensor_snapshot`/`get_power_info` with a single
/// backend-driven interval, per the frozen event contract.
#[derive(Serialize, Clone)]
struct SystemStatsPayload {
    sensors: SensorSnapshot,
    power: PowerInfo,
    usage: SystemUsage,
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
        .setup(|app| {
            // Background push loop: emits sensor + power readings every
            // ~2s so the frontend can `listen("system-stats", ...)`
            // instead of polling `get_sensor_snapshot`/`get_power_info` on
            // a timer. Snapshot fetches (which shell out to `sensors` and
            // read sysfs) still run inside `spawn_blocking`, same as every
            // other device-touching path in this file.
            let app_handle = app.handle().clone();
            let state = app.state::<AppState>();
            let sensors = state.sensors.clone();
            let power = state.power.clone();
            let usage = state.usage.clone();
            tauri::async_runtime::spawn(async move {
                let mut ticker = tokio::time::interval(Duration::from_secs(2));
                loop {
                    ticker.tick().await;
                    let sensors = sensors.clone();
                    let power = power.clone();
                    let usage = usage.clone();
                    let readings = tokio::task::spawn_blocking(move || {
                        (sensors.snapshot(), power.snapshot(), usage.snapshot())
                    })
                    .await;
                    if let Ok((sensors, power, usage)) = readings {
                        let _ = app_handle.emit(
                            "system-stats",
                            SystemStatsPayload { sensors, power, usage },
                        );
                    }
                }
            });

            // osd-lidmonitor-agent: background evdev listener that pops up
            // the `osd` window on Fn-hotkey presses. See hotkey_osd.rs and
            // CONTRACT.md's "osd-lidmonitor-agent" section.
            //
            // System tray icon (Show Control Center / Quick Clear / Exit)
            // plus hide-to-tray on window close - see tray.rs.
            //
            // NOTE: this, the tray setup, and the system-stats loop above
            // must stay inside this single `.setup()` closure - Tauri's
            // Builder::setup only keeps the last-registered closure, it
            // does not chain them. A prior auto-merge of two independent
            // agent branches each calling `.setup()` separately silently
            // dropped one of the two background tasks; caught and fixed
            // during integration.
            hotkey_osd::spawn(app.handle().clone());
            tray::setup(app)?;

            // Clear the keyboard on SIGTERM/SIGINT too (session logout,
            // `kill`, window-manager "force quit", systemd stop) - these
            // bypass both the tray "Exit" menu item and Tauri's window-
            // close interception entirely, terminating the process
            // directly via the OS's default signal disposition unless we
            // install a handler. `tokio`'s "signal" feature (part of the
            // "full" feature set already enabled below) makes this a
            // plain async task, no extra crate needed. SIGKILL (`kill
            // -9`) can never be caught by any process - that limitation
            // is unavoidable and not specific to this app.
            {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    use tokio::signal::unix::{signal, SignalKind};
                    let Ok(mut sigterm) = signal(SignalKind::terminate()) else {
                        return;
                    };
                    let Ok(mut sigint) = signal(SignalKind::interrupt()) else {
                        return;
                    };
                    tokio::select! {
                        _ = sigterm.recv() => {}
                        _ = sigint.recv() => {}
                    }
                    let rgb = app_handle.state::<AppState>().rgb.clone();
                    let _ = tokio::task::spawn_blocking(move || rgb.clear_all_keys()).await;
                    app_handle.exit(0);
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_connection_status,
            set_key_color,
            set_group_color,
            clear_all_keys,
            get_sensor_snapshot,
            get_power_info,
            get_system_usage,
            get_tlp_stats,
            start_effect,
            stop_effect,
            set_power_profile,
            set_fan_mode,
            list_flexikey_profiles,
            get_flexikey_profile,
            save_flexikey_profile,
            delete_flexikey_profile,
            set_active_flexikey_profile,
            capture_next_key,
            start_flexikey_engine,
            stop_flexikey_engine,
        ])
        .build(tauri::generate_context!())
        .expect("error while building the OriginPC Control Center application")
        .run(|app_handle, event| {
            // Clear the keyboard on every exit path (tray "Exit", window
            // manager quit, session logout/SIGTERM that tauri translates
            // to a runtime exit) - not just one button. Matches the old
            // Python app's `quit_application`, which cleared RGB before
            // terminating so the keyboard never gets left lit after the
            // app closes.
            //
            // `clear_all_keys` is a plain blocking call (real hidraw
            // writes, no async involved at the `RgbController` level) -
            // called directly and synchronously here rather than via
            // `spawn_blocking`, since `RunEvent::Exit` fires once, right
            // before the process actually terminates, and we need the
            // write to complete before that happens rather than racing it.
            if let tauri::RunEvent::Exit = event {
                let rgb = app_handle.state::<AppState>().rgb.clone();
                let _ = rgb.clear_all_keys();
            }
        });
}
