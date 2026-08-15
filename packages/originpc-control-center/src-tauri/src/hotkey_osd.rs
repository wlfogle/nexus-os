//! Fn-hotkey on-screen-display background task.
//!
//! Mirrors the UX of `packages/originpc-control-center/src/hotkey_osd.py`,
//! but scoped down to exactly the device the `clevo-hotkeys` kernel module
//! registers (`"Clevo WMI Hotkeys"`, see
//! `kernel/clevo-hotkeys/clevo-hotkeys.c`) rather than scanning every evdev
//! node on the system, per CONTRACT.md.
//!
//! The device node under `/dev/input` can shift across reboots/module
//! reloads, so it is always looked up by name via `evdev::enumerate()`,
//! never hardcoded. Discovery and the event read loop both run on the
//! Tokio runtime: discovery's blocking directory/file I/O is confined to
//! `spawn_blocking`, and once found the device is driven through
//! `evdev`'s `tokio`-feature `EventStream`, which is epoll-backed and
//! genuinely non-blocking - so, like every hardware-touching Tauri
//! command in `lib.rs`, this task never blocks a runtime worker thread.

use std::time::Duration;

use evdev::{Device, EventSummary, KeyCode};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

/// Name the `clevo-hotkeys` kernel module gives its input device (see
/// `input_dev->name` assignment in `clevo-hotkeys.c`). Never hardcode
/// `/dev/input/eventN` - the number shifts depending on enumeration order.
const HOTKEY_DEVICE_NAME: &str = "Clevo WMI Hotkeys";

/// Label of the always-on-top popup window defined in `tauri.conf.json`.
const OSD_WINDOW_LABEL: &str = "osd";

/// How long the OSD stays visible after the most recent hotkey event,
/// matching the Python OSD's `QTimer` auto-hide delay.
const OSD_AUTO_HIDE: Duration = Duration::from_millis(1500);

/// How long to wait before retrying device discovery when the
/// `clevo-hotkeys` module isn't loaded yet (or the device disappears,
/// e.g. after a module reload).
const DEVICE_RETRY_INTERVAL: Duration = Duration::from_secs(5);

/// Linux `KEY_LIGHTS_TOGGLE` (reading-light on/off, kernel scancode
/// `0x21e`) has no named associated constant in the `evdev` crate as of
/// 0.13. Construct it directly from the value in
/// `/usr/include/linux/input-event-codes.h` instead of guessing at a
/// constant name that may not exist.
const KEY_LIGHTS_TOGGLE: KeyCode = KeyCode::new(0x21e);

/// Mirrors `frontend/src/types.ts`'s `HotkeyEventPayload` (frozen in
/// CONTRACT.md).
#[derive(Clone, Serialize)]
struct HotkeyEventPayload {
    key: &'static str,
    label: &'static str,
    icon: &'static str,
}

/// Keycode -> (key name, label, icon) lookup. Source of truth is
/// `packages/originpc-control-center/src/hotkey_osd.py`'s `OSD_MESSAGES`
/// table, reproduced verbatim (labels, icons and the full set of keys) even
/// though the `clevo-hotkeys` module itself only ever emits a subset of
/// these (KB backlight, lights-toggle, touchpad, rfkill, battery gauge) -
/// brightness/volume/mute are handled by other input devices but are kept
/// here for parity with the reference table and in case a future kernel
/// revision routes them through this device too.
fn hotkey_info(code: KeyCode) -> Option<HotkeyEventPayload> {
    let (key, label, icon) = if code == KeyCode::KEY_KBDILLUMUP {
        ("KEY_KBDILLUMUP", "Keyboard Backlight +", "\u{2b06}")
    } else if code == KeyCode::KEY_KBDILLUMDOWN {
        ("KEY_KBDILLUMDOWN", "Keyboard Backlight -", "\u{2b07}")
    } else if code == KeyCode::KEY_KBDILLUMTOGGLE {
        ("KEY_KBDILLUMTOGGLE", "Keyboard Backlight Toggled", "\u{2728}")
    } else if code == KEY_LIGHTS_TOGGLE {
        ("KEY_LIGHTS_TOGGLE", "Keyboard Effect Cycled", "\u{1f308}")
    } else if code == KeyCode::KEY_F21 {
        ("KEY_F21", "Touchpad Toggled", "\u{1f5b1}")
    } else if code == KeyCode::KEY_RFKILL {
        ("KEY_RFKILL", "Wireless Toggled", "\u{1f4f6}")
    } else if code == KeyCode::KEY_PROG1 {
        ("KEY_PROG1", "Battery Gauge", "\u{1f50b}")
    } else if code == KeyCode::KEY_BRIGHTNESSUP {
        ("KEY_BRIGHTNESSUP", "Screen Brightness +", "\u{2600}")
    } else if code == KeyCode::KEY_BRIGHTNESSDOWN {
        ("KEY_BRIGHTNESSDOWN", "Screen Brightness -", "\u{1f505}")
    } else if code == KeyCode::KEY_VOLUMEUP {
        ("KEY_VOLUMEUP", "Volume +", "\u{1f50a}")
    } else if code == KeyCode::KEY_VOLUMEDOWN {
        ("KEY_VOLUMEDOWN", "Volume -", "\u{1f509}")
    } else if code == KeyCode::KEY_MUTE {
        ("KEY_MUTE", "Muted", "\u{1f507}")
    } else {
        return None;
    };
    Some(HotkeyEventPayload { key, label, icon })
}

/// Spawns the background hotkey-listening task. Called once from `run()`'s
/// `.setup()` hook at application startup.
pub fn spawn(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        run(app_handle).await;
    });
}

async fn run(app_handle: AppHandle) {
    loop {
        let Some(device) = find_hotkey_device().await else {
            tokio::time::sleep(DEVICE_RETRY_INTERVAL).await;
            continue;
        };

        let mut stream = match device.into_event_stream() {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("hotkey-osd: failed to open event stream for {HOTKEY_DEVICE_NAME}: {e}");
                tokio::time::sleep(DEVICE_RETRY_INTERVAL).await;
                continue;
            }
        };

        // Tracks the pending auto-hide task so a fresh event can cancel and
        // reschedule it - without this, rapid key presses would hide the
        // window mid-sequence instead of extending its visible time.
        let mut pending_hide: Option<tauri::async_runtime::JoinHandle<()>> = None;

        loop {
            match stream.next_event().await {
                Ok(event) => {
                    if let EventSummary::Key(_, code, value) = event.destructure() {
                        // value 1 = key down, 0 = key up, 2 = autorepeat.
                        if value == 1 {
                            if let Some(payload) = hotkey_info(code) {
                                show_and_schedule_hide(&app_handle, payload, &mut pending_hide);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "hotkey-osd: lost connection to {HOTKEY_DEVICE_NAME}: {e}; will retry discovery"
                    );
                    break;
                }
            }
        }
        // Device likely unplugged or the kernel module was reloaded (which
        // re-creates the input device under a new /dev/input/eventN node) -
        // loop back around to rediscover it by name.
    }
}

/// Blocking device enumeration (opens every node under `/dev/input`) run
/// off the async task via `spawn_blocking`, per the same rule that governs
/// every hardware-touching Tauri command in `lib.rs`.
async fn find_hotkey_device() -> Option<Device> {
    tokio::task::spawn_blocking(|| {
        evdev::enumerate()
            .map(|(_path, device)| device)
            .find(|device| device.name() == Some(HOTKEY_DEVICE_NAME))
    })
    .await
    .unwrap_or(None)
}

fn show_and_schedule_hide(
    app_handle: &AppHandle,
    payload: HotkeyEventPayload,
    pending_hide: &mut Option<tauri::async_runtime::JoinHandle<()>>,
) {
    if let Err(e) = app_handle.emit_to(OSD_WINDOW_LABEL, "hotkey-event", payload) {
        eprintln!("hotkey-osd: failed to emit hotkey-event to '{OSD_WINDOW_LABEL}': {e}");
    }

    match app_handle.get_webview_window(OSD_WINDOW_LABEL) {
        Some(window) => {
            if let Err(e) = window.show() {
                eprintln!("hotkey-osd: failed to show '{OSD_WINDOW_LABEL}' window: {e}");
            }
        }
        None => eprintln!("hotkey-osd: no window labeled '{OSD_WINDOW_LABEL}' found"),
    }

    // Cancel any hide scheduled by a previous event before scheduling a new
    // one, so the window stays up for OSD_AUTO_HIDE after the *last* event
    // in a burst, not the first.
    if let Some(handle) = pending_hide.take() {
        handle.abort();
    }

    let app_handle = app_handle.clone();
    *pending_hide = Some(tauri::async_runtime::spawn(async move {
        tokio::time::sleep(OSD_AUTO_HIDE).await;
        if let Some(window) = app_handle.get_webview_window(OSD_WINDOW_LABEL) {
            let _ = window.hide();
        }
    }));
}
