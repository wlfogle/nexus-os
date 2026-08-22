//! System tray icon with a right-click context menu: "Show Control
//! Center", "Quick Clear", and "Exit" - mirrors the old Python/PyQt5
//! app's `setup_system_tray_actual()`.
//!
//! Also intercepts the main window's close request so it hides to the
//! tray instead of quitting, matching the old app's
//! `app.setQuitOnLastWindowClosed(False)`. Without this, closing the
//! window would tear down the whole process and the tray icon would be
//! useless (nothing left to "show" back).
//!
//! Per the lesson documented in `lib.rs`'s `run()` (a prior auto-merge
//! silently dropped a background task because `Builder::setup` only
//! keeps the last registered closure), this module exposes a plain
//! function that is called from *inside* the single existing `.setup()`
//! closure rather than registering its own.

use tauri::menu::MenuBuilder;
use tauri::tray::TrayIconBuilder;
use tauri::{App, Manager, Runtime, WindowEvent};

use crate::AppState;

/// Menu item ids for the tray context menu.
mod ids {
    pub const SHOW: &str = "show";
    pub const QUICK_CLEAR: &str = "quick_clear";
    pub const EXIT: &str = "exit";
}

/// Builds the tray icon and its context menu, and wires the main
/// window's close button to hide-to-tray. Call once from `Builder::setup`.
pub fn setup<R: Runtime>(app: &App<R>) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text(ids::SHOW, "Show Control Center")
        .text(ids::QUICK_CLEAR, "Quick Clear")
        .separator()
        .text(ids::EXIT, "Exit")
        .build()?;

    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("OriginPC Control Center")
        .show_menu_on_left_click(true);
    // Reuse the app's own configured window icon rather than requiring a
    // separate tray-specific asset/feature (e.g. `image-png`).
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.on_menu_event(|app_handle, event| match event.id().as_ref() {
        ids::SHOW => {
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        ids::QUICK_CLEAR => {
            let rgb = app_handle.state::<AppState>().rgb.clone();
            tauri::async_runtime::spawn(async move {
                let _ = tokio::task::spawn_blocking(move || rgb.clear_all_keys()).await;
            });
        }
        ids::EXIT => {
            // Clear the keyboard directly here, synchronously, rather
            // than relying solely on `lib.rs`'s `RunEvent::Exit` handler
            // to fire - on Linux/GTK, `app_handle.exit()` can tear the
            // process down before that callback is guaranteed to run.
            // `clear_all_keys` is a plain blocking call (a few ms of real
            // hidraw writes), so blocking this menu-event thread briefly
            // to guarantee it completes before exit proceeds is correct.
            let _ = app_handle.state::<AppState>().rgb.clear_all_keys();
            app_handle.exit(0);
        }
        _ => {}
    })
    .build(app)?;

    if let Some(window) = app.get_webview_window("main") {
        let hide_window = window.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = hide_window.hide();
            }
        });
    }

    Ok(())
}
