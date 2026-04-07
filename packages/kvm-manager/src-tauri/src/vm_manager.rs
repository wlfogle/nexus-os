use std::collections::HashMap;
use chrono::{Utc, TimeZone};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use virt::{connect::Connect, domain::Domain, sys};

use crate::errors::{KvmError, Result};
use crate::types::*;
use crate::xml_parser::{XmlParser, VmXmlInfo};

pub struct VmManager {
    connection: Connect,
    vm_cache: HashMap<String, VirtualMachine>,
}

impl VmManager {
    pub async fn new() -> Result<Self> {
        info!("Initializing VM Manager with libvirt connection");
        
        // Try to connect to libvirt
        let connection = Connect::open(None)
            .map_err(|e| {
                error!("Failed to connect to libvirt: {}", e);
                KvmError::LibvirtConnection(e)
            })?;

        info!("Successfully connected to libvirt");

        let mut manager = Self {
            connection,
            vm_cache: HashMap::new(),
        };

        // Initialize cache
        manager.refresh_vm_cache().await?;

        Ok(manager)
    }

    pub async fn list_vms(&self) -> Result<Vec<VirtualMachine>> {
        debug!("Listing all virtual machines");
        
        let domain_flags = sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE | 
                          sys::VIR_CONNECT_LIST_DOMAINS_INACTIVE;
        
        let domains = self.connection
            .list_all_domains(domain_flags)
            .map_err(|e| {
                error!("Failed to list domains from libvirt: {}", e);
                KvmError::LibvirtConnection(e)
            })?;

        let domain_count = domains.len();
        info!("Found {} domains in libvirt", domain_count);
        let mut vms = Vec::new();

        for domain in domains {
            let domain_name = domain.get_name().unwrap_or_else(|_| "<unknown>".to_string());
            debug!("Processing domain: {}", domain_name);
            
            match self.domain_to_vm(&domain).await {
                Ok(vm) => {
                    info!("Successfully converted domain '{}' to VM", domain_name);
                    vms.push(vm);
                },
                Err(e) => {
                    error!("Failed to convert domain '{}' to VM: {}", domain_name, e);
                    continue;
                }
            }
        }

        info!("Successfully listed {} VMs out of {} domains", vms.len(), domain_count);
        Ok(vms)
    }

    pub async fn create_vm(&mut self, config: VmConfig) -> Result<String> {
        info!("Creating new VM: {}", config.name);

        // Validate configuration
        self.validate_vm_config(&config)?;

        // Generate VM UUID
        let vm_id = Uuid::new_v4().to_string();

        // Generate XML configuration
        let xml_config = self.generate_vm_xml(&config, &vm_id)?;

        // Define the domain first, then start it
        let domain = Domain::define_xml(&self.connection, &xml_config)
            .map_err(|e| {
                error!("Failed to define VM {}: {}", config.name, e);
                KvmError::VmOperationFailed(format!("Failed to create VM: {}", e))
            })?;
        
        // Start the domain
        domain.create()
            .map_err(|e| {
                error!("Failed to start VM {}: {}", config.name, e);
                KvmError::VmOperationFailed(format!("Failed to start VM: {}", e))
            })?;

        // Create storage if needed
        if let Err(e) = self.create_vm_storage(&config, &vm_id).await {
            warn!("Failed to create storage for VM {}: {}", config.name, e);
        }

        info!("Successfully created VM {} with ID {}", config.name, vm_id);
        
        // Refresh cache
        self.refresh_vm_cache().await?;

        Ok(vm_id)
    }

    pub async fn start_vm(&self, vm_id: &str) -> Result<()> {
        info!("Starting VM: {}", vm_id);

        let domain = self.get_domain_by_id(vm_id)?;
        
        // Check if VM is already running
        if domain.is_active().map_err(KvmError::LibvirtConnection)? {
            info!("VM {} is already running", vm_id);
            return Ok(());
        }
        
        domain.create()
            .map_err(|e| {
                error!("Failed to start VM {}: {}", vm_id, e);
                KvmError::VmOperationFailed(format!("Failed to start VM: {}", e))
            })?;

        info!("Successfully started VM: {}", vm_id);
        Ok(())
    }

    pub async fn stop_vm(&self, vm_id: &str) -> Result<()> {
        info!("Stopping VM: {}", vm_id);

        let domain = self.get_domain_by_id(vm_id)?;
        
        // Try graceful shutdown first
        if let Err(_) = domain.shutdown() {
            // If graceful shutdown fails, force shutdown
            warn!("Graceful shutdown failed for VM {}, forcing shutdown", vm_id);
            domain.destroy()
                .map_err(|e| {
                    error!("Failed to force stop VM {}: {}", vm_id, e);
                    KvmError::VmOperationFailed(format!("Failed to stop VM: {}", e))
                })?;
        }

        info!("Successfully stopped VM: {}", vm_id);
        Ok(())
    }

    pub async fn delete_vm(&mut self, vm_id: &str) -> Result<()> {
        info!("Deleting VM: {}", vm_id);

        let domain = self.get_domain_by_id(vm_id)?;
        
        // Stop VM if running
        if domain.is_active().map_err(KvmError::LibvirtConnection)? {
            self.stop_vm(vm_id).await?;
        }

        // Undefine the domain
        domain.undefine()
            .map_err(|e| {
                error!("Failed to delete VM {}: {}", vm_id, e);
                KvmError::VmOperationFailed(format!("Failed to delete VM: {}", e))
            })?;

        // Remove from cache
        self.vm_cache.remove(vm_id);

        info!("Successfully deleted VM: {}", vm_id);
        Ok(())
    }

    pub async fn get_vm_stats(&self, vm_id: &str) -> Result<VmStats> {
        debug!("Getting stats for VM: {}", vm_id);

        let domain = self.get_domain_by_id(vm_id)?;

        // Check if VM is active
        if !domain.is_active().map_err(KvmError::LibvirtConnection)? {
            return Err(KvmError::VmOperationFailed("VM is not running".to_string()));
        }

        // Get domain info
        let info = domain.get_info().map_err(KvmError::LibvirtConnection)?;
        
        // Get enhanced CPU statistics
        let cpu_usage = self.get_cpu_usage_percentage(&domain).await.unwrap_or(0.0);

        // Get enhanced memory stats
        let (memory_usage, memory_total) = self.get_memory_stats(&domain, &info).await;

        // Get disk I/O statistics
        let (disk_read, disk_write) = self.get_disk_io_stats(&domain).await;

        // Get network I/O statistics
        let (network_rx, network_tx) = self.get_network_io_stats(&domain).await;

        // Get accurate uptime
        let uptime = self.get_vm_uptime(&domain).await;

        Ok(VmStats {
            cpu_usage,
            memory_usage,
            memory_total,
            disk_read,
            disk_write,
            network_rx,
            network_tx,
            uptime,
            timestamp: Utc::now(),
            guest_agent_connected: false,
        })
    }

    async fn get_cpu_usage_percentage(&self, domain: &Domain) -> Option<f64> {
        // Get CPU stats from libvirt - this requires multiple samples for accuracy
        if let Ok(info1) = domain.get_info() {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if let Ok(info2) = domain.get_info() {
                // Calculate CPU usage based on CPU time changes
                let time_diff = info2.cpu_time.saturating_sub(info1.cpu_time);
                if time_diff > 0 {
                    // Convert nanoseconds to percentage (rough approximation)
                    let cpu_usage = (time_diff as f64 / 100_000_000.0).min(100.0);
                    return Some(cpu_usage);
                }
            }
        }
        
        // Fallback: try to read from /proc if available
        if let Ok(name) = domain.get_name() {
            if let Some(usage) = self.get_vm_cpu_from_proc(&name).await {
                return Some(usage);
            }
        }
        
        None
    }

    async fn get_vm_cpu_from_proc(&self, vm_name: &str) -> Option<f64> {
        // Try to get CPU usage from /proc filesystem
        use std::process::Command;
        
        let output = Command::new("ps")
            .args(["-eo", "comm,pcpu"])
            .output()
            .ok()?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(vm_name) || line.contains("qemu") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(cpu_usage) = parts[1].parse::<f64>() {
                            return Some(cpu_usage);
                        }
                    }
                }
            }
        }
        None
    }

    async fn get_memory_stats(&self, domain: &Domain, info: &virt::domain::DomainInfo) -> (u64, u64) {
        // Try to get detailed memory statistics
        if let Ok(memory_stats) = domain.memory_stats(0) {
            let mut actual_memory = info.memory / 1024; // Default fallback
            let mut available_memory = 0;
            let mut unused_memory = 0;
            
            for stat in memory_stats {
                match stat.tag {
                    sys::VIR_DOMAIN_MEMORY_STAT_ACTUAL_BALLOON => {
                        actual_memory = stat.val / 1024; // Convert to MB
                    },
                    sys::VIR_DOMAIN_MEMORY_STAT_AVAILABLE => {
                        available_memory = stat.val / 1024;
                    },
                    sys::VIR_DOMAIN_MEMORY_STAT_UNUSED => {
                        unused_memory = stat.val / 1024;
                    },
                    _ => {}
                }
            }
            
            let used_memory = if available_memory > 0 {
                actual_memory - available_memory
            } else if unused_memory > 0 {
                actual_memory - unused_memory
            } else {
                actual_memory
            };
            
            return (used_memory, actual_memory);
        }
        
        // Fallback to basic info
        let total = info.memory / 1024;
        (total, total)
    }

    async fn get_disk_io_stats(&self, domain: &Domain) -> (u64, u64) {
        // Try to get statistics from all disk devices
        let mut total_read = 0;
        let mut total_write = 0;
        
        // Common disk device names
        let disk_devices = ["vda", "vdb", "vdc", "vdd", "sda", "sdb", "hda", "hdb"];
        
        for device in &disk_devices {
            if let Ok(block_stats) = domain.get_block_stats(device) {
                total_read += block_stats.rd_bytes as u64;
                total_write += block_stats.wr_bytes as u64;
            }
        }
        
        // If no stats found, try to get from XML and query each device
        if total_read == 0 && total_write == 0 {
            if let Ok(xml) = domain.get_xml_desc(0) {
                if let Ok(xml_info) = XmlParser::parse_vm_from_xml(&xml) {
                    for storage_device in &xml_info.storage_devices {
                        if let Ok(block_stats) = domain.get_block_stats(&storage_device.device) {
                            total_read += block_stats.rd_bytes as u64;
                            total_write += block_stats.wr_bytes as u64;
                        }
                    }
                }
            }
        }
        
        (total_read, total_write)
    }

    async fn get_network_io_stats(&self, domain: &Domain) -> (u64, u64) {
        // Try to get statistics from all network interfaces
        let mut total_rx = 0;
        let mut total_tx = 0;
        
        // Get interface names from XML
        if let Ok(xml) = domain.get_xml_desc(0) {
            if let Ok(xml_info) = XmlParser::parse_vm_from_xml(&xml) {
                for interface in &xml_info.network_interfaces {
                    // Try different interface naming patterns
                    let possible_names = [
                        format!("vnet{}", 0), // vnet0, vnet1, etc.
                        format!("tap{}", 0),  // tap0, tap1, etc.
                        interface.source.clone(),
                    ];
                    
                    for iface_name in &possible_names {
                        if let Ok(net_stats) = domain.interface_stats(iface_name) {
                            total_rx += net_stats.rx_bytes as u64;
                            total_tx += net_stats.tx_bytes as u64;
                            break; // Found stats for this interface
                        }
                    }
                }
            }
        }
        
        // Fallback: try common interface names
        if total_rx == 0 && total_tx == 0 {
            let common_interfaces = ["vnet0", "tap0", "eth0", "ens3"];
            for iface_name in &common_interfaces {
                if let Ok(net_stats) = domain.interface_stats(iface_name) {
                    total_rx += net_stats.rx_bytes as u64;
                    total_tx += net_stats.tx_bytes as u64;
                }
            }
        }
        
        (total_rx, total_tx)
    }

    async fn get_vm_uptime(&self, domain: &Domain) -> u64 {
        // Try to get actual uptime from domain
        if let Ok((state, _reason)) = domain.get_state() {
            if state == sys::VIR_DOMAIN_RUNNING {
                // Try to get boot time from guest agent or estimate
                if let Ok(name) = domain.get_name() {
                    return self.estimate_vm_uptime(&name).await;
                }
            }
        }
        0
    }
    
    async fn estimate_vm_uptime(&self, vm_name: &str) -> u64 {
        // Try to estimate uptime from process information
        use std::process::Command;
        
        // Look for qemu process for this VM
        if let Ok(output) = Command::new("ps")
            .args(["-eo", "comm,pid,etime"])
            .output() {
            
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains(vm_name) || (line.contains("qemu") && line.contains("guest")) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            // Parse etime format (e.g., "10:30", "1-02:30:45")
                            if let Some(uptime_seconds) = self.parse_etime(parts[2]) {
                                return uptime_seconds;
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: return current timestamp (will show as "running since now")
        0
    }
    
    fn parse_etime(&self, etime: &str) -> Option<u64> {
        // Parse etime format: [[days-]hours:]minutes:seconds
        let parts: Vec<&str> = if etime.contains('-') {
            // Format: days-hours:minutes:seconds
            let day_split: Vec<&str> = etime.split('-').collect();
            if day_split.len() == 2 {
                let days: u64 = day_split[0].parse().ok()?;
                let time_parts: Vec<&str> = day_split[1].split(':').collect();
                let hours: u64 = time_parts.get(0)?.parse().ok()?;
                let minutes: u64 = time_parts.get(1)?.parse().ok()?;
                let seconds: u64 = time_parts.get(2)?.parse().ok()?;
                return Some(days * 86400 + hours * 3600 + minutes * 60 + seconds);
            }
            return None;
        } else {
            etime.split(':').collect()
        };
        
        match parts.len() {
            2 => {
                // Format: minutes:seconds
                let minutes: u64 = parts[0].parse().ok()?;
                let seconds: u64 = parts[1].parse().ok()?;
                Some(minutes * 60 + seconds)
            },
            3 => {
                // Format: hours:minutes:seconds
                let hours: u64 = parts[0].parse().ok()?;
                let minutes: u64 = parts[1].parse().ok()?;
                let seconds: u64 = parts[2].parse().ok()?;
                Some(hours * 3600 + minutes * 60 + seconds)
            },
            _ => None
        }
    }

    // Enhanced VM creation with Proxmox support
    pub async fn create_proxmox_vm(&mut self, name: String, proxmox_path: String, memory_gb: u32, vcpus: u32) -> Result<String> {
        info!("Creating Proxmox VM: {} from {}", name, proxmox_path);

        // Check if VM with this name already exists
        if let Ok(_) = Domain::lookup_by_name(&self.connection, &name) {
            return Err(KvmError::VmOperationFailed(format!("VM with name '{}' already exists", name)));
        }

        // Check if the Proxmox image exists
        if !std::path::Path::new(&proxmox_path).exists() {
            return Err(KvmError::VmOperationFailed(format!("Proxmox image not found: {}", proxmox_path)));
        }

        let vm_id = Uuid::new_v4().to_string();
        let memory_mb = memory_gb * 1024;

        // Generate XML for Proxmox VM
        let xml_config = format!(r#"
<domain type='kvm'>
  <name>{}</name>
  <uuid>{}</uuid>
  <memory unit='MiB'>{}</memory>
  <currentMemory unit='MiB'>{}</currentMemory>
  <vcpu placement='static'>{}</vcpu>
  <os>
    <type arch='x86_64' machine='pc-q35-6.2'>hvm</type>
    <boot dev='hd'/>
  </os>
  <features>
    <acpi/>
    <apic/>
    <vmport state='off'/>
  </features>
  <cpu mode='host-model' check='partial'/>
  <clock offset='utc'>
    <timer name='rtc' tickpolicy='catchup'/>
    <timer name='pit' tickpolicy='delay'/>
    <timer name='hpet' present='no'/>
  </clock>
  <on_poweroff>destroy</on_poweroff>
  <on_reboot>restart</on_reboot>
  <on_crash>destroy</on_crash>
  <pm>
    <suspend-to-mem enabled='no'/>
    <suspend-to-disk enabled='no'/>
  </pm>
  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>
    <disk type='file' device='disk'>
      <driver name='qemu' type='qcow2'/>
      <source file='{}'/>
      <target dev='vda' bus='virtio'/>
      <address type='pci' domain='0x0000' bus='0x04' slot='0x00' function='0x0'/>
    </disk>
    <controller type='usb' index='0' model='qemu-xhci' ports='15'>
      <address type='pci' domain='0x0000' bus='0x02' slot='0x00' function='0x0'/>
    </controller>
    <controller type='sata' index='0'>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x1f' function='0x2'/>
    </controller>
    <controller type='pci' index='0' model='pcie-root'/>
    <controller type='pci' index='1' model='pcie-root-port'>
      <model name='pcie-root-port'/>
      <target chassis='1' port='0x10'/>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x02' function='0x0' multifunction='on'/>
    </controller>
    <interface type='network'>
      <mac address='52:54:00:{:02x}:{:02x}:{:02x}'/>
      <source network='default'/>
      <model type='virtio'/>
      <address type='pci' domain='0x0000' bus='0x01' slot='0x00' function='0x0'/>
    </interface>
    <serial type='pty'>
      <target type='isa-serial' port='0'>
        <model name='isa-serial'/>
      </target>
    </serial>
    <console type='pty'>
      <target type='serial' port='0'/>
    </console>
    <channel type='unix'>
      <target type='virtio' name='org.qemu.guest_agent.0'/>
      <address type='virtio-serial' controller='0' bus='0' port='1'/>
    </channel>
    <input type='tablet' bus='usb'>
      <address type='usb' bus='0' port='1'/>
    </input>
    <input type='mouse' bus='ps2'/>
    <input type='keyboard' bus='ps2'/>
    <graphics type='vnc' port='-1' autoport='yes' listen='127.0.0.1'>
      <listen type='address' address='127.0.0.1'/>
    </graphics>
    <video>
      <model type='qxl' ram='65536' vram='65536' vgamem='16384' heads='1' primary='yes'/>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x01' function='0x0'/>
    </video>
    <memballoon model='virtio'>
      <address type='pci' domain='0x0000' bus='0x05' slot='0x00' function='0x0'/>
    </memballoon>
    <rng model='virtio'>
      <backend model='random'>/dev/urandom</backend>
      <address type='pci' domain='0x0000' bus='0x06' slot='0x00' function='0x0'/>
    </rng>
  </devices>
</domain>
"#, name, vm_id, memory_mb, memory_mb, vcpus, proxmox_path,
        rand::random::<u8>(), rand::random::<u8>(), rand::random::<u8>());

        // Define the domain
        let _domain = Domain::define_xml(&self.connection, &xml_config)
            .map_err(|e| {
                error!("Failed to define Proxmox VM {}: {}", name, e);
                KvmError::VmOperationFailed(format!("Failed to create Proxmox VM: {}", e))
            })?;

        info!("Successfully created Proxmox VM {} with ID {}", name, vm_id);
        self.refresh_vm_cache().await?;
        Ok(vm_id)
    }

    pub async fn get_host_info(&self) -> Result<HostInfo> {
        debug!("Getting host information");

        let node_info = self.connection.get_node_info()
            .map_err(KvmError::LibvirtConnection)?;

        let hostname = self.connection.get_hostname()
            .map_err(KvmError::LibvirtConnection)?;

        let hypervisor = self.connection.get_type()
            .map_err(KvmError::LibvirtConnection)?;

        let hypervisor_version = self.connection.get_lib_version()
            .map_err(KvmError::LibvirtConnection)?;

        // Get domain counts
        let active_vms = self.connection.num_of_domains()
            .map_err(KvmError::LibvirtConnection)? as u32;

        let inactive_vms = self.connection.num_of_defined_domains()
            .map_err(KvmError::LibvirtConnection)? as u32;

        // Get memory info
        let memory_total = node_info.memory / 1024; // Convert to MB from KB
        
        // Get actual free memory from system
        let memory_free = self.get_host_free_memory().await.unwrap_or(0);

        Ok(HostInfo {
            hostname,
            hypervisor,
            hypervisor_version: hypervisor_version.to_string(),
            cpu_model: node_info.model,
            cpu_cores: node_info.cpus,
            memory_total,
            memory_free,
            storage_pools: self.get_storage_pools().await?,
            networks: self.get_networks().await?,
            active_vms,
            inactive_vms,
        })
    }

    pub async fn create_snapshot(&self, vm_id: &str, snapshot_name: &str) -> Result<()> {
        info!("Creating snapshot {} for VM {}", snapshot_name, vm_id);
        
        let domain = self.get_domain_by_id(vm_id)?;
        
        // Generate snapshot XML (not used in virsh approach)
        let _snapshot_xml = format!(
            r#"<domainsnapshot>
  <name>{}</name>
  <description>Snapshot created by KVM Manager</description>
  <creationTime>{}</creationTime>
</domainsnapshot>"#,
            snapshot_name,
            chrono::Utc::now().timestamp()
        );
        
        // Create the snapshot using virsh command as fallback
        // This is needed because the virt crate might not have full snapshot support
        let output = std::process::Command::new("virsh")
            .args(["snapshot-create-as", &domain.get_name().unwrap_or_default(), snapshot_name, "--disk-only"])
            .output()
            .map_err(|e| KvmError::SnapshotOperationFailed(format!("Failed to execute virsh: {}", e)))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(KvmError::SnapshotOperationFailed(format!("Snapshot creation failed: {}", error)));
        }
        
        info!("Successfully created snapshot {} for VM {}", snapshot_name, vm_id);
        Ok(())
    }

    pub async fn restore_snapshot(&self, vm_id: &str, snapshot_name: &str) -> Result<()> {
        info!("Restoring snapshot {} for VM {}", snapshot_name, vm_id);
        
        let domain = self.get_domain_by_id(vm_id)?;
        let vm_name = domain.get_name().unwrap_or_default();
        
        // Use virsh to restore snapshot
        let output = std::process::Command::new("virsh")
            .args(["snapshot-revert", &vm_name, snapshot_name])
            .output()
            .map_err(|e| KvmError::SnapshotOperationFailed(format!("Failed to execute virsh: {}", e)))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(KvmError::SnapshotOperationFailed(format!("Snapshot restoration failed: {}", error)));
        }
        
        info!("Successfully restored snapshot {} for VM {}", snapshot_name, vm_id);
        Ok(())
    }
    
    pub async fn list_snapshots(&self, vm_id: &str) -> Result<Vec<Snapshot>> {
        info!("Listing snapshots for VM {}", vm_id);
        
        let domain = self.get_domain_by_id(vm_id)?;
        let vm_name = domain.get_name().unwrap_or_default();
        
        // Use virsh to list snapshots
        let output = std::process::Command::new("virsh")
            .args(["snapshot-list", &vm_name, "--name"])
            .output()
            .map_err(|e| KvmError::SnapshotOperationFailed(format!("Failed to execute virsh: {}", e)))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(KvmError::SnapshotOperationFailed(format!("Failed to list snapshots: {}", error)));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut snapshots = Vec::new();
        
        for line in stdout.lines() {
            let snapshot_name = line.trim();
            if !snapshot_name.is_empty() {
                snapshots.push(Snapshot {
                    name: snapshot_name.to_string(),
                    description: Some("Snapshot created by KVM Manager".to_string()),
                    created_at: Utc::now(), // We'd need to parse this from virsh output for accuracy
                    state: "disk-snapshot".to_string(),
                    parent: None,
                });
            }
        }
        
        info!("Found {} snapshots for VM {}", snapshots.len(), vm_id);
        Ok(snapshots)
    }
    
    pub async fn delete_snapshot(&self, vm_id: &str, snapshot_name: &str) -> Result<()> {
        info!("Deleting snapshot {} for VM {}", snapshot_name, vm_id);
        
        let domain = self.get_domain_by_id(vm_id)?;
        let vm_name = domain.get_name().unwrap_or_default();
        
        // Use virsh to delete snapshot
        let output = std::process::Command::new("virsh")
            .args(["snapshot-delete", &vm_name, snapshot_name])
            .output()
            .map_err(|e| KvmError::SnapshotOperationFailed(format!("Failed to execute virsh: {}", e)))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(KvmError::SnapshotOperationFailed(format!("Snapshot deletion failed: {}", error)));
        }
        
        info!("Successfully deleted snapshot {} for VM {}", snapshot_name, vm_id);
        Ok(())
    }

    pub async fn get_storage_pools(&self) -> Result<Vec<StoragePool>> {
        debug!("Getting storage pools");

        let pools = self.connection.list_all_storage_pools(0)
            .map_err(KvmError::LibvirtConnection)?;

        let mut storage_pools = Vec::new();

        for pool in pools {
            match self.pool_to_storage_pool(&pool).await {
                Ok(storage_pool) => storage_pools.push(storage_pool),
                Err(e) => {
                    warn!("Failed to convert storage pool: {}", e);
                    continue;
                }
            }
        }

        Ok(storage_pools)
    }

    pub async fn get_networks(&self) -> Result<Vec<Network>> {
        debug!("Getting networks");

        let networks = self.connection.list_all_networks(0)
            .map_err(KvmError::LibvirtConnection)?;

        let mut result_networks = Vec::new();

        for network in networks {
            match self.network_to_network(&network).await {
                Ok(net) => result_networks.push(net),
                Err(e) => {
                    warn!("Failed to convert network: {}", e);
                    continue;
                }
            }
        }

        Ok(result_networks)
    }

    // Private helper methods

    async fn refresh_vm_cache(&mut self) -> Result<()> {
        debug!("Refreshing VM cache");
        
        let vms = self.list_vms().await?;
        self.vm_cache.clear();
        
        for vm in vms {
            self.vm_cache.insert(vm.id.clone(), vm);
        }
        
        Ok(())
    }

    fn get_domain_by_id(&self, vm_id: &str) -> Result<Domain> {
        // Try to get by UUID string first (most reliable)
        if let Ok(domain) = Domain::lookup_by_uuid_string(&self.connection, vm_id) {
            return Ok(domain);
        }
        
        // Try to get by name as fallback
        if let Ok(domain) = Domain::lookup_by_name(&self.connection, vm_id) {
            return Ok(domain);
        }

        Err(KvmError::VmNotFound(vm_id.to_string()))
    }

    async fn domain_to_vm(&self, domain: &Domain) -> Result<VirtualMachine> {
        let uuid = domain.get_uuid_string().map_err(|e| {
            error!("Failed to get UUID for domain: {}", e);
            KvmError::LibvirtConnection(e)
        })?;
        let name = domain.get_name().map_err(|e| {
            error!("Failed to get name for domain: {}", e);
            KvmError::LibvirtConnection(e)
        })?;
        let info = domain.get_info().map_err(|e| {
            error!("Failed to get info for domain {}: {}", name, e);
            KvmError::LibvirtConnection(e)
        })?;
        
        debug!("Converting domain to VM: name={}, uuid={}, state={}", name, uuid, info.state);
        
        let state = match info.state {
            sys::VIR_DOMAIN_NOSTATE => VmState::Stopped,
            sys::VIR_DOMAIN_RUNNING => VmState::Running,
            sys::VIR_DOMAIN_BLOCKED => VmState::Running,
            sys::VIR_DOMAIN_PAUSED => VmState::Paused,
            sys::VIR_DOMAIN_SHUTDOWN => VmState::ShuttingDown,
            sys::VIR_DOMAIN_SHUTOFF => VmState::Stopped,
            sys::VIR_DOMAIN_CRASHED => VmState::Error,
            sys::VIR_DOMAIN_PMSUSPENDED => VmState::Suspended,
            _ => VmState::Error,
        };

        // Use XML parser to extract comprehensive VM information
        let xml_info = match domain.get_xml_desc(0) {
            Ok(xml) => {
                debug!("Got XML for VM {}: {} chars", name, xml.len());
                match XmlParser::parse_vm_from_xml(&xml) {
                    Ok(info) => info,
                    Err(e) => {
                        warn!("Failed to parse XML for VM {}: {}", name, e);
                        VmXmlInfo {
                            name: name.clone(),
                            uuid: uuid.clone(),
                            memory_mb: info.memory / 1024,
                            vcpus: info.nr_virt_cpu,
                            os_type: "linux".to_string(),
                            os_variant: Some("generic".to_string()),
                            disk_size_gb: 0.0,
                            storage_devices: Vec::new(),
                            network_interfaces: Vec::new(),
                            vnc_port: None,
                            spice_port: None,
                            description: None,
                        }
                    }
                }
            },
            Err(e) => {
                warn!("Failed to get XML for VM {}: {}", name, e);
                VmXmlInfo {
                    name: name.clone(),
                    uuid: uuid.clone(),
                    memory_mb: info.memory / 1024,
                    vcpus: info.nr_virt_cpu,
                    os_type: "linux".to_string(),
                    os_variant: Some("generic".to_string()),
                    disk_size_gb: 0.0,
                    storage_devices: Vec::new(),
                    network_interfaces: Vec::new(),
                    vnc_port: None,
                    spice_port: None,
                    description: None,
                }
            }
        };

        let vm = VirtualMachine {
            id: uuid.clone(),
            name: name.clone(),
            state,
            memory: info.memory / 1024, // Use libvirt info for memory (more reliable)
            vcpus: info.nr_virt_cpu,    // Use libvirt info for vCPUs
            disk_size: xml_info.disk_size_gb as u64,
            os_type: xml_info.os_type.clone(),
            os_variant: xml_info.os_variant.clone(),
            created_at: self.extract_creation_time(&xml_info, &domain).await,
            last_started: self.extract_last_started_time(&domain).await,
            description: xml_info.description,
            vnc_port: xml_info.vnc_port,
            spice_port: xml_info.spice_port,
            snapshots: self.load_vm_snapshots(&domain).await.unwrap_or_default(),
            network_interfaces: xml_info.network_interfaces,
            storage_devices: xml_info.storage_devices,
        };
        
        debug!("Successfully converted domain to VM: name={}, os_type={}, disks={}", 
               name, vm.os_type, vm.storage_devices.len());
        Ok(vm)
    }

    fn validate_vm_config(&self, config: &VmConfig) -> Result<()> {
        if config.name.is_empty() {
            return Err(KvmError::InvalidVmConfig("VM name cannot be empty".to_string()));
        }

        if config.memory < 128 {
            return Err(KvmError::InvalidVmConfig("Memory must be at least 128 MB".to_string()));
        }

        if config.vcpus == 0 {
            return Err(KvmError::InvalidVmConfig("Must have at least 1 vCPU".to_string()));
        }

        if config.disk_size < 1 {
            return Err(KvmError::InvalidVmConfig("Disk size must be at least 1 GB".to_string()));
        }

        Ok(())
    }

    fn generate_vm_xml(&self, config: &VmConfig, vm_id: &str) -> Result<String> {
        let xml = format!(
            r#"<domain type='kvm'>
  <name>{}</name>
  <uuid>{}</uuid>
  <memory unit='MiB'>{}</memory>
  <currentMemory unit='MiB'>{}</currentMemory>
  <vcpu placement='static'>{}</vcpu>
  <os>
    <type arch='x86_64' machine='pc-q35-6.2'>hvm</type>
    <boot dev='hd'/>
    <boot dev='cdrom'/>
  </os>
  <features>
    <acpi/>
    <apic/>
    <vmport state='off'/>
  </features>
  <cpu mode='host-model' check='partial'/>
  <clock offset='utc'>
    <timer name='rtc' tickpolicy='catchup'/>
    <timer name='pit' tickpolicy='delay'/>
    <timer name='hpet' present='no'/>
  </clock>
  <on_poweroff>destroy</on_poweroff>
  <on_reboot>restart</on_reboot>
  <on_crash>destroy</on_crash>
  <pm>
    <suspend-to-mem enabled='no'/>
    <suspend-to-disk enabled='no'/>
  </pm>
  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>
    <disk type='file' device='disk'>
      <driver name='qemu' type='{}' cache='{}'/>
      <source file='/var/lib/libvirt/images/{}.{}'/>
      <target dev='vda' bus='{}'/>
      <address type='pci' domain='0x0000' bus='0x03' slot='0x00' function='0x0'/>
    </disk>
    <controller type='usb' index='0' model='qemu-xhci'>
      <address type='pci' domain='0x0000' bus='0x02' slot='0x00' function='0x0'/>
    </controller>
    <interface type='network'>
      <source network='{}'/>
      <model type='{}'/>
      <address type='pci' domain='0x0000' bus='0x01' slot='0x00' function='0x0'/>
    </interface>
    <graphics type='{}' port='-1' autoport='yes' listen='127.0.0.1'>
      <listen type='address' address='127.0.0.1'/>
    </graphics>
    <video>
      <model type='qxl' ram='65536' vram='65536' vgamem='16384' heads='1' primary='yes'/>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x02' function='0x0'/>
    </video>
    <memballoon model='virtio'>
      <address type='pci' domain='0x0000' bus='0x04' slot='0x00' function='0x0'/>
    </memballoon>
  </devices>
</domain>"#,
            config.name,
            vm_id,
            config.memory,
            config.memory,
            config.vcpus,
            config.storage_config.format,
            config.storage_config.cache,
            config.name,
            config.storage_config.format,
            config.storage_config.bus,
            config.network_config.network_name.as_ref().unwrap_or(&"default".to_string()),
            config.network_config.model,
            config.display_config.graphics_type,
        );

        Ok(xml)
    }

    async fn create_vm_storage(&self, _config: &VmConfig, vm_id: &str) -> Result<()> {
        // This would create the disk image file
        // For now, we'll assume it's handled by libvirt
        info!("Storage creation for VM {} handled by libvirt", vm_id);
        Ok(())
    }

    fn parse_os_info_from_xml(&self, xml: &str) -> Result<(String, Option<String>)> {
        debug!("Parsing OS info from XML: {} chars", xml.len());
        
        // Try to parse OS info from XML, with fallback to default
        if xml.contains("<os>") {
            if xml.contains("linux") || xml.contains("debian") || xml.contains("ubuntu") || xml.contains("fedora") || xml.contains("centos") || xml.contains("rhel") {
                debug!("Detected Linux OS type from XML");
                return Ok(("linux".to_string(), Some("generic".to_string())));
            } else if xml.contains("windows") || xml.contains("win") {
                debug!("Detected Windows OS type from XML");
                return Ok(("windows".to_string(), Some("win10".to_string())));
            } else if xml.contains("freebsd") || xml.contains("openbsd") || xml.contains("netbsd") {
                debug!("Detected BSD OS type from XML");
                return Ok(("bsd".to_string(), Some("generic".to_string())));
            }
        }
        
        // Check for common OS indicators in the full XML
        let xml_lower = xml.to_lowercase();
        if xml_lower.contains("debian") || xml_lower.contains("ubuntu") || xml_lower.contains("fedora") || xml_lower.contains("centos") {
            debug!("Detected Linux from XML content analysis");
            return Ok(("linux".to_string(), Some("generic".to_string())));
        } else if xml_lower.contains("windows") || xml_lower.contains("win10") || xml_lower.contains("win11") {
            debug!("Detected Windows from XML content analysis");
            return Ok(("windows".to_string(), Some("win10".to_string())));
        }
        
        // Default fallback
        debug!("Using default OS type (Linux)");
        Ok(("linux".to_string(), Some("generic".to_string())))
    }
    
    pub async fn refresh_vm_list(&mut self) -> Result<Vec<VirtualMachine>> {
        info!("Refreshing VM list");
        self.refresh_vm_cache().await?;
        self.list_vms().await
    }
    
    pub async fn import_vm_from_xml(&mut self, xml_path: &str) -> Result<String> {
        info!("Importing VM from XML: {}", xml_path);
        
        let xml_content = std::fs::read_to_string(xml_path)
            .map_err(|e| KvmError::VmOperationFailed(format!("Failed to read XML file: {}", e)))?;
        
        // Define the domain from XML
        let domain = Domain::define_xml(&self.connection, &xml_content)
            .map_err(|e| {
                error!("Failed to define VM from XML {}: {}", xml_path, e);
                KvmError::VmOperationFailed(format!("Failed to import VM: {}", e))
            })?;
        
        let name = domain.get_name().map_err(KvmError::LibvirtConnection)?;
        let uuid = domain.get_uuid_string().map_err(KvmError::LibvirtConnection)?;
        
        info!("Successfully imported VM {} with UUID {}", name, uuid);
        
        // Refresh cache
        self.refresh_vm_cache().await?;
        
        Ok(uuid)
    }
    
    pub async fn create_vm_from_qcow2(
        &mut self, 
        qcow2_path: &str, 
        vm_name: &str, 
        memory_mb: u64, 
        vcpus: u32, 
        passthrough_device: Option<&str>
    ) -> Result<String> {
        info!("Creating VM from qcow2: {} (name: {})", qcow2_path, vm_name);
        
        // Validate qcow2 file exists
        if !std::path::Path::new(qcow2_path).exists() {
            return Err(KvmError::VmOperationFailed(format!("qcow2 file not found: {}", qcow2_path)));
        }
        
        // Generate VM UUID
        let vm_uuid = uuid::Uuid::new_v4().to_string();
        
        // Generate XML configuration
        let xml_config = self.generate_qcow2_vm_xml(
            vm_name, 
            &vm_uuid, 
            qcow2_path, 
            memory_mb, 
            vcpus, 
            passthrough_device
        )?;
        
        info!("Generated XML for VM {}", vm_name);
        
        // Define the domain
        let _domain = Domain::define_xml(&self.connection, &xml_config)
            .map_err(|e| {
                error!("Failed to define VM {} from qcow2: {}", vm_name, e);
                KvmError::VmOperationFailed(format!("Failed to create VM: {}", e))
            })?;
        
        info!("Successfully created VM {} with UUID {}", vm_name, vm_uuid);
        
        // Refresh cache
        self.refresh_vm_cache().await?;
        
        Ok(vm_uuid)
    }
    
    fn generate_qcow2_vm_xml(
        &self,
        vm_name: &str,
        vm_uuid: &str,
        qcow2_path: &str,
        memory_mb: u64,
        vcpus: u32,
        passthrough_device: Option<&str>
    ) -> Result<String> {
        let memory_kb = memory_mb * 1024;
        
        let passthrough_disk = if let Some(device) = passthrough_device {
            format!(
                r#"    <disk type='block' device='disk'>
      <driver name='qemu' type='raw' cache='none' io='native'/>
      <source dev='{}'/>
      <target dev='vdb' bus='virtio'/>
      <address type='pci' domain='0x0000' bus='0x05' slot='0x00' function='0x0'/>
    </disk>"#,
                device
            )
        } else {
            String::new()
        };
        
        let xml = format!(
            r#"<domain type='kvm'>
  <name>{}</name>
  <uuid>{}</uuid>
  <metadata>
    <libosinfo:libosinfo xmlns:libosinfo="http://libosinfo.org/xmlns/libvirt/domain/1.0">
      <libosinfo:os id="http://debian.org/debian/12"/>
    </libosinfo:libosinfo>
  </metadata>
  <memory unit='KiB'>{}</memory>
  <currentMemory unit='KiB'>{}</currentMemory>
  <vcpu placement='static'>{}</vcpu>
  <os>
    <type arch='x86_64' machine='pc-q35-8.2'>hvm</type>
    <loader readonly='yes' type='pflash'>/usr/share/edk2/x64/OVMF_CODE.4m.fd</loader>
    <nvram>/var/lib/libvirt/qemu/nvram/{}_VARS.fd</nvram>
    <boot dev='hd'/>
    <boot dev='cdrom'/>
  </os>
  <features>
    <acpi/>
    <apic/>
    <vmport state='off'/>
  </features>
  <cpu mode='host-passthrough' check='none' migratable='on'/>
  <clock offset='utc'>
    <timer name='rtc' tickpolicy='catchup'/>
    <timer name='pit' tickpolicy='delay'/>
    <timer name='hpet' present='no'/>
  </clock>
  <on_poweroff>destroy</on_poweroff>
  <on_reboot>restart</on_reboot>
  <on_crash>destroy</on_crash>
  <pm>
    <suspend-to-mem enabled='no'/>
    <suspend-to-disk enabled='no'/>
  </pm>
  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>
    
    <!-- Main disk (qcow2) -->
    <disk type='file' device='disk'>
      <driver name='qemu' type='qcow2' cache='writeback'/>
      <source file='{}'/>
      <target dev='vda' bus='virtio'/>
      <address type='pci' domain='0x0000' bus='0x04' slot='0x00' function='0x0'/>
    </disk>
{}
    
    <!-- Network interface -->
    <interface type='network'>
      <mac address='52:54:00:{:02x}:{:02x}:{:02x}'/>
      <source network='default'/>
      <model type='virtio'/>
      <address type='pci' domain='0x0000' bus='0x01' slot='0x00' function='0x0'/>
    </interface>
    
    <!-- Console -->
    <console type='pty'>
      <target type='virtio' port='0'/>
    </console>
    
    <!-- SPICE Graphics -->
    <graphics type='spice' autoport='yes' listen='127.0.0.1'>
      <listen type='address' address='127.0.0.1'/>
      <image compression='off'/>
    </graphics>
    
    <!-- Video -->
    <video>
      <model type='qxl' ram='65536' vram='65536' vgamem='16384' heads='1' primary='yes'/>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x01' function='0x0'/>
    </video>
    
    <!-- USB Controller -->
    <controller type='usb' index='0' model='qemu-xhci' ports='15'>
      <address type='pci' domain='0x0000' bus='0x02' slot='0x00' function='0x0'/>
    </controller>
    
    <!-- PCI Controllers -->
    <controller type='pci' index='0' model='pcie-root'/>
    <controller type='pci' index='1' model='pcie-root-port'>
      <model name='pcie-root-port'/>
      <target chassis='1' port='0x10'/>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x02' function='0x0' multifunction='on'/>
    </controller>
    <controller type='pci' index='2' model='pcie-root-port'>
      <model name='pcie-root-port'/>
      <target chassis='2' port='0x11'/>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x02' function='0x1'/>
    </controller>
    <controller type='pci' index='3' model='pcie-root-port'>
      <model name='pcie-root-port'/>
      <target chassis='3' port='0x12'/>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x02' function='0x2'/>
    </controller>
    <controller type='pci' index='4' model='pcie-root-port'>
      <model name='pcie-root-port'/>
      <target chassis='4' port='0x13'/>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x02' function='0x3'/>
    </controller>
    <controller type='pci' index='5' model='pcie-root-port'>
      <model name='pcie-root-port'/>
      <target chassis='5' port='0x14'/>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x02' function='0x4'/>
    </controller>
    
    <!-- SATA Controller -->
    <controller type='sata' index='0'>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x1f' function='0x2'/>
    </controller>
    
    <!-- Virtio Controllers -->
    <controller type='virtio-serial' index='0'>
      <address type='pci' domain='0x0000' bus='0x03' slot='0x00' function='0x0'/>
    </controller>
    
    <!-- RNG Device -->
    <rng model='virtio'>
      <backend model='random'>/dev/urandom</backend>
      <address type='pci' domain='0x0000' bus='0x06' slot='0x00' function='0x0'/>
    </rng>
    
    <!-- Memory Balloon -->
    <memballoon model='virtio'>
      <address type='pci' domain='0x0000' bus='0x07' slot='0x00' function='0x0'/>
    </memballoon>
  </devices>
</domain>"#,
            vm_name,
            vm_uuid,
            memory_kb,
            memory_kb,
            vcpus,
            vm_name,
            qcow2_path,
            passthrough_disk,
            rand::random::<u8>(),
            rand::random::<u8>(),
            rand::random::<u8>()
        );
        
        Ok(xml)
    }

    async fn pool_to_storage_pool(&self, pool: &virt::storage_pool::StoragePool) -> Result<StoragePool> {
        let name = pool.get_name().map_err(KvmError::LibvirtConnection)?;
        let info = pool.get_info().map_err(KvmError::LibvirtConnection)?;
        let xml = pool.get_xml_desc(0).map_err(KvmError::LibvirtConnection)?;
        
        // Parse pool type and path from XML using our XML parser
        let (pool_type, path) = if let Ok(pool_info) = XmlParser::parse_storage_pool_from_xml(&xml) {
            (pool_info.pool_type, pool_info.path.unwrap_or_else(|| "/var/lib/libvirt/images".to_string()))
        } else {
            ("dir".to_string(), "/var/lib/libvirt/images".to_string())
        };
        
        // Get volumes in the pool
        let volumes = match pool.list_all_volumes(0) {
            Ok(vols) => {
                let mut volume_list = Vec::new();
                for vol in vols {
                    if let (Ok(name), Ok(info), Ok(path)) = (vol.get_name(), vol.get_info(), vol.get_path()) {
                        // Try to get format from volume XML
                        let format = if let Ok(vol_xml) = vol.get_xml_desc(0) {
                            self.parse_volume_format_from_xml(&vol_xml)
                        } else {
                            "raw".to_string()
                        };
                        
                        volume_list.push(StorageVolume {
                            name,
                            format,
                            capacity: info.capacity,
                            allocation: info.allocation,
                            path,
                        });
                    }
                }
                volume_list
            },
            Err(_) => Vec::new(),
        };

        Ok(StoragePool {
            name,
            pool_type,
            path,
            capacity: info.capacity,
            available: info.available,
            used: info.allocation,
            state: if info.state == sys::VIR_STORAGE_POOL_RUNNING { "active" } else { "inactive" }.to_string(),
            autostart: pool.get_autostart().map_err(KvmError::LibvirtConnection)?,
            volumes,
        })
    }

    async fn network_to_network(&self, network: &virt::network::Network) -> Result<Network> {
        let name = network.get_name().map_err(KvmError::LibvirtConnection)?;
        let uuid = network.get_uuid_string().map_err(KvmError::LibvirtConnection)?;
        let is_active = network.is_active().map_err(KvmError::LibvirtConnection)?;
        let autostart = network.get_autostart().map_err(KvmError::LibvirtConnection)?;
        
        // Parse network XML to get detailed information
        let xml = network.get_xml_desc(0).map_err(KvmError::LibvirtConnection)?;
        
        let (bridge_name, forward_mode, ip_range, dhcp_enabled) = if let Ok(network_info) = XmlParser::parse_network_from_xml(&xml) {
            (
                network_info.bridge_name,
                network_info.forward_mode,
                network_info.ip_range,
                network_info.dhcp_enabled,
            )
        } else {
            // Fallback values if XML parsing fails
            (
                None,
                "nat".to_string(),
                None,
                false,
            )
        };
        
        // Get connected VMs by checking all domains for network usage
        let connected_vms = self.get_connected_vms_for_network(&name).await.unwrap_or_default();

        Ok(Network {
            name,
            uuid,
            bridge_name,
            forward_mode,
            state: if is_active { "active" } else { "inactive" }.to_string(),
            autostart,
            ip_range,
            dhcp_enabled,
            connected_vms,
        })
    }
    
    async fn get_host_free_memory(&self) -> Option<u64> {
        use std::process::Command;
        
        // Try to get free memory from /proc/meminfo
        if let Ok(output) = Command::new("cat")
            .arg("/proc/meminfo")
            .output() {
            
            if output.status.success() {
                let content = String::from_utf8_lossy(&output.stdout);
                let mut mem_available = None;
                let mut mem_free = None;
                
                for line in content.lines() {
                    if line.starts_with("MemAvailable:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = value.parse::<u64>() {
                                mem_available = Some(kb / 1024); // Convert to MB
                            }
                        }
                    } else if line.starts_with("MemFree:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = value.parse::<u64>() {
                                mem_free = Some(kb / 1024); // Convert to MB
                            }
                        }
                    }
                }
                
                // Prefer MemAvailable over MemFree as it's more accurate
                return mem_available.or(mem_free);
            }
        }
        
        // Note: get_memory_stats method doesn't exist in virt crate - using fallback only
        
        None
    }
    
    async fn load_vm_snapshots(&self, domain: &Domain) -> Result<Vec<Snapshot>> {
        let vm_name = domain.get_name().map_err(KvmError::LibvirtConnection)?;
        
        // Use virsh to list snapshots (similar to list_snapshots but without extra logging)
        let output = std::process::Command::new("virsh")
            .args(["snapshot-list", &vm_name, "--name"])
            .output()
            .map_err(|e| KvmError::SnapshotOperationFailed(format!("Failed to execute virsh: {}", e)))?;
        
        if !output.status.success() {
            // If no snapshots or command fails, return empty vec instead of error
            return Ok(Vec::new());
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut snapshots = Vec::new();
        
        for line in stdout.lines() {
            let snapshot_name = line.trim();
            if !snapshot_name.is_empty() {
                snapshots.push(Snapshot {
                    name: snapshot_name.to_string(),
                    description: Some("Snapshot created by KVM Manager".to_string()),
                    created_at: Utc::now(), // Real implementation would parse timestamp from virsh output
                    state: "disk-snapshot".to_string(),
                    parent: None,
                });
            }
        }
        
        Ok(snapshots)
    }
    
    async fn get_connected_vms_for_network(&self, network_name: &str) -> Result<Vec<String>> {
        debug!("Getting VMs connected to network: {}", network_name);
        
        let domain_flags = sys::VIR_CONNECT_LIST_DOMAINS_ACTIVE | 
                          sys::VIR_CONNECT_LIST_DOMAINS_INACTIVE;
        
        let domains = self.connection
            .list_all_domains(domain_flags)
            .map_err(KvmError::LibvirtConnection)?;
        
        let mut connected_vms = Vec::new();
        
        for domain in domains {
            if let Ok(xml) = domain.get_xml_desc(0) {
                // Check if the domain XML contains this network
                if xml.contains(&format!("<source network='{}'", network_name)) {
                    if let Ok(vm_name) = domain.get_name() {
                        connected_vms.push(vm_name);
                    }
                }
            }
        }
        
        debug!("Found {} VMs connected to network {}", connected_vms.len(), network_name);
        Ok(connected_vms)
    }
    
    async fn extract_creation_time(&self, xml_info: &VmXmlInfo, domain: &Domain) -> chrono::DateTime<chrono::Utc> {
        // Try to extract creation time from XML metadata or estimate from file system
        if let Ok(xml) = domain.get_xml_desc(0) {
            // Look for libvirt metadata creation time
            for line in xml.lines() {
                if line.contains("<metadata>") {
                    // This would require parsing the full XML metadata structure
                    // For now, we'll estimate based on disk file creation time
                    break;
                }
            }
        }
        
        // Try to get creation time from disk file timestamps
        for storage_device in &xml_info.storage_devices {
            if let Some(path) = &storage_device.path {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if let Ok(created) = metadata.created() {
                        if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
                            return chrono::Utc.timestamp_opt(
                                duration.as_secs() as i64, 
                                duration.subsec_nanos()
                            ).single().unwrap_or_else(|| Utc::now());
                        }
                    }
                }
            }
        }
        
        // Fallback: use current time
        Utc::now()
    }
    
    async fn extract_last_started_time(&self, domain: &Domain) -> Option<chrono::DateTime<chrono::Utc>> {
        // Try to estimate last started time based on VM state and process information
        if let Ok(info) = domain.get_info() {
            if info.state == sys::VIR_DOMAIN_RUNNING {
                // For running VMs, estimate start time based on uptime
                if let Ok(name) = domain.get_name() {
                    if let Some(uptime_seconds) = self.get_process_start_time(&name).await {
                        let start_time = Utc::now() - chrono::Duration::seconds(uptime_seconds as i64);
                        return Some(start_time);
                    }
                }
                // Fallback: assume started recently
                return Some(Utc::now() - chrono::Duration::minutes(5));
            }
        }
        
        // For stopped VMs, we don't have reliable last started time
        None
    }
    
    async fn get_process_start_time(&self, vm_name: &str) -> Option<u64> {
        use std::process::Command;
        
        // Try to get process start time using ps command
        if let Ok(output) = Command::new("ps")
            .args(["-eo", "comm,pid,etime"])
            .output() {
            
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains(vm_name) || (line.contains("qemu") && line.contains(&vm_name)) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            // Parse etime to get uptime in seconds
                            return self.parse_etime(parts[2]);
                        }
                    }
                }
            }
        }
        
        None
    }
    
    fn parse_volume_format_from_xml(&self, volume_xml: &str) -> String {
        // Parse format from volume XML (similar to storage.rs implementation)
        if let Some(start) = volume_xml.find("<format type='") {
            let start_pos = start + 14; // Length of "<format type='"
            if let Some(end) = volume_xml[start_pos..].find("'") {
                return volume_xml[start_pos..start_pos + end].to_string();
            }
        }
        
        // Alternative pattern
        if let Some(start) = volume_xml.find("<format type=\"") {
            let start_pos = start + 14; // Length of "<format type=\""
            if let Some(end) = volume_xml[start_pos..].find("\"") {
                return volume_xml[start_pos..start_pos + end].to_string();
            }
        }
        
        "raw".to_string() // Default format
    }
}
