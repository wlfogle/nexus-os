use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};
use dirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub search_settings: SearchSettings,
    pub ui_settings: UISettings,
    pub indexing_settings: IndexingSettings,
    pub ai_settings: AISettings,
    pub cloud_settings: CloudSettings,
    pub paths: PathSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSettings {
    pub max_results: usize,
    pub fuzzy_threshold: f64,
    pub include_hidden_files: bool,
    pub search_content_by_default: bool,
    pub enable_real_time_search: bool,
    pub search_timeout_ms: u64,
    pub result_cache_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    pub theme: Theme,
    pub window_width: u32,
    pub window_height: u32,
    pub show_file_preview: bool,
    pub show_thumbnails: bool,
    pub font_size: u16,
    pub compact_mode: bool,
    pub show_hidden_results: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Auto,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingSettings {
    pub auto_index_on_startup: bool,
    pub watch_file_changes: bool,
    pub index_file_content: bool,
    pub max_file_size_mb: u64,
    pub excluded_extensions: Vec<String>,
    pub included_extensions: Vec<String>,
    pub indexing_threads: usize,
    pub index_update_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISettings {
    pub enable_ai_processing: bool,
    pub enable_smart_suggestions: bool,
    pub model_cache_path: PathBuf,
    pub max_suggestions: usize,
    pub suggestion_threshold: f32,
    pub enable_content_understanding: bool,
    pub use_local_models_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSettings {
    pub enabled_providers: Vec<CloudProvider>,
    pub sync_interval_minutes: u64,
    pub cache_cloud_metadata: bool,
    pub max_cloud_cache_size_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProvider {
    GoogleDrive,
    Dropbox,
    OneDrive,
    NextCloud,
    TeraBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSettings {
    pub search_paths: Vec<String>,
    pub excluded_paths: Vec<String>,
    pub database_path: PathBuf,
    pub cache_path: PathBuf,
    pub logs_path: PathBuf,
    pub models_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home"));
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| home_dir.join(".config"))
            .join("omniosearch");
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| home_dir.join(".local/share"))
            .join("omniosearch");
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| home_dir.join(".cache"))
            .join("omniosearch");

        Self {
            search_settings: SearchSettings {
                max_results: 1000,
                fuzzy_threshold: 0.6,
                include_hidden_files: false,
                search_content_by_default: true,
                enable_real_time_search: true,
                search_timeout_ms: 5000,
                result_cache_size: 10000,
            },
            ui_settings: UISettings {
                theme: Theme::Auto,
                window_width: 1200,
                window_height: 800,
                show_file_preview: true,
                show_thumbnails: true,
                font_size: 14,
                compact_mode: false,
                show_hidden_results: false,
            },
            indexing_settings: IndexingSettings {
                auto_index_on_startup: true,
                watch_file_changes: true,
                index_file_content: true,
                max_file_size_mb: 100,
                excluded_extensions: vec![
                    "tmp".to_string(),
                    "log".to_string(),
                    "cache".to_string(),
                    "bak".to_string(),
                    "swp".to_string(),
                ],
                included_extensions: vec![], // Empty means all extensions
                indexing_threads: num_cpus::get().max(2),
                index_update_interval_ms: 1000,
            },
            ai_settings: AISettings {
                enable_ai_processing: true,
                enable_smart_suggestions: true,
                model_cache_path: data_dir.join("models"),
                max_suggestions: 5,
                suggestion_threshold: 0.7,
                enable_content_understanding: true,
                use_local_models_only: true,
            },
            cloud_settings: CloudSettings {
                enabled_providers: vec![],
                sync_interval_minutes: 15,
                cache_cloud_metadata: true,
                max_cloud_cache_size_mb: 500,
            },
            paths: PathSettings {
                search_paths: vec![
                    home_dir.to_string_lossy().to_string(),
                    "/usr/share/applications".to_string(),
                    "/opt".to_string(),
                ],
                excluded_paths: vec![
                    format!("{}/.cache", home_dir.to_string_lossy()),
                    format!("{}/.local/share/Trash", home_dir.to_string_lossy()),
                    "/proc".to_string(),
                    "/sys".to_string(),
                    "/dev".to_string(),
                    "/tmp".to_string(),
                ],
                database_path: data_dir.join("omniosearch.db"),
                cache_path: cache_dir,
                logs_path: data_dir.join("logs"),
                models_path: data_dir.join("models"),
            },
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if config_path.exists() {
            info!("📄 Loading configuration from: {}", config_path.display());
            Self::from_file(&config_path).await
        } else {
            info!("⚙️ Using default configuration");
            let config = Self::default();
            config.save().await?;
            Ok(config)
        }
    }

    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = tokio::fs::read_to_string(path).await
            .context("Failed to read config file")?;
        
        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;
        
        debug!("✅ Configuration loaded from {}", path.display());
        Ok(config)
    }

    pub async fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        
        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        tokio::fs::write(&config_path, content).await
            .context("Failed to write config file")?;
        
        info!("💾 Configuration saved to: {}", config_path.display());
        Ok(())
    }

    fn config_file_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            anyhow::anyhow!("Could not determine home directory")
        })?;
        
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| home_dir.join(".config"))
            .join("omniosearch");
        
        Ok(config_dir.join("config.toml"))
    }

    // Getter methods for easy access
    pub fn database_path(&self) -> &Path {
        &self.paths.database_path
    }

    pub fn cache_path(&self) -> &Path {
        &self.paths.cache_path
    }

    pub fn logs_path(&self) -> &Path {
        &self.paths.logs_path
    }

    pub fn ai_models_path(&self) -> &Path {
        &self.paths.models_path
    }

    pub fn search_paths(&self) -> &[String] {
        &self.paths.search_paths
    }

    pub fn excluded_paths(&self) -> &[String] {
        &self.paths.excluded_paths
    }

    pub fn is_path_excluded(&self, path: &str) -> bool {
        let path_lower = path.to_lowercase();
        
        for excluded in &self.paths.excluded_paths {
            if path_lower.starts_with(&excluded.to_lowercase()) {
                return true;
            }
        }
        
        false
    }

    pub fn is_extension_excluded(&self, extension: &str) -> bool {
        if self.indexing_settings.excluded_extensions.is_empty() {
            return false;
        }
        
        let ext_lower = extension.to_lowercase();
        self.indexing_settings.excluded_extensions
            .iter()
            .any(|excluded| excluded.to_lowercase() == ext_lower)
    }

    pub fn should_index_file(&self, path: &str, size: u64) -> bool {
        // Check if path is excluded
        if self.is_path_excluded(path) {
            return false;
        }
        
        // Check file size limit
        let max_size = self.indexing_settings.max_file_size_mb * 1024 * 1024;
        if size > max_size {
            return false;
        }
        
        // Check extension
        if let Some(extension) = Path::new(path).extension() {
            if let Some(ext_str) = extension.to_str() {
                if self.is_extension_excluded(ext_str) {
                    return false;
                }
                
                // If included_extensions is not empty, only index those extensions
                if !self.indexing_settings.included_extensions.is_empty() {
                    let ext_lower = ext_str.to_lowercase();
                    return self.indexing_settings.included_extensions
                        .iter()
                        .any(|included| included.to_lowercase() == ext_lower);
                }
            }
        }
        
        true
    }

    pub fn ensure_directories(&self) -> Result<()> {
        let dirs_to_create = [
            &self.paths.database_path.parent().unwrap(),
            &self.paths.cache_path,
            &self.paths.logs_path,
            &self.paths.models_path,
        ];

        for dir in &dirs_to_create {
            if !dir.exists() {
                std::fs::create_dir_all(dir)
                    .context(format!("Failed to create directory: {}", dir.display()))?;
                debug!("📁 Created directory: {}", dir.display());
            }
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        // Validate search paths exist
        for path in &self.paths.search_paths {
            if !Path::new(path).exists() {
                warn!("⚠️ Search path does not exist: {}", path);
            }
        }

        // Validate settings ranges
        if self.search_settings.fuzzy_threshold < 0.0 || self.search_settings.fuzzy_threshold > 1.0 {
            return Err(anyhow::anyhow!("fuzzy_threshold must be between 0.0 and 1.0"));
        }

        if self.search_settings.max_results == 0 {
            return Err(anyhow::anyhow!("max_results must be greater than 0"));
        }

        if self.indexing_settings.max_file_size_mb == 0 {
            return Err(anyhow::anyhow!("max_file_size_mb must be greater than 0"));
        }

        if self.ai_settings.max_suggestions == 0 {
            return Err(anyhow::anyhow!("max_suggestions must be greater than 0"));
        }

        if self.ai_settings.suggestion_threshold < 0.0 || self.ai_settings.suggestion_threshold > 1.0 {
            return Err(anyhow::anyhow!("suggestion_threshold must be between 0.0 and 1.0"));
        }

        debug!("✅ Configuration validation passed");
        Ok(())
    }

    pub fn add_search_path(&mut self, path: String) {
        if !self.paths.search_paths.contains(&path) {
            self.paths.search_paths.push(path);
        }
    }

    pub fn remove_search_path(&mut self, path: &str) {
        self.paths.search_paths.retain(|p| p != path);
    }

    pub fn add_excluded_path(&mut self, path: String) {
        if !self.paths.excluded_paths.contains(&path) {
            self.paths.excluded_paths.push(path);
        }
    }

    pub fn remove_excluded_path(&mut self, path: &str) {
        self.paths.excluded_paths.retain(|p| p != path);
    }

    pub fn toggle_cloud_provider(&mut self, provider: CloudProvider) {
        if let Some(pos) = self.cloud_settings.enabled_providers
            .iter()
            .position(|p| std::mem::discriminant(p) == std::mem::discriminant(&provider)) {
            self.cloud_settings.enabled_providers.remove(pos);
        } else {
            self.cloud_settings.enabled_providers.push(provider);
        }
    }
}
