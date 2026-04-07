use clap::{Parser, Subcommand};
use garuda_hello_common::{
    AuthConfig, BiometricDevice, IpcMessage, AuthRequest, Result,
    SOCKET_PATH, GarudaHelloError, VersionInfo, User
};
use serde_json;
use std::io::{self, Write};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{info, error, warn};

#[derive(Parser)]
#[command(name = "garuda-hello")]
#[command(about = "Garuda Hello biometric authentication system")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Enroll biometric templates for a user
    Enroll {
        /// Username to enroll
        #[arg(short, long)]
        username: Option<String>,
        
        /// Device ID to use for enrollment
        #[arg(short, long)]
        device: Option<String>,
    },
    
    /// Test biometric authentication for a user
    Verify {
        /// Username to verify
        #[arg(short, long)]
        username: Option<String>,
        
        /// Device ID to use for verification
        #[arg(short, long)]
        device: Option<String>,
    },
    
    /// List available biometric devices
    ListDevices,
    
    /// Show system configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
    
    /// Start the daemon
    Daemon,
    
    /// Stop the daemon
    Stop,
    
    /// Show system status
    Status,
    
    /// List enrolled users
    ListUsers,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    
    /// Set configuration values
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    
    /// Reset configuration to defaults
    Reset,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();
    
    match cli.command {
        Commands::Enroll { username, device } => {
            let username = get_username(username)?;
            enroll_user(&username, device).await
        }
        Commands::Verify { username, device } => {
            let username = get_username(username)?;
            verify_user(&username, device).await
        }
        Commands::ListDevices => list_devices().await,
        Commands::Config { action } => handle_config(action).await,
        Commands::Daemon => start_daemon().await,
        Commands::Stop => stop_daemon().await,
        Commands::Status => show_status().await,
        Commands::ListUsers => list_users().await,
    }
}

async fn enroll_user(username: &str, device_id: Option<String>) -> Result<()> {
    let devices = get_available_devices().await?;
    
    if devices.is_empty() {
        eprintln!("❌ No biometric devices available");
        return Ok(());
    }
    
    let device_id = if let Some(id) = device_id {
        if !devices.iter().any(|d| d.id == id) {
            eprintln!("❌ Device '{}' not found", id);
            return Ok(());
        }
        id
    } else {
        select_device(&devices)?
    };
    
    println!("🔐 Enrolling biometric template for user: {}", username);
    println!("📱 Please interact with the biometric device...");
    
    let message = IpcMessage::EnrollRequest {
        username: username.to_string(),
        device_id: device_id.clone(),
    };
    
    match send_message_to_daemon(message).await? {
        IpcMessage::EnrollResponse { success, message } => {
            if success {
                println!("✅ {}", message);
            } else {
                eprintln!("❌ {}", message);
            }
        }
        _ => {
            eprintln!("❌ Unexpected response from daemon");
        }
    }
    
    Ok(())
}

async fn verify_user(username: &str, device_id: Option<String>) -> Result<()> {
    println!("🔍 Verifying biometric authentication for user: {}", username);
    println!("📱 Please interact with the biometric device...");
    
    let request = AuthRequest {
        username: username.to_string(),
        device_id,
        timeout: Some(30),
    };
    
    let message = IpcMessage::AuthRequest(request);
    
    match send_message_to_daemon(message).await? {
        IpcMessage::AuthResponse(response) => {
            if response.success {
                println!("✅ {}", response.message);
                if let Some(device) = response.device_used {
                    println!("📱 Device used: {}", device);
                }
            } else {
                eprintln!("❌ {}", response.message);
            }
        }
        _ => {
            eprintln!("❌ Unexpected response from daemon");
        }
    }
    
    Ok(())
}

async fn list_devices() -> Result<()> {
    let devices = get_available_devices().await?;
    
    if devices.is_empty() {
        println!("📱 No biometric devices available");
        return Ok(());
    }
    
    println!("📱 Available biometric devices:");
    println!();
    
    for device in devices {
        let status = if device.is_available { "✅ Available" } else { "❌ Unavailable" };
        let device_type = match device.device_type {
            garuda_hello_common::BiometricDeviceType::Fingerprint => "👆 Fingerprint",
            garuda_hello_common::BiometricDeviceType::Face => "👤 Face",
            garuda_hello_common::BiometricDeviceType::Iris => "👁️ Iris",
            garuda_hello_common::BiometricDeviceType::Voice => "🗣️ Voice",
        };
        
        println!("  {} {}", device_type, device.name);
        println!("    ID: {}", device.id);
        println!("    Status: {}", status);
        
        if !device.capabilities.is_empty() {
            println!("    Capabilities: {}", device.capabilities.join(", "));
        }
        
        if let (Some(vid), Some(pid)) = (device.vendor_id, device.product_id) {
            println!("    Vendor:Product ID: {:04x}:{:04x}", vid, pid);
        }
        
        println!();
    }
    
    Ok(())
}

async fn handle_config(action: Option<ConfigAction>) -> Result<()> {
    match action {
        Some(ConfigAction::Show) | None => show_config().await,
        Some(ConfigAction::Set { key, value }) => set_config(&key, &value).await,
        Some(ConfigAction::Reset) => reset_config().await,
    }
}

async fn show_config() -> Result<()> {
    let config = AuthConfig::load()?;
    
    println!("⚙️  Garuda Hello Configuration:");
    println!();
    println!("  Authentication timeout: {} seconds", config.timeout);
    println!("  Maximum attempts: {}", config.max_attempts);
    println!("  Fallback enabled: {}", config.fallback_enabled);
    println!("  Security level: {:?}", config.security_level);
    println!("  Template storage path: {}", config.template_storage_path);
    
    if !config.device_preferences.is_empty() {
        println!("  Device preferences:");
        for (i, device) in config.device_preferences.iter().enumerate() {
            println!("    {}. {}", i + 1, device);
        }
    } else {
        println!("  Device preferences: None set");
    }
    
    Ok(())
}

async fn set_config(key: &str, value: &str) -> Result<()> {
    let mut config = AuthConfig::load()?;
    
    match key {
        "timeout" => {
            config.timeout = value.parse()
                .map_err(|_| GarudaHelloError::Config("Invalid timeout value".to_string()))?;
        }
        "max_attempts" => {
            config.max_attempts = value.parse()
                .map_err(|_| GarudaHelloError::Config("Invalid max_attempts value".to_string()))?;
        }
        "fallback_enabled" => {
            config.fallback_enabled = value.parse()
                .map_err(|_| GarudaHelloError::Config("Invalid fallback_enabled value".to_string()))?;
        }
        "security_level" => {
            config.security_level = match value.to_lowercase().as_str() {
                "low" => garuda_hello_common::SecurityLevel::Low,
                "medium" => garuda_hello_common::SecurityLevel::Medium,
                "high" => garuda_hello_common::SecurityLevel::High,
                _ => return Err(GarudaHelloError::Config("Invalid security level".to_string())),
            };
        }
        "template_storage_path" => {
            config.template_storage_path = value.to_string();
        }
        _ => {
            return Err(GarudaHelloError::Config(format!("Unknown configuration key: {}", key)));
        }
    }
    
    config.save()?;
    println!("✅ Configuration updated: {} = {}", key, value);
    
    Ok(())
}

async fn reset_config() -> Result<()> {
    print!("⚠️  Are you sure you want to reset configuration to defaults? [y/N]: ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    if input.trim().to_lowercase() == "y" {
        let config = AuthConfig::default();
        config.save()?;
        println!("✅ Configuration reset to defaults");
    } else {
        println!("❌ Configuration reset cancelled");
    }
    
    Ok(())
}

async fn start_daemon() -> Result<()> {
    println!("🚀 Starting Garuda Hello daemon...");
    
    // Check if daemon is already running
    if is_daemon_running().await {
        eprintln!("❌ Daemon is already running");
        return Ok(());
    }
    
    // Start daemon process
    let _status = tokio::process::Command::new("garuda_hello_daemon")
        .spawn()
        .map_err(|e| GarudaHelloError::Device(format!("Failed to start daemon: {}", e)))?;
    
    // Give it a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    if is_daemon_running().await {
        println!("✅ Daemon started successfully");
    } else {
        eprintln!("❌ Failed to start daemon");
    }
    
    Ok(())
}

async fn stop_daemon() -> Result<()> {
    println!("🛑 Stopping Garuda Hello daemon...");
    
    if !is_daemon_running().await {
        eprintln!("❌ Daemon is not running");
        return Ok(());
    }
    
    match send_message_to_daemon(IpcMessage::Shutdown).await {
        Ok(_) => {
            println!("✅ Daemon stopped successfully");
        }
        Err(_) => {
            eprintln!("❌ Failed to stop daemon gracefully");
        }
    }
    
    Ok(())
}

async fn show_status() -> Result<()> {
    let version_info = VersionInfo::new();
    println!("📊 Garuda Hello Status");
    println!("  Version: {}", version_info.version);
    
    let daemon_status = if is_daemon_running().await {
        "✅ Running"
    } else {
        "❌ Not running"
    };
    
    println!("  Daemon: {}", daemon_status);
    
    if is_daemon_running().await {
        let devices = match get_available_devices().await {
            Ok(devices) => devices,
            Err(_) => Vec::new(),
        };
        
        println!("  Devices: {} available", devices.len());
        
        // Count enrolled users
        let user_count = count_enrolled_users()?;
        println!("  Enrolled users: {}", user_count);
    }
    
    Ok(())
}

async fn list_users() -> Result<()> {
    let template_dir = &garuda_hello_common::TEMPLATE_DIR;
    
    if !std::path::Path::new(template_dir).exists() {
        println!("👥 No users enrolled yet");
        return Ok(());
    }
    
    let entries = std::fs::read_dir(template_dir)?;
    let mut users = Vec::new();
    
    for entry in entries {
        if let Ok(entry) = entry {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".toml") {
                    let username = filename.trim_end_matches(".toml");
                    if let Ok(Some(user)) = User::load(username) {
                        users.push(user);
                    }
                }
            }
        }
    }
    
    if users.is_empty() {
        println!("👥 No users enrolled yet");
        return Ok(());
    }
    
    println!("👥 Enrolled users:");
    println!();
    
    for user in users {
        println!("  👤 {}", user.username);
        println!("    UID: {}", user.uid);
        println!("    Enrolled devices: {} device(s)", user.enrolled_devices.len());
        println!("    Templates: {} template(s)", user.templates.len());
        
        if !user.enrolled_devices.is_empty() {
            for device_id in &user.enrolled_devices {
                let template_count = user.get_templates_for_device(device_id).len();
                println!("      📱 {} ({} template(s))", device_id, template_count);
            }
        }
        
        println!();
    }
    
    Ok(())
}

// Helper functions

fn get_username(username: Option<String>) -> Result<String> {
    if let Some(username) = username {
        Ok(username)
    } else {
        // Get current user
        std::env::var("USER").or_else(|_| std::env::var("USERNAME"))
            .map_err(|_| GarudaHelloError::Auth("Could not determine username".to_string()))
    }
}

fn select_device(devices: &[BiometricDevice]) -> Result<String> {
    if devices.len() == 1 {
        return Ok(devices[0].id.clone());
    }
    
    println!("📱 Available devices:");
    for (i, device) in devices.iter().enumerate() {
        let device_type = match device.device_type {
            garuda_hello_common::BiometricDeviceType::Fingerprint => "👆",
            garuda_hello_common::BiometricDeviceType::Face => "👤",
            garuda_hello_common::BiometricDeviceType::Iris => "👁️",
            garuda_hello_common::BiometricDeviceType::Voice => "🗣️",
        };
        println!("  {}. {} {} ({})", i + 1, device_type, device.name, device.id);
    }
    
    print!("Select device [1-{}]: ", devices.len());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let selection: usize = input.trim().parse()
        .map_err(|_| GarudaHelloError::Device("Invalid selection".to_string()))?;
    
    if selection < 1 || selection > devices.len() {
        return Err(GarudaHelloError::Device("Invalid selection".to_string()));
    }
    
    Ok(devices[selection - 1].id.clone())
}

async fn get_available_devices() -> Result<Vec<BiometricDevice>> {
    match send_message_to_daemon(IpcMessage::ListDevices).await? {
        IpcMessage::DeviceList(devices) => Ok(devices),
        _ => Err(GarudaHelloError::Device("Unexpected response from daemon".to_string())),
    }
}

async fn send_message_to_daemon(message: IpcMessage) -> Result<IpcMessage> {
    let mut stream = UnixStream::connect(SOCKET_PATH).await
        .map_err(|_| GarudaHelloError::Device("Failed to connect to daemon. Is it running?".to_string()))?;
    
    // Serialize and send message
    let message_bytes = serde_json::to_vec(&message)?;
    let len_bytes = (message_bytes.len() as u32).to_le_bytes();
    
    stream.write_all(&len_bytes).await?;
    stream.write_all(&message_bytes).await?;
    stream.flush().await?;
    
    // Read response
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).await?;
    let response_len = u32::from_le_bytes(len_bytes) as usize;
    
    let mut response_bytes = vec![0u8; response_len];
    stream.read_exact(&mut response_bytes).await?;
    
    let response: IpcMessage = serde_json::from_slice(&response_bytes)
        .map_err(|e| GarudaHelloError::Device(format!("Failed to deserialize response: {}", e)))?;
    
    Ok(response)
}

async fn is_daemon_running() -> bool {
    UnixStream::connect(SOCKET_PATH).await.is_ok()
}

fn count_enrolled_users() -> Result<usize> {
    let template_dir = &garuda_hello_common::TEMPLATE_DIR;
    
    if !std::path::Path::new(template_dir).exists() {
        return Ok(0);
    }
    
    let entries = std::fs::read_dir(template_dir)?;
    let count = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_name().to_str()
                .map(|name| name.ends_with(".toml"))
                .unwrap_or(false)
        })
        .count();
    
    Ok(count)
}
