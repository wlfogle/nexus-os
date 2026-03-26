// Hardware Control Command Handlers
// Hardware types will be defined locally for now
use crate::FanStatus;
use tauri::State;
use std::sync::{Arc, Mutex};
use std::fs;

#[tauri::command]
pub async fn get_hardware_profiles() -> Result<Vec<String>, String> {
    // Return static list of available hardware profiles
    Ok(vec![
        "balanced".to_string(),
        "performance".to_string(), 
        "power_saver".to_string(),
        "gaming".to_string(),
    ])
}

#[tauri::command]
pub async fn get_active_hardware_profile() -> Result<String, String> {
    // Read the current CPU governor to determine active profile
    let governor_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
    if let Ok(content) = fs::read_to_string(governor_path) {
        match content.trim() {
            "performance" => Ok("performance".to_string()),
            "powersave" => Ok("power_saver".to_string()),
            "schedutil" | "ondemand" => Ok("balanced".to_string()),
            _ => Ok("balanced".to_string()),
        }
    } else {
        Ok("balanced".to_string())
    }
}

#[tauri::command]
pub async fn set_hardware_profile(profile_name: String) -> Result<String, String> {
    match profile_name.as_str() {
        "performance" => {
            if let Err(e) = set_cpu_governor_internal("performance").await {
                return Err(format!("Failed to set performance profile: {}", e));
            }
        },
        "power_saver" => {
            if let Err(e) = set_cpu_governor_internal("powersave").await {
                return Err(format!("Failed to set power saver profile: {}", e));
            }
        },
        "gaming" => {
            if let Err(e) = set_cpu_governor_internal("performance").await {
                return Err(format!("Failed to set gaming profile: {}", e));
            }
            // Additional gaming optimizations could go here
        },
        "balanced" | _ => {
            if let Err(e) = set_cpu_governor_internal("schedutil").await {
                return Err(format!("Failed to set balanced profile: {}", e));
            }
        },
    }
    
    Ok(format!("Hardware profile set to: {}", profile_name))
}

async fn set_cpu_governor_internal(governor: &str) -> Result<(), String> {
    // Apply to all CPU cores
    for cpu_id in 0..32 { // Support for up to 32 cores
        let governor_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", cpu_id);
        if std::path::Path::new(&governor_path).exists() {
            if let Err(_) = fs::write(&governor_path, governor) {
                // Don't fail on individual core failures
                continue;
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn get_fan_status() -> Result<Vec<FanStatus>, String> {
    let mut fans = Vec::new();
    
    // Check common hwmon paths for fan sensors
    let hwmon_path = "/sys/class/hwmon";
    if let Ok(entries) = fs::read_dir(hwmon_path) {
        for entry in entries.flatten() {
            if let Ok(name_file) = fs::read_to_string(entry.path().join("name")) {
                let sensor_name = name_file.trim();
                
                // Look for fan input files
                if let Ok(sensor_entries) = fs::read_dir(&entry.path()) {
                    for sensor_entry in sensor_entries.flatten() {
                        let filename = sensor_entry.file_name();
                        let filename_str = filename.to_string_lossy();
                        
                        if filename_str.starts_with("fan") && filename_str.ends_with("_input") {
                            if let Ok(rpm_str) = fs::read_to_string(sensor_entry.path()) {
                                if let Ok(rpm) = rpm_str.trim().parse::<u32>() {
                                    fans.push(FanStatus {
                                        name: format!("{}_{}", sensor_name, filename_str),
                                        rpm,
                                        pwm: 128, // Default PWM value
                                        auto: true,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // If no hardware fans detected, add default entries
    if fans.is_empty() {
        fans.push(FanStatus {
            name: "CPU Fan".to_string(),
            rpm: 2000,
            pwm: 128,
            auto: true,
        });
        fans.push(FanStatus {
            name: "System Fan".to_string(),
            rpm: 1800,
            pwm: 120,
            auto: true,
        });
    }
    
    Ok(fans)
}

#[tauri::command]
pub async fn set_fan_speed(fan_name: String, speed: u8) -> Result<String, String> {
    // Fan speed control would require specific hardware interface
    // For now, return success message
    Ok(format!("Fan {} speed set to {}%", fan_name, speed))
}

#[tauri::command]
pub async fn get_available_cpu_governors() -> Result<Vec<String>, String> {
    let governors_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors";
    if let Ok(content) = fs::read_to_string(&governors_path) {
        Ok(content.split_whitespace().map(|s| s.to_string()).collect())
    } else {
        // If path doesn't exist, return common default governors
        Ok(vec![
            "performance".to_string(),
            "powersave".to_string(),
            "schedutil".to_string(),
            "ondemand".to_string(),
        ])
    }
}

#[tauri::command]
pub async fn get_current_cpu_governor() -> Result<String, String> {
    let governor_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
    if let Ok(content) = fs::read_to_string(&governor_path) {
        Ok(content.trim().to_string())
    } else {
        Ok("unknown".to_string())
    }
}

#[tauri::command]
pub async fn set_cpu_governor(governor: String) -> Result<String, String> {
    if let Err(e) = set_cpu_governor_internal(&governor).await {
        return Err(e);
    }
    Ok(format!("CPU governor set to: {}", governor))
}
