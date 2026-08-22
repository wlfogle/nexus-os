//! CPU/memory/disk/load-average/uptime readers, matching the data shown in
//! the original app's "System Monitor" panel (CPU/Memory usage gauges) and
//! "System Information" text block (`Memory: NN% (usedGB / totalGB)`,
//! `Disk: ...`, `Load Average: ...`, `Uptime: ...`).
//!
//! Dependency-free by design, consistent with the rest of this crate:
//! `/proc/stat`/`/proc/meminfo`/`/proc/loadavg`/`/proc/uptime` are read
//! directly, and disk usage shells out to the standard `df` CLI rather than
//! pulling in a filesystem-stats crate for one number.

use std::fs;
use std::process::Command;
use std::sync::Mutex;
use std::time::Instant;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default)]
struct CpuTimes {
    idle: u64,
    total: u64,
}

fn read_cpu_times() -> Option<CpuTimes> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    let line = stat.lines().next()?; // aggregate "cpu  ..." line
    let fields: Vec<u64> = line
        .split_whitespace()
        .skip(1) // skip the "cpu" label
        .filter_map(|f| f.parse().ok())
        .collect();
    // user, nice, system, idle, iowait, irq, softirq, steal[, guest, guest_nice]
    if fields.len() < 4 {
        return None;
    }
    let idle = fields[3] + fields.get(4).copied().unwrap_or(0); // idle + iowait
    let total: u64 = fields.iter().sum();
    Some(CpuTimes { idle, total })
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemUsage {
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub memory_used_gb: f32,
    pub memory_total_gb: f32,
    pub disk_percent: f32,
    pub disk_used_gb: f32,
    pub disk_total_gb: f32,
    pub load_avg: (f32, f32, f32),
    pub uptime_secs: u64,
}

struct Cache {
    last_cpu: Option<CpuTimes>,
    last_cpu_percent: f32,
    /// Disk usage barely changes second to second; re-running `df` every
    /// poll tick is wasted work, so it's refreshed on its own slower cycle
    /// (same TTL-cache pattern as `SensorReader`/`PowerReader`).
    disk: Option<(f32, f32, f32)>,
    disk_checked_at: Option<Instant>,
}

pub struct UsageReader {
    cache: Mutex<Cache>,
}

impl Default for UsageReader {
    fn default() -> Self {
        Self::new()
    }
}

impl UsageReader {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(Cache {
                last_cpu: None,
                last_cpu_percent: 0.0,
                disk: None,
                disk_checked_at: None,
            }),
        }
    }

    pub fn snapshot(&self) -> SystemUsage {
        let mut cache = self.cache.lock().unwrap();

        // CPU%: delta of (non-idle time) / (total time) between this call
        // and the previous one - matches how `top`/psutil compute it.
        // Returns 0 on the very first call (no prior sample yet), which is
        // the same behavior the Python app's `psutil.cpu_percent()` has on
        // its first invocation.
        let cpu_percent = if let Some(now) = read_cpu_times() {
            let percent = match cache.last_cpu {
                Some(prev) if now.total > prev.total => {
                    let total_delta = (now.total - prev.total) as f32;
                    let idle_delta = now.idle.saturating_sub(prev.idle) as f32;
                    ((total_delta - idle_delta) / total_delta * 100.0).clamp(0.0, 100.0)
                }
                _ => cache.last_cpu_percent,
            };
            cache.last_cpu = Some(now);
            cache.last_cpu_percent = percent;
            percent
        } else {
            0.0
        };

        let (memory_percent, memory_used_gb, memory_total_gb) = Self::read_memory();

        if cache.disk_checked_at.map(|t| t.elapsed().as_secs() >= 10).unwrap_or(true) {
            cache.disk = Self::read_disk();
            cache.disk_checked_at = Some(Instant::now());
        }
        let (disk_percent, disk_used_gb, disk_total_gb) = cache.disk.unwrap_or_default();

        SystemUsage {
            cpu_percent,
            memory_percent,
            memory_used_gb,
            memory_total_gb,
            disk_percent,
            disk_used_gb,
            disk_total_gb,
            load_avg: Self::read_load_avg(),
            uptime_secs: Self::read_uptime(),
        }
    }

    fn read_memory() -> (f32, f32, f32) {
        let Ok(meminfo) = fs::read_to_string("/proc/meminfo") else {
            return (0.0, 0.0, 0.0);
        };
        let mut total_kb = 0u64;
        let mut available_kb = 0u64;
        for line in meminfo.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                total_kb = Self::parse_kb(rest);
            } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
                available_kb = Self::parse_kb(rest);
            }
        }
        if total_kb == 0 {
            return (0.0, 0.0, 0.0);
        }
        let used_kb = total_kb.saturating_sub(available_kb);
        let percent = used_kb as f32 / total_kb as f32 * 100.0;
        (percent, used_kb as f32 / 1_048_576.0, total_kb as f32 / 1_048_576.0)
    }

    fn parse_kb(field: &str) -> u64 {
        field
            .split_whitespace()
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }

    fn read_disk() -> Option<(f32, f32, f32)> {
        // `--output=size,used,pcent` in 1K blocks (POSIX default), parsed
        // from the second line (first is the header).
        let output = Command::new("df").args(["--output=size,used,pcent", "/"]).output().ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let data_line = text.lines().nth(1)?;
        let mut fields = data_line.split_whitespace();
        let size_kb: f64 = fields.next()?.parse().ok()?;
        let used_kb: f64 = fields.next()?.parse().ok()?;
        let percent: f32 = fields.next()?.trim_end_matches('%').parse().ok()?;
        Some((percent, (used_kb / 1_048_576.0) as f32, (size_kb / 1_048_576.0) as f32))
    }

    fn read_load_avg() -> (f32, f32, f32) {
        let Ok(content) = fs::read_to_string("/proc/loadavg") else {
            return (0.0, 0.0, 0.0);
        };
        let mut parts = content.split_whitespace();
        let one = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let five = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let fifteen = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        (one, five, fifteen)
    }

    fn read_uptime() -> u64 {
        fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|s| s.split_whitespace().next().map(str::to_string))
            .and_then(|s| s.parse::<f64>().ok())
            .map(|secs| secs as u64)
            .unwrap_or(0)
    }
}

/// Formats seconds as "`Nd HHh MMm`", matching the Python app's
/// `format_uptime`.
pub fn format_uptime(total_secs: u64) -> String {
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    format!("{days}d {hours}h {minutes}m")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_never_panics() {
        let reader = UsageReader::new();
        let first = reader.snapshot();
        // First call has no prior CPU sample, so percent must be a
        // well-defined default rather than garbage/NaN.
        assert!(first.cpu_percent >= 0.0 && first.cpu_percent <= 100.0);
        let second = reader.snapshot();
        assert!(second.cpu_percent >= 0.0 && second.cpu_percent <= 100.0);
    }

    #[test]
    fn format_uptime_matches_expected_shape() {
        assert_eq!(format_uptime(0), "0d 0h 0m");
        assert_eq!(format_uptime(90061), "1d 1h 1m");
    }
}
