use crate::errors::AppError;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::fs;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ai: AIConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub performance: PerformanceConfig,
    pub ui: UIConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub default_model: String,
    pub api_timeout: u64,
    pub max_retries: u32,
    pub cache_ttl: u64,
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub base_url: String,
    pub api_key_env: String,
    pub models: Vec<String>,
    pub rate_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub max_connections: u32,
    pub backup_interval: u64,
    pub auto_vacuum: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_rotation: bool,
    pub max_file_size: u64,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_https: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub cors_origins: Vec<String>,
    pub rate_limiting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub worker_threads: u32,
    pub max_memory_mb: u64,
    pub gc_interval: u64,
    pub metrics_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: String,
    pub font_size: u32,
    pub auto_save: bool,
    pub keybindings: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai: AIConfig {
                default_model: "gpt-3.5-turbo".to_string(),
                api_timeout: 30,
                max_retries: 3,
                cache_ttl: 3600,
                providers: HashMap::new(),
            },
            database: DatabaseConfig {
                path: "./data/app.db".to_string(),
                max_connections: 10,
                backup_interval: 86400, // 24 hours
                auto_vacuum: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_rotation: true,
                max_file_size: 100 * 1024 * 1024, // 100MB
                retention_days: 30,
            },
            security: SecurityConfig {
                enable_https: false,
                cert_path: None,
                key_path: None,
                cors_origins: vec!["http://localhost:3000".to_string()],
                rate_limiting: true,
            },
            performance: PerformanceConfig {
                worker_threads: num_cpus::get() as u32,
                max_memory_mb: 1024,
                gc_interval: 300, // 5 minutes
                metrics_enabled: true,
            },
            ui: UIConfig {
                theme: "dark".to_string(),
                font_size: 14,
                auto_save: true,
                keybindings: HashMap::new(),
            },
        }
    }
}

pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
    _watcher: Option<RecommendedWatcher>,
    reload_tx: Option<mpsc::UnboundedSender<()>>,
}

impl ConfigManager {
    pub async fn new(config_path: impl AsRef<Path>) -> Result<Self, AppError> {
        let config_path = config_path.as_ref().to_path_buf();
        let config = Arc::new(RwLock::new(Self::load_config(&config_path).await?));

        Ok(Self {
            config,
            config_path,
            _watcher: None,
            reload_tx: None,
        })
    }

    pub async fn with_hot_reload(config_path: impl AsRef<Path>) -> Result<Self, AppError> {
        let config_path = config_path.as_ref().to_path_buf();
        let config = Arc::new(RwLock::new(Self::load_config(&config_path).await?));
        
        let (reload_tx, mut reload_rx) = mpsc::unbounded_channel();
        let config_clone = Arc::clone(&config);
        let path_clone = config_path.clone();

        // Spawn hot-reload task
        tokio::spawn(async move {
            while let Some(_) = reload_rx.recv().await {
                match Self::load_config(&path_clone).await {
                    Ok(new_config) => {
                        let mut config_guard = config_clone.write().unwrap();
                        *config_guard = new_config;
                        info!("Configuration hot-reloaded successfully");
                    }
                    Err(e) => {
                        error!("Failed to hot-reload configuration: {}", e);
                    }
                }
            }
        });

        // Set up file watcher
        let watcher_tx = reload_tx.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            match res {
                Ok(_) => {
                    if let Err(e) = watcher_tx.send(()) {
                        error!("Failed to send reload signal: {}", e);
                    }
                }
                Err(e) => error!("File watcher error: {:?}", e),
            }
        }).map_err(|e| AppError::Internal(format!("Failed to create watcher: {}", e)))?;

        watcher.watch(&config_path, RecursiveMode::NonRecursive)
            .map_err(|e| AppError::Internal(format!("Failed to watch config file: {}", e)))?;

        Ok(Self {
            config,
            config_path,
            _watcher: Some(watcher),
            reload_tx: Some(reload_tx),
        })
    }

    async fn load_config(path: &Path) -> Result<AppConfig, AppError> {
        if !path.exists() {
            let default_config = AppConfig::default();
            Self::save_config(path, &default_config).await?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(path).await
            .map_err(|e| AppError::FileSystem(format!("Failed to read config: {}", e)))?;

        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| AppError::Validation(format!("Invalid config format: {}", e)))?;

        Ok(config)
    }

    async fn save_config(path: &Path, config: &AppConfig) -> Result<(), AppError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| AppError::FileSystem(format!("Failed to create config dir: {}", e)))?;
        }

        let content = toml::to_string_pretty(config)
            .map_err(|e| AppError::Internal(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, content).await
            .map_err(|e| AppError::FileSystem(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    pub fn get(&self) -> AppConfig {
        self.config.read().unwrap().clone()
    }

    pub fn trigger_reload(&self) -> Result<(), AppError> {
        if let Some(ref tx) = self.reload_tx {
            tx.send(()).map_err(|_| AppError::Internal("Failed to trigger config reload".to_string()))?;
        }
        Ok(())
    }

    pub async fn update<F>(&self, updater: F) -> Result<(), AppError>
    where
        F: FnOnce(&mut AppConfig),
    {
        {
            let mut config = self.config.write().unwrap();
            updater(&mut config);
        }

        let config = self.get();
        Self::save_config(&self.config_path, &config).await?;
        info!("Configuration updated and saved");
        Ok(())
    }

    pub fn get_api_key(&self, provider: &str) -> Option<String> {
        let config = self.get();
        if let Some(provider_config) = config.ai.providers.get(provider) {
            std::env::var(&provider_config.api_key_env).ok()
        } else {
            None
        }
    }

    pub async fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let config = self.get();

        // Validate database path
        if let Some(parent) = Path::new(&config.database.path).parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent).await {
                    errors.push(format!("Cannot create database directory: {}", e));
                }
            }
        }

        // Validate AI providers
        for (name, provider) in &config.ai.providers {
            if std::env::var(&provider.api_key_env).is_err() {
                warn!("API key not found for provider '{}': {}", name, provider.api_key_env);
            }
        }

        // Validate security settings
        if config.security.enable_https {
            if config.security.cert_path.is_none() || config.security.key_path.is_none() {
                errors.push("HTTPS enabled but certificate paths not configured".to_string());
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_creation() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let manager = ConfigManager::new(&config_path).await.unwrap();
        let config = manager.get();

        assert_eq!(config.ai.default_model, "gpt-3.5-turbo");
        assert!(config_path.exists());
    }

    #[tokio::test]
    async fn test_config_update() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let manager = ConfigManager::new(&config_path).await.unwrap();
        
        manager.update(|config| {
            config.ai.default_model = "gpt-4".to_string();
        }).await.unwrap();

        let updated_config = manager.get();
        assert_eq!(updated_config.ai.default_model, "gpt-4");
    }
}
