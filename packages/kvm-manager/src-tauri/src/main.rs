// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod vm_manager;
mod storage;
mod network;
mod monitoring;
mod system_monitor;
mod types;
mod errors;
mod xml_parser;

use tracing::{info, error, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

use vm_manager::VmManager;
use types::*;

type AppState = Arc<RwLock<VmManager>>;

#[tauri::command]
async fn get_vms(state: tauri::State<'_, AppState>) -> Result<Vec<VirtualMachine>, String> {
    let manager = state.read().await;
    manager.list_vms().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_vm(
    state: tauri::State<'_, AppState>,
    config: VmConfig,
) -> Result<String, String> {
    let mut manager = state.write().await;
    manager.create_vm(config).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_vm(
    state: tauri::State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    let manager = state.read().await;
    manager.start_vm(&vm_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_vm(
    state: tauri::State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    let manager = state.read().await;
    manager.stop_vm(&vm_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_vm(
    state: tauri::State<'_, AppState>,
    vm_id: String,
) -> Result<(), String> {
    let mut manager = state.write().await;
    manager.delete_vm(&vm_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_vm_stats(
    state: tauri::State<'_, AppState>,
    vm_id: String,
) -> Result<VmStats, String> {
    let manager = state.read().await;
    manager.get_vm_stats(&vm_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_host_info(state: tauri::State<'_, AppState>) -> Result<HostInfo, String> {
    let manager = state.read().await;
    manager.get_host_info().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_snapshot(
    state: tauri::State<'_, AppState>,
    vm_id: String,
    snapshot_name: String,
) -> Result<(), String> {
    let manager = state.read().await;
    manager.create_snapshot(&vm_id, &snapshot_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn restore_snapshot(
    state: tauri::State<'_, AppState>,
    vm_id: String,
    snapshot_name: String,
) -> Result<(), String> {
    let manager = state.read().await;
    manager.restore_snapshot(&vm_id, &snapshot_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_storage_pools(state: tauri::State<'_, AppState>) -> Result<Vec<StoragePool>, String> {
    let manager = state.read().await;
    manager.get_storage_pools().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_networks(state: tauri::State<'_, AppState>) -> Result<Vec<Network>, String> {
    let manager = state.read().await;
    manager.get_networks().await.map_err(|e| e.to_string())
}

// Enhanced Proxmox-specific commands
#[tauri::command]
async fn create_proxmox_vm(
    state: tauri::State<'_, AppState>,
    name: String,
    proxmox_path: String,
    memory_gb: u32,
    vcpus: u32,
) -> Result<String, String> {
    let mut manager = state.write().await;
    manager.create_proxmox_vm(name, proxmox_path, memory_gb, vcpus).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_vm_from_xml(
    state: tauri::State<'_, AppState>,
    xml_path: String,
) -> Result<String, String> {
    let mut manager = state.write().await;
    manager.import_vm_from_xml(&xml_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_vm_from_qcow2(
    state: tauri::State<'_, AppState>,
    qcow2_path: String,
    vm_name: String,
    memory_mb: u64,
    vcpus: u32,
    passthrough_device: Option<String>,
) -> Result<String, String> {
    let mut manager = state.write().await;
    manager.create_vm_from_qcow2(&qcow2_path, &vm_name, memory_mb, vcpus, passthrough_device.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn refresh_vms(state: tauri::State<'_, AppState>) -> Result<Vec<VirtualMachine>, String> {
    let mut manager = state.write().await;
    manager.refresh_vm_list().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_vm_snapshots(
    state: tauri::State<'_, AppState>,
    vm_id: String,
) -> Result<Vec<Snapshot>, String> {
    let manager = state.read().await;
    manager.list_snapshots(&vm_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_vm_snapshot(
    state: tauri::State<'_, AppState>,
    vm_id: String,
    snapshot_name: String,
) -> Result<(), String> {
    let manager = state.read().await;
    manager.delete_snapshot(&vm_id, &snapshot_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn browse_qcow2_files() -> Result<Vec<String>, String> {
    use std::process::Command;
    
    // Find QCOW2 files in common directories
    let common_dirs = [
        "/var/lib/libvirt/images",
        "/home",
        "/mnt",
        "/media"
    ];
    
    let mut qcow2_files = Vec::new();
    
    for dir in &common_dirs {
        if let Ok(output) = Command::new("find")
            .args([dir, "-name", "*.qcow2", "-type", "f", "-readable", "2>/dev/null"])
            .output() {
            
            if output.status.success() {
                let files: Vec<String> = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| line.to_string())
                    .collect();
                qcow2_files.extend(files);
            }
        }
    }
    
    Ok(qcow2_files)
}

#[tauri::command]
async fn browse_xml_files() -> Result<Vec<String>, String> {
    use std::process::Command;
    
    let common_dirs = [
        "/etc/libvirt/qemu",
        "/var/lib/libvirt/qemu",
        "/home"
    ];
    
    let mut xml_files = Vec::new();
    
    for dir in &common_dirs {
        if let Ok(output) = Command::new("find")
            .args([dir, "-name", "*.xml", "-type", "f", "-readable", "2>/dev/null"])
            .output() {
            
            if output.status.success() {
                let files: Vec<String> = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| line.to_string())
                    .collect();
                xml_files.extend(files);
            }
        }
    }
    
    Ok(xml_files)
}

#[tauri::command]
async fn get_profiles() -> Result<Vec<VmProfile>, String> {
    use std::path::Path;
    
    // Try multiple possible locations for profiles directory
    let possible_paths = [
        "profiles",
        "./profiles", 
        "/mnt/home/lou/github/kvm-manager/profiles",
        "../profiles",
    ];
    
    let mut profiles_dir: Option<&Path> = None;
    for path_str in &possible_paths {
        let path = Path::new(path_str);
        if path.exists() {
            profiles_dir = Some(path);
            info!("Found profiles directory at: {}", path_str);
            break;
        }
    }
    
    let profiles_dir = match profiles_dir {
        Some(dir) => dir,
        None => {
            warn!("No profiles directory found in any of the expected locations");
            return Ok(Vec::new());
        }
    };
    
    let mut profiles = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(profiles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(profile) = serde_json::from_str::<VmProfile>(&content) {
                        profiles.push(profile);
                    }
                }
            }
        }
    }
    
    Ok(profiles)
}

#[tauri::command]
async fn create_vm_from_profile(
    state: tauri::State<'_, AppState>,
    profile_name: String,
) -> Result<String, String> {
    let profiles = get_profiles().await?;
    let profile = profiles.into_iter()
        .find(|p| p.name == profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;
    
    let mut manager = state.write().await;
    
    // Check if we have XML file for this profile
    let xml_filename = format!("{}.xml", profile_name.to_lowercase().replace(" ", "-"));
    let possible_xml_paths = [
        format!("profiles/{}", xml_filename),
        format!("./profiles/{}", xml_filename),
        format!("/mnt/home/lou/github/kvm-manager/profiles/{}", xml_filename),
        format!("../profiles/{}", xml_filename),
    ];
    
    let mut xml_path: Option<String> = None;
    for path_str in &possible_xml_paths {
        if std::path::Path::new(path_str).exists() {
            xml_path = Some(path_str.clone());
            info!("Found XML file at: {}", path_str);
            break;
        }
    }
    
    if let Some(xml_path) = xml_path {
        manager.import_vm_from_xml(&xml_path).await.map_err(|e| e.to_string())
    } else {
        // Create VM from QCOW2 if storage devices are specified
        if let Some(storage_device) = profile.storage_devices.first() {
            let passthrough_device = if profile.storage_devices.len() > 1 {
                Some(profile.storage_devices.get(1).unwrap().source.as_str())
            } else {
                None
            };
            
            manager.create_vm_from_qcow2(
                &storage_device.source,
                &profile.name,
                profile.memory as u64, // Profile memory is already in MB
                profile.vcpus,
                passthrough_device,
            ).await.map_err(|e| e.to_string())
        } else {
            Err("Profile has no storage devices defined".to_string())
        }
    }
}

#[tauri::command]
async fn get_qcow2_info(path: String) -> Result<QcowInfo, String> {
    use std::process::Command;
    
    info!("Getting QCOW2 info for: {}", path);
    
    // Check if file exists
    if !std::path::Path::new(&path).exists() {
        return Err(format!("File not found: {}", path));
    }
    
    // Use qemu-img info to get details
    let output = Command::new("qemu-img")
        .args(["info", "--output=json", &path])
        .output()
        .map_err(|e| format!("Failed to run qemu-img: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("qemu-img failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let info_json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse qemu-img output: {}", e))?;
    
    let filename = std::path::Path::new(&path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    let size_gb = info_json["actual-size"]
        .as_u64()
        .unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0;
    
    let virtual_size_gb = info_json["virtual-size"]
        .as_u64()
        .unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0;
    
    let format = info_json["format"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    
    let cluster_size = info_json["cluster-size"].as_u64();
    
    let backing_file = info_json["backing-filename"]
        .as_str()
        .map(|s| s.to_string());
    
    Ok(QcowInfo {
        path: path.clone(),
        filename,
        size_gb,
        format,
        virtual_size_gb,
        cluster_size,
        backing_file,
    })
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct QcowInfo {
    pub path: String,
    pub filename: String,
    pub size_gb: f64,
    pub format: String,
    pub virtual_size_gb: f64,
    pub cluster_size: Option<u64>,
    pub backing_file: Option<String>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting KVM Manager application");

    // Initialize VM Manager
    let vm_manager = match VmManager::new().await {
        Ok(manager) => Arc::new(RwLock::new(manager)),
        Err(e) => {
            error!("Failed to initialize VM Manager: {}", e);
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(vm_manager)
        .invoke_handler(tauri::generate_handler![
            get_vms,
            create_vm,
            start_vm,
            stop_vm,
            delete_vm,
            get_vm_stats,
            get_host_info,
            create_snapshot,
            restore_snapshot,
            list_vm_snapshots,
            delete_vm_snapshot,
            get_storage_pools,
            get_networks,
            create_proxmox_vm,
            import_vm_from_xml,
            create_vm_from_qcow2,
            refresh_vms,
            get_qcow2_info,
            browse_qcow2_files,
            browse_xml_files,
            get_profiles,
            create_vm_from_profile,
            system_monitor::get_system_statistics,
            system_monitor::get_proxmox_info,
            system_monitor::get_system_history,
            system_monitor::start_system_monitoring
        ])
        .setup(|_app| {
    info!("Application setup complete");
            
            // Test Proxmox detection
            tokio::spawn(async {
                let proxmox_path = "/run/media/garuda/Data/proxmox-ve.qcow2";
                match system_monitor::SystemMonitor::get_proxmox_vm_info(proxmox_path) {
                    Ok(info) => info!("Proxmox VM detected: {} GB, running: {}", info.size_gb, info.is_running),
                    Err(e) => error!("Failed to detect Proxmox VM: {}", e),
                }
            });
            
            // Start system monitoring
            tokio::spawn(async {
                if let Err(e) = system_monitor::start_system_monitoring().await {
                    error!("Failed to start system monitoring: {}", e);
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
