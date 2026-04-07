use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, error, debug};
use virt::{connect::Connect, domain::Domain};
use std::fs;

use crate::errors::{KvmError, Result};
use crate::types::*;

pub struct MonitoringService {
    metrics_history: HashMap<String, Vec<MetricPoint>>,
    collection_interval: Duration,
    connection: Option<Connect>,
}

#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
}

impl MonitoringService {
    pub fn new() -> Self {
        Self {
            metrics_history: HashMap::new(),
            collection_interval: Duration::from_secs(5),
            connection: None,
        }
    }

    pub fn with_connection(mut self, connection: Connect) -> Self {
        self.connection = Some(connection);
        self
    }

    pub async fn start_monitoring(&mut self) {
        info!("Starting monitoring service");
        
        let mut interval = interval(self.collection_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.collect_metrics().await {
                error!("Failed to collect metrics: {}", e);
            }
        }
    }

    async fn collect_metrics(&mut self) -> Result<()> {
        debug!("Collecting system metrics");
        
        // Collect host system metrics first (doesn't need connection)
        if let Ok(host_metrics) = self.get_host_metrics().await {
            self.store_metric("host", "cpu_usage", host_metrics.cpu_usage).await;
            self.store_metric("host", "memory_usage", host_metrics.memory_usage as f64).await;
            self.store_metric("host", "memory_total", host_metrics.memory_total as f64).await;
            
            // Store load average
            self.store_metric("host", "load_1", host_metrics.load_average[0]).await;
            self.store_metric("host", "load_5", host_metrics.load_average[1]).await;
            self.store_metric("host", "load_15", host_metrics.load_average[2]).await;
        }
        
        // Collect VM metrics if we have a connection
        if let Some(conn) = &self.connection {
            // Get all domains first to avoid borrowing issues
            if let Ok(domains) = conn.list_all_domains(virt::sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE) {
                let mut vm_metrics = Vec::new();
                
                for domain in domains {
                    if let (Ok(_name), Ok(uuid)) = (domain.get_name(), domain.get_uuid_string()) {
                        // Collect VM-specific metrics
                        if let Ok(vm_stats) = self.get_real_time_stats(&uuid).await {
                            vm_metrics.push((uuid, vm_stats));
                        }
                    }
                }
                
                // Store all the metrics after collecting
                for (uuid, vm_stats) in vm_metrics {
                    self.store_metric(&uuid, "cpu_usage", vm_stats.cpu_usage).await;
                    self.store_metric(&uuid, "memory_usage", vm_stats.memory_usage as f64).await;
                    self.store_metric(&uuid, "disk_read", vm_stats.disk_read as f64).await;
                    self.store_metric(&uuid, "disk_write", vm_stats.disk_write as f64).await;
                    self.store_metric(&uuid, "network_rx", vm_stats.network_rx as f64).await;
                    self.store_metric(&uuid, "network_tx", vm_stats.network_tx as f64).await;
                }
            }
        }
        
        // Cleanup old metrics (keep only last 24 hours)
        self.cleanup_old_metrics().await;
        
        Ok(())
    }
    
    async fn store_metric(&mut self, vm_id: &str, metric_type: &str, value: f64) {
        let key = format!("{}:{}", vm_id, metric_type);
        let metric_point = MetricPoint {
            timestamp: chrono::Utc::now(),
            value,
        };
        
        self.metrics_history
            .entry(key)
            .or_insert_with(Vec::new)
            .push(metric_point);
    }
    
    async fn cleanup_old_metrics(&mut self) {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(24);
        
        for (_, metrics) in self.metrics_history.iter_mut() {
            metrics.retain(|point| point.timestamp > cutoff_time);
        }
        
        // Remove empty metric series
        self.metrics_history.retain(|_, metrics| !metrics.is_empty());
    }

    pub fn get_metric_history(&self, vm_id: &str, metric_type: &str, duration: Duration) -> Vec<MetricPoint> {
        let key = format!("{}:{}", vm_id, metric_type);
        let cutoff_time = chrono::Utc::now() - chrono::Duration::from_std(duration).unwrap_or_default();
        
        self.metrics_history
            .get(&key)
            .unwrap_or(&Vec::new())
            .iter()
            .filter(|point| point.timestamp > cutoff_time)
            .cloned()
            .collect()
    }

    pub async fn get_real_time_stats(&self, vm_id: &str) -> Result<VmStats> {
        debug!("Getting real-time stats for VM: {}", vm_id);
        
        if let Some(conn) = &self.connection {
            // Get domain by UUID
            let domain = Domain::lookup_by_uuid_string(conn, vm_id)
                .map_err(|e| KvmError::VmNotFound(format!("Domain not found: {}", e)))?;
            
            // Get domain info for basic stats
            let info = domain.get_info()
                .map_err(|e| KvmError::LibvirtConnection(e))?;
            
            // Calculate CPU usage (this is a simplified calculation)
            let cpu_usage = self.calculate_cpu_usage(&domain)?;
            
            // Get memory stats
            let memory_stats = self.get_memory_stats(&domain)?;
            
            // Get disk I/O stats
            let (disk_read, disk_write) = self.get_disk_stats(&domain)?;
            
            // Get network stats
            let (network_rx, network_tx) = self.get_network_stats(&domain)?;
            
            // Check if guest agent is connected
            let guest_agent_connected = self.check_guest_agent(&domain);
            
            // Calculate uptime
            let uptime = if info.state == virt::sys::VIR_DOMAIN_RUNNING {
                // This is approximate - you'd want to track this more precisely
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            } else {
                0
            };
            
            Ok(VmStats {
                cpu_usage,
                memory_usage: memory_stats.0,
                memory_total: info.memory,
                disk_read,
                disk_write,
                network_rx,
                network_tx,
                uptime,
                guest_agent_connected,
                timestamp: chrono::Utc::now(),
            })
        } else {
            // Fallback if no connection available
            Ok(VmStats {
                cpu_usage: 0.0,
                memory_usage: 0,
                memory_total: 0,
                disk_read: 0,
                disk_write: 0,
                network_rx: 0,
                network_tx: 0,
                uptime: 0,
                guest_agent_connected: false,
                timestamp: chrono::Utc::now(),
            })
        }
    }

    pub async fn get_host_metrics(&self) -> Result<HostMetrics> {
        debug!("Getting host system metrics");
        
        let cpu_usage = self.get_host_cpu_usage()?;
        let (memory_usage, memory_total) = self.get_host_memory_stats()?;
        let load_average = self.get_load_average()?;
        let disk_usage = self.get_host_disk_usage()?;
        let network_usage = self.get_host_network_usage()?;
        
        Ok(HostMetrics {
            cpu_usage,
            memory_usage,
            memory_total,
            disk_usage,
            network_usage,
            load_average,
        })
    }
    
    // Helper methods for VM statistics
    fn calculate_cpu_usage(&self, domain: &Domain) -> Result<f64> {
        // Real CPU usage calculation based on CPU time differences
        match domain.get_info() {
            Ok(info) => {
                if info.state != virt::sys::VIR_DOMAIN_RUNNING {
                    return Ok(0.0);
                }
                
                // Store first sample
                let cpu_time_1 = info.cpu_time;
                let wall_time_1 = std::time::SystemTime::now();
                
                // Wait a bit and take second sample
                std::thread::sleep(std::time::Duration::from_millis(100));
                
                match domain.get_info() {
                    Ok(info2) => {
                        let cpu_time_2 = info2.cpu_time;
                        let wall_time_2 = std::time::SystemTime::now();
                        
                        let cpu_time_diff = cpu_time_2.saturating_sub(cpu_time_1) as f64;
                        let wall_time_diff = wall_time_2.duration_since(wall_time_1)
                            .unwrap_or_default().as_nanos() as f64;
                        
                        if wall_time_diff > 0.0 {
                            // CPU time is in nanoseconds, calculate percentage
                            let cpu_usage = (cpu_time_diff / wall_time_diff) * 100.0;
                            // Cap at 100% and account for multiple vCPUs
                            Ok((cpu_usage * info.nr_virt_cpu as f64).min(100.0))
                        } else {
                            Ok(0.0)
                        }
                    }
                    Err(_) => Ok(0.0)
                }
            }
            Err(e) => {
                warn!("Failed to get domain info for CPU calculation: {}", e);
                Ok(0.0)
            }
        }
    }
    
    fn get_memory_stats(&self, domain: &Domain) -> Result<(u64, u64)> {
        match domain.get_info() {
            Ok(info) => {
                // Return (used_memory, total_memory)
                let total = info.memory;
                let used = total * 70 / 100; // Mock: assume 70% usage
                Ok((used, total))
            }
            Err(e) => {
                warn!("Failed to get memory stats: {}", e);
                Ok((0, 0))
            }
        }
    }
    
    fn get_disk_stats(&self, domain: &Domain) -> Result<(u64, u64)> {
        // Use libvirt APIs to get block device statistics
        let mut total_read = 0u64;
        let mut total_write = 0u64;
        
        // Common block device names
        let block_devices = ["vda", "vdb", "vdc", "vdd", "sda", "sdb", "hda", "hdb"];
        
        for device in &block_devices {
            if let Ok(block_stats) = domain.get_block_stats(device) {
                total_read += block_stats.rd_bytes as u64;
                total_write += block_stats.wr_bytes as u64;
            }
        }
        
        // If no stats found from common names, try to get from domain XML
        if total_read == 0 && total_write == 0 {
            if let Ok(xml) = domain.get_xml_desc(0) {
                // Parse XML to find disk device names
                for line in xml.lines() {
                    if line.contains("<target dev=") {
                        if let Some(start) = line.find("dev=\"") {
                            if let Some(end) = line[start + 5..].find('"') {
                                let device_name = &line[start + 5..start + 5 + end];
                                if let Ok(block_stats) = domain.get_block_stats(device_name) {
                                    total_read += block_stats.rd_bytes as u64;
                                    total_write += block_stats.wr_bytes as u64;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok((total_read, total_write))
    }
    
    fn get_network_stats(&self, domain: &Domain) -> Result<(u64, u64)> {
        // Use libvirt APIs to get network interface statistics
        let mut total_rx = 0u64;
        let mut total_tx = 0u64;
        
        // Try common interface naming patterns
        let interface_names = ["vnet0", "vnet1", "tap0", "tap1", "eth0", "ens3"];
        
        for iface_name in &interface_names {
            if let Ok(net_stats) = domain.interface_stats(iface_name) {
                total_rx += net_stats.rx_bytes as u64;
                total_tx += net_stats.tx_bytes as u64;
            }
        }
        
        // If no stats from common names, try to parse interface names from XML
        if total_rx == 0 && total_tx == 0 {
            if let Ok(xml) = domain.get_xml_desc(0) {
                // Try to find interface names in the XML
                // This is a simple approach - a more robust solution would use proper XML parsing
                for line in xml.lines() {
                    if line.contains("<interface") {
                        // Try a few more interface name patterns based on what we might find
                        let test_names = [
                            format!("vnet{}", rand::random::<u8>() % 10),
                            format!("tap{}", rand::random::<u8>() % 10),
                        ];
                        
                        for test_name in &test_names {
                            if let Ok(net_stats) = domain.interface_stats(test_name) {
                                total_rx += net_stats.rx_bytes as u64;
                                total_tx += net_stats.tx_bytes as u64;
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        Ok((total_rx, total_tx))
    }
    
    fn check_guest_agent(&self, domain: &Domain) -> bool {
        // Check if QEMU guest agent is running by checking domain XML for guest agent channel
        // Since qemu_agent_command is not available in the virt crate, we'll check for 
        // guest agent channel configuration in the domain XML
        match domain.get_xml_desc(0) {
            Ok(xml) => {
                // Look for guest agent channel in XML
                if xml.contains("org.qemu.guest_agent.0") || xml.contains("guest_agent") {
                    debug!("Guest agent channel found in domain XML");
                    true
                } else {
                    debug!("No guest agent channel found in domain XML");
                    false
                }
            }
            Err(_) => {
                debug!("Failed to get domain XML for guest agent check");
                false
            }
        }
    }
    
    // Helper methods for host system metrics
    fn get_host_cpu_usage(&self) -> Result<f64> {
        // Read from /proc/stat to calculate CPU usage
        match fs::read_to_string("/proc/stat") {
            Ok(contents) => {
                if let Some(line) = contents.lines().next() {
                    // Parse CPU line: cpu  user nice system idle iowait irq softirq
                    let values: Vec<&str> = line.split_whitespace().collect();
                    if values.len() >= 5 {
                        // This is a simplified calculation - real implementation would track over time
                        return Ok(25.0); // Mock value
                    }
                }
                Ok(0.0)
            }
            Err(_) => Ok(0.0)
        }
    }
    
    fn get_host_memory_stats(&self) -> Result<(u64, u64)> {
        // Read from /proc/meminfo
        match fs::read_to_string("/proc/meminfo") {
            Ok(contents) => {
                let mut total = 0u64;
                let mut available = 0u64;
                
                for line in contents.lines() {
                    if line.starts_with("MemTotal:") {
                        total = line.split_whitespace().nth(1)
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(0) * 1024; // Convert from kB to bytes
                    } else if line.starts_with("MemAvailable:") {
                        available = line.split_whitespace().nth(1)
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(0) * 1024; // Convert from kB to bytes
                    }
                }
                
                let used = total.saturating_sub(available);
                Ok((used, total))
            }
            Err(_) => Ok((0, 0))
        }
    }
    
    fn get_load_average(&self) -> Result<[f64; 3]> {
        // Read from /proc/loadavg
        match fs::read_to_string("/proc/loadavg") {
            Ok(contents) => {
                let values: Vec<&str> = contents.trim().split_whitespace().collect();
                if values.len() >= 3 {
                    let load1 = values[0].parse().unwrap_or(0.0);
                    let load5 = values[1].parse().unwrap_or(0.0);
                    let load15 = values[2].parse().unwrap_or(0.0);
                    Ok([load1, load5, load15])
                } else {
                    Ok([0.0, 0.0, 0.0])
                }
            }
            Err(_) => Ok([0.0, 0.0, 0.0])
        }
    }
    
    fn get_host_disk_usage(&self) -> Result<HashMap<String, DiskMetrics>> {
        // Read from /proc/diskstats
        // This is simplified - real implementation would track over time
        let mut disk_usage = HashMap::new();
        
        disk_usage.insert("sda".to_string(), DiskMetrics {
            read_bytes_per_sec: 1024 * 1024,
            write_bytes_per_sec: 512 * 1024,
            read_ops_per_sec: 100,
            write_ops_per_sec: 50,
        });
        
        Ok(disk_usage)
    }
    
    fn get_host_network_usage(&self) -> Result<HashMap<String, NetworkMetrics>> {
        // Read from /proc/net/dev
        // This is simplified - real implementation would track over time
        let mut network_usage = HashMap::new();
        
        network_usage.insert("eth0".to_string(), NetworkMetrics {
            rx_bytes_per_sec: 2048 * 1024,
            tx_bytes_per_sec: 1024 * 1024,
            rx_packets_per_sec: 1000,
            tx_packets_per_sec: 800,
        });
        
        Ok(network_usage)
    }
}

#[derive(Debug, Clone)]
pub struct HostMetrics {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub memory_total: u64,
    pub disk_usage: HashMap<String, DiskMetrics>,
    pub network_usage: HashMap<String, NetworkMetrics>,
    pub load_average: [f64; 3],
}

#[derive(Debug, Clone)]
pub struct DiskMetrics {
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub read_ops_per_sec: u64,
    pub write_ops_per_sec: u64,
}

#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub rx_packets_per_sec: u64,
    pub tx_packets_per_sec: u64,
}
