//! Shared types and utilities for Garuda Hello.

use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::{Aead, OsRng};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;
use thiserror::Error;

pub const PROJECT_NAME: &str = "garuda-hello";
pub const CONFIG_DIR: &str = "/etc/garuda-hello";
pub const USER_CONFIG_DIR: &str = ".config/garuda-hello";
pub const TEMPLATE_DIR: &str = "/var/lib/garuda-hello/templates";
pub const SOCKET_PATH: &str = "/run/garuda-hello/daemon.sock";

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version: &'static str,
}

impl VersionInfo {
    pub const fn new() -> Self {
        Self { version: env!("CARGO_PKG_VERSION") }
    }
}

/// Custom Result type for the application
pub type Result<T> = std::result::Result<T, GarudaHelloError>;

/// Main error type for Garuda Hello
#[derive(Error, Debug)]
pub enum GarudaHelloError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Device error: {0}")]
    Device(String),
    #[error("Template error: {0}")]
    Template(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::de::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Nix error: {0}")]
    Nix(#[from] nix::Error),
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub timeout: u64,
    pub max_attempts: u32,
    pub fallback_enabled: bool,
    pub device_preferences: Vec<String>,
    pub security_level: SecurityLevel,
    pub template_storage_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            timeout: 30,
            max_attempts: 3,
            fallback_enabled: true,
            device_preferences: vec![],
            security_level: SecurityLevel::Medium,
            template_storage_path: TEMPLATE_DIR.to_string(),
        }
    }
}

impl AuthConfig {
    pub fn load() -> Result<Self> {
        let config_path = format!("{}/config.toml", CONFIG_DIR);
        if Path::new(&config_path).exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: AuthConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    pub fn save(&self) -> Result<()> {
        fs::create_dir_all(CONFIG_DIR)?;
        let config_path = format!("{}/config.toml", CONFIG_DIR);
        let content = toml::to_string_pretty(self)
            .map_err(|e| GarudaHelloError::Config(format!("Serialization failed: {}", e)))?;
        fs::write(&config_path, content)?;
        Ok(())
    }
}

/// Represents a biometric device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricDevice {
    pub id: String,
    pub name: String,
    pub device_type: BiometricDeviceType,
    pub capabilities: Vec<String>,
    pub is_available: bool,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BiometricDeviceType {
    Fingerprint,
    Face,
    Iris,
    Voice,
}

impl BiometricDevice {
    /// Discover available biometric devices
    pub async fn discover() -> Result<Vec<Self>> {
        let mut devices = Vec::new();
        
        // Discover USB fingerprint devices
        devices.extend(Self::discover_usb_fingerprint_devices()?);
        
        // Discover camera devices for face recognition
        devices.extend(Self::discover_camera_devices()?);
        
        Ok(devices)
    }
    
    fn discover_usb_fingerprint_devices() -> Result<Vec<Self>> {
        let mut devices = Vec::new();
        
        // Read USB devices from /sys/bus/usb/devices
        if let Ok(entries) = fs::read_dir("/sys/bus/usb/devices") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if let (Ok(vendor), Ok(product)) = (
                        fs::read_to_string(path.join("idVendor")),
                        fs::read_to_string(path.join("idProduct"))
                    ) {
                        let vendor_id = u16::from_str_radix(vendor.trim(), 16).unwrap_or(0);
                        let product_id = u16::from_str_radix(product.trim(), 16).unwrap_or(0);
                        
                        // Known fingerprint device vendor/product IDs
                        if Self::is_fingerprint_device(vendor_id, product_id) {
                            let device_name = fs::read_to_string(path.join("product"))
                                .unwrap_or_else(|_| "Unknown Fingerprint Device".to_string());
                            
                            devices.push(BiometricDevice {
                                id: format!("{:04x}:{:04x}", vendor_id, product_id),
                                name: device_name.trim().to_string(),
                                device_type: BiometricDeviceType::Fingerprint,
                                capabilities: vec!["enroll".to_string(), "verify".to_string()],
                                is_available: true,
                                vendor_id: Some(vendor_id),
                                product_id: Some(product_id),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(devices)
    }
    
    fn discover_camera_devices() -> Result<Vec<Self>> {
        let mut devices = Vec::new();
        
        // Check for video devices
        for i in 0..10 {
            let device_path = format!("/dev/video{}", i);
            if Path::new(&device_path).exists() {
                devices.push(BiometricDevice {
                    id: format!("video{}", i),
                    name: format!("Camera {}", i),
                    device_type: BiometricDeviceType::Face,
                    capabilities: vec!["enroll".to_string(), "verify".to_string()],
                    is_available: true,
                    vendor_id: None,
                    product_id: None,
                });
            }
        }
        
        Ok(devices)
    }
    
    fn is_fingerprint_device(vendor_id: u16, product_id: u16) -> bool {
        // Common fingerprint device vendor/product IDs
        match (vendor_id, product_id) {
            // ITE Tech fingerprint devices (common in power buttons)
            (0x048d, 0x8910) => true,
            // ELAN fingerprint devices
            (0x04f3, _) if product_id >= 0x0c00 => true,
            // AuthenTec devices
            (0x08ff, _) => true,
            // Validity/Synaptics devices
            (0x138a, _) => true,
            // Upek devices
            (0x147e, _) => true,
            // LighTuning devices
            (0x1c7a, _) => true,
            // Goodix devices
            (0x27c6, _) => true,
            _ => false,
        }
    }
    
    pub fn open(&self) -> Result<DeviceHandle> {
        DeviceHandle::new(self.clone())
    }
}

/// Handle to an opened biometric device
pub struct DeviceHandle {
    device: BiometricDevice,
}

impl DeviceHandle {
    fn new(device: BiometricDevice) -> Result<Self> {
        Ok(DeviceHandle { device })
    }
    
    pub fn enroll(&self, username: &str) -> Result<BiometricTemplate> {
        // Simulate biometric enrollment
        let template_data = self.capture_biometric_data()?;
        
        Ok(BiometricTemplate {
            username: username.to_string(),
            device_id: self.device.id.clone(),
            device_type: self.device.device_type.clone(),
            template_data,
            created_at: std::time::SystemTime::now(),
        })
    }
    
    pub fn verify(&self, template: &BiometricTemplate) -> Result<bool> {
        // Simulate biometric verification
        let captured_data = self.capture_biometric_data()?;
        
        // In a real implementation, this would compare the captured data
        // against the stored template using appropriate biometric algorithms
        Ok(self.compare_templates(&captured_data, &template.template_data))
    }
    
    fn capture_biometric_data(&self) -> Result<Vec<u8>> {
        // Simulate capturing biometric data
        match self.device.device_type {
            BiometricDeviceType::Fingerprint => {
                // Simulate fingerprint capture
                let mut data = vec![0u8; 256];
                rand::thread_rng().fill_bytes(&mut data);
                Ok(data)
            },
            BiometricDeviceType::Face => {
                // Simulate face capture
                let mut data = vec![0u8; 512];
                rand::thread_rng().fill_bytes(&mut data);
                Ok(data)
            },
            _ => Err(GarudaHelloError::Device("Device type not supported".to_string())),
        }
    }
    
    fn compare_templates(&self, data1: &[u8], data2: &[u8]) -> bool {
        // Simple comparison for simulation
        // In reality, this would use sophisticated biometric matching algorithms
        let hash1 = Sha256::digest(data1);
        let hash2 = Sha256::digest(data2);
        hash1 == hash2
    }
}

/// User information for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub uid: u32,
    pub enrolled_devices: Vec<String>,
    pub templates: Vec<BiometricTemplate>,
}

impl User {
    pub fn load(username: &str) -> Result<Option<Self>> {
        let user_file = format!("{}/{}.toml", TEMPLATE_DIR, username);
        if Path::new(&user_file).exists() {
            let content = fs::read_to_string(&user_file)?;
            let user: User = toml::from_str(&content)?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        fs::create_dir_all(TEMPLATE_DIR)?;
        let user_file = format!("{}/{}.toml", TEMPLATE_DIR, self.username);
        let content = toml::to_string_pretty(self)
            .map_err(|e| GarudaHelloError::Template(format!("Serialization failed: {}", e)))?;
        fs::write(&user_file, content)?;
        Ok(())
    }
    
    pub fn add_template(&mut self, template: BiometricTemplate) {
        let device_id = template.device_id.clone();
        self.templates.push(template);
        if !self.enrolled_devices.contains(&device_id) {
            self.enrolled_devices.push(device_id);
        }
    }
    
    pub fn get_templates_for_device(&self, device_id: &str) -> Vec<&BiometricTemplate> {
        self.templates.iter()
            .filter(|t| t.device_id == device_id)
            .collect()
    }
}

/// Biometric template storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricTemplate {
    pub username: String,
    pub device_id: String,
    pub device_type: BiometricDeviceType,
    pub template_data: Vec<u8>,
    pub created_at: std::time::SystemTime,
}

/// Template storage with encryption
pub struct TemplateStorage {
    cipher: Aes256Gcm,
}

impl TemplateStorage {
    pub fn new() -> Result<Self> {
        let key_data = Self::load_or_generate_key()?;
        let key = Key::<Aes256Gcm>::from_slice(&key_data);
        let cipher = Aes256Gcm::new(key);
        
        Ok(TemplateStorage { cipher })
    }
    
    fn load_or_generate_key() -> Result<[u8; 32]> {
        let key_path = format!("{}/master.key", CONFIG_DIR);
        
        if Path::new(&key_path).exists() {
            let key_data = fs::read(&key_path)?;
            if key_data.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_data);
                Ok(key)
            } else {
                Err(GarudaHelloError::Encryption("Invalid key file".to_string()))
            }
        } else {
            // Generate new key
            let mut key = [0u8; 32];
            OsRng.fill_bytes(&mut key);
            
            fs::create_dir_all(CONFIG_DIR)?;
            fs::write(&key_path, &key)?;
            
            // Set restrictive permissions
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&key_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&key_path, perms)?;
            
            Ok(key)
        }
    }
    
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher.encrypt(nonce, data)
            .map_err(|e| GarudaHelloError::Encryption(format!("Encryption failed: {}", e)))?;
        
        let mut result = Vec::new();
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }
    
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            return Err(GarudaHelloError::Encryption("Invalid encrypted data".to_string()));
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| GarudaHelloError::Encryption(format!("Decryption failed: {}", e)))
    }
}

/// Authentication request
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub device_id: Option<String>,
    pub timeout: Option<u64>,
}

/// Authentication response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub device_used: Option<String>,
}

/// IPC message types
#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessage {
    AuthRequest(AuthRequest),
    AuthResponse(AuthResponse),
    EnrollRequest {
        username: String,
        device_id: String,
    },
    EnrollResponse {
        success: bool,
        message: String,
    },
    ListDevices,
    DeviceList(Vec<BiometricDevice>),
    Shutdown,
}
