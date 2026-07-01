// src-tauri/src/commands/nvme.rs
use serde::{Serialize, Deserialize};
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
pub struct Partition {
    pub name: String,
    pub mountpoint: Option<String>,
    pub size: String, // lsblk returns size as string
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NVMeDevice {
    pub name: String,
    pub model: String,
    pub size: String,
    pub children: Option<Vec<Partition>>, // lsblk nests partitions under 'children'
}

#[derive(Deserialize)]
struct LsblkOutput {
    blockdevices: Vec<NVMeDevice>,
}

#[tauri::command]
pub async fn get_nvme_devices() -> Result<Vec<NVMeDevice>, String> {
    // Execute lsblk to list NVMe devices in JSON format
    let output = Command::new("lsblk")
    .args(["-J", "-o", "NAME,MODEL,SIZE,MOUNTPOINT", "-e", "7,1,11"])
    .output()
    .map_err(|e| e.to_string())?;

    let parsed: LsblkOutput = serde_json::from_slice(&output.stdout)
    .map_err(|e| e.to_string())?;

    // Filter to only include devices that are NVMe (often identified by 'nvme' in name)
    let nvme_only: Vec<NVMeDevice> = parsed.blockdevices
    .into_iter()
    .filter(|d| d.name.contains("nvme"))
    .collect();

    Ok(nvme_only)
}
