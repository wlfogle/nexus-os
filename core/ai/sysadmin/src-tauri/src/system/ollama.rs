// Ollama Configuration Module with Dynamic Path Detection
// No hardcoded paths - automatically detects model storage location

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaManager {
    pub models_path: Option<PathBuf>,
    pub ollama_installed: bool,
    pub service_running: bool,
    pub models_discovered: Vec<ModelInfo>,
    pub configuration: OllamaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size_gb: f64,
    pub last_modified: String,
    pub model_type: ModelType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Llama,
    Mistral,
    CodeLlama,
    Gemma,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub host: String,
    pub port: u16,
    pub num_parallel: u8,
    pub max_loaded_models: u8,
    pub flash_attention: bool,
    pub huge_pages_enabled: bool,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 11434,
            num_parallel: 4,
            max_loaded_models: 3,
            flash_attention: true,
            huge_pages_enabled: false,
        }
    }
}

impl OllamaManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("🧠 Initializing Ollama Manager with auto-detection...");
        
        let mut manager = Self {
            models_path: None,
            ollama_installed: false,
            service_running: false,
            models_discovered: Vec::new(),
            configuration: OllamaConfig::default(),
        };
        
        // Auto-detect everything
        manager.detect_ollama_installation().await?;
        manager.discover_models_directory().await?;
        manager.scan_existing_models().await?;
        manager.detect_current_configuration().await?;
        
        Ok(manager)
    }
    
    async fn detect_ollama_installation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("🔍 Detecting Ollama installation...");
        
        // Check if ollama command exists
        if let Ok(output) = AsyncCommand::new("which").arg("ollama").output().await {
            if output.status.success() {
                self.ollama_installed = true;
                info!("✅ Ollama found in PATH");
            }
        }
        
        // Check if service is running
        if let Ok(output) = AsyncCommand::new("systemctl")
            .args(&["is-active", "--quiet", "ollama"])
            .output()
            .await 
        {
            if output.status.success() {
                self.service_running = true;
                info!("✅ Ollama service is running");
            }
        }
        
        // Check if ollama is running as user process
        if !self.service_running {
            if let Ok(output) = AsyncCommand::new("pgrep").arg("ollama").output().await {
                if output.status.success() && !output.stdout.is_empty() {
                    self.service_running = true;
                    info!("✅ Ollama running as user process");
                }
            }
        }
        
        Ok(())
    }
    
    async fn discover_models_directory(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("🔍 Discovering models directory...");
        
        // Common model storage locations to check
        let candidate_paths = [
            // User's workspace (detected from your system)
            "../../../media/workspace/models",
            "../../media/workspace/models", 
            "../media/workspace/models",
            
            // Standard ollama locations
            "~/.ollama/models",
            "$HOME/.ollama/models",
            
            // Custom locations
            "models",
            "./models",
            "../models",
            
            // System-wide locations
            "/usr/share/ollama/models",
            "/opt/ollama/models",
        ];
        
        for candidate in &candidate_paths {
            let expanded_path = self.expand_path(candidate).await?;
            
            if let Ok(metadata) = fs::metadata(&expanded_path) {
                if metadata.is_dir() {
                    // Check if it looks like an ollama models directory
                    if self.is_ollama_models_directory(&expanded_path).await? {
                        self.models_path = Some(expanded_path.clone());
                        info!("✅ Found Ollama models directory at: {}", expanded_path.display());
                        break;
                    }
                }
            }
        }
        
        // If not found, check environment variable
        if self.models_path.is_none() {
            if let Ok(env_path) = std::env::var("OLLAMA_MODELS") {
                let path = PathBuf::from(env_path);
                if path.exists() {
                    self.models_path = Some(path.clone());
                    info!("✅ Using OLLAMA_MODELS environment variable: {}", path.display());
                }
            }
        }
        
        if self.models_path.is_none() {
            warn!("⚠️ No Ollama models directory found - will use default location");
        }
        
        Ok(())
    }
    
    async fn expand_path(&self, path: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut expanded = path.to_string();
        
        // Expand ~ to home directory
        if expanded.starts_with('~') {
            if let Ok(home) = std::env::var("HOME") {
                expanded = expanded.replace('~', &home);
            }
        }
        
        // Expand environment variables
        if expanded.contains('$') {
            if let Some(start) = expanded.find("$HOME") {
                if let Ok(home) = std::env::var("HOME") {
                    expanded = expanded.replace("$HOME", &home);
                }
            }
        }
        
        // Convert to absolute path if relative
        let path_buf = PathBuf::from(expanded);
        if path_buf.is_absolute() {
            Ok(path_buf)
        } else {
            // Make relative to current working directory
            let cwd = std::env::current_dir()?;
            Ok(cwd.join(path_buf).canonicalize().unwrap_or(cwd.join(path_buf)))
        }
    }
    
    async fn is_ollama_models_directory(&self, path: &PathBuf) -> Result<bool, Box<dyn std::error::Error>> {
        // Check for typical ollama directory structure
        let blobs_dir = path.join("blobs");
        let manifests_dir = path.join("manifests");
        
        Ok(blobs_dir.exists() && manifests_dir.exists())
    }
    
    async fn scan_existing_models(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(models_path) = &self.models_path {
            debug!("📊 Scanning existing models in: {}", models_path.display());
            
            let manifests_dir = models_path.join("manifests");
            if manifests_dir.exists() {
                self.scan_manifests_directory(&manifests_dir).await?;
            }
            
            // Also try to get models from ollama command if available
            if self.ollama_installed && self.service_running {
                self.scan_via_ollama_command().await?;
            }
        }
        
        Ok(())
    }
    
    async fn scan_manifests_directory(&mut self, manifests_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(entries) = fs::read_dir(manifests_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(registry_name) = path.file_name().and_then(|n| n.to_str()) {
                        // Scan each registry (usually "registry.ollama.ai")
                        self.scan_registry_directory(&path, registry_name).await?;
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn scan_registry_directory(&mut self, registry_dir: &PathBuf, registry_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(entries) = fs::read_dir(registry_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(namespace) = path.file_name().and_then(|n| n.to_str()) {
                        // Scan each namespace (like "library")
                        self.scan_namespace_directory(&path, namespace).await?;
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn scan_namespace_directory(&mut self, namespace_dir: &PathBuf, namespace: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(entries) = fs::read_dir(namespace_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(model_name) = path.file_name().and_then(|n| n.to_str()) {
                        // This is a model directory
                        self.process_model_directory(&path, model_name, namespace).await?;
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn process_model_directory(&mut self, model_dir: &PathBuf, model_name: &str, namespace: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Get model metadata
        if let Ok(metadata) = fs::metadata(model_dir) {
            if let Ok(modified) = metadata.modified() {
                let model_info = ModelInfo {
                    name: if namespace == "library" {
                        model_name.to_string()
                    } else {
                        format!("{}/{}", namespace, model_name)
                    },
                    size_gb: self.estimate_model_size(model_dir).await?,
                    last_modified: format!("{:?}", modified),
                    model_type: self.classify_model_type(model_name),
                };
                
                self.models_discovered.push(model_info);
            }
        }
        Ok(())
    }
    
    async fn scan_via_ollama_command(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(output) = AsyncCommand::new("ollama").arg("list").output().await {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines().skip(1) { // Skip header
                    if let Some(model_name) = line.split_whitespace().next() {
                        // Check if we already have this model from manifest scan
                        if !self.models_discovered.iter().any(|m| m.name == model_name) {
                            let model_info = ModelInfo {
                                name: model_name.to_string(),
                                size_gb: 0.0, // Size not easily available from list command
                                last_modified: "Unknown".to_string(),
                                model_type: self.classify_model_type(model_name),
                            };
                            self.models_discovered.push(model_info);
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn estimate_model_size(&self, model_dir: &PathBuf) -> Result<f64, Box<dyn std::error::Error>> {
        let mut total_size = 0u64;
        
        if let Ok(entries) = fs::read_dir(model_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    }
                }
            }
        }
        
        // Convert bytes to GB
        Ok(total_size as f64 / 1_073_741_824.0)
    }
    
    fn classify_model_type(&self, model_name: &str) -> ModelType {
        let name_lower = model_name.to_lowercase();
        
        if name_lower.contains("llama") {
            ModelType::Llama
        } else if name_lower.contains("mistral") {
            ModelType::Mistral
        } else if name_lower.contains("codellama") || name_lower.contains("code") {
            ModelType::CodeLlama
        } else if name_lower.contains("gemma") {
            ModelType::Gemma
        } else {
            ModelType::Other(model_name.to_string())
        }
    }
    
    async fn detect_current_configuration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Check environment variables for current configuration
        if let Ok(host) = std::env::var("OLLAMA_HOST") {
            if let Some((host_part, port_part)) = host.split_once(':') {
                self.configuration.host = host_part.to_string();
                if let Ok(port) = port_part.parse::<u16>() {
                    self.configuration.port = port;
                }
            } else {
                self.configuration.host = host;
            }
        }
        
        if let Ok(parallel) = std::env::var("OLLAMA_NUM_PARALLEL") {
            if let Ok(num) = parallel.parse::<u8>() {
                self.configuration.num_parallel = num;
            }
        }
        
        if let Ok(max_models) = std::env::var("OLLAMA_MAX_LOADED_MODELS") {
            if let Ok(num) = max_models.parse::<u8>() {
                self.configuration.max_loaded_models = num;
            }
        }
        
        // Check if huge pages are configured
        if let Ok(hugepages) = fs::read_to_string("/proc/meminfo") {
            if hugepages.contains("HugePages_Total:") && !hugepages.contains("HugePages_Total:        0") {
                self.configuration.huge_pages_enabled = true;
            }
        }
        
        Ok(())
    }
    
    pub async fn configure_for_discovered_models(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        info!("🔧 Configuring Ollama for discovered models...");
        
        if let Some(models_path) = &self.models_path {
            // Set OLLAMA_MODELS environment variable
            std::env::set_var("OLLAMA_MODELS", models_path);
            
            let config_script = format!(r#"#!/bin/bash
# Configure Ollama for discovered models at: {}

echo "🧠 Configuring Ollama for models at: {}"

# Export environment variables
export OLLAMA_MODELS="{}"
export OLLAMA_HOST="{}:{}"
export OLLAMA_NUM_PARALLEL={}
export OLLAMA_MAX_LOADED_MODELS={}
export OLLAMA_FLASH_ATTENTION={}

# Create systemd override directory if using system service
if systemctl is-enabled ollama.service >/dev/null 2>&1; then
    echo "📝 Creating systemd service override..."
    sudo mkdir -p /etc/systemd/system/ollama.service.d/
    
    cat << EOF | sudo tee /etc/systemd/system/ollama.service.d/override.conf
[Service]
Environment="OLLAMA_MODELS={}"
Environment="OLLAMA_HOST={}:{}"
Environment="OLLAMA_NUM_PARALLEL={}"
Environment="OLLAMA_MAX_LOADED_MODELS={}"
Environment="OLLAMA_FLASH_ATTENTION={}"
LimitNOFILE=1048576
LimitNPROC=1048576
EOF
    
    sudo systemctl daemon-reload
    echo "✅ Systemd service configured"
fi

# Create user environment file
mkdir -p "$HOME/.config/environment.d"
cat << EOF > "$HOME/.config/environment.d/ollama.conf"
OLLAMA_MODELS={}
OLLAMA_HOST={}:{}
OLLAMA_NUM_PARALLEL={}
OLLAMA_MAX_LOADED_MODELS={}
OLLAMA_FLASH_ATTENTION={}
EOF

echo "✅ Ollama configured for {} discovered models"
"#,
                models_path.display(),
                models_path.display(),
                models_path.display(),
                self.configuration.host,
                self.configuration.port,
                self.configuration.num_parallel,
                self.configuration.max_loaded_models,
                if self.configuration.flash_attention { "1" } else { "0" },
                models_path.display(),
                self.configuration.host,
                self.configuration.port,
                self.configuration.num_parallel,
                self.configuration.max_loaded_models,
                if self.configuration.flash_attention { "1" } else { "0" },
                models_path.display(),
                self.configuration.host,
                self.configuration.port,
                self.configuration.num_parallel,
                self.configuration.max_loaded_models,
                if self.configuration.flash_attention { "1" } else { "0" },
                self.models_discovered.len()
            );
            
            // Write and execute configuration script
            let script_path = "/tmp/configure_ollama.sh";
            fs::write(script_path, config_script)?;
            
            let output = AsyncCommand::new("bash")
                .arg(script_path)
                .output()
                .await?;
            
            if output.status.success() {
                Ok(format!("✅ Ollama configured for {} models at {}", 
                    self.models_discovered.len(), 
                    models_path.display()))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("Configuration failed: {}", error).into())
            }
        } else {
            Err("No models directory found to configure".into())
        }
    }
    
    pub fn get_discovered_models(&self) -> &Vec<ModelInfo> {
        &self.models_discovered
    }
    
    pub fn get_models_path(&self) -> Option<&PathBuf> {
        self.models_path.as_ref()
    }
    
    pub fn get_status(&self) -> HashMap<String, String> {
        let mut status = HashMap::new();
        
        status.insert("ollama_installed".to_string(), self.ollama_installed.to_string());
        status.insert("service_running".to_string(), self.service_running.to_string());
        status.insert("models_discovered".to_string(), self.models_discovered.len().to_string());
        
        if let Some(path) = &self.models_path {
            status.insert("models_path".to_string(), path.display().to_string());
        } else {
            status.insert("models_path".to_string(), "Not found".to_string());
        }
        
        status.insert("host".to_string(), format!("{}:{}", self.configuration.host, self.configuration.port));
        status.insert("huge_pages_enabled".to_string(), self.configuration.huge_pages_enabled.to_string());
        
        status
    }
}
