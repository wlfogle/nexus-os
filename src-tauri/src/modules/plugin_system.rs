use crate::errors::AppError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::fs;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub entry_point: String,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
    pub ai_providers: Vec<AIProviderConfig>,
    pub language_analyzers: Vec<LanguageAnalyzerConfig>,
    pub integrations: Vec<IntegrationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderConfig {
    pub name: String,
    pub base_url: String,
    pub api_version: String,
    pub supported_models: Vec<String>,
    pub rate_limits: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageAnalyzerConfig {
    pub language: String,
    pub file_extensions: Vec<String>,
    pub analyzer_type: String, // "eslint", "clippy", "custom", etc.
    pub config_path: Option<String>,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub service: String, // "github", "gitlab", "vscode", etc.
    pub auth_type: String, // "oauth", "token", "basic"
    pub endpoints: HashMap<String, String>,
    pub scopes: Vec<String>,
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn initialize(&mut self, config: &PluginManifest) -> Result<(), AppError>;
    async fn execute(&self, operation: &str, params: &serde_json::Value) -> Result<serde_json::Value, AppError>;
    async fn cleanup(&self) -> Result<(), AppError>;
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn process_request(&self, model: &str, prompt: &str, config: &serde_json::Value) -> Result<String, AppError>;
    fn supported_models(&self) -> Vec<String>;
    fn rate_limit(&self, model: &str) -> Option<u32>;
}

#[async_trait]
pub trait LanguageAnalyzer: Send + Sync {
    async fn analyze(&self, file_path: &Path, content: &str) -> Result<AnalysisResult, AppError>;
    fn supported_languages(&self) -> Vec<String>;
    async fn fix_issues(&self, file_path: &Path, issues: &[Issue]) -> Result<String, AppError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub issues: Vec<Issue>,
    pub metrics: HashMap<String, serde_json::Value>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub severity: String, // "error", "warning", "info"
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub rule: Option<String>,
    pub fix_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub title: String,
    pub description: String,
    pub line_start: u32,
    pub line_end: u32,
    pub replacement: Option<String>,
}

pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
    ai_providers: Arc<RwLock<HashMap<String, Box<dyn AIProvider>>>>,
    language_analyzers: Arc<RwLock<HashMap<String, Box<dyn LanguageAnalyzer>>>>,
    plugin_directory: PathBuf,
    loaded_manifests: Arc<RwLock<HashMap<String, PluginManifest>>>,
}

impl PluginManager {
    pub fn new(plugin_directory: impl AsRef<Path>) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            ai_providers: Arc::new(RwLock::new(HashMap::new())),
            language_analyzers: Arc::new(RwLock::new(HashMap::new())),
            plugin_directory: plugin_directory.as_ref().to_path_buf(),
            loaded_manifests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn discover_plugins(&self) -> Result<Vec<PluginManifest>, AppError> {
        let mut manifests = Vec::new();
        
        if !self.plugin_directory.exists() {
            fs::create_dir_all(&self.plugin_directory).await
                .map_err(|e| AppError::FileSystem(format!("Failed to create plugin directory: {}", e)))?;
            return Ok(manifests);
        }

        let mut entries = fs::read_dir(&self.plugin_directory).await
            .map_err(|e| AppError::FileSystem(format!("Failed to read plugin directory: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AppError::FileSystem(format!("Failed to read directory entry: {}", e)))? {
            
            if entry.file_type().await.map_err(|e| AppError::FileSystem(format!("Failed to get file type: {}", e)))?.is_dir() {
                let manifest_path = entry.path().join("plugin.toml");
                if manifest_path.exists() {
                    match self.load_manifest(&manifest_path).await {
                        Ok(manifest) => {
                            info!("Discovered plugin: {} v{}", manifest.name, manifest.version);
                            manifests.push(manifest);
                        }
                        Err(e) => {
                            warn!("Failed to load plugin manifest from {:?}: {}", manifest_path, e);
                        }
                    }
                }
            }
        }

        Ok(manifests)
    }

    async fn load_manifest(&self, path: &Path) -> Result<PluginManifest, AppError> {
        let content = fs::read_to_string(path).await
            .map_err(|e| AppError::FileSystem(format!("Failed to read manifest: {}", e)))?;

        let manifest: PluginManifest = toml::from_str(&content)
            .map_err(|e| AppError::Validation(format!("Invalid manifest format: {}", e)))?;

        Ok(manifest)
    }

    pub async fn load_plugin(&self, manifest: PluginManifest) -> Result<(), AppError> {
        // In a real implementation, this would use dynamic loading (libloading crate)
        // For now, we'll simulate plugin loading
        info!("Loading plugin: {} v{}", manifest.name, manifest.version);

        // Store the manifest
        {
            let mut manifests = self.loaded_manifests.write().unwrap();
            manifests.insert(manifest.name.clone(), manifest.clone());
        }

        // Register AI providers
        for provider_config in &manifest.ai_providers {
            info!("Registering AI provider: {}", provider_config.name);
            // In real implementation, instantiate the provider from the plugin
            // let provider = plugin.create_ai_provider(&provider_config)?;
            // self.register_ai_provider(provider_config.name.clone(), provider);
        }

        // Register language analyzers
        for analyzer_config in &manifest.language_analyzers {
            info!("Registering language analyzer for: {}", analyzer_config.language);
            // In real implementation, instantiate the analyzer from the plugin
            // let analyzer = plugin.create_language_analyzer(&analyzer_config)?;
            // self.register_language_analyzer(analyzer_config.language.clone(), analyzer);
        }

        Ok(())
    }

    pub fn register_ai_provider(&self, name: String, provider: Box<dyn AIProvider>) {
        let mut providers = self.ai_providers.write().unwrap();
        providers.insert(name, provider);
    }

    pub fn register_language_analyzer(&self, language: String, analyzer: Box<dyn LanguageAnalyzer>) {
        let mut analyzers = self.language_analyzers.write().unwrap();
        analyzers.insert(language, analyzer);
    }

    pub async fn analyze_code(&self, language: &str, file_path: &Path, content: &str) -> Result<AnalysisResult, AppError> {
        let analyzers = self.language_analyzers.read().unwrap();
        
        if let Some(analyzer) = analyzers.get(language) {
            analyzer.analyze(file_path, content).await
        } else {
            // Fallback to basic analysis
            Ok(AnalysisResult {
                issues: Vec::new(),
                metrics: HashMap::new(),
                suggestions: Vec::new(),
            })
        }
    }

    pub async fn process_ai_request(&self, provider: &str, model: &str, prompt: &str, config: &serde_json::Value) -> Result<String, AppError> {
        let providers = self.ai_providers.read().unwrap();
        
        if let Some(ai_provider) = providers.get(provider) {
            ai_provider.process_request(model, prompt, config).await
        } else {
            Err(AppError::Validation(format!("AI provider '{}' not found", provider)))
        }
    }

    pub fn list_plugins(&self) -> Vec<PluginManifest> {
        let manifests = self.loaded_manifests.read().unwrap();
        manifests.values().cloned().collect()
    }

    pub fn list_ai_providers(&self) -> Vec<String> {
        let providers = self.ai_providers.read().unwrap();
        providers.keys().cloned().collect()
    }

    pub fn list_language_analyzers(&self) -> Vec<String> {
        let analyzers = self.language_analyzers.read().unwrap();
        analyzers.keys().cloned().collect()
    }

    pub async fn create_plugin_template(&self, name: &str, author: &str) -> Result<PathBuf, AppError> {
        let plugin_dir = self.plugin_directory.join(name);
        fs::create_dir_all(&plugin_dir).await
            .map_err(|e| AppError::FileSystem(format!("Failed to create plugin directory: {}", e)))?;

        let manifest = PluginManifest {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: format!("A plugin for {}", name),
            author: author.to_string(),
            entry_point: "src/lib.rs".to_string(),
            dependencies: Vec::new(),
            permissions: Vec::new(),
            ai_providers: Vec::new(),
            language_analyzers: Vec::new(),
            integrations: Vec::new(),
        };

        let manifest_content = toml::to_string_pretty(&manifest)
            .map_err(|e| AppError::Internal(format!("Failed to serialize manifest: {}", e)))?;

        let manifest_path = plugin_dir.join("plugin.toml");
        fs::write(&manifest_path, manifest_content).await
            .map_err(|e| AppError::FileSystem(format!("Failed to write manifest: {}", e)))?;

        // Create basic plugin structure
        let src_dir = plugin_dir.join("src");
        fs::create_dir_all(&src_dir).await
            .map_err(|e| AppError::FileSystem(format!("Failed to create src directory: {}", e)))?;

        let lib_rs_content = format!(r#"
use async_trait::async_trait;
use serde_json;
use plugin_system::{{Plugin, AppError, PluginManifest}};

pub struct {}Plugin;

#[async_trait]
impl Plugin for {}Plugin {{
    fn name(&self) -> &str {{
        "{}"
    }}

    fn version(&self) -> &str {{
        "0.1.0"
    }}

    async fn initialize(&mut self, config: &PluginManifest) -> Result<(), AppError> {{
        // Initialize plugin here
        Ok(())
    }}

    async fn execute(&self, operation: &str, params: &serde_json::Value) -> Result<serde_json::Value, AppError> {{
        match operation {{
            "hello" => Ok(serde_json::json!({{"message": "Hello from {}!"}})),
            _ => Err(AppError::Validation(format!("Unknown operation: {{}}", operation))),
        }}
    }}

    async fn cleanup(&self) -> Result<(), AppError> {{
        // Cleanup plugin resources
        Ok(())
    }}
}}
"#, name, name, name, name);

        let lib_rs_path = src_dir.join("lib.rs");
        fs::write(&lib_rs_path, lib_rs_content).await
            .map_err(|e| AppError::FileSystem(format!("Failed to write lib.rs: {}", e)))?;

        info!("Created plugin template at: {:?}", plugin_dir);
        Ok(plugin_dir)
    }
}

// Built-in ESLint analyzer example
pub struct ESLintAnalyzer {
    config_path: Option<PathBuf>,
}

impl ESLintAnalyzer {
    pub fn new(config_path: Option<PathBuf>) -> Self {
        Self { config_path }
    }
}

#[async_trait]
impl LanguageAnalyzer for ESLintAnalyzer {
    async fn analyze(&self, _file_path: &Path, _content: &str) -> Result<AnalysisResult, AppError> {
        // In real implementation, this would call ESLint
        // For now, return mock data
        let issues = vec![
            Issue {
                severity: "warning".to_string(),
                message: "Missing semicolon".to_string(),
                line: 1,
                column: 10,
                rule: Some("semi".to_string()),
                fix_available: true,
            }
        ];

        Ok(AnalysisResult {
            issues,
            metrics: HashMap::new(),
            suggestions: Vec::new(),
        })
    }

    fn supported_languages(&self) -> Vec<String> {
        vec!["javascript".to_string(), "typescript".to_string()]
    }

    async fn fix_issues(&self, _file_path: &Path, _issues: &[Issue]) -> Result<String, AppError> {
        // Implement automatic fixes
        Ok("Fixed issues".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_plugin_discovery() {
        let temp_dir = tempdir().unwrap();
        let manager = PluginManager::new(temp_dir.path());

        let manifests = manager.discover_plugins().await.unwrap();
        assert!(manifests.is_empty()); // No plugins in empty directory
    }

    #[tokio::test]
    async fn test_plugin_template_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = PluginManager::new(temp_dir.path());

        let plugin_dir = manager.create_plugin_template("test_plugin", "test_author").await.unwrap();
        assert!(plugin_dir.join("plugin.toml").exists());
        assert!(plugin_dir.join("src/lib.rs").exists());
    }
}
