use serde::{Deserialize, Serialize};
use sysinfo::System;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tokio::time::{interval, Duration};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use tracing::{info, error};

// Global system info cache
static SYSTEM_CACHE: Lazy<DashMap<String, SystemStats>> = Lazy::new(|| DashMap::new());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_percentage: f32,
    pub swap_used: u64,
    pub swap_total: u64,
    pub disk_stats: Vec<DiskInfo>,
    pub network_stats: NetworkInfo,
    pub load_average: LoadAverage,
    pub uptime: u64,
    pub running_vms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub usage_percentage: f32,
    pub file_system: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub total_bytes_received: u64,
    pub total_bytes_transmitted: u64,
    pub total_packets_received: u64,
    pub total_packets_transmitted: u64,
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub bytes_received: u64,
    pub bytes_transmitted: u64,
    pub packets_received: u64,
    pub packets_transmitted: u64,
    pub errors_received: u64,
    pub errors_transmitted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VMStatistics {
    pub name: String,
    pub status: String,
    pub cpu_time: u64,
    pub cpu_percentage: f64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_percentage: f64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub uptime: u64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxmoxVMInfo {
    pub path: String,
    pub size_gb: f64,
    pub format: String,
    pub last_modified: DateTime<Utc>,
    pub is_running: bool,
    pub estimated_memory_usage: u64,
}

pub struct SystemMonitor {
    system: System,
    last_cpu_times: HashMap<String, u64>,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system,
            last_cpu_times: HashMap::new(),
        }
    }

    pub fn get_system_stats(&mut self) -> SystemStats {
        self.system.refresh_all();

        let cpu_usage = self.system.global_cpu_usage();
        let memory_used = self.system.used_memory();
        let memory_total = self.system.total_memory();
        let memory_percentage = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        let swap_used = self.system.used_swap();
        let swap_total = self.system.total_swap();

        let disk_stats = vec![]; // Simplified for now - sysinfo API changes

        let network_stats = self.get_network_stats();
        let load_average = self.get_load_average();
        let uptime = System::uptime();
        let running_vms = self.count_running_vms();

        SystemStats {
            timestamp: Utc::now(),
            cpu_usage,
            memory_used,
            memory_total,
            memory_percentage,
            swap_used,
            swap_total,
            disk_stats,
            network_stats,
            load_average,
            uptime,
            running_vms,
        }
    }

    fn get_network_stats(&self) -> NetworkInfo {
        use std::fs;
        use std::collections::HashMap;
        
        let mut interfaces = Vec::new();
        let mut total_rx_bytes = 0u64;
        let mut total_tx_bytes = 0u64;
        let mut total_rx_packets = 0u64;
        let mut total_tx_packets = 0u64;
        
        // Read network statistics from /proc/net/dev
        if let Ok(content) = fs::read_to_string("/proc/net/dev") {
            for line in content.lines().skip(2) { // Skip header lines
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() >= 17 {
                    let name = parts[0].trim_end_matches(':').to_string();
                    
                    // Skip loopback interface
                    if name == "lo" {
                        continue;
                    }
                    
                    if let (Ok(rx_bytes), Ok(tx_bytes), Ok(rx_packets), Ok(tx_packets)) = (
                        parts[1].parse::<u64>(),
                        parts[9].parse::<u64>(),
                        parts[2].parse::<u64>(),
                        parts[10].parse::<u64>()
                    ) {
                        interfaces.push(NetworkInterface {
                            name: name.clone(),
                            bytes_received: rx_bytes,
                            bytes_transmitted: tx_bytes,
                            packets_received: rx_packets,
                            packets_transmitted: tx_packets,
                            errors_received: parts[3].parse().unwrap_or(0),
                            errors_transmitted: parts[11].parse().unwrap_or(0),
                        });
                        
                        total_rx_bytes += rx_bytes;
                        total_tx_bytes += tx_bytes;
                        total_rx_packets += rx_packets;
                        total_tx_packets += tx_packets;
                    }
                }
            }
        }
        
        NetworkInfo {
            total_bytes_received: total_rx_bytes,
            total_bytes_transmitted: total_tx_bytes,
            total_packets_received: total_rx_packets,
            total_packets_transmitted: total_tx_packets,
            interfaces,
        }
    }

    fn get_load_average(&self) -> LoadAverage {
        let load_avg = sysinfo::System::load_average();
        LoadAverage {
            one: load_avg.one as f64,
            five: load_avg.five as f64,
            fifteen: load_avg.fifteen as f64,
        }
    }

    fn count_running_vms(&self) -> u32 {
        self.system.processes()
            .iter()
            .filter(|(_, process)| {
                let name = process.name().to_string_lossy();
                name.contains("qemu") || name.contains("kvm") || name.contains("virt")
            })
            .count() as u32
    }

    pub fn get_proxmox_vm_info(vm_path: &str) -> Result<ProxmoxVMInfo, String> {
        use std::process::Command;
        use std::path::Path;
        
        info!("Checking Proxmox VM info for path: {}", vm_path);
        
        // First check if the path exists
        if !Path::new(vm_path).exists() {
            error!("VM file does not exist: {}", vm_path);
            return Err(format!("VM file does not exist: {}", vm_path));
        }
        
        // Use stat command to get file info (works better with different permissions)
        let stat_output = Command::new("stat")
            .args(["-c", "%s", vm_path])
            .output();
            
        let size_bytes = match stat_output {
            Ok(output) if output.status.success() => {
                let size_str = String::from_utf8_lossy(&output.stdout);
                size_str.trim().parse::<u64>().unwrap_or(0)
            },
            _ => {
                // Fallback to ls -l if stat fails
                let ls_output = Command::new("ls")
                    .args(["-l", vm_path])
                    .output();
                    
                match ls_output {
                    Ok(output) if output.status.success() => {
                        let output_str = String::from_utf8_lossy(&output.stdout);
                        // Parse ls output to extract file size
                        output_str.split_whitespace()
                            .nth(4)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0)
                    },
                    _ => {
                        error!("Cannot access file size for: {}", vm_path);
                        return Err(format!("Cannot access file: {}", vm_path));
                    }
                }
            }
        };
        
        let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        
        // Get last modified time using stat command
        let stat_time_output = Command::new("stat")
            .args(["-c", "%Y", vm_path])
            .output();
            
        let last_modified = match stat_time_output {
            Ok(output) if output.status.success() => {
                let timestamp_str = String::from_utf8_lossy(&output.stdout);
                let timestamp = timestamp_str.trim().parse::<i64>().unwrap_or(0);
                DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now())
            },
            _ => Utc::now() // Fallback to current time
        };
        
        // Check if VM is currently running by looking for qemu processes using this image
        let is_running = Self::is_vm_running_by_image(vm_path);
        
        // Estimate memory usage based on typical Proxmox configurations
        let estimated_memory_usage = if is_running {
            // Proxmox VE typically uses 2-8GB of RAM depending on configuration
            4 * 1024 * 1024 * 1024 // 4GB estimate
        } else {
            0
        };

        Ok(ProxmoxVMInfo {
            path: vm_path.to_string(),
            size_gb,
            format: "qcow2".to_string(),
            last_modified,
            is_running,
            estimated_memory_usage,
        })
    }

    fn is_vm_running_by_image(vm_path: &str) -> bool {
        use std::process::Command;
        
        if let Ok(output) = Command::new("pgrep")
            .args(&["-f", vm_path])
            .output()
        {
            !output.stdout.is_empty()
        } else {
            false
        }
    }

    pub async fn start_monitoring() -> Result<(), String> {
        let mut monitor = SystemMonitor::new();
        let mut interval = interval(Duration::from_secs(5)); // Update every 5 seconds

        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                let stats = monitor.get_system_stats();
                SYSTEM_CACHE.insert("current".to_string(), stats);
                
                // Keep only the last 100 readings for historical data
                let history_key = format!("history_{}", Utc::now().timestamp());
                SYSTEM_CACHE.insert(history_key, SYSTEM_CACHE.get("current").unwrap().clone());
                
                // Cleanup old entries
                if SYSTEM_CACHE.len() > 100 {
                    let oldest_keys: Vec<String> = SYSTEM_CACHE
                        .iter()
                        .filter(|entry| entry.key().starts_with("history_"))
                        .take(SYSTEM_CACHE.len() - 100)
                        .map(|entry| entry.key().clone())
                        .collect();
                    
                    for key in oldest_keys {
                        SYSTEM_CACHE.remove(&key);
                    }
                }
            }
        });

        Ok(())
    }

    pub fn get_cached_stats() -> Option<SystemStats> {
        SYSTEM_CACHE.get("current").map(|entry| entry.clone())
    }

    pub fn get_historical_stats() -> Vec<SystemStats> {
        SYSTEM_CACHE
            .iter()
            .filter(|entry| entry.key().starts_with("history_"))
            .map(|entry| entry.value().clone())
            .collect()
    }
}

// Tauri command functions
#[tauri::command]
pub async fn get_system_statistics() -> Result<SystemStats, String> {
    if let Some(stats) = SystemMonitor::get_cached_stats() {
        Ok(stats)
    } else {
        let mut monitor = SystemMonitor::new();
        Ok(monitor.get_system_stats())
    }
}

#[tauri::command]
pub async fn get_proxmox_info(vm_path: String) -> Result<ProxmoxVMInfo, String> {
    SystemMonitor::get_proxmox_vm_info(&vm_path)
}

#[tauri::command]
pub async fn get_system_history() -> Result<Vec<SystemStats>, String> {
    Ok(SystemMonitor::get_historical_stats())
}

#[tauri::command]
pub async fn start_system_monitoring() -> Result<String, String> {
    SystemMonitor::start_monitoring().await?;
    Ok("System monitoring started".to_string())
}
