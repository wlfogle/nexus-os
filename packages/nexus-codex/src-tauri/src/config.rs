use anyhow::Result;
use std::path::PathBuf;

use crate::types::Config;

/// Returns the path to the persisted config file.
/// Stored at $HOME/.config/nexus-codex/config.json
fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    PathBuf::from(home).join(".config").join("nexus-codex").join("config.json")
}

/// Load config from disk, falling back to defaults if not found or invalid.
pub fn load_config() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let data = std::fs::read_to_string(&path)?;
    let cfg: Config = serde_json::from_str(&data)?;
    Ok(cfg)
}

/// Persist config to disk.
pub fn save_config(config: &Config) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, data)?;
    Ok(())
}
