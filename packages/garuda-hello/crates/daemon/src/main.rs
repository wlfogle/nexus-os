use garuda_hello_common::{
    AuthConfig, BiometricDevice, User, IpcMessage, AuthRequest, AuthResponse,
    Result, SOCKET_PATH, TemplateStorage, GarudaHelloError
};
use libc::{kill, pid_t};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{info, error, warn, debug};

type DeviceRegistry = Arc<RwLock<HashMap<String, BiometricDevice>>>;

struct DaemonState {
    devices: DeviceRegistry,
    config: AuthConfig,
    template_storage: TemplateStorage,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting Garuda Hello daemon v{}", env!("CARGO_PKG_VERSION"));
    
    // Check if already running
    if is_daemon_running()? {
        error!("Daemon is already running");
        std::process::exit(1);
    }
    
    // Load configuration
    let config = AuthConfig::load()?;
    info!("Loaded configuration: timeout={}s, max_attempts={}", 
          config.timeout, config.max_attempts);
    
    // Initialize template storage
    let template_storage = TemplateStorage::new()?;
    info!("Initialized encrypted template storage");
    
    // Initialize biometric devices
    let discovered_devices = BiometricDevice::discover().await?;
    info!("Discovered {} biometric devices", discovered_devices.len());
    
    let mut device_map = HashMap::new();
    for device in discovered_devices {
        info!("Device: {} ({})", device.name, device.id);
        device_map.insert(device.id.clone(), device);
    }
    let devices = Arc::new(RwLock::new(device_map));
    
    let state = Arc::new(DaemonState {
        devices,
        config,
        template_storage,
    });
    
    // Set up Unix socket for IPC
    std::fs::create_dir_all("/run/garuda-hello")?;
    
    // Remove existing socket if present
    if std::fs::metadata(SOCKET_PATH).is_ok() {
        std::fs::remove_file(SOCKET_PATH)?;
    }
    
    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("Daemon listening on {}", SOCKET_PATH);
    
    // Set socket permissions
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(SOCKET_PATH)?.permissions();
    perms.set_mode(0o666); // Allow all users to connect
    std::fs::set_permissions(SOCKET_PATH, perms)?;
    
    // Write PID file
    write_pid_file()?;
    
    // Set up signal handlers
    setup_signal_handlers().await?;
    
    info!("Garuda Hello daemon started successfully");
    
    // Main event loop
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, state).await {
                        error!("Client handler error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept client connection: {}", e);
            }
        }
    }
}

async fn handle_client(mut stream: UnixStream, state: Arc<DaemonState>) -> Result<()> {
    debug!("New client connected");
    
    loop {
        // Read message length
        let mut len_bytes = [0u8; 4];
        if stream.read_exact(&mut len_bytes).await.is_err() {
            debug!("Client disconnected");
            break;
        }
        
        let msg_len = u32::from_le_bytes(len_bytes) as usize;
        if msg_len > 1024 * 1024 { // 1MB limit
            warn!("Message too large: {} bytes", msg_len);
            break;
        }
        
        // Read message content
        let mut msg_bytes = vec![0u8; msg_len];
        stream.read_exact(&mut msg_bytes).await?;
        
        // Deserialize message
        let message: IpcMessage = match serde_json::from_slice(&msg_bytes) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to deserialize message: {}", e);
                continue;
            }
        };
        
        debug!("Received message: {:?}", message);
        
        // Process message
        let response = match message {
            IpcMessage::AuthRequest(req) => {
                handle_auth_request(req, &state).await
            }
            IpcMessage::EnrollRequest { username, device_id } => {
                handle_enroll_request(username, device_id, &state).await
            }
            IpcMessage::ListDevices => {
                let devices = state.devices.read().await;
                let device_list: Vec<_> = devices.values().cloned().collect();
                IpcMessage::DeviceList(device_list)
            }
            IpcMessage::Shutdown => {
                info!("Received shutdown request");
                std::process::exit(0);
            }
            _ => {
                warn!("Unexpected message type");
                continue;
            }
        };
        
        // Send response
        let response_bytes = serde_json::to_vec(&response)?;
        let len_bytes = (response_bytes.len() as u32).to_le_bytes();
        
        stream.write_all(&len_bytes).await?;
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;
    }
    
    Ok(())
}

async fn handle_auth_request(req: AuthRequest, state: &DaemonState) -> IpcMessage {
    info!("Authentication request for user: {}", req.username);
    
    // Load user data
    let user = match User::load(&req.username) {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("User {} not enrolled", req.username);
            return IpcMessage::AuthResponse(AuthResponse {
                success: false,
                message: "User not enrolled for biometric authentication".to_string(),
                device_used: None,
            });
        }
        Err(e) => {
            error!("Failed to load user data: {}", e);
            return IpcMessage::AuthResponse(AuthResponse {
                success: false,
                message: "Internal error".to_string(),
                device_used: None,
            });
        }
    };
    
    // Get available devices
    let devices = state.devices.read().await;
    
    // Determine which devices to try
    let devices_to_try: Vec<_> = if let Some(device_id) = &req.device_id {
        // Specific device requested
        if let Some(device) = devices.get(device_id) {
            vec![device]
        } else {
            warn!("Requested device {} not found", device_id);
            return IpcMessage::AuthResponse(AuthResponse {
                success: false,
                message: "Requested device not available".to_string(),
                device_used: None,
            });
        }
    } else {
        // Try all enrolled devices in preference order
        let mut available_devices = Vec::new();
        for pref_device in &state.config.device_preferences {
            if let Some(device) = devices.get(pref_device) {
                if user.enrolled_devices.contains(pref_device) {
                    available_devices.push(device);
                }
            }
        }
        // Add any remaining enrolled devices
        for device_id in &user.enrolled_devices {
            if let Some(device) = devices.get(device_id) {
                if !available_devices.iter().any(|d| d.id == device.id) {
                    available_devices.push(device);
                }
            }
        }
        available_devices
    };
    
    if devices_to_try.is_empty() {
        warn!("No suitable devices available for user {}", req.username);
        return IpcMessage::AuthResponse(AuthResponse {
            success: false,
            message: "No suitable biometric devices available".to_string(),
            device_used: None,
        });
    }
    
    // Try authentication with each device
    for device in devices_to_try {
        info!("Trying authentication with device: {}", device.name);
        
        let device_handle = match device.open() {
            Ok(handle) => handle,
            Err(e) => {
                error!("Failed to open device {}: {}", device.name, e);
                continue;
            }
        };
        
        // Get templates for this device
        let templates = user.get_templates_for_device(&device.id);
        if templates.is_empty() {
            warn!("No templates found for device {} and user {}", device.id, req.username);
            continue;
        }
        
        // Try verification against each template
        for template in templates {
            match device_handle.verify(template) {
                Ok(true) => {
                    info!("Authentication successful for user {} with device {}", 
                          req.username, device.name);
                    return IpcMessage::AuthResponse(AuthResponse {
                        success: true,
                        message: "Authentication successful".to_string(),
                        device_used: Some(device.id.clone()),
                    });
                }
                Ok(false) => {
                    debug!("Verification failed for template");
                }
                Err(e) => {
                    error!("Verification error: {}", e);
                }
            }
        }
    }
    
    warn!("Authentication failed for user {}", req.username);
    IpcMessage::AuthResponse(AuthResponse {
        success: false,
        message: "Authentication failed".to_string(),
        device_used: None,
    })
}

async fn handle_enroll_request(username: String, device_id: String, state: &DaemonState) -> IpcMessage {
    info!("Enrollment request for user {} with device {}", username, device_id);
    
    let devices = state.devices.read().await;
    let device = match devices.get(&device_id) {
        Some(device) => device,
        None => {
            warn!("Device {} not found for enrollment", device_id);
            return IpcMessage::EnrollResponse {
                success: false,
                message: "Device not found".to_string(),
            };
        }
    };
    
    let device_handle = match device.open() {
        Ok(handle) => handle,
        Err(e) => {
            error!("Failed to open device {}: {}", device.name, e);
            return IpcMessage::EnrollResponse {
                success: false,
                message: format!("Failed to open device: {}", e),
            };
        }
    };
    
    // Perform enrollment
    let template = match device_handle.enroll(&username) {
        Ok(template) => template,
        Err(e) => {
            error!("Enrollment failed: {}", e);
            return IpcMessage::EnrollResponse {
                success: false,
                message: format!("Enrollment failed: {}", e),
            };
        }
    };
    
    // Load or create user
    let mut user = User::load(&username).unwrap_or_else(|_| None)
        .unwrap_or_else(|| {
            let uid = get_uid_for_username(&username).unwrap_or(1000);
            User {
                username: username.clone(),
                uid,
                enrolled_devices: Vec::new(),
                templates: Vec::new(),
            }
        });
    
    // Add template
    user.add_template(template);
    
    // Save user data
    if let Err(e) = user.save() {
        error!("Failed to save user data: {}", e);
        return IpcMessage::EnrollResponse {
            success: false,
            message: format!("Failed to save enrollment: {}", e),
        };
    }
    
    info!("Enrollment successful for user {} with device {}", username, device.name);
    IpcMessage::EnrollResponse {
        success: true,
        message: "Enrollment successful".to_string(),
    }
}

fn is_daemon_running() -> Result<bool> {
    let pid_file = "/run/garuda-hello/daemon.pid";
    if let Ok(pid_str) = std::fs::read_to_string(pid_file) {
        if let Ok(pid) = pid_str.trim().parse::<pid_t>() {
            // Check if process is still running by sending signal 0
            let result = unsafe { kill(pid, 0) };
            if result == 0 {
                return Ok(true); // Process exists
            } else {
                // Process doesn't exist, remove stale PID file
                let _ = std::fs::remove_file(pid_file);
            }
        }
    }
    Ok(false)
}

fn write_pid_file() -> Result<()> {
    let pid_file = "/run/garuda-hello/daemon.pid";
    let pid = std::process::id();
    std::fs::write(pid_file, pid.to_string())?;
    Ok(())
}

async fn setup_signal_handlers() -> Result<()> {
    tokio::spawn(async {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to register SIGTERM handler");
        let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .expect("Failed to register SIGINT handler");
        
        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down gracefully");
            }
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down gracefully");
            }
        }
        
        cleanup_and_exit();
    });
    
    Ok(())
}

fn cleanup_and_exit() {
    // Remove socket and PID file
    let _ = std::fs::remove_file(SOCKET_PATH);
    let _ = std::fs::remove_file("/run/garuda-hello/daemon.pid");
    
    info!("Garuda Hello daemon shutdown complete");
    std::process::exit(0);
}

fn get_uid_for_username(username: &str) -> Result<u32> {
    use std::ffi::CString;
    use libc::{getpwnam, passwd};
    
    let c_username = CString::new(username)
        .map_err(|_| GarudaHelloError::Auth("Invalid username".to_string()))?;
    
    unsafe {
        let passwd_ptr: *mut passwd = getpwnam(c_username.as_ptr());
        if passwd_ptr.is_null() {
            return Err(GarudaHelloError::Auth(format!("User {} not found", username)));
        }
        
        let passwd = &*passwd_ptr;
        Ok(passwd.pw_uid)
    }
}
