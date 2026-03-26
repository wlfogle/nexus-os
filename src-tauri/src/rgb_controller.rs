// RGB Controller - OriginPC/Clevo compatible RGB management
use anyhow::{Result, Context, anyhow};
use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::Duration;

/// Device path for the Clevo RGB keyboard
const CLEVO_RGB_DEVICE: &str = "/dev/hidraw0";

/// Manages RGB lighting for Clevo/OriginPC keyboards
pub struct RGBManager {
    /// Path to the RGB device (usually /dev/hidraw0)
    device_path: String,
    /// Whether the device is accessible and compatible
    is_compatible: bool,
    /// Current RGB state (on/off)
    is_enabled: bool,
    /// Current brightness level (0-100)
    brightness: u8,
    /// Current RGB color values
    color: (u8, u8, u8),
}

impl RGBManager {
    /// Creates a new RGB manager for OriginPC systems
    pub async fn new_originpc() -> Result<Self> {
        let device_path = CLEVO_RGB_DEVICE.to_string();
        let mut manager = Self {
            device_path,
            is_compatible: false,
            is_enabled: true,
            brightness: 100,
            color: (255, 0, 0), // Default to red
        };
        
        // Check if device is accessible
        manager.is_compatible = manager.test_device_access().await.is_ok();
        
        Ok(manager)
    }
    
    /// Tests if the RGB device is accessible
    async fn test_device_access(&self) -> Result<()> {
        // Try to open the device for reading to test access
        OpenOptions::new()
            .read(true)
            .open(&self.device_path)
            .context(format!("Cannot access RGB device at {}", self.device_path))?;
            
        Ok(())
    }
    
    /// Sends raw command bytes to the RGB device
    async fn send_command(&self, data: &[u8]) -> Result<()> {
        // Early return if device is not compatible
        if !self.is_compatible {
            return Err(anyhow!("RGB device is not compatible or accessible"));
        }
        
        // Open device for writing
        let mut device = OpenOptions::new()
            .write(true)
            .open(&self.device_path)
            .context(format!("Failed to open RGB device at {}", self.device_path))?;
        
        // Write data to device
        device.write_all(data)
            .context("Failed to write data to RGB device")?;
            
        // Flush to ensure command is sent
        device.flush()
            .context("Failed to flush data to RGB device")?;
            
        Ok(())
    }
    
    /// Clears all RGB effects (turns off RGB lighting)
    pub async fn clear_effects(&mut self) -> Result<()> {
        // This is the minimal clear command from the Python script
        let data = [0xCC, 0x01, 0x53, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        self.send_command(&data).await?;
        self.is_enabled = false;
        Ok(())
    }
    
    /// Sets a solid color for the entire keyboard
    pub async fn set_color(&mut self, r: u8, g: u8, b: u8) -> Result<()> {
        // Store the new color
        self.color = (r, g, b);
        
        // Only apply if RGB is enabled
        if self.is_enabled {
            // Command format based on reverse engineering
            // Exact structure may need adjustment based on testing
            let mut data = [0xCC, 0x01, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            
            // Set RGB values
            data[3] = r;
            data[4] = g;
            data[5] = b;
            
            // Set brightness
            data[6] = self.brightness;
            
            self.send_command(&data).await?;
        }
        
        Ok(())
    }
    
    /// Toggles RGB effects on/off
    pub async fn toggle_effects(&mut self) -> Result<()> {
        if self.is_enabled {
            self.clear_effects().await?
        } else {
            self.is_enabled = true;
            let (r, g, b) = self.color;
            self.set_color(r, g, b).await?
        }
        
        Ok(())
    }
    
    /// Sets the brightness level (0-100)
    pub async fn set_brightness(&mut self, brightness: u8) -> Result<()> {
        // Clamp brightness to 0-100
        self.brightness = brightness.min(100);
        
        // Apply the change if RGB is enabled
        if self.is_enabled {
            let (r, g, b) = self.color;
            self.set_color(r, g, b).await?
        }
        
        Ok(())
    }
    
    /// Initializes the RGB system and starts the default effect
    pub async fn start_effect_engine(&mut self) -> Result<()> {
        // First clear any existing effects
        self.clear_effects().await?;
        
        // Short delay to ensure the clear command is processed
        thread::sleep(Duration::from_millis(100));
        
        // Enable RGB and set to default color
        self.is_enabled = true;
        let (r, g, b) = self.color;
        self.set_color(r, g, b).await?;
        
        Ok(())
    }
    
    /// Checks if RGB is currently enabled
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }
    
    /// Gets the current RGB color
    pub fn get_color(&self) -> (u8, u8, u8) {
        self.color
    }
    
    /// Gets the current brightness
    pub fn get_brightness(&self) -> u8 {
        self.brightness
    }
}
