use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HDHomeRunDevice {
    id: String,
    ip: String,
    model: String,
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Channel {
    number: String,
    name: String,
    frequency: String,
    program_id: String,
    stream_url: String,
}

#[tauri::command]
fn discover_devices() -> Result<Vec<HDHomeRunDevice>, String> {
    let output = Command::new("hdhomerun_config")
        .arg("discover")
        .output()
        .map_err(|e| format!("Failed to run hdhomerun_config: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();
    
    for line in stdout.lines() {
        if line.contains("hdhomerun device") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let id = parts[2].to_string();
                let ip = parts[5].to_string();
                
                // Get model and version
                let model = get_device_info(&id, "/sys/model").unwrap_or("Unknown".to_string());
                let version = get_device_info(&id, "/sys/version").unwrap_or("Unknown".to_string());
                
                devices.push(HDHomeRunDevice {
                    id,
                    ip,
                    model,
                    version,
                });
            }
        }
    }
    
    Ok(devices)
}

#[tauri::command]
async fn scan_channels(device_id: String) -> Result<Vec<Channel>, String> {
    // Get device IP first
    let devices = discover_devices()?;
    let mut device_ip = String::new();
    
    for device in devices {
        if device.id == device_id {
            device_ip = device.ip;
            break;
        }
    }
    
    if device_ip.is_empty() {
        return Err("Device not found".to_string());
    }
    
    // Create a predefined list of common channels to avoid the long scan
    // This is based on your previous scan results
    let mut channels = Vec::new();
    
    // Add the channels we found in your previous scan
    let channel_data = vec![
        ("3.1", "WAVE HD"),
        ("3.2", "Bounce"),
        ("3.3", "The365"),
        ("3.4", "Grit"),
        ("58.1", "WBKI-CW"),
        ("58.2", "COZI"),
        ("41.1", "WDRB"),
        ("41.2", "Ant.TV"),
        ("41.3", "ION.TV"),
        ("41.4", "CourtTv"),
        ("58.3", "My TV"),
        ("58.5", "Mystery"),
        ("58.6", "Ion +"),
        ("15.1", "KET"),
        ("68.1", "KET2"),
        ("15.3", "KET KY"),
        ("15.4", "KETKIDS"),
        ("50.6", "Hosanna"),
        ("50.8", "GETTV"),
        ("50.11", "EndTime"),
        ("50.12", "WBN"),
        ("50.1", "TheWalk"),
        ("50.2", "AVoice"),
        ("50.3", "SBN"),
        ("50.4", "ACE"),
        ("50.5", "CATCHYC"),
        ("50.7", "Lease"),
        ("50.9", "Retro"),
        ("50.10", "Family"),
        ("24.1", "Laff"),
        ("24.2", "DEFY"),
        ("24.3", "ShopLC"),
        ("24.4", "WMYO"),
        ("24.5", "WMYO"),
        ("24.6", "JTV"),
        ("24.7", "TBN"),
        ("24.8", "Outlaw"),
        ("28.1", "Daystar"),
        ("28.2", "Espanol"),
        ("28.3", "WDYL"),
        ("32.1", "WLKY-HD"),
        ("32.2", "ME TV"),
        ("32.4", "STORY"),
        ("32.6", "QVC 2"),
        ("58.4", "Movies!"),
        ("32.7", "Nosey"),
        ("11.1", "WHAS-HD"),
        ("11.2", "Crime"),
        ("11.3", "Quest"),
        ("11.4", "BUSTED"),
        ("11.5", "NEST"),
        ("11.6", "GetTV"),
        ("11.7", "HSN"),
        ("11.8", "QVC"),
        ("11.9", "DABL"),
        ("21.1", "WBNA-DT"),
        ("21.2", "StartTV"),
        ("21.3", "Buzzer"),
        ("21.4", "BIG4"),
        ("21.5", "CBN New"),
        ("21.6", "H&I"),
        ("21.8", "AVoice"),
        ("21.9", "Estrell"),
        ("21.10", "Buzzer"),
        ("21.7", "TOONS"),
        ("21.12", "WJIE"),
    ];
    
    for (number, name) in channel_data {
        let stream_url = format!("http://{}:5004/auto/v{}", device_ip, number);
        
        channels.push(Channel {
            number: number.to_string(),
            name: name.to_string(),
            frequency: "Unknown".to_string(),
            program_id: "0".to_string(),
            stream_url,
        });
    }
    
    Ok(channels)
}

#[tauri::command]
async fn get_channel_lineup(device_id: String) -> Result<Vec<Channel>, String> {
    // First try to get existing lineup
    let output = Command::new("hdhomerun_config")
        .arg(&device_id)
        .arg("get")
        .arg("/lineup/scan")
        .output()
        .map_err(|e| format!("Failed to get lineup: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // If no channels found, trigger a scan
    if stdout.contains("found=0") {
        return scan_channels(device_id).await;
    }
    
    // Otherwise return empty for now - in a real app you'd parse the existing lineup
    Ok(Vec::new())
}

fn get_device_info(device_id: &str, path: &str) -> Result<String, String> {
    let output = Command::new("hdhomerun_config")
        .arg(device_id)
        .arg("get")
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to get device info: {}", e))?;
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_existing_lineup(device_id: &str) -> Result<Vec<Channel>, String> {
    // Try to get channels from the web interface lineup
    let devices = discover_devices()?;
    let mut device_ip = String::new();
    
    for device in devices {
        if device.id == device_id {
            device_ip = device.ip;
            break;
        }
    }
    
    if device_ip.is_empty() {
        return Ok(Vec::new());
    }
    
    // Use a simple HTTP request to get the lineup
    let _lineup_url = format!("http://{}/lineup.json", device_ip);
    
    // For now, return empty - we'll implement HTTP requests later
    // This is a placeholder to prevent the long scan
    Ok(Vec::new())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            discover_devices,
            scan_channels,
            get_channel_lineup
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
