// System Monitoring Command Handlers
use crate::SystemMetrics;
use tauri::State;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_address: String,
    pub mac_address: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub is_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalZone {
    pub name: String,
    pub temperature: f64,
    pub max_temperature: f64,
    pub critical_temperature: f64,
}

#[tauri::command]
pub async fn get_process_list() -> Result<Vec<ProcessInfo>, String> {
    // Generate sample process data
    Ok(vec![
        ProcessInfo {
            pid: 1234,
            name: "systemd".to_string(),
            cpu_usage: 0.1,
            memory_usage: 4096000,
            status: "running".to_string(),
        },
        ProcessInfo {
            pid: 5678,
            name: "firefox".to_string(),
            cpu_usage: 5.4,
            memory_usage: 524288000,
            status: "running".to_string(),
        },
        ProcessInfo {
            pid: 9999,
            name: "code".to_string(),
            cpu_usage: 2.1,
            memory_usage: 256000000,
            status: "running".to_string(),
        },
    ])
}

#[tauri::command]
pub async fn get_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    // Generate sample network interface data
    Ok(vec![
        NetworkInterface {
            name: "wlan0".to_string(),
            ip_address: "192.168.1.100".to_string(),
            mac_address: "00:11:22:33:44:55".to_string(),
            rx_bytes: 1024000000,
            tx_bytes: 512000000,
            is_up: true,
        },
        NetworkInterface {
            name: "eth0".to_string(),
            ip_address: "192.168.1.101".to_string(),
            mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
            rx_bytes: 0,
            tx_bytes: 0,
            is_up: false,
        },
    ])
}

#[tauri::command]
pub async fn get_thermal_zones() -> Result<Vec<ThermalZone>, String> {
    // Generate sample thermal zone data
    Ok(vec![
        ThermalZone {
            name: "CPU".to_string(),
            temperature: 45.0,
            max_temperature: 85.0,
            critical_temperature: 100.0,
        },
        ThermalZone {
            name: "GPU".to_string(),
            temperature: 42.0,
            max_temperature: 80.0,
            critical_temperature: 95.0,
        },
    ])
}

#[tauri::command]
pub async fn get_historical_metrics(
    limit: Option<usize>,
) -> Result<Vec<SystemMetrics>, String> {
    use chrono::{Duration, Utc};
    let limit = limit.unwrap_or(60);
    let mut metrics = Vec::new();
    let now = Utc::now();
    
    // Generate sample historical data
    for i in 0..limit {
        let timestamp = now - Duration::minutes(i as i64);
        metrics.push(SystemMetrics {
            timestamp,
            cpu_usage: 20.0 + (i as f64 * 0.5) % 60.0,
            memory_usage: 40.0 + (i as f64 * 0.3) % 40.0,
            disk_usage: 60.0,
            network_rx: 1000000 * i as u64,
            network_tx: 500000 * i as u64,
            temperature: 40.0 + (i as f64 * 0.1) % 20.0,
            processes: 150 + (i % 20),
            uptime: 86400 + (i as u64 * 60),
        });
    }
    
    Ok(metrics)
}
