//! Temperature/fan telemetry via `sensors -j` (lm-sensors), matching the
//! Python app's data source but with a TTL cache shared across all callers
//! instead of spawning a subprocess per widget per poll tick.

use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// How long a cached reading is considered fresh. Matches the ~1-2s
/// cadence the UI actually needs; temperatures don't change meaningfully
/// faster than this.
const CACHE_TTL: Duration = Duration::from_millis(1500);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub label: String,
    pub celsius: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SensorSnapshot {
    pub cpu: Vec<TemperatureReading>,
    pub gpu: Vec<TemperatureReading>,
    pub nvme: Vec<TemperatureReading>,
    pub fans_rpm: Vec<TemperatureReading>,
}

struct Cache {
    snapshot: SensorSnapshot,
    fetched_at: Instant,
}

pub struct SensorReader {
    cache: Mutex<Option<Cache>>,
}

impl Default for SensorReader {
    fn default() -> Self {
        Self::new()
    }
}

impl SensorReader {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(None),
        }
    }

    /// Returns a cached snapshot if fresh, otherwise re-runs `sensors -j`
    /// and refreshes the cache. Never returns an error to callers - a
    /// missing `sensors` binary or malformed output just yields an empty
    /// snapshot, matching the Python app's "no sensors found" fallback.
    pub fn snapshot(&self) -> SensorSnapshot {
        let mut guard = self.cache.lock().unwrap();
        if let Some(cache) = guard.as_ref() {
            if cache.fetched_at.elapsed() < CACHE_TTL {
                return cache.snapshot.clone();
            }
        }
        let snapshot = Self::fetch().unwrap_or_default();
        *guard = Some(Cache {
            snapshot: snapshot.clone(),
            fetched_at: Instant::now(),
        });
        snapshot
    }

    fn fetch() -> Option<SensorSnapshot> {
        let output = Command::new("sensors").arg("-j").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let json: Value = serde_json::from_slice(&output.stdout).ok()?;
        let mut snapshot = SensorSnapshot::default();

        if let Value::Object(chips) = json {
            for (chip, fields) in chips {
                let chip_lower = chip.to_lowercase();
                if let Value::Object(entries) = fields {
                    for (name, sub) in entries {
                        let Value::Object(sub_fields) = sub else { continue };
                        for (sub_key, value) in &sub_fields {
                            let Some(num) = value.as_f64() else { continue };
                            if sub_key.ends_with("_input") && sub_key.contains("temp") {
                                let reading = TemperatureReading {
                                    label: format!("{chip} {name}"),
                                    celsius: num as f32,
                                };
                                if chip_lower.contains("coretemp") || chip_lower.contains("cpu") {
                                    snapshot.cpu.push(reading);
                                } else if chip_lower.contains("nvme") {
                                    snapshot.nvme.push(reading);
                                } else if chip_lower.contains("amdgpu") || chip_lower.contains("nvidia") || chip_lower.contains("gpu") {
                                    snapshot.gpu.push(reading);
                                }
                            } else if sub_key.ends_with("_input") && sub_key.contains("fan") {
                                snapshot.fans_rpm.push(TemperatureReading {
                                    label: format!("{chip} {name}"),
                                    celsius: num as f32,
                                });
                            }
                        }
                    }
                }
            }
        }
        Some(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_never_panics_even_without_sensors_binary() {
        // Environment-independent: just asserts this never panics/errors.
        let reader = SensorReader::new();
        let _ = reader.snapshot();
    }
}
