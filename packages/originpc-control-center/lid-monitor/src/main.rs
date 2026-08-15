//! OriginPC lid-monitor daemon.
//!
//! Two modes:
//! - `--clear-once`: one-shot manual recovery tool (opens the RGB device
//!   via `clevo-hw` and clears every key, then exits). Useful for testing
//!   and manual recovery without touching the persistent loop below.
//! - no arguments (the mode `packaging/originpc-lid-monitor.service`
//!   actually runs): the persistent daemon. Polls lid state once a second
//!   using multiple detection methods - mirroring
//!   `packages/originpc-control-center/src/lid-monitor-daemon.py`'s
//!   `_check_lid_state()` - and calls `RgbController::clear_all_keys()` the
//!   moment a closure is detected. Unlike the Python version's
//!   `ultra_aggressive_clear()` (10 brute-force passes over every key
//!   index, working around a residual-color bug by sheer repetition), this
//!   relies on `clevo_hw::RgbController::clear_all_keys()`, which already
//!   forces a write to every mapped key exactly once - the residual-color
//!   issue was a symptom of the old per-write-reopen-device pattern that
//!   the shared `clevo-hw` crate's persistent-handle design already fixes.
//!
//! Detection methods, in priority order (ACPI is checked first because
//! it has been confirmed present and reliable on this exact hardware;
//! the rest are fallbacks for robustness if ACPI is ever unavailable):
//! 1. ACPI lid button state: `/proc/acpi/button/lid/*/state`.
//! 2. Display power state via `xset q` (a closed lid typically blanks the
//!    display through the DPMS/backlight chain).
//! 3. Session lock state via `loginctl show-session self`.
//! 4. Whether the system is mid-suspend (`systemctl is-active
//!    suspend.target`).
//! 5. A manual test hook (`/tmp/test_lid_closed`) for exercising the
//!    closure path without physically closing the lid.

use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use clevo_hw::RgbController;

/// Minimum time between consecutive RGB clears, so a flapping detection
/// method can't hammer the hidraw device.
const CLEAR_COOLDOWN: Duration = Duration::from_secs(5);

/// Number of consecutive "closed" readings required before acting, so a
/// single spurious reading from one of the fallback methods doesn't trigger
/// a clear.
const REQUIRED_CONSECUTIVE_CLOSED: u32 = 2;

const POLL_INTERVAL: Duration = Duration::from_secs(1);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--clear-once") {
        run_clear_once();
        return;
    }
    if let Some(other) = args.get(1) {
        eprintln!("Unknown argument: {other}");
        eprintln!("Usage: occ-lid-monitor [--clear-once]");
        eprintln!("  (no arguments)  run the persistent lid-monitoring daemon");
        eprintln!("  --clear-once    clear the RGB keyboard once and exit");
        std::process::exit(2);
    }
    run_daemon();
}

fn run_clear_once() {
    let controller = RgbController::new();
    match controller.clear_all_keys() {
        Ok(()) => println!("RGB keyboard cleared."),
        Err(e) => {
            eprintln!("Failed to clear RGB keyboard: {e}");
            std::process::exit(1);
        }
    }
}

fn run_daemon() {
    log(&format!(
        "OriginPC lid-monitor daemon starting (pid {})",
        std::process::id()
    ));

    let controller = RgbController::new();

    let term_requested = Arc::new(AtomicBool::new(false));
    for sig in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGINT] {
        if let Err(e) = signal_hook::flag::register(sig, Arc::clone(&term_requested)) {
            log(&format!(
                "warning: failed to register handler for signal {sig}: {e}; \
                 the process will still exit on this signal, just without a clean shutdown log line"
            ));
        }
    }

    let mut lid_was_open = true;
    let mut consecutive_closed: u32 = 0;
    // Allow an immediate clear on the very first detected closure.
    let mut last_clear = Instant::now() - CLEAR_COOLDOWN - Duration::from_secs(1);

    log("Lid monitoring loop started");

    while !term_requested.load(Ordering::Relaxed) {
        let open = lid_is_open();

        if open {
            if !lid_was_open {
                log("Lid reopened");
            }
            lid_was_open = true;
            consecutive_closed = 0;
        } else {
            consecutive_closed += 1;
            if lid_was_open
                && consecutive_closed >= REQUIRED_CONSECUTIVE_CLOSED
                && last_clear.elapsed() > CLEAR_COOLDOWN
            {
                log("Lid closure detected - clearing RGB keyboard");
                match controller.clear_all_keys() {
                    Ok(()) => log("RGB keyboard cleared successfully"),
                    Err(e) => log(&format!("Failed to clear RGB keyboard: {e}")),
                }
                last_clear = Instant::now();
                lid_was_open = false;
            }
        }

        std::thread::sleep(POLL_INTERVAL);
    }

    log("Received shutdown signal, stopping lid-monitor daemon");
}

/// Multi-method lid-state check. Returns `true` if the lid is (as far as
/// any method can tell) open.
fn lid_is_open() -> bool {
    // Method 1 (primary): ACPI lid button state - confirmed present and
    // reliable on this hardware.
    if let Some(closed) = check_acpi_lid_closed() {
        return !closed;
    }

    // Method 2: display power state.
    if let Some((success, output)) = run_with_timeout("xset", &["q"], Duration::from_secs(3)) {
        if success {
            let lower = output.to_lowercase();
            if lower.contains("monitor is off") || lower.contains("monitor is in standby") {
                return false;
            }
        }
    }

    // Method 3: session lock status.
    if let Some((success, output)) = run_with_timeout(
        "loginctl",
        &["show-session", "self"],
        Duration::from_secs(3),
    ) {
        if success && output.contains("LockedHint=yes") {
            return false;
        }
    }

    // Method 4: system is mid-suspend.
    if let Some((success, output)) = run_with_timeout(
        "systemctl",
        &["is-active", "suspend.target"],
        Duration::from_secs(2),
    ) {
        if success && output.trim() == "active" {
            return false;
        }
    }

    // Method 5: manual test hook, for exercising the closure path without
    // physically closing the lid.
    if Path::new("/tmp/test_lid_closed").exists() {
        return false;
    }

    true // default to open
}

/// Checks every `/proc/acpi/button/lid/*/state` entry. Returns `Some(true)`
/// if any reports "closed", `Some(false)` if entries exist and all report
/// open, or `None` if the ACPI lid button interface isn't present at all
/// (so the caller should fall back to the other detection methods).
fn check_acpi_lid_closed() -> Option<bool> {
    let entries = std::fs::read_dir("/proc/acpi/button/lid").ok()?;
    let mut found_any = false;
    for entry in entries.flatten() {
        let state_path = entry.path().join("state");
        if let Ok(content) = std::fs::read_to_string(&state_path) {
            found_any = true;
            if content.to_lowercase().contains("closed") {
                return Some(true);
            }
        }
    }
    if found_any {
        Some(false)
    } else {
        None
    }
}

/// Runs `cmd` with `args`, killing it if it hasn't exited within `timeout`.
/// Mirrors the Python daemon's `subprocess.run(..., timeout=N)` calls: the
/// monitor loop must never be able to wedge on a hung helper binary.
/// Returns `(exit_success, captured_stdout)`, or `None` if the command
/// could not even be spawned (e.g. not installed).
fn run_with_timeout(cmd: &str, args: &[&str], timeout: Duration) -> Option<(bool, String)> {
    let mut child: Child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let mut stdout_pipe = child.stdout.take()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = stdout_pipe.read_to_string(&mut buf);
        let _ = tx.send(buf);
    });

    let deadline = Instant::now() + timeout;
    let success = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.success(),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    break false;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(_) => break false,
        }
    };

    let output = rx.recv_timeout(Duration::from_millis(500)).unwrap_or_default();
    Some((success, output))
}

/// Logs a timestamped line to stderr (captured by the journal under
/// `systemd --user`) and to the same XDG state-dir log file the Python
/// daemon used, for continuity with existing troubleshooting docs.
fn log(message: &str) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("{timestamp} - {message}");
    eprintln!("{line}");

    if let Some(state_dir) = state_dir() {
        if std::fs::create_dir_all(&state_dir).is_ok() {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(state_dir.join("lid-monitor.log"))
            {
                let _ = writeln!(file, "{line}");
            }
        }
    }
}

fn state_dir() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| dirs_home().map(|home| home.join(".local").join("state")))?;
    Some(base.join("originpc-control-center"))
}

fn dirs_home() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}
