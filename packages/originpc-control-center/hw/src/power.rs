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
    /// Performance/Balanced/Power Save buttons. Requires TLP to be
    /// installed; returns an error string otherwise (surfaced to the UI,
    /// not a panic).
    pub fn set_profile(profile: PowerProfile) -> Result<(), String> {
        let mode = match profile {
            PowerProfile::Performance => "ac",
            PowerProfile::Balanced => "auto",
            PowerProfile::PowerSave => "bat",
        };
        Command::new("tlp")
            .arg(mode)
            .status()
            .map_err(|e| format!("failed to run tlp: {e}"))
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    Err(format!("tlp {mode} exited with {status}"))
                }
            })
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
