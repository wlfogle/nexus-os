//! PAM module for Garuda Hello biometric authentication
//! 
//! This module provides a complete PAM (Pluggable Authentication Modules) implementation
//! that integrates with the Garuda Hello biometric authentication daemon.

use garuda_hello_common::{IpcMessage, AuthRequest, SOCKET_PATH};
use libc::{c_char, c_int, c_void};
use serde_json;
use std::ffi::{CStr, CString};
use std::ptr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{debug, error, info, warn};

// PAM constants (from security/pam_modules.h)
const PAM_SUCCESS: c_int = 0;
const PAM_OPEN_ERR: c_int = 1;
const PAM_SYMBOL_ERR: c_int = 2;
const PAM_SERVICE_ERR: c_int = 3;
const PAM_SYSTEM_ERR: c_int = 4;
const PAM_BUF_ERR: c_int = 5;
const PAM_PERM_DENIED: c_int = 6;
const PAM_AUTH_ERR: c_int = 7;
const PAM_CRED_INSUFFICIENT: c_int = 8;
const PAM_AUTHINFO_UNAVAIL: c_int = 9;
const PAM_USER_UNKNOWN: c_int = 10;
const PAM_MAXTRIES: c_int = 11;
const PAM_NEW_AUTHTOK_REQD: c_int = 12;
const PAM_ACCT_EXPIRED: c_int = 13;
const PAM_SESSION_ERR: c_int = 14;
const PAM_CRED_UNAVAIL: c_int = 15;
const PAM_CRED_EXPIRED: c_int = 16;
const PAM_CRED_ERR: c_int = 17;
const PAM_NO_MODULE_DATA: c_int = 18;
const PAM_CONV_ERR: c_int = 19;
const PAM_AUTHTOK_ERR: c_int = 20;
const PAM_AUTHTOK_RECOVER_ERR: c_int = 21;
const PAM_AUTHTOK_LOCK_BUSY: c_int = 22;
const PAM_AUTHTOK_DISABLE_AGING: c_int = 23;
const PAM_TRY_AGAIN: c_int = 24;
const PAM_IGNORE: c_int = 25;
const PAM_ABORT: c_int = 26;
const PAM_AUTHTOK_EXPIRED: c_int = 27;
const PAM_MODULE_UNKNOWN: c_int = 28;
const PAM_BAD_ITEM: c_int = 29;
const PAM_CONV_AGAIN: c_int = 30;
const PAM_INCOMPLETE: c_int = 31;

// PAM flags
const PAM_SILENT: c_int = 0x8000;
const PAM_DISALLOW_NULL_AUTHTOK: c_int = 0x0001;
const PAM_ESTABLISH_CRED: c_int = 0x0002;
const PAM_DELETE_CRED: c_int = 0x0004;
const PAM_REINITIALIZE_CRED: c_int = 0x0008;
const PAM_REFRESH_CRED: c_int = 0x0010;
const PAM_CHANGE_EXPIRED_AUTHTOK: c_int = 0x0020;

// PAM items
const PAM_SERVICE: c_int = 1;
const PAM_USER: c_int = 2;
const PAM_TTY: c_int = 3;
const PAM_RHOST: c_int = 4;
const PAM_CONV: c_int = 5;
const PAM_AUTHTOK: c_int = 6;
const PAM_OLDAUTHTOK: c_int = 7;
const PAM_RUSER: c_int = 8;
const PAM_USER_PROMPT: c_int = 9;
const PAM_FAIL_DELAY: c_int = 10;
const PAM_XDISPLAY: c_int = 11;
const PAM_XAUTHDATA: c_int = 12;
const PAM_AUTHTOK_TYPE: c_int = 13;

// PAM handle structure (opaque)
#[repr(C)]
struct PamHandle {
    _private: [u8; 0],
}

// PAM function type definitions
type PamGetItemFunc = unsafe extern "C" fn(
    pamh: *const PamHandle,
    item_type: c_int,
    item: *mut *const c_void,
) -> c_int;

type PamSetItemFunc = unsafe extern "C" fn(
    pamh: *mut PamHandle,
    item_type: c_int,
    item: *const c_void,
) -> c_int;

// External PAM functions
extern "C" {
    fn pam_get_item(pamh: *const PamHandle, item_type: c_int, item: *mut *const c_void) -> c_int;
    fn pam_set_item(pamh: *mut PamHandle, item_type: c_int, item: *const c_void) -> c_int;
}

/// Initialize logging for the PAM module
fn init_logging() {
    // Only initialize once
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .with_file(true)
            .with_line_number(true)
            .try_init();
    });
}

/// Get username from PAM handle
fn get_username_from_pam(pamh: *const PamHandle) -> Result<String, c_int> {
    let mut user_ptr: *const c_void = ptr::null();
    
    let result = unsafe {
        pam_get_item(pamh, PAM_USER, &mut user_ptr)
    };
    
    if result != PAM_SUCCESS {
        error!("Failed to get username from PAM: {}", result);
        return Err(PAM_USER_UNKNOWN);
    }
    
    if user_ptr.is_null() {
        error!("Username is null");
        return Err(PAM_USER_UNKNOWN);
    }
    
    let c_str = unsafe { CStr::from_ptr(user_ptr as *const c_char) };
    match c_str.to_str() {
        Ok(username) => Ok(username.to_string()),
        Err(_) => {
            error!("Invalid UTF-8 in username");
            Err(PAM_USER_UNKNOWN)
        }
    }
}

/// Connect to daemon and send authentication request
fn authenticate_with_daemon_sync(username: &str) -> Result<bool, c_int> {
    // Create a new tokio runtime for this authentication attempt
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to create tokio runtime: {}", e);
            return Err(PAM_SYSTEM_ERR);
        }
    };
    
    rt.block_on(async {
        authenticate_with_daemon_async(username).await
    })
}

/// Async function to handle daemon communication
async fn authenticate_with_daemon_async(username: &str) -> Result<bool, c_int> {
    debug!("Connecting to daemon for user: {}", username);
    
    // Connect to daemon
    let mut stream = match UnixStream::connect(SOCKET_PATH).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to connect to daemon: {}", e);
            return Err(PAM_AUTHINFO_UNAVAIL);
        }
    };
    
    // Create authentication request
    let request = AuthRequest {
        username: username.to_string(),
        device_id: None, // Let daemon choose best device
        timeout: Some(30),
    };
    
    let message = IpcMessage::AuthRequest(request);
    
    // Serialize and send message
    let message_bytes = match serde_json::to_vec(&message) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to serialize request: {}", e);
            return Err(PAM_SYSTEM_ERR);
        }
    };
    
    let len_bytes = (message_bytes.len() as u32).to_le_bytes();
    
    if let Err(e) = stream.write_all(&len_bytes).await {
        error!("Failed to send message length: {}", e);
        return Err(PAM_SYSTEM_ERR);
    }
    
    if let Err(e) = stream.write_all(&message_bytes).await {
        error!("Failed to send message: {}", e);
        return Err(PAM_SYSTEM_ERR);
    }
    
    if let Err(e) = stream.flush().await {
        error!("Failed to flush stream: {}", e);
        return Err(PAM_SYSTEM_ERR);
    }
    
    // Read response
    let mut len_bytes = [0u8; 4];
    if let Err(e) = stream.read_exact(&mut len_bytes).await {
        error!("Failed to read response length: {}", e);
        return Err(PAM_SYSTEM_ERR);
    }
    
    let response_len = u32::from_le_bytes(len_bytes) as usize;
    
    if response_len > 1024 * 1024 { // 1MB limit
        error!("Response too large: {} bytes", response_len);
        return Err(PAM_SYSTEM_ERR);
    }
    
    let mut response_bytes = vec![0u8; response_len];
    if let Err(e) = stream.read_exact(&mut response_bytes).await {
        error!("Failed to read response: {}", e);
        return Err(PAM_SYSTEM_ERR);
    }
    
    // Deserialize response
    let response: IpcMessage = match serde_json::from_slice(&response_bytes) {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to deserialize response: {}", e);
            return Err(PAM_SYSTEM_ERR);
        }
    };
    
    // Handle response
    match response {
        IpcMessage::AuthResponse(auth_response) => {
            if auth_response.success {
                info!("Authentication successful for user: {}", username);
                if let Some(device) = auth_response.device_used {
                    info!("Device used: {}", device);
                }
                Ok(true)
            } else {
                warn!("Authentication failed for user {}: {}", username, auth_response.message);
                Ok(false)
            }
        }
        _ => {
            error!("Unexpected response from daemon");
            Err(PAM_SYSTEM_ERR)
        }
    }
}

/// PAM authentication function
/// This is the main entry point called by PAM when authentication is required
#[no_mangle]
pub extern "C" fn pam_sm_authenticate(
    pamh: *mut PamHandle,
    flags: c_int,
    argc: c_int,
    argv: *const *const c_char,
) -> c_int {
    init_logging();
    
    debug!("PAM authentication requested with flags: {}", flags);
    
    // Get username
    let username = match get_username_from_pam(pamh) {
        Ok(username) => username,
        Err(code) => return code,
    };
    
    debug!("Authentication request for user: {}", username);
    
    // Perform biometric authentication
    match authenticate_with_daemon_sync(&username) {
        Ok(true) => {
            info!("Biometric authentication successful for user: {}", username);
            PAM_SUCCESS
        }
        Ok(false) => {
            info!("Biometric authentication failed for user: {}", username);
            PAM_AUTH_ERR
        }
        Err(code) => {
            error!("Authentication system error for user: {}", username);
            code
        }
    }
}

/// PAM account management function
/// Called to verify that the user account is valid
#[no_mangle]
pub extern "C" fn pam_sm_acct_mgmt(
    pamh: *mut PamHandle,
    flags: c_int,
    argc: c_int,
    argv: *const *const c_char,
) -> c_int {
    init_logging();
    
    debug!("PAM account management requested with flags: {}", flags);
    
    // For biometric authentication, if the user successfully authenticated,
    // their account is considered valid
    PAM_SUCCESS
}

/// PAM session open function
/// Called when a session is being established
#[no_mangle]
pub extern "C" fn pam_sm_open_session(
    pamh: *mut PamHandle,
    flags: c_int,
    argc: c_int,
    argv: *const *const c_char,
) -> c_int {
    init_logging();
    
    debug!("PAM session open requested with flags: {}", flags);
    
    // No special session handling needed for biometric authentication
    PAM_SUCCESS
}

/// PAM session close function
/// Called when a session is being terminated
#[no_mangle]
pub extern "C" fn pam_sm_close_session(
    pamh: *mut PamHandle,
    flags: c_int,
    argc: c_int,
    argv: *const *const c_char,
) -> c_int {
    init_logging();
    
    debug!("PAM session close requested with flags: {}", flags);
    
    // No special session handling needed for biometric authentication
    PAM_SUCCESS
}

/// PAM credential setting function
/// Called to establish/refresh/delete user credentials
#[no_mangle]
pub extern "C" fn pam_sm_setcred(
    pamh: *mut PamHandle,
    flags: c_int,
    argc: c_int,
    argv: *const *const c_char,
) -> c_int {
    init_logging();
    
    debug!("PAM setcred requested with flags: {}", flags);
    
    // For biometric authentication, we don't manage traditional credentials
    match flags & 0x001f {
        PAM_ESTABLISH_CRED => {
            debug!("Establishing credentials");
            PAM_SUCCESS
        }
        PAM_DELETE_CRED => {
            debug!("Deleting credentials");
            PAM_SUCCESS
        }
        PAM_REINITIALIZE_CRED => {
            debug!("Reinitializing credentials");
            PAM_SUCCESS
        }
        PAM_REFRESH_CRED => {
            debug!("Refreshing credentials");
            PAM_SUCCESS
        }
        _ => {
            warn!("Unknown credential operation: {}", flags);
            PAM_SUCCESS
        }
    }
}

/// PAM password change function
/// Called when the user needs to change their password/authentication token
#[no_mangle]
pub extern "C" fn pam_sm_chauthtok(
    pamh: *mut PamHandle,
    flags: c_int,
    argc: c_int,
    argv: *const *const c_char,
) -> c_int {
    init_logging();
    
    debug!("PAM chauthtok requested with flags: {}", flags);
    
    // Biometric templates are managed through the CLI tool, not through PAM
    // Return success to indicate no password change is needed
    PAM_SUCCESS
}
