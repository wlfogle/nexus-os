// Fan Controller - Intelligent fan curve management for i9-13900HX gaming laptops
use anyhow::{Result, anyhow, Context};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanCurvePoint {
    pub temperature: f32,
    pub fan_speed: u8, // PWM percentage 0-100
    pub hysteresis: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanProfile {
    pub name: String,
    pub curve_points: Vec<FanCurvePoint>,
    pub sensor_source: String, // "cpu", "gpu", "average"
    pub response_delay: u32, // milliseconds
    pub minimum_speed: u8,
    pub maximum_speed: u8,
}

#[derive(Debug, Clone)]
pub struct FanDevice {
    pub name: String,
    pub pwm_path: PathBuf,
    pub enable_path: PathBuf,
    pub rpm_path: Option<PathBuf>,
    pub current_pwm: u8,
    pub current_rpm: Option<u32>,
    pub is_controllable: bool,
}

pub struct FanManager {
    pub devices: HashMap<String, FanDevice>,
    pub profiles: HashMap<String, FanProfile>,
    pub active_profile: Option<String>,
    pub intelligent_mode: bool,
    pub temperature_history: Vec<(f32, SystemTime)>,
    pub last_adjustment: SystemTime,
    pub adjustment_interval: Duration,
    pub work_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl FanManager {
    pub async fn new_intelligent() -> Result<Self> {
        info!("🌪️ Initializing intelligent fan management for i9-13900HX");
        
        let current_dir = std::env::current_dir()?;
        let work_dir = current_dir.clone();
        let config_dir = work_dir.join("config").join("fan_control");
        
        // Ensure config directory exists
        fs::create_dir_all(&config_dir)?;
        
        let mut manager = Self {
            devices: HashMap::new(),
            profiles: HashMap::new(),
            active_profile: None,
            intelligent_mode: true,
            temperature_history: Vec::new(),
            last_adjustment: SystemTime::now(),
            adjustment_interval: Duration::from_secs(3),
            work_dir,
            config_dir,
        };
        
        // Detect fan devices
        manager.detect_fan_devices().await?;
        
        // Load or create default profiles
        manager.load_profiles().await?;
        manager.create_default_profiles().await?;
        
        // Set default profile
        manager.set_active_profile("balanced").await?;
        
        info!("✅ Fan management initialized with {} devices and {} profiles", 
              manager.devices.len(), manager.profiles.len());
        
        Ok(manager)
    }
    
    async fn detect_fan_devices(&mut self) -> Result<()> {
        debug!("🔍 Detecting fan control devices");
        
        let hwmon_path = PathBuf::from("/sys/class/hwmon");
        if !hwmon_path.exists() {
            return Ok(());
        }
        
        if let Ok(entries) = fs::read_dir(&hwmon_path) {
            for entry in entries.flatten() {
                if let Ok(name_file) = fs::read_to_string(entry.path().join("name")) {
                    let sensor_name = name_file.trim();
                    
                    // Look for PWM controls
                    if let Ok(hwmon_entries) = fs::read_dir(&entry.path()) {
                        for hwmon_entry in hwmon_entries.flatten() {
                            let filename = hwmon_entry.file_name();
                            let filename_str = filename.to_string_lossy();
                            
                            // Check for PWM controls (pwm1, pwm2, etc.)
                            if filename_str.starts_with("pwm") && !filename_str.contains("_") {
                                let pwm_num = filename_str.strip_prefix("pwm").unwrap_or("1");
                                let enable_path = entry.path().join(format!("pwm{}_enable", pwm_num));
                                let rpm_path = entry.path().join(format!("fan{}_input", pwm_num));
                                
                                let device_name = format!("{}_{}", sensor_name, filename_str);
                                
                                let device = FanDevice {
                                    name: device_name.clone(),
                                    pwm_path: hwmon_entry.path(),
                                    enable_path,
                                    rpm_path: if rpm_path.exists() { Some(rpm_path) } else { None },
                                    current_pwm: 128, // Default 50%
                                    current_rpm: None,
                                    is_controllable: self.test_fan_control(&hwmon_entry.path()).await,
                                };
                                
                                self.devices.insert(device_name, device);
                            }
                        }
                    }
                }
            }
        }
        
        debug!("🌪️ Detected {} fan devices", self.devices.len());
        Ok(())
    }
    
    async fn test_fan_control(&self, pwm_path: &PathBuf) -> bool {
        // Test if we can actually control this fan
        if let Ok(current_pwm) = fs::read_to_string(pwm_path) {
            if let Ok(current_value) = current_pwm.trim().parse::<u8>() {
                // Try to write the same value back (should be safe)
                if fs::write(pwm_path, current_value.to_string()).is_ok() {
                    return true;
                }
            }
        }
        false
    }
    
    async fn create_default_profiles(&mut self) -> Result<()> {
        // Silent profile - prioritizes quiet operation
        let silent_profile = FanProfile {
            name: "silent".to_string(),
            curve_points: vec![
                FanCurvePoint { temperature: 30.0, fan_speed: 20, hysteresis: 2.0 },
                FanCurvePoint { temperature: 40.0, fan_speed: 25, hysteresis: 2.0 },
                FanCurvePoint { temperature: 50.0, fan_speed: 35, hysteresis: 3.0 },
                FanCurvePoint { temperature: 60.0, fan_speed: 45, hysteresis: 3.0 },
                FanCurvePoint { temperature: 70.0, fan_speed: 60, hysteresis: 4.0 },
                FanCurvePoint { temperature: 80.0, fan_speed: 80, hysteresis: 4.0 },
                FanCurvePoint { temperature: 85.0, fan_speed: 100, hysteresis: 5.0 },
            ],
            sensor_source: "cpu".to_string(),
            response_delay: 5000, // 5 seconds for quiet transitions
            minimum_speed: 15,
            maximum_speed: 100,
        };
        
        // Balanced profile - good balance of cooling and noise
        let balanced_profile = FanProfile {
            name: "balanced".to_string(),
            curve_points: vec![
                FanCurvePoint { temperature: 30.0, fan_speed: 25, hysteresis: 2.0 },
                FanCurvePoint { temperature: 40.0, fan_speed: 35, hysteresis: 2.0 },
                FanCurvePoint { temperature: 50.0, fan_speed: 45, hysteresis: 3.0 },
                FanCurvePoint { temperature: 60.0, fan_speed: 60, hysteresis: 3.0 },
                FanCurvePoint { temperature: 70.0, fan_speed: 75, hysteresis: 3.0 },
                FanCurvePoint { temperature: 80.0, fan_speed: 90, hysteresis: 4.0 },
                FanCurvePoint { temperature: 85.0, fan_speed: 100, hysteresis: 4.0 },
            ],
            sensor_source: "cpu".to_string(),
            response_delay: 3000, // 3 seconds
            minimum_speed: 20,
            maximum_speed: 100,
        };
        
        // Performance profile - aggressive cooling for gaming/heavy workloads
        let performance_profile = FanProfile {
            name: "performance".to_string(),
            curve_points: vec![
                FanCurvePoint { temperature: 30.0, fan_speed: 40, hysteresis: 2.0 },
                FanCurvePoint { temperature: 40.0, fan_speed: 50, hysteresis: 2.0 },
                FanCurvePoint { temperature: 50.0, fan_speed: 65, hysteresis: 2.0 },
                FanCurvePoint { temperature: 60.0, fan_speed: 80, hysteresis: 3.0 },
                FanCurvePoint { temperature: 70.0, fan_speed: 90, hysteresis: 3.0 },
                FanCurvePoint { temperature: 80.0, fan_speed: 100, hysteresis: 3.0 },
            ],
            sensor_source: "cpu".to_string(),
            response_delay: 1000, // 1 second for fast response
            minimum_speed: 35,
            maximum_speed: 100,
        };
        
        self.profiles.insert("silent".to_string(), silent_profile);
        self.profiles.insert("balanced".to_string(), balanced_profile);
        self.profiles.insert("performance".to_string(), performance_profile);
        
        self.save_profiles().await?;
        Ok(())
    }
    
    async fn load_profiles(&mut self) -> Result<()> {
        let profiles_file = self.config_dir.join("fan_profiles.json");
        if profiles_file.exists() {
            let content = fs::read_to_string(&profiles_file)?;
            self.profiles = serde_json::from_str(&content).unwrap_or_default();
        }
        Ok(())
    }
    
    async fn save_profiles(&self) -> Result<()> {
        let profiles_file = self.config_dir.join("fan_profiles.json");
        let content = serde_json::to_string_pretty(&self.profiles)?;
        fs::write(&profiles_file, content)?;
        Ok(())
    }
    
    pub async fn set_active_profile(&mut self, profile_name: &str) -> Result<()> {
        if !self.profiles.contains_key(profile_name) {
            return Err(anyhow!("Fan profile '{}' not found", profile_name));
        }
        
        self.active_profile = Some(profile_name.to_string());
        info!("🎯 Set active fan profile: {}", profile_name);
        Ok(())
    }
    
    pub async fn start_intelligent_control(&mut self) -> Result<()> {
        info!("🧠 Starting intelligent fan control");
        
        self.intelligent_mode = true;
        
        // Enable all controllable fans
        for device in self.devices.values_mut() {
            if device.is_controllable {
                // Enable PWM control
                if device.enable_path.exists() {
                    if let Err(e) = fs::write(&device.enable_path, "1") {
                        warn!("Failed to enable fan control for {}: {}", device.name, e);
                    } else {
                        debug!("✅ Enabled PWM control for {}", device.name);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn update_fan_speeds(&mut self, cpu_temp: f32, gpu_temp: Option<f32>) -> Result<()> {
        if !self.intelligent_mode {
            return Ok(());
        }
        
        // Check if enough time has passed since last adjustment
        if self.last_adjustment.elapsed().unwrap_or(Duration::ZERO) < self.adjustment_interval {
            return Ok(());
        }
        
        // Store temperature in history
        self.temperature_history.push((cpu_temp, SystemTime::now()));
        if self.temperature_history.len() > 100 {
            self.temperature_history.remove(0);
        }
        
        let active_profile_name = if let Some(ref profile_name) = self.active_profile {
            profile_name.clone()
        } else {
            return Err(anyhow!("No active fan profile set"));
        };
        
        let profile = self.profiles.get(&active_profile_name)
            .ok_or_else(|| anyhow!("Active profile '{}' not found", active_profile_name))?;
        
        // Determine which temperature to use
        let target_temp = match profile.sensor_source.as_str() {
            "gpu" => gpu_temp.unwrap_or(cpu_temp),
            "average" => {
                if let Some(gpu_temp) = gpu_temp {
                    (cpu_temp + gpu_temp) / 2.0
                } else {
                    cpu_temp
                }
            },
            _ => cpu_temp, // Default to CPU
        };
        
        // Calculate target fan speed using the curve
        let target_speed = self.calculate_fan_speed_from_curve(&profile, target_temp);
        
        // Apply speed to all controllable fans
        for device in self.devices.values_mut() {
            if device.is_controllable {
                self.set_fan_speed(device, target_speed).await?;
            }
        }
        
        self.last_adjustment = SystemTime::now();
        debug!("🌪️ Updated fan speeds to {}% for temperature {:.1}°C", target_speed, target_temp);
        
        Ok(())
    }
    
    fn calculate_fan_speed_from_curve(&self, profile: &FanProfile, temperature: f32) -> u8 {
        // Find the appropriate curve point
        let mut target_speed = profile.minimum_speed;
        
        for i in 0..profile.curve_points.len() {
            let point = &profile.curve_points[i];
            
            if temperature >= point.temperature {
                target_speed = point.fan_speed;
                
                // Interpolate between curve points
                if i + 1 < profile.curve_points.len() {
                    let next_point = &profile.curve_points[i + 1];
                    if temperature < next_point.temperature {
                        // Linear interpolation
                        let temp_range = next_point.temperature - point.temperature;
                        let speed_range = next_point.fan_speed as f32 - point.fan_speed as f32;
                        let temp_offset = temperature - point.temperature;
                        
                        let interpolated_speed = point.fan_speed as f32 + 
                            (temp_offset / temp_range) * speed_range;
                        
                        target_speed = interpolated_speed.round() as u8;
                        break;
                    }
                }
            } else {
                break;
            }
        }
        
        // Apply hysteresis to prevent fan speed oscillation
        target_speed = self.apply_hysteresis(target_speed, temperature);
        
        // Ensure speed is within bounds
        target_speed.max(profile.minimum_speed).min(profile.maximum_speed)
    }
    
    fn apply_hysteresis(&self, target_speed: u8, current_temp: f32) -> u8 {
        // Simple hysteresis: if temperature recently dropped, be more conservative
        if self.temperature_history.len() >= 2 {
            let recent_temps: Vec<f32> = self.temperature_history
                .iter()
                .rev()
                .take(5)
                .map(|(temp, _)| *temp)
                .collect();
            
            if recent_temps.len() >= 3 {
                let temp_trend = recent_temps[0] - recent_temps[2];
                
                // If temperature is dropping, reduce fan speed more gradually
                if temp_trend < -2.0 {
                    return (target_speed as f32 * 0.9) as u8;
                }
                
                // If temperature is rising quickly, increase fan speed more aggressively
                if temp_trend > 3.0 {
                    return (target_speed as f32 * 1.1).min(100.0) as u8;
                }
            }
        }
        
        target_speed
    }
    
    async fn set_fan_speed(&mut self, device: &mut FanDevice, speed_percent: u8) -> Result<()> {
        let pwm_value = (speed_percent as f32 / 100.0 * 255.0) as u8;
        
        if let Err(e) = fs::write(&device.pwm_path, pwm_value.to_string()) {
            warn!("Failed to set fan speed for {}: {}", device.name, e);
            return Err(anyhow!("Failed to control fan {}: {}", device.name, e));
        }
        
        device.current_pwm = pwm_value;
        
        // Update RPM reading if available
        if let Some(ref rpm_path) = device.rpm_path {
            if let Ok(rpm_str) = fs::read_to_string(rpm_path) {
                device.current_rpm = rpm_str.trim().parse().ok();
            }
        }
        
        Ok(())
    }
    
    pub async fn set_manual_fan_speed(&mut self, device_name: &str, speed_percent: u8) -> Result<()> {
        if let Some(device) = self.devices.get_mut(device_name) {
            if !device.is_controllable {
                return Err(anyhow!("Fan {} is not controllable", device_name));
            }
            
            self.intelligent_mode = false; // Disable intelligent mode for manual control
            self.set_fan_speed(device, speed_percent).await?;
            
            info!("🎛️ Set manual fan speed for {}: {}%", device_name, speed_percent);
        } else {
            return Err(anyhow!("Fan device '{}' not found", device_name));
        }
        
        Ok(())
    }
    
    pub async fn enable_intelligent_mode(&mut self) -> Result<()> {
        self.intelligent_mode = true;
        info!("🧠 Intelligent fan control enabled");
        Ok(())
    }
    
    pub async fn disable_intelligent_mode(&mut self) -> Result<()> {
        self.intelligent_mode = false;
        info!("✋ Intelligent fan control disabled - manual mode active");
        Ok(())
    }
    
    pub async fn create_custom_profile(&mut self, name: String, curve_points: Vec<FanCurvePoint>) -> Result<()> {
        let profile = FanProfile {
            name: name.clone(),
            curve_points,
            sensor_source: "cpu".to_string(),
            response_delay: 3000,
            minimum_speed: 20,
            maximum_speed: 100,
        };
        
        self.profiles.insert(name.clone(), profile);
        self.save_profiles().await?;
        
        info!("✅ Created custom fan profile: {}", name);
        Ok(())
    }
    
    pub fn get_device_status(&self) -> HashMap<String, serde_json::Value> {
        let mut status = HashMap::new();
        
        for (name, device) in &self.devices {
            status.insert(name.clone(), serde_json::json!({
                "controllable": device.is_controllable,
                "current_pwm": device.current_pwm,
                "current_rpm": device.current_rpm,
                "pwm_path": device.pwm_path.to_string_lossy(),
            }));
        }
        
        status
    }
    
    pub fn get_profile_names(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }
    
    pub fn get_active_profile(&self) -> Option<FanProfile> {
        if let Some(ref profile_name) = self.active_profile {
            self.profiles.get(profile_name).cloned()
        } else {
            None
        }
    }
    
    pub fn is_intelligent_mode(&self) -> bool {
        self.intelligent_mode
    }
    
    pub fn get_temperature_history(&self, minutes: u32) -> Vec<(f32, u64)> {
        let cutoff_time = SystemTime::now() - Duration::from_secs(minutes as u64 * 60);
        
        self.temperature_history
            .iter()
            .filter(|(_, time)| *time > cutoff_time)
            .map(|(temp, time)| (*temp, time.duration_since(UNIX_EPOCH).unwrap().as_secs()))
            .collect()
    }
}
