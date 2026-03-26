// RGB Control Command Handlers
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbStatus {
    pub enabled: bool,
    pub color: [u8; 3],
    pub brightness: u8,
}

// Simple RGB state management
static RGB_STATE: Mutex<RgbStatus> = Mutex::new(RgbStatus {
    enabled: true,
    color: [255, 0, 0], // Default red
    brightness: 100,
});

#[tauri::command]
pub async fn get_rgb_status() -> Result<RgbStatus, String> {
    let state = RGB_STATE.lock().map_err(|e| e.to_string())?;
    Ok(state.clone())
}

#[tauri::command]
pub async fn toggle_rgb() -> Result<String, String> {
    let (enabled, color, brightness) = {
        let mut state = RGB_STATE.lock().map_err(|e| e.to_string())?;
        state.enabled = !state.enabled;
        (state.enabled, state.color, state.brightness)
    }; // Mutex guard is dropped here
    
    if enabled {
        // Try to set color through hidraw device
        let _ = send_rgb_command(&color, brightness).await;
        Ok("RGB lighting enabled".to_string())
    } else {
        // Try to clear RGB effects
        let _ = clear_rgb_effects().await;
        Ok("RGB lighting disabled".to_string())
    }
}

#[tauri::command]
pub async fn set_rgb_color(r: u8, g: u8, b: u8) -> Result<String, String> {
    let (enabled, color, brightness) = {
        let mut state = RGB_STATE.lock().map_err(|e| e.to_string())?;
        state.color = [r, g, b];
        (state.enabled, state.color.clone(), state.brightness)
    };
    
    if enabled {
        if let Err(e) = send_rgb_command(&color, brightness).await {
            return Err(format!("Failed to set RGB color: {}", e));
        }
    }
    
    Ok(format!("RGB color set to ({}, {}, {})", r, g, b))
}

#[tauri::command]
pub async fn set_rgb_brightness(brightness: u8) -> Result<String, String> {
    let (enabled, color) = {
        let mut state = RGB_STATE.lock().map_err(|e| e.to_string())?;
        state.brightness = brightness.min(100);
        (state.enabled, state.color.clone())
    };
    
    if enabled {
        if let Err(e) = send_rgb_command(&color, brightness).await {
            return Err(format!("Failed to set RGB brightness: {}", e));
        }
    }
    
    Ok(format!("RGB brightness set to {}%", brightness))
}

async fn send_rgb_command(color: &[u8; 3], brightness: u8) -> Result<(), String> {
    use std::fs::OpenOptions;
    use std::io::Write;
    
    const CLEVO_RGB_DEVICE: &str = "/dev/hidraw0";
    
    // Try to open the RGB device
    let mut device = OpenOptions::new()
        .write(true)
        .open(CLEVO_RGB_DEVICE)
        .map_err(|e| format!("Failed to open RGB device: {}", e))?;
    
    // Command format based on Clevo RGB protocol
    let mut data = [0xCC, 0x01, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    
    // Set RGB values
    data[3] = color[0]; // Red
    data[4] = color[1]; // Green
    data[5] = color[2]; // Blue
    data[6] = brightness; // Brightness
    
    // Write command to device
    device.write_all(&data)
        .map_err(|e| format!("Failed to write RGB command: {}", e))?;
    
    device.flush()
        .map_err(|e| format!("Failed to flush RGB command: {}", e))?;
    
    Ok(())
}

async fn clear_rgb_effects() -> Result<(), String> {
    use std::fs::OpenOptions;
    use std::io::Write;
    
    const CLEVO_RGB_DEVICE: &str = "/dev/hidraw0";
    
    // Try to open the RGB device
    let mut device = OpenOptions::new()
        .write(true)
        .open(CLEVO_RGB_DEVICE)
        .map_err(|e| format!("Failed to open RGB device: {}", e))?;
    
    // Clear effects command
    let data = [0xCC, 0x01, 0x53, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    
    // Write command to device
    device.write_all(&data)
        .map_err(|e| format!("Failed to write RGB clear command: {}", e))?;
    
    device.flush()
        .map_err(|e| format!("Failed to flush RGB clear command: {}", e))?;
    
    Ok(())
}
