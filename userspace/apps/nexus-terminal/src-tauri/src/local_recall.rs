use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// LocalRecall client for managing knowledge base and memory
#[derive(Debug, Clone)]
pub struct LocalRecallClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    default_collection: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub name: String,
    pub entries_count: Option<u32>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub max_results: Option<u32>,
    pub threshold: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub score: f32,
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub query: String,
    pub total_results: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddTextRequest {
    pub content: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalSourceRequest {
    pub url: String,
    pub update_interval: Option<u32>, // minutes
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl LocalRecallClient {
    /// Create a new LocalRecall client
    pub fn new(base_url: Option<String>, api_key: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| {
            std::env::var("LOCALRECALL_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
        });
        
        let api_key = api_key.or_else(|| std::env::var("LOCALRECALL_API_KEY").ok());
        
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url,
            api_key,
            default_collection: "nexus_terminal".to_string(),
        }
    }

    /// Initialize the client and ensure default collection exists
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing LocalRecall client at {}", self.base_url);
        
        // Test connection
        self.health_check().await?;
        
        // Ensure default collection exists
        self.ensure_collection_exists(&self.default_collection).await?;
        
        info!("LocalRecall client initialized successfully");
        Ok(())
    }

    /// Check if LocalRecall service is healthy
    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/health", self.base_url);
        
        let mut request = self.client.get(&url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        match request.send().await {
            Ok(response) if response.status().is_success() => {
                debug!("LocalRecall health check passed");
                Ok(())
            }
            Ok(response) => {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(anyhow::anyhow!("LocalRecall health check failed: {} - {}", status, error_text))
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to connect to LocalRecall: {}", e))
            }
        }
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<Collection>> {
        let url = format!("{}/api/collections", self.base_url);
        
        let mut request = self.client.get(&url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await
            .context("Failed to list collections")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Failed to list collections: {}", error_text));
        }

        let collections: Vec<Collection> = response.json().await
            .context("Failed to parse collections response")?;

        Ok(collections)
    }

    /// Create a new collection
    pub async fn create_collection(&self, name: &str, description: Option<String>) -> Result<()> {
        let url = format!("{}/api/collections", self.base_url);
        
        let request_body = CreateCollectionRequest {
            name: name.to_string(),
            description,
        };

        let mut request = self.client.post(&url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .json(&request_body)
            .send()
            .await
            .context("Failed to create collection")?;

        if response.status().is_success() {
            info!("Successfully created collection: {}", name);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("Failed to create collection '{}': {}", name, error_text))
        }
    }

    /// Ensure a collection exists, create it if it doesn't
    pub async fn ensure_collection_exists(&self, name: &str) -> Result<()> {
        match self.list_collections().await {
            Ok(collections) => {
                if collections.iter().any(|c| c.name == name) {
                    debug!("Collection '{}' already exists", name);
                    return Ok(());
                }
            }
            Err(e) => {
                warn!("Could not list collections: {}", e);
            }
        }

        // Collection doesn't exist, create it
        self.create_collection(name, Some(format!("Auto-created collection for {}", name))).await
    }

    /// Add text content to a collection
    pub async fn add_text(&self, collection: &str, content: &str, metadata: Option<HashMap<String, serde_json::Value>>, source: Option<String>) -> Result<()> {
        let url = format!("{}/api/collections/{}/add", self.base_url, collection);
        
        let request_body = AddTextRequest {
            content: content.to_string(),
            metadata,
            source,
        };

        let mut request = self.client.post(&url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .json(&request_body)
            .send()
            .await
            .context("Failed to add text to collection")?;

        if response.status().is_success() {
            debug!("Successfully added text to collection: {}", collection);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("Failed to add text to collection '{}': {}", collection, error_text))
        }
    }

    /// Search within a collection
    pub async fn search(&self, collection: &str, query: &str, max_results: Option<u32>, threshold: Option<f32>) -> Result<SearchResponse> {
        let url = format!("{}/api/collections/{}/search", self.base_url, collection);
        
        let request_body = SearchRequest {
            query: query.to_string(),
            max_results: max_results.or(Some(5)),
            threshold,
        };

        let mut request = self.client.post(&url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .json(&request_body)
            .send()
            .await
            .context("Failed to search collection")?;

        if response.status().is_success() {
            let search_response: SearchResponse = response.json().await
                .context("Failed to parse search response")?;
            
            debug!("Search in '{}' returned {} results", collection, search_response.results.len());
            Ok(search_response)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("Failed to search collection '{}': {}", collection, error_text))
        }
    }

    /// Add external source (web page, git repo, etc.)
    pub async fn add_external_source(&self, collection: &str, url: &str, update_interval: Option<u32>, metadata: Option<HashMap<String, serde_json::Value>>) -> Result<()> {
        let api_url = format!("{}/api/collections/{}/sources", self.base_url, collection);
        
        let request_body = ExternalSourceRequest {
            url: url.to_string(),
            update_interval,
            metadata,
        };

        let mut request = self.client.post(&api_url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .json(&request_body)
            .send()
            .await
            .context("Failed to add external source")?;

        if response.status().is_success() {
            info!("Successfully added external source '{}' to collection '{}'", url, collection);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("Failed to add external source '{}' to collection '{}': {}", url, collection, error_text))
        }
    }

    /// Upload a file to a collection
    pub async fn upload_file(&self, collection: &str, file_path: &str, metadata: Option<HashMap<String, serde_json::Value>>) -> Result<()> {
        let _url = format!("{}/api/collections/{}/upload", self.base_url, collection);
        
        // Read file content
        let content = tokio::fs::read_to_string(file_path).await
            .context(format!("Failed to read file: {}", file_path))?;

        // Use add_text with file metadata
        let mut file_metadata = metadata.unwrap_or_default();
        file_metadata.insert("file_path".to_string(), serde_json::Value::String(file_path.to_string()));
        file_metadata.insert("content_type".to_string(), serde_json::Value::String("file".to_string()));

        self.add_text(collection, &content, Some(file_metadata), Some(file_path.to_string())).await
    }

    /// Reset a collection (clear all entries)
    pub async fn reset_collection(&self, collection: &str) -> Result<()> {
        let url = format!("{}/api/collections/{}/reset", self.base_url, collection);
        
        let mut request = self.client.post(&url);
        if let Some(ref api_key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await
            .context("Failed to reset collection")?;

        if response.status().is_success() {
            info!("Successfully reset collection: {}", collection);
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow::anyhow!("Failed to reset collection '{}': {}", collection, error_text))
        }
    }

    /// Get contextual information for AI prompts
    pub async fn get_context_for_prompt(&self, query: &str, max_results: Option<u32>) -> Result<String> {
        let search_results = self.search(&self.default_collection, query, max_results, Some(0.7)).await?;
        
        if search_results.results.is_empty() {
            return Ok(String::new());
        }

        let mut context_parts = Vec::new();
        context_parts.push("## 🧠 Knowledge Base Context:".to_string());

        for (i, result) in search_results.results.iter().enumerate() {
            let relevance_indicator = if result.score > 0.9 {
                "🎯"
            } else if result.score > 0.8 {
                "📌" 
            } else {
                "💡"
            };

            let source_info = result.source.as_ref()
                .map(|s| format!(" (Source: {})", s))
                .unwrap_or_default();

            context_parts.push(format!(
                "{} **Result {}** (Relevance: {:.1}%){}\n{}",
                relevance_indicator,
                i + 1,
                result.score * 100.0,
                source_info,
                result.content.chars().take(500).collect::<String>() + 
                if result.content.len() > 500 { "..." } else { "" }
            ));
        }

        context_parts.push("---".to_string());
        context_parts.push("Based on this knowledge, here's my response:".to_string());

        Ok(context_parts.join("\n\n"))
    }

    /// Index command history for future context retrieval
    pub async fn index_command(&self, command: &str, output: &str, working_dir: &str, exit_code: i32, duration_ms: u64) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), serde_json::Value::String("command".to_string()));
        metadata.insert("working_dir".to_string(), serde_json::Value::String(working_dir.to_string()));
        metadata.insert("exit_code".to_string(), serde_json::Value::Number(serde_json::Number::from(exit_code)));
        metadata.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(duration_ms)));
        metadata.insert("timestamp".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));

        let content = format!(
            "Command: {}\nWorking Directory: {}\nExit Code: {}\nOutput:\n{}", 
            command, working_dir, exit_code, output.chars().take(2000).collect::<String>()
        );

        let source = format!("command_history_{}", chrono::Utc::now().timestamp());

        self.add_text("command_history", &content, Some(metadata), Some(source)).await
    }

    /// Index AI conversation for learning and context
    pub async fn index_conversation(&self, messages: &[(&str, &str)], context: Option<&str>) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), serde_json::Value::String("conversation".to_string()));
        metadata.insert("message_count".to_string(), serde_json::Value::Number(serde_json::Number::from(messages.len())));
        metadata.insert("timestamp".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
        
        if let Some(ctx) = context {
            metadata.insert("context".to_string(), serde_json::Value::String(ctx.to_string()));
        }

        let conversation_content = messages.iter()
            .map(|(role, content)| format!("{}: {}", role, content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let source = format!("conversation_{}", chrono::Utc::now().timestamp());

        self.add_text("conversations", &conversation_content, Some(metadata), Some(source)).await
    }

    /// Index codebase files for project context
    pub async fn index_codebase(&self, project_path: &str, files: &[String]) -> Result<()> {
        info!("Indexing codebase at: {}", project_path);
        
        // Ensure collection exists for this project
        let project_collection = format!("project_{}", 
            project_path.replace(['/', '\\', '.'], "_").trim_start_matches('_'));
        self.ensure_collection_exists(&project_collection).await?;

        let mut indexed_count = 0;
        for file_path in files {
            if let Ok(content) = tokio::fs::read_to_string(file_path).await {
                let mut metadata = HashMap::new();
                metadata.insert("type".to_string(), serde_json::Value::String("source_file".to_string()));
                metadata.insert("project_path".to_string(), serde_json::Value::String(project_path.to_string()));
                metadata.insert("file_extension".to_string(), serde_json::Value::String(
                    file_path.split('.').last().unwrap_or("unknown").to_string()
                ));
                metadata.insert("indexed_at".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));

                if let Err(e) = self.add_text(&project_collection, &content, Some(metadata), Some(file_path.clone())).await {
                    warn!("Failed to index file {}: {}", file_path, e);
                } else {
                    indexed_count += 1;
                }
            } else {
                warn!("Could not read file: {}", file_path);
            }
        }

        info!("Successfully indexed {} files from codebase", indexed_count);
        Ok(())
    }

    /// Auto-index common project files when in a new directory
    pub async fn auto_index_project(&self, working_dir: &str) -> Result<()> {
        let important_files = [
            "README.md", "README.txt", "README",
            "package.json", "Cargo.toml", "go.mod", "requirements.txt", "pom.xml",
            ".env.example", "docker-compose.yml", "Dockerfile",
            "CHANGELOG.md", "CONTRIBUTING.md", "LICENSE", "LICENSE.md",
        ];

        let mut found_files = Vec::new();
        
        for file in &important_files {
            let file_path = std::path::Path::new(working_dir).join(file);
            if file_path.exists() {
                found_files.push(file_path.to_string_lossy().to_string());
            }
        }

        if !found_files.is_empty() {
            info!("Auto-indexing {} project files in {}", found_files.len(), working_dir);
            self.index_codebase(working_dir, &found_files).await?;
        }

        Ok(())
    }

    /// Get the default collection name
    pub fn get_default_collection(&self) -> &str {
        &self.default_collection
    }

    /// Set the default collection
    pub fn set_default_collection(&mut self, collection: String) {
        self.default_collection = collection;
    }

    /// Get statistics about the knowledge base
    pub async fn get_stats(&self) -> Result<HashMap<String, serde_json::Value>> {
        let collections = self.list_collections().await.unwrap_or_default();
        
        let mut stats = HashMap::new();
        stats.insert("total_collections".to_string(), serde_json::Value::Number(serde_json::Number::from(collections.len())));
        stats.insert("default_collection".to_string(), serde_json::Value::String(self.default_collection.clone()));
        stats.insert("base_url".to_string(), serde_json::Value::String(self.base_url.clone()));
        stats.insert("api_key_configured".to_string(), serde_json::Value::Bool(self.api_key.is_some()));
        
        let collection_names: Vec<serde_json::Value> = collections.into_iter()
            .map(|c| serde_json::Value::String(c.name))
            .collect();
        stats.insert("collections".to_string(), serde_json::Value::Array(collection_names));

        Ok(stats)
    }
}

impl Default for LocalRecallClient {
    fn default() -> Self {
        Self::new(None, None)
    }
}

/// Helper function to start LocalRecall service if not running
pub async fn ensure_localrecall_running() -> Result<()> {
    use tokio::process::Command;
    use std::path::Path;

    info!("Ensuring LocalRecall service is running...");

    // Check if already running
    let client = LocalRecallClient::default();
    if client.health_check().await.is_ok() {
        info!("LocalRecall is already running");
        return Ok(());
    }

    // Try to start LocalRecall
    info!("LocalRecall not responding, attempting to start...");

    // Check if localrecall binary exists
    let localrecall_paths = [
        "/usr/local/bin/localrecall",
        "/usr/bin/localrecall", 
        "./localrecall",
        "../localrecall",
        "localrecall",
    ];

    let mut localrecall_cmd = None;
    for path in &localrecall_paths {
        if Path::new(path).exists() || *path == "localrecall" {
            localrecall_cmd = Some(*path);
            break;
        }
    }

    if let Some(cmd) = localrecall_cmd {
        info!("Starting LocalRecall with: {}", cmd);

        let mut command = Command::new(cmd);
        
        // Configure environment variables
        command.env("COLLECTION_DB_PATH", std::env::var("LOCALRECALL_DB_PATH").unwrap_or_else(|_| "/tmp/localrecall".to_string()));
        command.env("EMBEDDING_MODEL", std::env::var("LOCALRECALL_EMBEDDING_MODEL").unwrap_or_else(|_| "granite-embedding-107m-multilingual".to_string()));
        command.env("FILE_ASSETS", std::env::var("LOCALRECALL_FILE_ASSETS").unwrap_or_else(|_| "/tmp/localrecall_assets".to_string()));
        command.env("LISTENING_ADDRESS", std::env::var("LOCALRECALL_ADDRESS").unwrap_or_else(|_| ":8080".to_string()));
        command.env("OPENAI_BASE_URL", std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()));
        command.env("OPENAI_API_KEY", "sk-localrecall");

        // Start in background
        match command.spawn() {
            Ok(mut child) => {
                tokio::spawn(async move {
                    let _ = child.wait().await;
                });
                
                // Wait for service to be ready
                for attempt in 1..=10 {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    if client.health_check().await.is_ok() {
                        info!("LocalRecall started successfully after {} attempts", attempt);
                        return Ok(());
                    }
                }
                
                Err(anyhow::anyhow!("LocalRecall failed to start within timeout"))
            }
            Err(e) => {
                error!("Failed to start LocalRecall: {}", e);
                Err(anyhow::anyhow!("Could not start LocalRecall: {}", e))
            }
        }
    } else {
        warn!("LocalRecall binary not found. Please install LocalRecall or ensure it's in PATH");
        Err(anyhow::anyhow!("LocalRecall binary not found"))
    }
}
