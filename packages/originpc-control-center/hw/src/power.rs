//! Battery/AC/TLP power state, read directly from sysfs (no dependency on
//! any particular power-management daemon being installed), with the same
//! TTL caching approach as `sensors.rs`.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const CACHE_TTL: Duration = Duration::from_millis(2000);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerInfo {
    pub battery_percent: f32,
    pub ac_connected: bool,
    pub status: String,
    pub tlp_active: bool,
}

impl Default for PowerInfo {
    fn default() -> Self {
        Self {
            battery_percent: 0.0,
            ac_connected: true,
            status: "unknown".to_string(),
            tlp_active: false,
        }
    }
}

struct Cache {
    info: PowerInfo,
    fetched_at: Instant,
}

pub struct PowerReader {
    cache: Mutex<Option<Cache>>,
}

impl Default for PowerReader {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerReader {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(None),
        }
    }

    pub fn snapshot(&self) -> PowerInfo {
        let mut guard = self.cache.lock().unwrap();
        if let Some(cache) = guard.as_ref() {
            if cache.fetched_at.elapsed() < CACHE_TTL {
                return cache.info.clone();
            }
        }
        let info = Self::fetch();
        *guard = Some(Cache {
            info: info.clone(),
            fetched_at: Instant::now(),
        });
        info
    }

    fn fetch() -> PowerInfo {
        let mut info = PowerInfo::default();

        if let Some(battery_dir) = Self::find_battery_dir() {
            if let Ok(capacity) = fs::read_to_string(battery_dir.join("capacity")) {
                info.battery_percent = capacity.trim().parse().unwrap_or(0.0);
            }
            if let Ok(status) = fs::read_to_string(battery_dir.join("status")) {
                let status = status.trim().to_string();
                info.ac_connected = status == "Charging" || status == "Full";
                info.status = status;
            }
        }

        info.tlp_active = Command::new("systemctl")
            .args(["is-active", "tlp"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        info
    }

    fn find_battery_dir() -> Option<std::path::PathBuf> {
        let power_supply = Path::new("/sys/class/power_supply");
        let entries = fs::read_dir(power_supply).ok()?;
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("BAT") {
                return Some(entry.path());
            }
        }
        None
    }

    /// Apply a named power profile via TLP, matching the Python app's
    /// Performance/Balanced/Power Save buttons.
    ///
    /// `tlp`'s real CLI has no "auto"/"balanced" subcommand - valid modes
    /// are `start` (apply whichever profile matches the current power
    /// source), `ac` (force the AC profile), and `bat` (force the battery
    /// profile). An earlier version of this function passed `tlp auto`,
    /// which is not a real subcommand and would have failed every time
    /// "Balanced" was clicked - caught by checking the actual `tlp` CLI
    /// rather than assuming a 3-way performance/balanced/powersave mapping
    /// existed on the TLP side.
    ///
    /// `tlp ac|bat|start` all write sysfs power-management settings and
    /// therefore require root - running bare `tlp <mode>` from this GUI
    /// process (no controlling TTY) always fails with "missing root
    /// privilege", exit status 1. We call it through `sudo -n` instead,
    /// scoped to exactly these three invocations via a dedicated
    /// `/etc/sudoers.d/originpc-control-center-tlp` NOPASSWD rule (see
    /// packaging notes) - `-n` (non-interactive) makes sudo fail
    /// immediately if that rule is ever missing rather than hang waiting
    /// on a TTY password prompt that can never arrive, the same class of
    /// GUI-hang bug already fixed once in this app's RGB-clear path.
    ///
    /// Requires TLP to be installed and the sudoers rule to be present;
    /// returns an error string otherwise (surfaced to the UI, not a
    /// panic).
    pub fn set_profile(profile: PowerProfile) -> Result<(), String> {
        let mode = match profile {
            PowerProfile::Performance => "ac",
            PowerProfile::Balanced => "start",
            PowerProfile::PowerSave => "bat",
        };
        Command::new("sudo")
            .args(["-n", "tlp", mode])
            .status()
            .map_err(|e| format!("failed to run sudo tlp: {e}"))
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    Err(format!(
                        "sudo tlp {mode} exited with {status} - is the \
                         /etc/sudoers.d/originpc-control-center-tlp NOPASSWD rule installed?"
                    ))
                }
            })
    }

    /// Text output of `tlp-stat -s` (status summary), for the "TLP Stats"
    /// detail view.
    ///
    /// The Python app ran the full, unfiltered `sudo tlp-stat` in an
    /// external terminal. That is deliberately not reproduced here:
    /// shelling out to `sudo` from a GUI process blocks forever waiting on
    /// a TTY password prompt that can never arrive (the same class of bug
    /// already found and removed elsewhere in this app's RGB-clear path).
    /// `-s` is the same summary view `get_power_info`'s TLP detection
    /// already reads without root, so this stays consistent with the rest
    /// of this module rather than needing new privileges.
    pub fn tlp_stats() -> Result<String, String> {
        let output = Command::new("tlp-stat")
            .arg("-s")
            .output()
            .map_err(|e| format!("failed to run tlp-stat: {e}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if output.status.success() && !stdout.trim().is_empty() {
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("tlp-stat -s produced no output: {stderr}").trim().to_string())
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PowerProfile {
    Performance,
    Balanced,
    PowerSave,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_never_panics_without_a_battery_present() {
        let reader = PowerReader::new();
        let info = reader.snapshot();
        assert!(info.battery_percent >= 0.0);
    }
}
