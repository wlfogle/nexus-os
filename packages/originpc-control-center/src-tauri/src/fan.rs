//! Fan speed control via NBFC (Notebook FanControl).
//!
//! The Python app never reverse-engineered a raw EC/PWM fan protocol for
//! the EON17-X - `hardware_optimizations.py`'s `HardwareOptimizer` only
//! ever touches CPU governor/P-state and GPU power-mode sysfs knobs (a
//! different subsystem from fan speed). The actual fan control code path
//! is `FanController.set_fan_mode` in
//! `enhanced-professional-control-center.py`, which shells out to the
//! `nbfc` CLI. This module reproduces that exact mechanism rather than
//! inventing a new one:
//! - `"auto"` -> `nbfc set -a` (hand fan control back to NBFC's
//!   configured automatic curve)
//! - `"silent"` -> `nbfc set -s 30` (manual fixed 30% fan speed)

use std::process::Command;

/// Applies a fan mode by invoking `nbfc`. Returns an error string (surfaced
/// to the UI) if `nbfc` is not installed/configured or exits non-zero,
/// rather than panicking - fan control is best-effort on hardware where
/// NBFC isn't set up.
pub fn set_fan_mode(mode: &str) -> Result<(), String> {
    let args: &[&str] = match mode {
        "auto" => &["set", "-a"],
        "silent" => &["set", "-s", "30"],
        other => {
            return Err(format!(
                "unknown fan mode '{other}' (expected \"auto\" or \"silent\")"
            ))
        }
    };

    let output = Command::new("nbfc")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run nbfc: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "nbfc {} exited with {}: {}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_mode() {
        let err = set_fan_mode("turbo").unwrap_err();
        assert!(err.contains("unknown fan mode"));
    }
}
