use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};

use adb_client::server::ADBServer;
use adb_client::server_device::ADBServerDevice;
use adb_client::ADBDeviceExt;
use serde::Serialize;

const ADB_SERVER_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 5037);

/// Connects to the local adb-server, starting it first if it isn't already
/// reachable (mirrors what the `adb` CLI does implicitly on first use).
pub fn server() -> ADBServer {
    let mut srv = ADBServer::new(ADB_SERVER_ADDR);
    if srv.version().is_err() {
        let _ = ADBServer::start(&HashMap::new(), &None);
        srv = ADBServer::new(ADB_SERVER_ADDR);
    }
    srv
}

/// Opens a handle to a device: the one matching `serial` if given, otherwise
/// the sole attached device (erroring clearly if there is not exactly one).
pub fn open_device(serial: Option<String>) -> Result<ADBServerDevice, String> {
    let mut srv = server();
    match serial {
        Some(s) => srv
            .get_device_by_name(&s)
            .map_err(|e| format!("Failed to open device '{}': {}", s, e)),
        None => srv
            .get_device()
            .map_err(|e| format!("Failed to open device (select a device if more than one is attached): {}", e)),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub identifier: String,
    pub state: String,
    pub model: Option<String>,
    pub transport_id: Option<u32>,
}

#[tauri::command]
pub async fn list_devices() -> Result<Vec<DeviceInfo>, String> {
    tokio::task::spawn_blocking(|| {
        let mut srv = server();
        let devices = srv
            .devices_long()
            .map_err(|e| format!("Failed to list devices: {}", e))?;

        Ok(devices
            .into_iter()
            .map(|d| DeviceInfo {
                identifier: d.identifier,
                state: d.state.to_string(),
                model: if d.model.trim().is_empty() {
                    None
                } else {
                    Some(d.model)
                },
                transport_id: Some(d.transport_id),
            })
            .collect())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// Extracts the bare file/directory name from a directory listing entry,
/// matching the plain-name output previously produced by `adb shell ls -1`.
pub fn list_item_name(item: &adb_client::ADBListItemType) -> String {
    use adb_client::ADBListItemType::*;
    match item {
        Fifo(i) | CharacterDevice(i) | Directory(i) | BlockDevice(i) | File(i) | Symlink(i)
        | Socket(i) | Other(i) => i.name.clone(),
    }
}

/// Pushes the APK at `local_path` to `/data/local/tmp/<filename>`, installs
/// it with `pm install -r -g` (auto-granting runtime permissions, matching
/// the previous `adb install -r -g` behavior), then cleans up the staged
/// file on the device regardless of install outcome.
pub fn install_apk_on_device(
    device: &mut ADBServerDevice,
    local_path: &str,
    filename: &str,
) -> Result<(), String> {
    let remote_path = format!("/data/local/tmp/{}", filename);

    let mut file =
        std::fs::File::open(local_path).map_err(|e| format!("Failed to open APK: {}", e))?;
    device
        .push(&mut file, &remote_path)
        .map_err(|e| format!("Push failed: {}", e))?;

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let install_cmd = format!("pm install -r -g {}", remote_path);
    let install_result = device.shell_command(&install_cmd, Some(&mut stdout), Some(&mut stderr));

    let cleanup_cmd = format!("rm {}", remote_path);
    let _ = device.shell_command(&cleanup_cmd, None, None);

    install_result.map_err(|e| format!("Install failed: {}", e))?;

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&stdout),
        String::from_utf8_lossy(&stderr)
    );
    if combined.to_lowercase().contains("failure") {
        return Err(format!("Install failed: {}", combined.trim()));
    }

    Ok(())
}
