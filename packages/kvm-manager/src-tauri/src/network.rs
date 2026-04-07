use tracing::{info, error};
use virt::{connect::Connect, network::Network as LibvirtNetwork};
use crate::errors::{KvmError, Result};

pub struct NetworkManager {
    connection: Connect,
}

impl NetworkManager {
    pub fn new(connection: Connect) -> Self {
        Self { connection }
    }

    pub async fn create_network(&self, network_config: &NetworkCreateConfig) -> Result<String> {
        info!("Creating network: {}", network_config.name);
        
        let network_xml = self.generate_network_xml(network_config)?;
        
        // Define the network
        let network = LibvirtNetwork::define_xml(&self.connection, &network_xml)
            .map_err(|e| {
                error!("Failed to define network {}: {}", network_config.name, e);
                KvmError::NetworkOperationFailed(format!("Failed to create network: {}", e))
            })?;
        
        // Start the network if requested
        if network_config.auto_start {
            network.create()
                .map_err(|e| {
                    error!("Failed to start network {}: {}", network_config.name, e);
                    KvmError::NetworkOperationFailed(format!("Failed to start network: {}", e))
                })?;
        }
        
        info!("Successfully created network: {}", network_config.name);
        Ok(network_config.name.clone())
    }

    pub async fn delete_network(&self, network_name: &str) -> Result<()> {
        info!("Deleting network: {}", network_name);
        
        let network = LibvirtNetwork::lookup_by_name(&self.connection, network_name)
            .map_err(|e| {
                error!("Failed to find network {}: {}", network_name, e);
                KvmError::NetworkOperationFailed(format!("Network not found: {}", e))
            })?;
        
        // Stop network if it's active
        if network.is_active().map_err(KvmError::LibvirtConnection)? {
            network.destroy()
                .map_err(|e| {
                    error!("Failed to stop network {}: {}", network_name, e);
                    KvmError::NetworkOperationFailed(format!("Failed to stop network: {}", e))
                })?;
        }
        
        // Undefine the network
        network.undefine()
            .map_err(|e| {
                error!("Failed to delete network {}: {}", network_name, e);
                KvmError::NetworkOperationFailed(format!("Failed to delete network: {}", e))
            })?;
        
        info!("Successfully deleted network: {}", network_name);
        Ok(())
    }

    pub async fn start_network(&self, network_name: &str) -> Result<()> {
        info!("Starting network: {}", network_name);
        
        let network = LibvirtNetwork::lookup_by_name(&self.connection, network_name)
            .map_err(|e| {
                error!("Failed to find network {}: {}", network_name, e);
                KvmError::NetworkOperationFailed(format!("Network not found: {}", e))
            })?;
        
        network.create()
            .map_err(|e| {
                error!("Failed to start network {}: {}", network_name, e);
                KvmError::NetworkOperationFailed(format!("Failed to start network: {}", e))
            })?;
        
        info!("Successfully started network: {}", network_name);
        Ok(())
    }

    pub async fn stop_network(&self, network_name: &str) -> Result<()> {
        info!("Stopping network: {}", network_name);
        
        let network = LibvirtNetwork::lookup_by_name(&self.connection, network_name)
            .map_err(|e| {
                error!("Failed to find network {}: {}", network_name, e);
                KvmError::NetworkOperationFailed(format!("Network not found: {}", e))
            })?;
        
        network.destroy()
            .map_err(|e| {
                error!("Failed to stop network {}: {}", network_name, e);
                KvmError::NetworkOperationFailed(format!("Failed to stop network: {}", e))
            })?;
        
        info!("Successfully stopped network: {}", network_name);
        Ok(())
    }
    
    fn generate_network_xml(&self, config: &NetworkCreateConfig) -> Result<String> {
        let dhcp_section = if config.dhcp_enabled {
            let start = config.dhcp_range_start.as_deref().unwrap_or("192.168.1.2");
            let end = config.dhcp_range_end.as_deref().unwrap_or("192.168.1.254");
            format!("      <dhcp>\n        <range start='{}' end='{}'/>\n      </dhcp>", start, end)
        } else {
            String::new()
        };
        
        let bridge_section = if let Some(bridge) = &config.bridge_name {
            format!("  <bridge name='{}' stp='on' delay='0'/>\n", bridge)
        } else {
            "  <bridge name='virbr0' stp='on' delay='0'/>\n".to_string()
        };
        
        let forward_section = match config.forward_mode.as_str() {
            "nat" => "  <forward mode='nat'>\n    <nat>\n      <port start='1024' end='65535'/>\n    </nat>\n  </forward>\n",
            "route" => "  <forward mode='route'/>\n",
            "bridge" => "  <forward mode='bridge'/>\n",
            "none" => "",
            _ => "  <forward mode='nat'/>\n",
        };
        
        let ip_section = if let Some(ip_range) = &config.ip_range {
            format!("    <ip address='{}' netmask='255.255.255.0'>\n{}\n    </ip>", 
                   ip_range.split('/').next().unwrap_or("192.168.1.1"),
                   dhcp_section)
        } else {
            format!("    <ip address='192.168.1.1' netmask='255.255.255.0'>\n{}\n    </ip>", dhcp_section)
        };
        
        let xml = format!(
            r#"<network>\n  <name>{}</name>\n{}{}{}\n</network>"#,
            config.name,
            forward_section,
            bridge_section,
            ip_section
        );
        
        Ok(xml)
    }
}

#[derive(Debug, Clone)]
pub struct NetworkCreateConfig {
    pub name: String,
    pub forward_mode: String,
    pub bridge_name: Option<String>,
    pub ip_range: Option<String>,
    pub dhcp_enabled: bool,
    pub dhcp_range_start: Option<String>,
    pub dhcp_range_end: Option<String>,
    pub auto_start: bool,
}
