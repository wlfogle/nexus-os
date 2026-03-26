use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid;
use crate::ai::AIConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    pub temp_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionConfig {
    pub ocr_engine: String,
    pub vision_model: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ai: AIConfig,
    pub terminal: TerminalConfig,
    pub appearance: AppearanceConfig,
    pub shortcuts: ShortcutsConfig,
    pub paths: PathsConfig,
    pub vision: VisionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub default_shell: Option<String>,
    pub font_family: String,
    pub font_size: u16,
    pub cursor_blink: bool,
    pub cursor_style: String,
    pub scroll_back: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub theme: String,
    pub opacity: f32,
    pub blur_background: bool,
    pub show_tabs: bool,
    pub show_title_bar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutsConfig {
    pub new_terminal: String,
    pub close_terminal: String,
    pub copy: String,
    pub paste: String,
    pub find: String,
    pub ai_chat: String,
    pub command_palette: String,
}

impl Default for PathsConfig {
    fn default() -> Self {
        let app_dir = dirs::data_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap().join(".data"))
            .join("nexus-terminal");
        
        Self {
            temp_dir: app_dir.join("temp"),
            cache_dir: app_dir.join("cache"),
            data_dir: app_dir.join("data"),
            log_dir: app_dir.join("logs"),
        }
    }
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            ocr_engine: std::env::var("OCR_ENGINE").unwrap_or_else(|_| "tesseract".to_string()),
            vision_model: std::env::var("VISION_MODEL").unwrap_or_else(|_| "llava".to_string()),
            enabled: std::env::var("VISION_ENABLED")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(true), // Enabled by default with full implementation
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai: AIConfig::default(),
            terminal: TerminalConfig::default(),
            appearance: AppearanceConfig::default(),
            shortcuts: ShortcutsConfig::default(),
            paths: PathsConfig::default(),
            vision: VisionConfig::default(),
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            default_shell: None,
            font_family: "JetBrains Mono".to_string(),
            font_size: 14,
            cursor_blink: true,
            cursor_style: "block".to_string(),
            scroll_back: 10000,
        }
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            opacity: 0.95,
            blur_background: true,
            show_tabs: true,
            show_title_bar: false,
        }
    }
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            new_terminal: "Ctrl+Shift+T".to_string(),
            close_terminal: "Ctrl+Shift+W".to_string(),
            copy: "Ctrl+Shift+C".to_string(),
            paste: "Ctrl+Shift+V".to_string(),
            find: "Ctrl+Shift+F".to_string(),
            ai_chat: "Ctrl+Shift+A".to_string(),
            command_palette: "Ctrl+Shift+P".to_string(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("nexus-terminal");
        
        std::fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
        
        Ok(config_dir.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            
            let config: AppConfig = toml::from_str(&content)
                .context("Failed to parse config file")?;
            
            Ok(config)
        } else {
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        std::fs::write(&config_path, content)
            .context("Failed to write config file")?;
        
        Ok(())
    }
    
    /// Ensure all configured directories exist
    pub fn ensure_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.paths.temp_dir)
            .context("Failed to create temp directory")?;
        std::fs::create_dir_all(&self.paths.cache_dir)
            .context("Failed to create cache directory")?;
        std::fs::create_dir_all(&self.paths.data_dir)
            .context("Failed to create data directory")?;
        std::fs::create_dir_all(&self.paths.log_dir)
            .context("Failed to create log directory")?;
        Ok(())
    }
    
    /// Get a temporary file path with the given name
    pub fn temp_file_path(&self, name: &str) -> PathBuf {
        self.paths.temp_dir.join(name)
    }
    
    /// Get a cache file path with the given name
    pub fn cache_file_path(&self, name: &str) -> PathBuf {
        self.paths.cache_dir.join(name)
    }
    
    /// Get the AI models cache directory
    pub fn ai_models_cache_dir(&self) -> PathBuf {
        self.cache_file_path("ai_models")
    }
    
    /// Get the screenshots temp directory
    pub fn screenshots_temp_dir(&self) -> PathBuf {
        self.temp_file_path("screenshots")
    }
    
    /// Create a temporary screenshot file path
    pub fn create_temp_screenshot_path(&self) -> Result<PathBuf> {
        let temp_dir = self.screenshots_temp_dir();
        std::fs::create_dir_all(&temp_dir).context("Failed to create screenshots temp dir")?;
        let filename = format!("screenshot_{}.png", uuid::Uuid::new_v4());
        Ok(temp_dir.join(filename))
    }
}
