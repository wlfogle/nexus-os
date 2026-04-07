use std::process::Command;
use std::env;
use std::path::Path;
use serde_json::Value;
use tokio::time::{sleep, Duration};
use serde::{Deserialize, Serialize};

const OLLAMA_DEFAULT_HOST: &str = "http://127.0.0.1:11434";

// Get models path from environment variable with fallback
fn get_models_path() -> String {
    env::var("OLLAMA_MODELS")
        .or_else(|_| env::var("OLLAMA_MODELS_PATH"))
        .unwrap_or_else(|_| {
            // Fallback to default Ollama models directory
            let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{}/.ollama/models", home)
        })
}

#[derive(Debug, thiserror::Error)]
pub enum OllamaConfigError {
    #[error("Ollama is not installed or not found in PATH")]
    NotInstalled,
    #[error("External models directory not found: {0}")]
    ModelsDirectoryNotFound(String),
    #[error("Failed to start Ollama service: {0}")]
    StartupFailed(String),
    #[error("Failed to configure Ollama: {0}")]
    ConfigurationFailed(String),
    #[error("Ollama API error: {0}")]
    ApiError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub is_installed: bool,
    pub is_running: bool,
    pub models_path: String,
    pub host: String,
    pub available_models: Vec<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            is_installed: false,
            is_running: false,
            models_path: get_models_path(),
            host: OLLAMA_DEFAULT_HOST.to_string(),
            available_models: Vec::new(),
        }
    }
}

pub async fn check_ollama_installation() -> Result<bool, OllamaConfigError> {
    // Check if ollama is installed
    let output = Command::new("which")
        .arg("ollama")
        .output()
        .map_err(|_e| OllamaConfigError::NotInstalled)?;
    
    if !output.status.success() {
        return Err(OllamaConfigError::NotInstalled);
    }
    
    println!("✓ Ollama found in PATH");
    Ok(true)
}

pub async fn check_external_models() -> Result<bool, OllamaConfigError> {
    let external_models_path = get_models_path();
    let models_path = Path::new(&external_models_path);
    
    if !models_path.exists() {
        return Err(OllamaConfigError::ModelsDirectoryNotFound(external_models_path.clone()));
    }
    
    // Check if there are actual model files - try both manifests (new) and blobs (older)
    let manifests_path = models_path.join("manifests");
    let blobs_path = models_path.join("blobs");
    
    if !manifests_path.exists() && !blobs_path.exists() {
        return Err(OllamaConfigError::ModelsDirectoryNotFound(format!("{} (no manifests or blobs found)", external_models_path)));
    }
    
    println!("✓ External models directory found at {}", external_models_path);
    Ok(true)
}

pub async fn configure_ollama_models_path() -> Result<String, OllamaConfigError> {
    let external_models_path = get_models_path();
    
    // Set OLLAMA_MODELS environment variable to point to external models
    env::set_var("OLLAMA_MODELS", &external_models_path);
    
    // Verify the environment variable was set correctly
    match env::var("OLLAMA_MODELS") {
        Ok(value) if value == external_models_path => {
            println!("✓ Set OLLAMA_MODELS environment variable to {}", external_models_path);
            Ok(external_models_path)
        },
        Ok(value) => {
            Err(OllamaConfigError::ConfigurationFailed(
                format!("OLLAMA_MODELS was set to '{}' but expected '{}'", value, external_models_path)
            ))
        },
        Err(e) => {
            Err(OllamaConfigError::ConfigurationFailed(
                format!("Failed to verify OLLAMA_MODELS environment variable: {}", e)
            ))
        }
    }
}

pub async fn start_ollama_service() -> Result<(), OllamaConfigError> {
    // Check if Ollama is already running
    if is_ollama_running().await {
        println!("✓ Ollama is already running");
        return Ok(());
    }
    
    println!("🚀 Starting Ollama service...");
    
    // Start Ollama serve in background
    let external_models_path = get_models_path();
    let mut cmd = Command::new("ollama");
    cmd.arg("serve");
    cmd.env("OLLAMA_MODELS", &external_models_path);
    cmd.env("OLLAMA_HOST", OLLAMA_DEFAULT_HOST);
    
    let child = cmd.spawn()
        .map_err(|e| OllamaConfigError::StartupFailed(format!("Failed to spawn ollama serve: {}", e)))?;
    
    println!("✓ Ollama service started with PID {}", child.id());
    
    // Wait a moment for Ollama to start up
    sleep(Duration::from_secs(3)).await;
    
    // Verify it's running
    if !is_ollama_running().await {
        return Err(OllamaConfigError::StartupFailed("Ollama failed to start properly".to_string()));
    }
    
    println!("✓ Ollama service is running and ready");
    Ok(())
}

pub async fn is_ollama_running() -> bool {
    // Try to make a simple HTTP request to Ollama
    let client = reqwest::Client::new();
    match client.head(OLLAMA_DEFAULT_HOST).send().await {
        Ok(response) => {
            response.status().is_success()
        },
        Err(_) => false,
    }
}

pub async fn get_available_models_from_ollama() -> Result<Vec<String>, OllamaConfigError> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/tags", OLLAMA_DEFAULT_HOST);
    
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| OllamaConfigError::ApiError(format!("Failed to fetch models: {}", e)))?;
    
    if !response.status().is_success() {
        return Err(OllamaConfigError::ApiError(format!("API returned status: {}", response.status())));
    }
    
    let json: Value = response.json()
        .await
        .map_err(|e| OllamaConfigError::ApiError(format!("Failed to parse JSON: {}", e)))?;
    
    let models = json["models"].as_array()
        .ok_or_else(|| OllamaConfigError::ApiError("Invalid response format".to_string()))?;
    
    let model_names: Vec<String> = models
        .iter()
        .filter_map(|model| model["name"].as_str())
        .map(|name| name.to_string())
        .collect();
    
    println!("✓ Found {} available models", model_names.len());
    Ok(model_names)
}

pub async fn initialize_ollama_config() -> Result<OllamaConfig, OllamaConfigError> {
    let mut config = OllamaConfig::default();
    
    println!("🔧 Initializing Ollama configuration...");
    
    // Step 1: Check if Ollama is installed
    config.is_installed = check_ollama_installation().await.is_ok();
    if !config.is_installed {
        return Err(OllamaConfigError::NotInstalled);
    }
    
    // Step 2: Check external models directory
    check_external_models().await?;
    
    // Step 3: Configure models path
    config.models_path = configure_ollama_models_path().await?;
    
    // Step 4: Start Ollama service if not running
    start_ollama_service().await?;
    config.is_running = true;
    
    // Step 5: Get available models
    config.available_models = get_available_models_from_ollama().await.unwrap_or_default();
    
    println!("✅ Ollama configuration complete!");
    println!("   - Models path: {}", config.models_path);
    println!("   - Host: {}", config.host);
    println!("   - Available models: {}", config.available_models.len());
    
    Ok(config)
}

pub async fn ensure_ollama_configured() -> Result<(), OllamaConfigError> {
    // This is the main function to call at app startup
    match initialize_ollama_config().await {
        Ok(_) => {
            println!("🎉 Ollama is ready for AI operations!");
            Ok(())
        },
        Err(e) => {
            eprintln!("❌ Ollama configuration failed: {}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_external_models_check() {
        // This test assumes the external models directory exists
        let result = check_external_models().await;
        println!("External models check result: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_ollama_installation() {
        let result = check_ollama_installation().await;
        println!("Ollama installation check result: {:?}", result);
    }
}
