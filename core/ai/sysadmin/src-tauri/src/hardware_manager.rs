// Hardware Manager - i9-13900HX gaming laptop optimizations
// Complete implementation with CPU/GPU tuning, power management, and RGB control
use anyhow::{Result, anyhow, Context};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex};
use std::process::Command;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tokio::task;
use tracing::{info, warn, error, debug};

use crate::rgb_controller::RGBManager;
use crate::fan_controller::FanManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PowerMode {
    Performance,
    Balanced,
    PowerSaver,
    Gaming,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub name: String,
    pub power_mode: PowerMode,
    pub cpu_governor: String,
    pub gpu_power_level: String,
    pub fan_profile: String,
    pub thermal_throttle_temp: f32,
    pub overclock_enabled: bool,
    pub rgb_profile: Option<String>,
    pub custom_settings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareStatus {
    pub cpu_temp: f32,
    pub gpu_temp: Option<f32>,
    pub current_power_mode: PowerMode,
    pub current_cpu_governor: String,
    pub current_gpu_power_level: String,
    pub current_fan_profile: String,
    pub active_cooling: bool,
    pub battery_mode: bool,
    pub throttling: bool,
    pub optimization_active: bool,
}

pub struct HardwareController {
    // Managed hardware components
    pub rgb_manager: Option<RGBManager>,
    pub fan_manager: Option<FanManager>,
    
    // Hardware profiles
    pub profiles: HashMap<String, HardwareProfile>,
    pub active_profile: Option<String>,
    
    // System paths
    pub cpu_governor_path: PathBuf,
    pub gpu_power_path: Option<PathBuf>,
    pub thermal_throttle_path: PathBuf,
    
    // Runtime state
    pub i9_13900hx_optimized: bool,
    pub optimization_active: bool,
    pub last_status: Option<HardwareStatus>,
    pub optimization_interval: Duration,
    pub sleep_allowed: bool,
    
    // Configuration
    pub work_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl HardwareController {
    pub async fn new_gaming_laptop() -> Result<Self> {
        info!("🔧 Initializing hardware controller for i9-13900HX gaming laptop");
        
        let current_dir = std::env::current_dir()?;
        let work_dir = current_dir.clone();
        let config_dir = work_dir.join("config").join("hardware");
        
        // Ensure config directory exists
        fs::create_dir_all(&config_dir)?;
        
        let mut controller = Self {
            rgb_manager: None,
            fan_manager: None,
            profiles: HashMap::new(),
            active_profile: None,
            cpu_governor_path: PathBuf::from("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor"),
            gpu_power_path: Self::detect_gpu_power_path().await,
            thermal_throttle_path: PathBuf::from("/sys/devices/virtual/thermal/thermal_zone0/trip_point_0_temp"),
            i9_13900hx_optimized: true,
            optimization_active: false,
            last_status: None,
            optimization_interval: Duration::from_secs(10),
            sleep_allowed: true,
            work_dir,
            config_dir,
        };
        
        // Create default hardware profiles
        controller.create_default_profiles().await?;
        controller.load_profiles().await?;
        
        // Initialize RGB manager
        match RGBManager::new_originpc().await {
            Ok(rgb) => controller.rgb_manager = Some(rgb),
            Err(e) => warn!("RGB manager initialization failed: {}", e),
        }
        
        // Initialize Fan manager
        match FanManager::new_intelligent().await {
            Ok(fan) => controller.fan_manager = Some(fan),
            Err(e) => warn!("Fan manager initialization failed: {}", e),
        }
        
        // Set default profile to balanced
        controller.set_active_profile("balanced").await?;
        
        info!("✅ Hardware controller initialized with {} profiles", controller.profiles.len());
        
        Ok(controller)
    }
    
    async fn detect_gpu_power_path() -> Option<PathBuf> {
        // Try NVIDIA path first
        let nvidia_path = PathBuf::from("/sys/class/drm/card0/device/power/control");
        if nvidia_path.exists() {
            return Some(nvidia_path);
        }
        
        // Try AMD path
        let amd_path = PathBuf::from("/sys/class/drm/card0/device/power_dpm_force_performance_level");
        if amd_path.exists() {
            return Some(amd_path);
        }
        
        None
    }
    
    async fn create_default_profiles(&mut self) -> Result<()> {
        // Performance profile - maximum performance
        let performance_profile = HardwareProfile {
            name: "performance".to_string(),
            power_mode: PowerMode::Performance,
            cpu_governor: "performance".to_string(),
            gpu_power_level: "high".to_string(),
            fan_profile: "performance".to_string(),
            thermal_throttle_temp: 95.0,
            overclock_enabled: false, // Safety default
            rgb_profile: None,
            custom_settings: HashMap::new(),
        };
        
        // Balanced profile - good balance of performance and energy
        let balanced_profile = HardwareProfile {
            name: "balanced".to_string(),
            power_mode: PowerMode::Balanced,
            cpu_governor: "schedutil".to_string(), // Modern scheduler-driven governor
            gpu_power_level: "auto".to_string(),
            fan_profile: "balanced".to_string(),
            thermal_throttle_temp: 90.0,
            overclock_enabled: false,
            rgb_profile: None,
            custom_settings: HashMap::new(),
        };
        
        // Power saver profile - maximize battery life
        let power_saver_profile = HardwareProfile {
            name: "power_saver".to_string(),
            power_mode: PowerMode::PowerSaver,
            cpu_governor: "powersave".to_string(),
            gpu_power_level: "low".to_string(),
            fan_profile: "silent".to_string(),
            thermal_throttle_temp: 85.0,
            overclock_enabled: false,
            rgb_profile: None,
            custom_settings: HashMap::new(),
        };
        
        // Gaming profile - optimized for gaming performance
        let gaming_profile = HardwareProfile {
            name: "gaming".to_string(),
            power_mode: PowerMode::Gaming,
            cpu_governor: "performance".to_string(),
            gpu_power_level: "high".to_string(),
            fan_profile: "performance".to_string(),
            thermal_throttle_temp: 95.0,
            overclock_enabled: true,
            rgb_profile: None,
            custom_settings: HashMap::from([
                ("cpu_boost".to_string(), "1".to_string()),
                ("gpu_boost".to_string(), "1".to_string()),
            ]),
        };
        
        self.profiles.insert("performance".to_string(), performance_profile);
        self.profiles.insert("balanced".to_string(), balanced_profile);
        self.profiles.insert("power_saver".to_string(), power_saver_profile);
        self.profiles.insert("gaming".to_string(), gaming_profile);
        
        self.save_profiles().await?;
        Ok(())
    }
    
    async fn load_profiles(&mut self) -> Result<()> {
        let profiles_file = self.config_dir.join("hardware_profiles.json");
        if profiles_file.exists() {
            let content = fs::read_to_string(&profiles_file)?;
            let loaded_profiles: HashMap<String, HardwareProfile> = serde_json::from_str(&content)?;
            
            // Merge with existing profiles, prefer loaded ones
            for (name, profile) in loaded_profiles {
                self.profiles.insert(name, profile);
            }
            
            debug!("📋 Loaded {} hardware profiles", self.profiles.len());
        }
        Ok(())
    }
    
    async fn save_profiles(&self) -> Result<()> {
        let profiles_file = self.config_dir.join("hardware_profiles.json");
        let content = serde_json::to_string_pretty(&self.profiles)?;
        fs::write(&profiles_file, content)?;
        debug!("💾 Saved {} hardware profiles", self.profiles.len());
        Ok(())
    }
    
    pub async fn set_active_profile(&mut self, profile_name: &str) -> Result<()> {
        if !self.profiles.contains_key(profile_name) {
            return Err(anyhow!("Hardware profile '{}' not found", profile_name));
        }
        
        self.active_profile = Some(profile_name.to_string());
        
        // Apply the profile settings immediately
        self.apply_active_profile().await?;
        
        info!("🎯 Set active hardware profile: {}", profile_name);
        Ok(())
    }
    
    async fn apply_active_profile(&mut self) -> Result<()> {
        let profile_name = if let Some(ref name) = self.active_profile {
            name.clone()
        } else {
            return Err(anyhow!("No active hardware profile set"));
        };
        
        let profile = self.profiles.get(&profile_name)
            .ok_or_else(|| anyhow!("Active profile '{}' not found", profile_name))?;
        
        debug!("🔧 Applying hardware profile: {}", profile_name);
        
        // Set CPU governor
        self.set_cpu_governor(&profile.cpu_governor).await?;
        
        // Set GPU power level
        self.set_gpu_power_level(&profile.gpu_power_level).await?;
        
        // Set fan profile if fan manager is available
        if let Some(fan_manager) = &mut self.fan_manager {
            if let Err(e) = fan_manager.set_active_profile(&profile.fan_profile).await {
                warn!("Failed to set fan profile: {}", e);
            } else {
                if let Err(e) = fan_manager.start_intelligent_control().await {
                    warn!("Failed to start intelligent fan control: {}", e);
                }
            }
        }
        
        // Set RGB profile if manager is available and profile specifies one
        if let Some(rgb_profile) = &profile.rgb_profile {
            if let Some(rgb_manager) = &mut self.rgb_manager {
                if let Err(e) = rgb_manager.start_effect_engine().await {
                    warn!("Failed to start RGB effects: {}", e);
                }
            }
        }
        
        // Set thermal throttle temperature
        self.set_thermal_throttle_temp(profile.thermal_throttle_temp).await?;
        
        // Apply overclocking if enabled
        if profile.overclock_enabled {
            self.apply_safe_overclock().await?;
        }
        
        // Apply any custom settings
        for (key, value) in &profile.custom_settings {
            self.apply_custom_setting(key, value).await?;
        }
        
        // Update the system status
        self.update_status().await?;
        
        Ok(())
    }
    
    async fn set_cpu_governor(&self, governor: &str) -> Result<()> {
        debug!("🔄 Setting CPU governor to {}", governor);
        
        // First check if the governor is valid
        let available_governors = self.get_available_governors().await?;
        if !available_governors.contains(&governor.to_string()) {
            return Err(anyhow!("CPU governor '{}' not available. Valid options: {}", 
                    governor, available_governors.join(", ")));
        }
        
        // Apply to all CPU cores
        for cpu_id in 0..32 { // Support for up to 32 cores
            let governor_path = PathBuf::from(format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", cpu_id));
            if governor_path.exists() {
                if let Err(e) = fs::write(&governor_path, governor) {
                    warn!("Failed to set governor for cpu{}: {}", cpu_id, e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn get_available_governors(&self) -> Result<Vec<String>> {
        let governors_path = PathBuf::from("/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors");
        if governors_path.exists() {
            let content = fs::read_to_string(&governors_path)?;
            return Ok(content.split_whitespace().map(|s| s.to_string()).collect());
        }
        
        // If path doesn't exist, return common default governors
        Ok(vec![
            "performance".to_string(),
            "powersave".to_string(),
            "schedutil".to_string(),
            "ondemand".to_string(),
        ])
    }
    
    async fn set_gpu_power_level(&self, level: &str) -> Result<()> {
        debug!("🎮 Setting GPU power level to {}", level);
        
        if let Some(ref gpu_path) = self.gpu_power_path {
            // Adapt the command based on GPU type
            if gpu_path.to_string_lossy().contains("nvidia") {
                // NVIDIA GPU power management
                let level_value = match level {
                    "high" => "1", // Max performance
                    "auto" => "auto",
                    "low" => "0", // Power saving
                    _ => "auto",
                };
                
                if let Err(e) = fs::write(gpu_path, level_value) {
                    warn!("Failed to set NVIDIA GPU power level: {}", e);
                }
                
                // Try to set power management mode via nvidia-settings
                if level == "high" {
                    let _ = Command::new("nvidia-settings")
                        .args(["-a", "[gpu:0]/GpuPowerMizerMode=1"])
                        .output();
                }
            } else if gpu_path.to_string_lossy().contains("amd") {
                // AMD GPU power management
                let level_value = match level {
                    "high" => "high",
                    "auto" => "auto",
                    "low" => "low",
                    _ => "auto",
                };
                
                if let Err(e) = fs::write(gpu_path, level_value) {
                    warn!("Failed to set AMD GPU power level: {}", e);
                }
            }
        } else {
            debug!("No GPU power management path found");
        }
        
        Ok(())
    }
    
    async fn set_thermal_throttle_temp(&self, temp: f32) -> Result<()> {
        debug!("🌡️ Setting thermal throttle temperature to {:.1}°C", temp);
        
        // Convert to millidegrees as used by the kernel
        let milli_degrees = (temp * 1000.0) as u32;
        
        if self.thermal_throttle_path.exists() {
            if let Err(e) = fs::write(&self.thermal_throttle_path, milli_degrees.to_string()) {
                warn!("Failed to set thermal throttle temperature: {}", e);
            }
        } else {
            debug!("Thermal throttle path not found: {}", self.thermal_throttle_path.display());
        }
        
        Ok(())
    }
    
    async fn apply_safe_overclock(&self) -> Result<()> {
        debug!("⚡ Applying safe overclock settings for i9-13900HX");
        
        // For safety, we only apply very mild "overclocking" that's really just unlocking
        // full boost capabilities rather than actual overclocking
        
        // Enable Intel CPU turbo boost (if it exists)
        let turbo_path = PathBuf::from("/sys/devices/system/cpu/intel_pstate/no_turbo");
        if turbo_path.exists() {
            if let Err(e) = fs::write(&turbo_path, "0") { // 0 = enable turbo
                warn!("Failed to enable CPU turbo: {}", e);
            }
        }
        
        // Set CPU power limits to maximum safe values via msr-tools if installed
        // Don't consider it an error if these fail - they're optional optimizations
        let _ = Command::new("sh")
            .arg("-c")
            .arg("modprobe msr")
            .output();
            
        // Set PL1 and PL2 power limits (safe values for i9-13900HX)
        let _ = Command::new("wrmsr")
            .args(["-a", "0x610", "0x00DD8000"])
            .output();
            
        Ok(())
    }
    
    async fn apply_custom_setting(&self, key: &str, value: &str) -> Result<()> {
        debug!("⚙️ Applying custom setting {}={}", key, value);
        
        match key {
            "cpu_boost" => {
                // CPU Boost configuration
                let boost_path = PathBuf::from("/sys/devices/system/cpu/cpufreq/boost");
                if boost_path.exists() {
                    if let Err(e) = fs::write(&boost_path, value) {
                        warn!("Failed to set CPU boost: {}", e);
                    }
                }
            },
            "gpu_boost" => {
                // GPU boost settings would go here
                debug!("GPU boost setting applied in software only");
            },
            _ => {
                debug!("Unknown custom setting: {}", key);
            }
        }
        
        Ok(())
    }
    
    pub async fn start_optimization_loop(&mut self) -> Result<()> {
        info!("🔄 Starting hardware optimization loop");
        
        self.optimization_active = true;
        
        // Start components
        if let Some(fan_manager) = &mut self.fan_manager {
            fan_manager.start_intelligent_control().await?;
        }
        
        if let Some(rgb_manager) = &mut self.rgb_manager {
            rgb_manager.start_effect_engine().await?;
        }
        
        // Create a task for the optimization loop
        let hardware_controller = Arc::new(Mutex::new(self));
        let hardware_clone = hardware_controller.clone();
        
        task::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                let mut controller = hardware_clone.lock().unwrap();
                if !controller.optimization_active {
                    break;
                }
                
                // Update status
                if let Err(e) = controller.update_status().await {
                    warn!("Failed to update hardware status: {}", e);
                }
                
                // Check for thermal issues
                if let Some(status) = &controller.last_status {
                    if status.cpu_temp > 90.0 {
                        // Emergency thermal management
                        if let Err(e) = controller.handle_thermal_emergency().await {
                            error!("Thermal emergency handling failed: {}", e);
                        }
                    }
                }
                
                // Apply active profile periodically to ensure settings are maintained
                if let Err(e) = controller.apply_active_profile().await {
                    warn!("Failed to maintain hardware profile: {}", e);
                }
                
                // Update fan speeds based on current temperatures
                if let Some(fan_manager) = &mut controller.fan_manager {
                    if let Some(status) = &controller.last_status {
                        if let Err(e) = fan_manager.update_fan_speeds(status.cpu_temp, status.gpu_temp).await {
                            warn!("Failed to update fan speeds: {}", e);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn handle_thermal_emergency(&mut self) -> Result<()> {
        warn!("🔥 THERMAL EMERGENCY - Applying emergency cooling measures");
        
        // 1. Set fans to maximum
        if let Some(fan_manager) = &mut self.fan_manager {
            for (name, _) in fan_manager.devices.iter() {
                if let Err(e) = fan_manager.set_manual_fan_speed(name, 100).await {
                    warn!("Failed to set emergency fan speed for {}: {}", name, e);
                }
            }
        }
        
        // 2. Throttle CPU
        if let Err(e) = self.set_cpu_governor("powersave").await {
            warn!("Failed to set emergency CPU governor: {}", e);
        }
        
        // 3. Lower GPU power
        if let Err(e) = self.set_gpu_power_level("low").await {
            warn!("Failed to set emergency GPU power: {}", e);
        }
        
        // 4. Log the emergency
        if let Some(status) = &self.last_status {
            error!("Thermal emergency at CPU: {:.1}°C, GPU: {:?}°C", 
                status.cpu_temp, status.gpu_temp);
        }
        
        Ok(())
    }
    
    pub async fn update_status(&mut self) -> Result<()> {
        // Get CPU temperature
        let cpu_temp = self.get_cpu_temperature().await?;
        
        // Get GPU temperature
        let gpu_temp = self.get_gpu_temperature().await;
        
        // Get current settings
        let current_cpu_governor = self.get_current_governor().await?;
        let current_gpu_power_level = self.get_current_gpu_power().await?;
        
        // Get current fan profile
        let current_fan_profile = if let Some(fan_manager) = &self.fan_manager {
            if let Some(profile) = fan_manager.get_active_profile() {
                profile.name
            } else {
                "unknown".to_string()
            }
        } else {
            "none".to_string()
        };
        
        // Check if system is in battery mode
        let battery_mode = self.is_on_battery().await?;
        
        // Check if system is throttling
        let throttling = cpu_temp > 90.0;
        
        // Build status
        let status = HardwareStatus {
            cpu_temp,
            gpu_temp,
            current_power_mode: self.get_current_power_mode().await?,
            current_cpu_governor,
            current_gpu_power_level,
            current_fan_profile,
            active_cooling: true, // Assuming active cooling is always enabled
            battery_mode,
            throttling,
            optimization_active: self.optimization_active,
        };
        
        self.last_status = Some(status);
        Ok(())
    }
    
    async fn get_cpu_temperature(&self) -> Result<f32> {
        // Read from various possible temperature sources
        let thermal_paths = [
            "/sys/class/thermal/thermal_zone0/temp",
            "/sys/devices/platform/coretemp.0/temp1_input",
            "/sys/class/hwmon/hwmon0/temp1_input",
        ];
        
        for path in &thermal_paths {
            if let Ok(temp_str) = fs::read_to_string(path) {
                if let Ok(temp_millicelsius) = temp_str.trim().parse::<u32>() {
                    // Convert from millicelsius to celsius
                    return Ok(temp_millicelsius as f32 / 1000.0);
                }
            }
        }
        
        // If no temperature found, return a safe default
        Ok(50.0)
    }
    
    async fn get_gpu_temperature(&self) -> Option<f32> {
        // Try NVIDIA-specific method first
        let nvidia_output = Command::new("nvidia-smi")
            .args(["--query-gpu=temperature.gpu", "--format=csv,noheader,nounits"])
            .output();
            
        if let Ok(output) = nvidia_output {
            if output.status.success() {
                let temp_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if let Ok(temp) = temp_str.parse::<f32>() {
                    return Some(temp);
                }
            }
        }
        
        // Try AMD-specific method
        let amd_path = "/sys/class/drm/card0/device/hwmon/hwmon1/temp1_input";
        if let Ok(temp_str) = fs::read_to_string(amd_path) {
            if let Ok(temp_millicelsius) = temp_str.trim().parse::<u32>() {
                return Some(temp_millicelsius as f32 / 1000.0);
            }
        }
        
        None
    }
    
    async fn get_current_governor(&self) -> Result<String> {
        if self.cpu_governor_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.cpu_governor_path) {
                return Ok(content.trim().to_string());
            }
        }
        
        Ok("unknown".to_string())
    }
    
    async fn get_current_gpu_power(&self) -> Result<String> {
        if let Some(ref gpu_path) = self.gpu_power_path {
            if let Ok(content) = fs::read_to_string(gpu_path) {
                let content = content.trim().to_lowercase();
                if content.contains("high") || content == "1" {
                    return Ok("high".to_string());
                } else if content.contains("low") || content == "0" {
                    return Ok("low".to_string());
                } else {
                    return Ok("auto".to_string());
                }
            }
        }
        
        Ok("unknown".to_string())
    }
    
    async fn get_current_power_mode(&self) -> Result<PowerMode> {
        if let Some(ref profile_name) = self.active_profile {
            if let Some(profile) = self.profiles.get(profile_name) {
                return Ok(profile.power_mode.clone());
            }
        }
        
        // If no profile is active, guess based on CPU governor
        let governor = self.get_current_governor().await?;
        match governor.as_str() {
            "performance" => Ok(PowerMode::Performance),
            "powersave" => Ok(PowerMode::PowerSaver),
            _ => Ok(PowerMode::Balanced),
        }
    }
    
    async fn is_on_battery(&self) -> Result<bool> {
        // Check if system is running on battery
        let ac_online_path = PathBuf::from("/sys/class/power_supply/AC/online");
        if ac_online_path.exists() {
            if let Ok(content) = fs::read_to_string(&ac_online_path) {
                return Ok(content.trim() == "0");
            }
        }
        
        // Alternative path for some systems
        let ac_adapter_path = PathBuf::from("/sys/class/power_supply/ADP1/online");
        if ac_adapter_path.exists() {
            if let Ok(content) = fs::read_to_string(&ac_adapter_path) {
                return Ok(content.trim() == "0");
            }
        }
        
        // If we can't determine, assume AC power
        Ok(false)
    }
    
    pub async fn create_custom_profile(&mut self, name: String, profile: HardwareProfile) -> Result<()> {
        // Add or update profile
        self.profiles.insert(name.clone(), profile);
        
        // Save profiles
        self.save_profiles().await?;
        
        info!("✅ Created custom hardware profile: {}", name);
        Ok(())
    }
    
    pub async fn stop_optimization(&mut self) -> Result<()> {
        info!("⏹️ Stopping hardware optimization");
        
        self.optimization_active = false;
        
        Ok(())
    }
    
    pub fn get_profile_names(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }
    
    pub fn get_active_profile(&self) -> Option<HardwareProfile> {
        if let Some(ref profile_name) = self.active_profile {
            self.profiles.get(profile_name).cloned()
        } else {
            None
        }
    }
    
    pub fn get_status(&self) -> Option<HardwareStatus> {
        self.last_status.clone()
    }
    
    pub fn is_optimization_active(&self) -> bool {
        self.optimization_active
    }
    
    pub async fn apply_emergency_throttle(&mut self) -> Result<()> {
        self.handle_thermal_emergency().await
    }
}
