use tracing::debug;
use crate::errors::Result;
use crate::types::*;

pub struct XmlParser;

impl XmlParser {
    /// Parse VM properties from libvirt XML
    pub fn parse_vm_from_xml(xml: &str) -> Result<VmXmlInfo> {
        debug!("Parsing VM XML: {} chars", xml.len());
        
        let mut vm_info = VmXmlInfo::default();
        
        // Parse basic info
        vm_info.name = Self::extract_between_tags(xml, "name")
            .unwrap_or_else(|| "unknown".to_string());
        
        vm_info.uuid = Self::extract_between_tags(xml, "uuid")
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        // Parse memory (in KiB, convert to MB)
        if let Some(memory_str) = Self::extract_between_tags(xml, "memory") {
            if let Ok(memory_kib) = memory_str.parse::<u64>() {
                vm_info.memory_mb = memory_kib / 1024;
            }
        }
        
        // Parse vCPUs
        if let Some(vcpus_str) = Self::extract_between_tags(xml, "vcpu") {
            if let Ok(vcpus) = vcpus_str.parse::<u32>() {
                vm_info.vcpus = vcpus;
            }
        }
        
        // Parse OS info
        let (os_type, os_variant) = Self::parse_os_info(xml);
        vm_info.os_type = os_type;
        vm_info.os_variant = os_variant;
        
        // Parse storage devices
        vm_info.storage_devices = Self::parse_storage_devices(xml);
        vm_info.disk_size_gb = vm_info.storage_devices.iter()
            .map(|d| d.size_gb)
            .sum();
        
        // Parse network interfaces
        vm_info.network_interfaces = Self::parse_network_interfaces(xml);
        
        // Parse graphics ports
        vm_info.vnc_port = Self::parse_vnc_port(xml);
        vm_info.spice_port = Self::parse_spice_port(xml);
        
        // Parse description from metadata
        vm_info.description = Self::extract_description(xml);
        
        debug!("Parsed VM info: name={}, memory={}MB, vcpus={}, disks={}", 
               vm_info.name, vm_info.memory_mb, vm_info.vcpus, vm_info.storage_devices.len());
        
        Ok(vm_info)
    }
    
    fn extract_between_tags(xml: &str, tag: &str) -> Option<String> {
        let start_tag = format!("<{}>", tag);
        let end_tag = format!("</{}>", tag);
        
        if let Some(start_pos) = xml.find(&start_tag) {
            let content_start = start_pos + start_tag.len();
            if let Some(end_pos) = xml[content_start..].find(&end_tag) {
                return Some(xml[content_start..content_start + end_pos].trim().to_string());
            }
        }
        None
    }
    
    fn parse_os_info(xml: &str) -> (String, Option<String>) {
        // Look for libosinfo metadata first
        if xml.contains("libosinfo:os") {
            if let Some(os_id) = Self::extract_attribute_value(xml, "libosinfo:os", "id") {
                if os_id.contains("debian") {
                    return ("linux".to_string(), Some("debian".to_string()));
                } else if os_id.contains("ubuntu") {
                    return ("linux".to_string(), Some("ubuntu".to_string()));
                } else if os_id.contains("fedora") {
                    return ("linux".to_string(), Some("fedora".to_string()));
                } else if os_id.contains("rhel") || os_id.contains("centos") {
                    return ("linux".to_string(), Some("rhel".to_string()));
                } else if os_id.contains("windows") {
                    return ("windows".to_string(), Some("win10".to_string()));
                }
            }
        }
        
        // Fallback to content-based detection
        let xml_lower = xml.to_lowercase();
        if xml_lower.contains("windows") || xml_lower.contains("win10") || xml_lower.contains("win11") {
            ("windows".to_string(), Some("win10".to_string()))
        } else if xml_lower.contains("freebsd") || xml_lower.contains("openbsd") {
            ("bsd".to_string(), Some("generic".to_string()))
        } else {
            ("linux".to_string(), Some("generic".to_string()))
        }
    }
    
    fn parse_storage_devices(xml: &str) -> Vec<StorageDevice> {
        let mut devices = Vec::new();
        
        // Find all disk elements
        let disk_pattern = r#"<disk\s+[^>]*>"#;
        if let Ok(regex) = regex::Regex::new(disk_pattern) {
            for disk_match in regex.find_iter(xml) {
                let disk_start = disk_match.start();
                
                // Find the closing </disk> tag
                if let Some(disk_end) = xml[disk_start..].find("</disk>") {
                    let disk_xml = &xml[disk_start..disk_start + disk_end + 7];
                    
                    if let Some(device) = Self::parse_single_disk(disk_xml) {
                        devices.push(device);
                    }
                }
            }
        }
        
        devices
    }
    
    fn parse_single_disk(disk_xml: &str) -> Option<StorageDevice> {
        let device_type = Self::extract_attribute_value(disk_xml, "disk", "device")?;
        
        if device_type != "disk" {
            return None; // Skip CD-ROM, floppy, etc.
        }
        
        let driver_type = Self::extract_attribute_value(disk_xml, "driver", "type")
            .unwrap_or_else(|| "raw".to_string());
        
        let target_dev = Self::extract_attribute_value(disk_xml, "target", "dev")
            .unwrap_or_else(|| "vda".to_string());
        
        let target_bus = Self::extract_attribute_value(disk_xml, "target", "bus")
            .unwrap_or_else(|| "virtio".to_string());
        
        // Handle both file-based and block device sources
        let source_file = Self::extract_attribute_value(disk_xml, "source", "file");
        let source_dev = Self::extract_attribute_value(disk_xml, "source", "dev");
        let source_path = source_file.or(source_dev);
        
        // Determine disk type based on XML type attribute
        let disk_type = Self::extract_attribute_value(disk_xml, "disk", "type")
            .unwrap_or_else(|| "file".to_string());
        
        // Try to get disk size from file/device if available
        let size_gb = if let Some(path) = &source_path {
            if disk_type == "block" {
                // For block devices, try to get size from /sys/block or blockdev
                Self::get_block_device_size(path).unwrap_or(0.0)
            } else {
                // For file-based images, use qemu-img
                Self::get_disk_size_from_file(path).unwrap_or(0.0)
            }
        } else {
            0.0
        };
        
        Some(StorageDevice {
            device: target_dev,
            type_: driver_type,
            size_gb,
            path: source_path,
            bus: target_bus,
            cache: Self::extract_attribute_value(disk_xml, "driver", "cache"),
        })
    }
    
    fn get_disk_size_from_file(file_path: &str) -> Option<f64> {
        use std::process::Command;
        
        let output = Command::new("qemu-img")
            .args(["info", "--output=json", file_path])
            .output()
            .ok()?;
        
        if output.status.success() {
            let info_json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
            let size_bytes = info_json["virtual-size"].as_u64()?;
            Some(size_bytes as f64 / 1024.0 / 1024.0 / 1024.0)
        } else {
            None
        }
    }
    
    fn get_block_device_size(device_path: &str) -> Option<f64> {
        use std::process::Command;
        
        // Try blockdev --getsize64 first (most reliable for block devices)
        let output = Command::new("blockdev")
            .args(["--getsize64", device_path])
            .output()
            .ok()?;
        
        if output.status.success() {
            let size_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(size_bytes) = size_str.trim().parse::<u64>() {
                return Some(size_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            }
        }
        
        // Fallback: try to get size from sysfs
        let device_name = device_path.trim_start_matches("/dev/");
        let sysfs_path = format!("/sys/block/{}/size", device_name);
        
        if let Ok(size_str) = std::fs::read_to_string(&sysfs_path) {
            if let Ok(size_sectors) = size_str.trim().parse::<u64>() {
                // Each sector is 512 bytes
                let size_bytes = size_sectors * 512;
                return Some(size_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
            }
        }
        
        None
    }
    
    fn parse_network_interfaces(xml: &str) -> Vec<NetworkInterface> {
        let mut interfaces = Vec::new();
        
        // Find all interface elements
        let interface_pattern = r#"<interface\s+[^>]*>"#;
        if let Ok(regex) = regex::Regex::new(interface_pattern) {
            for interface_match in regex.find_iter(xml) {
                let interface_start = interface_match.start();
                
                // Find the closing </interface> tag
                if let Some(interface_end) = xml[interface_start..].find("</interface>") {
                    let interface_xml = &xml[interface_start..interface_start + interface_end + 12];
                    
                    if let Some(interface) = Self::parse_single_interface(interface_xml) {
                        interfaces.push(interface);
                    }
                }
            }
        }
        
        interfaces
    }
    
    fn parse_single_interface(interface_xml: &str) -> Option<NetworkInterface> {
        let interface_type = Self::extract_attribute_value(interface_xml, "interface", "type")?;
        
        let mac_address = Self::extract_attribute_value(interface_xml, "mac", "address");
        let network_source = Self::extract_attribute_value(interface_xml, "source", "network");
        let bridge_source = Self::extract_attribute_value(interface_xml, "source", "bridge");
        let model_type = Self::extract_attribute_value(interface_xml, "model", "type")
            .unwrap_or_else(|| "rtl8139".to_string());
        
        Some(NetworkInterface {
            type_: interface_type,
            mac_address,
            source: network_source.or(bridge_source).unwrap_or_else(|| "default".to_string()),
            model: model_type,
            connected: true, // Assume connected if defined
        })
    }
    
    fn parse_vnc_port(xml: &str) -> Option<u16> {
        if xml.contains("type='vnc'") {
            Self::extract_attribute_value(xml, "graphics", "port")
                .and_then(|port_str| {
                    if port_str == "-1" {
                        None // Auto-allocated port
                    } else {
                        port_str.parse().ok()
                    }
                })
        } else {
            None
        }
    }
    
    fn parse_spice_port(xml: &str) -> Option<u16> {
        if xml.contains("type='spice'") {
            Self::extract_attribute_value(xml, "graphics", "port")
                .and_then(|port_str| {
                    if port_str == "-1" {
                        None // Auto-allocated port
                    } else {
                        port_str.parse().ok()
                    }
                })
        } else {
            None
        }
    }
    
    fn extract_description(xml: &str) -> Option<String> {
        // Look for description in metadata
        Self::extract_between_tags(xml, "description")
            .or_else(|| Self::extract_between_tags(xml, "title"))
    }
    
    /// Parse network configuration from libvirt XML
    pub fn parse_network_from_xml(xml: &str) -> Result<NetworkXmlInfo> {
        debug!("Parsing network XML: {} chars", xml.len());
        
        let mut network_info = NetworkXmlInfo::default();
        
        // Parse basic info
        network_info.name = Self::extract_between_tags(xml, "name")
            .unwrap_or_else(|| "unknown".to_string());
        
        network_info.uuid = Self::extract_between_tags(xml, "uuid")
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        // Parse forward mode
        network_info.forward_mode = Self::extract_attribute_value(xml, "forward", "mode")
            .unwrap_or_else(|| "nat".to_string());
        
        // Parse bridge name
        network_info.bridge_name = Self::extract_attribute_value(xml, "bridge", "name");
        
        // Parse IP configuration
        if let Some(ip_address) = Self::extract_attribute_value(xml, "ip", "address") {
            let netmask = Self::extract_attribute_value(xml, "ip", "netmask")
                .unwrap_or_else(|| "255.255.255.0".to_string());
            network_info.ip_range = Some(format!("{}/{}", ip_address, Self::netmask_to_cidr(&netmask)));
        }
        
        // Parse DHCP configuration
        network_info.dhcp_enabled = xml.contains("<dhcp>");
        if network_info.dhcp_enabled {
            network_info.dhcp_start = Self::extract_attribute_value(xml, "range", "start");
            network_info.dhcp_end = Self::extract_attribute_value(xml, "range", "end");
        }
        
        // Parse domain name
        network_info.domain = Self::extract_attribute_value(xml, "domain", "name");
        
        debug!("Parsed network info: name={}, mode={}, dhcp={}", 
               network_info.name, network_info.forward_mode, network_info.dhcp_enabled);
        
        Ok(network_info)
    }
    
    /// Parse storage pool configuration from libvirt XML
    pub fn parse_storage_pool_from_xml(xml: &str) -> Result<StoragePoolXmlInfo> {
        debug!("Parsing storage pool XML: {} chars", xml.len());
        
        let mut pool_info = StoragePoolXmlInfo::default();
        
        // Parse basic info
        pool_info.name = Self::extract_between_tags(xml, "name")
            .unwrap_or_else(|| "unknown".to_string());
        
        // Parse pool type from root element
        if let Some(start) = xml.find("<pool type='") {
            let start_pos = start + 12; // Length of "<pool type='"
            if let Some(end) = xml[start_pos..].find("'") {
                pool_info.pool_type = xml[start_pos..start_pos + end].to_string();
            }
        }
        
        // Parse target path
        if let Some(path_section) = Self::extract_section(xml, "target") {
            pool_info.path = Self::extract_between_tags(&path_section, "path");
        }
        
        // Parse source information for different pool types
        if let Some(source_section) = Self::extract_section(xml, "source") {
            match pool_info.pool_type.as_str() {
                "logical" => {
                    pool_info.source_name = Self::extract_between_tags(&source_section, "name");
                }
                "iscsi" => {
                    pool_info.source_host = Self::extract_attribute_value(&source_section, "host", "name");
                    pool_info.source_device = Self::extract_attribute_value(&source_section, "device", "path");
                }
                _ => {}
            }
        }
        
        debug!("Parsed storage pool info: name={}, type={}, path={:?}", 
               pool_info.name, pool_info.pool_type, pool_info.path);
        
        Ok(pool_info)
    }
    
    fn extract_section(xml: &str, section_name: &str) -> Option<String> {
        let start_tag = format!("<{}>", section_name);
        let end_tag = format!("</{}>", section_name);
        
        if let Some(start_pos) = xml.find(&start_tag) {
            if let Some(end_pos) = xml[start_pos..].find(&end_tag) {
                let section_end = start_pos + end_pos + end_tag.len();
                return Some(xml[start_pos..section_end].to_string());
            }
        }
        None
    }
    
    fn netmask_to_cidr(netmask: &str) -> u8 {
        match netmask {
            "255.255.255.255" => 32,
            "255.255.255.254" => 31,
            "255.255.255.252" => 30,
            "255.255.255.248" => 29,
            "255.255.255.240" => 28,
            "255.255.255.224" => 27,
            "255.255.255.192" => 26,
            "255.255.255.128" => 25,
            "255.255.255.0" => 24,
            "255.255.254.0" => 23,
            "255.255.252.0" => 22,
            "255.255.248.0" => 21,
            "255.255.240.0" => 20,
            "255.255.224.0" => 19,
            "255.255.192.0" => 18,
            "255.255.128.0" => 17,
            "255.255.0.0" => 16,
            "255.254.0.0" => 15,
            "255.252.0.0" => 14,
            "255.248.0.0" => 13,
            "255.240.0.0" => 12,
            "255.224.0.0" => 11,
            "255.192.0.0" => 10,
            "255.128.0.0" => 9,
            "255.0.0.0" => 8,
            _ => 24, // Default to /24
        }
    }
    
    fn extract_attribute_value(xml: &str, element: &str, attribute: &str) -> Option<String> {
        let pattern = format!(r#"<{}\s+[^>]*{}=['""]([^'"]*)['""]"#, element, attribute);
        if let Ok(regex) = regex::Regex::new(&pattern) {
            if let Some(captures) = regex.captures(xml) {
                return captures.get(1).map(|m| m.as_str().to_string());
            }
        }
        
        // Try alternative pattern with different quote order
        let pattern2 = format!(r#"<{}\s+[^>]*{}=([^>\s]*)"#, element, attribute);
        if let Ok(regex) = regex::Regex::new(&pattern2) {
            if let Some(captures) = regex.captures(xml) {
                return captures.get(1).map(|m| m.as_str().trim_matches('"').trim_matches('\'').to_string());
            }
        }
        
        None
    }
}

#[derive(Debug, Default, Clone)]
pub struct VmXmlInfo {
    pub name: String,
    pub uuid: String,
    pub memory_mb: u64,
    pub vcpus: u32,
    pub os_type: String,
    pub os_variant: Option<String>,
    pub disk_size_gb: f64,
    pub storage_devices: Vec<StorageDevice>,
    pub network_interfaces: Vec<NetworkInterface>,
    pub vnc_port: Option<u16>,
    pub spice_port: Option<u16>,
    pub description: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct NetworkXmlInfo {
    pub name: String,
    pub uuid: String,
    pub forward_mode: String,
    pub bridge_name: Option<String>,
    pub ip_range: Option<String>,
    pub dhcp_enabled: bool,
    pub dhcp_start: Option<String>,
    pub dhcp_end: Option<String>,
    pub domain: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct StoragePoolXmlInfo {
    pub name: String,
    pub pool_type: String,
    pub path: Option<String>,
    pub source_name: Option<String>,
    pub source_host: Option<String>,
    pub source_device: Option<String>,
}
