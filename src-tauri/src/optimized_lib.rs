// Optimized library with modular structure
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use reqwest::{Client, ClientBuilder};
use tauri::State;

// AI-optimized configuration constants
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);
const CONNECTION_POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(90);
const MAX_RETRIES: u32 = 3;
const BACKOFF_MULTIPLIER: u64 = 2;

// Global HTTP client with optimized settings
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    ClientBuilder::new()
        .timeout(DEFAULT_TIMEOUT)
        .pool_idle_timeout(CONNECTION_POOL_IDLE_TIMEOUT)
        .pool_max_idle_per_host(10)
        .tcp_keepalive(Duration::from_secs(60))
        .build()
        .expect("Failed to create HTTP client")
});

// Advanced language detection patterns
static LANGUAGE_PATTERNS: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(|| {
    let mut patterns = HashMap::new();
    patterns.insert("python", vec!["def ", "import ", "from ", "class ", "if __name__", "print(", "self.", "#!/usr/bin/env python"]);
    patterns.insert("javascript", vec!["function ", "const ", "let ", "var ", "=>", "require(", "module.exports", "console.log"]);
    patterns.insert("typescript", vec!["interface ", "type ", "enum ", ": string", ": number", ": boolean", "export type"]);
    patterns.insert("rust", vec!["fn ", "let mut", "use std::", "impl ", "struct ", "enum ", "match ", "Ok(", "Err("]);
    patterns.insert("go", vec!["func ", "package ", "import \"", "type ", "var ", "fmt.Print", "go func(", "defer "]);
    patterns.insert("java", vec!["public class", "private ", "protected ", "import java", "System.out", "public static void main"]);
    patterns.insert("c++", vec!["#include", "using namespace", "int main(", "std::", "class ", "template<", "cout <<", "cin >>"]);
    patterns.insert("c", vec!["#include <", "int main(", "printf(", "scanf(", "malloc(", "free(", "struct ", "typedef"]);
    patterns.insert("csharp", vec!["using System", "namespace ", "public class", "Console.WriteLine", "public static void Main"]);
    patterns.insert("php", vec!["<?php", "$", "function ", "class ", "echo ", "require_once", "namespace "]);
    patterns.insert("ruby", vec!["def ", "class ", "module ", "puts ", "require ", "attr_accessor", "end"]);
    patterns.insert("swift", vec!["func ", "var ", "let ", "import Foundation", "class ", "struct ", "enum "]);
    patterns.insert("kotlin", vec!["fun ", "val ", "var ", "class ", "object ", "import ", "package "]);
    patterns.insert("scala", vec!["def ", "val ", "var ", "object ", "class ", "trait ", "case class", "import "]);
    patterns
});

// AI model selection with performance optimization
static MODEL_SELECTION: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut models = HashMap::new();
    models.insert("rust", "codellama:7b");
    models.insert("python", "qwen2.5-coder:7b");
    models.insert("javascript", "codegemma:7b");
    models.insert("typescript", "codegemma:7b");
    models.insert("go", "codegemma:7b");
    models.insert("java", "codellama:7b");
    models.insert("c++", "magicoder:7b");
    models.insert("c", "magicoder:7b");
    models.insert("csharp", "codellama:7b");
    models.insert("php", "codellama:7b");
    models.insert("ruby", "codellama:7b");
    models.insert("swift", "codellama:7b");
    models.insert("kotlin", "codellama:7b");
    models.insert("scala", "codellama:7b");
    models
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ai_container_url: String,
    pub ai_container_port: u16,
    pub max_code_length: usize,
    pub enable_caching: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai_container_url: "http://192.168.122.172".to_string(),
            ai_container_port: 11434,
            max_code_length: 50_000, // 50KB limit
            enable_caching: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub code: String,
    pub language: String,
    pub operation: String,
    pub context: Option<String>,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub result: String,
    pub success: bool,
    pub model_used: Option<String>,
    pub processing_time_ms: u64,
    pub tokens_used: Option<u32>,
    pub confidence_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub retry_after: Option<u64>,
}

// Cache for analysis results
type AnalysisCache = Arc<RwLock<HashMap<String, (AnalysisResponse, Instant)>>>;

pub struct AppState {
    pub config: Arc<RwLock<AppConfig>>,
    pub analysis_cache: AnalysisCache,
    pub connection_status: Arc<RwLock<bool>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(AppConfig::default())),
            analysis_cache: Arc::new(RwLock::new(HashMap::new())),
            connection_status: Arc::new(RwLock::new(false)),
        }
    }
}

// AI-enhanced language detection
fn detect_language_advanced(code: &str) -> String {
    let mut scores: HashMap<&str, i32> = HashMap::new();
    
    for (language, patterns) in LANGUAGE_PATTERNS.iter() {
        let mut score = 0;
        for pattern in patterns {
            let count = code.matches(pattern).count() as i32;
            score += count * 2; // Weight pattern matches
        }
        
        // Additional scoring based on file structure
        if language == &"python" && code.contains(":\n    ") {
            score += 5; // Python indentation
        }
        if language == &"javascript" && code.contains("{\n") {
            score += 3; // JS brace style
        }
        if language == &"rust" && code.contains("Result<") {
            score += 4; // Rust Result type
        }
        
        scores.insert(language, score);
    }
    
    scores.into_iter()
        .max_by_key(|(_, score)| *score)
        .map(|(lang, _)| lang.to_string())
        .unwrap_or_else(|| "text".to_string())
}

// Enhanced prompt creation with context awareness
fn create_enhanced_prompt(request: &AnalysisRequest) -> String {
    let base_context = match request.operation.as_str() {
        "analyze" => "As an expert code reviewer, analyze this code for:",
        "fix_bugs" => "As a debugging specialist, identify and fix bugs in this code:",
        "optimize" => "As a performance expert, optimize this code for better performance:",
        "document" => "As a technical writer, create comprehensive documentation for this code:",
        "test" => "As a test engineer, generate comprehensive unit tests for this code:",
        "security" => "As a security expert, analyze this code for security vulnerabilities:",
        "refactor" => "As a software architect, refactor this code for better maintainability:",
        _ => "As a software engineer, review this code:",
    };

    let language_specific_guidance = match request.language.as_str() {
        "rust" => "Focus on memory safety, ownership, borrowing, and idiomatic Rust patterns.",
        "python" => "Focus on PEP 8 compliance, performance, and Pythonic patterns.",
        "javascript" => "Focus on modern ES6+ features, async/await patterns, and performance.",
        "typescript" => "Focus on type safety, interfaces, and TypeScript best practices.",
        "go" => "Focus on Go idioms, error handling, and goroutine safety.",
        "java" => "Focus on object-oriented design, performance, and Java best practices.",
        "c++" => "Focus on memory management, RAII, and modern C++ features.",
        _ => "Focus on code quality, performance, and best practices.",
    };

    let context_info = request.context
        .as_ref()
        .map(|ctx| format!("\nAdditional context: {ctx}"))
        .unwrap_or_default();

    let file_info = request.file_path
        .as_ref()
        .map(|path| format!("\nFile: {path}"))
        .unwrap_or_default();

    format!(
        "{}\n\n{}\n\nProvide specific, actionable feedback with line numbers where applicable.{}{}\n\nCode:\n```{}\n{}\n```",
        base_context,
        language_specific_guidance,
        context_info,
        file_info,
        request.language,
        request.code
    )
}

// AI-optimized model selection
fn select_optimal_model(language: &str, code_length: usize) -> String {
    let base_model = MODEL_SELECTION
        .get(language)
        .unwrap_or(&"codellama:7b")
        .to_string();
    
    // For larger code files, prefer more capable models
    if code_length > 10_000 {
        match language {
            "python" => "qwen2.5-coder:14b".to_string(),
            "rust" | "c++" => "magicoder:14b".to_string(),
            _ => base_model,
        }
    } else {
        base_model
    }
}

// Exponential backoff retry logic
async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_retries: u32,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempt = 0;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                attempt += 1;
                if attempt >= max_retries {
                    return Err(error);
                }
                
                let delay = Duration::from_millis(100 * BACKOFF_MULTIPLIER.pow(attempt));
                tokio::time::sleep(delay).await;
            }
        }
    }
}

// Enhanced code analysis with caching and optimization
#[tauri::command]
pub async fn analyze_code_optimized(
    state: State<'_, AppState>,
    request: AnalysisRequest,
) -> Result<AnalysisResponse, ErrorDetails> {
    let start_time = Instant::now();
    
    // Validate input
    if request.code.trim().is_empty() {
        return Err(ErrorDetails {
            code: "INVALID_INPUT".to_string(),
            message: "Code input cannot be empty".to_string(),
            details: None,
            retry_after: None,
        });
    }
    
    let config = state.config.read().await;
    
    // Check code length limit
    if request.code.len() > config.max_code_length {
        return Err(ErrorDetails {
            code: "CODE_TOO_LARGE".to_string(),
            message: format!("Code exceeds maximum length of {} characters", config.max_code_length),
            details: Some("Consider breaking the code into smaller chunks".to_string()),
            retry_after: None,
        });
    }
    
    // Generate cache key
    let mut hasher = DefaultHasher::new();
    request.code.hash(&mut hasher);
    request.language.hash(&mut hasher);
    request.operation.hash(&mut hasher);
    let cache_key = format!("{:x}", hasher.finish());
    
    // Check cache if enabled
    if config.enable_caching {
        let cache = state.analysis_cache.read().await;
        if let Some((cached_response, cached_time)) = cache.get(&cache_key) {
            if cached_time.elapsed() < Duration::from_secs(3600) { // 1 hour cache
                return Ok(cached_response.clone());
            }
        }
    }
    
    // Detect language if not specified or auto
    let detected_language = if request.language == "auto" || request.language.is_empty() {
        detect_language_advanced(&request.code)
    } else {
        request.language.clone()
    };
    
    // Create enhanced prompt
    let enhanced_request = AnalysisRequest {
        language: detected_language.clone(),
        ..request
    };
    let prompt = create_enhanced_prompt(&enhanced_request);
    
    // Select optimal model
    let model = select_optimal_model(&detected_language, enhanced_request.code.len());
    
    // Prepare request body
    let mut request_body = HashMap::new();
    request_body.insert("model", model.clone());
    request_body.insert("prompt", prompt);
    request_body.insert("stream", "false".to_string());
    request_body.insert("options", serde_json::json!({
        "temperature": 0.1,
        "top_p": 0.9,
        "stop": ["```", "---"]
    }).to_string());
    
    let ai_url = format!("{}:{}/api/generate", config.ai_container_url, config.ai_container_port);
    
    // Execute request with retry logic
    let response_result = retry_with_backoff(
        || async {
            HTTP_CLIENT
                .post(&ai_url)
                .json(&request_body)
                .send()
                .await
        },
        MAX_RETRIES,
    ).await;
    
    drop(config); // Release read lock
    
    let response = response_result.map_err(|e| ErrorDetails {
        code: "CONNECTION_ERROR".to_string(),
        message: format!("Failed to connect to AI container: {e}"),
        details: Some("Check if the AI container is running and accessible".to_string()),
        retry_after: Some(5),
    })?;
    
    if !response.status().is_success() {
        return Err(ErrorDetails {
            code: "AI_ERROR".to_string(),
            message: format!("AI container returned error: {}", response.status()),
            details: None,
            retry_after: Some(10),
        });
    }
    
    let json: HashMap<String, serde_json::Value> = response
        .json()
        .await
        .map_err(|e| ErrorDetails {
            code: "PARSE_ERROR".to_string(),
            message: format!("Failed to parse response: {e}"),
            details: None,
            retry_after: None,
        })?;
    
    let result = json
        .get("response")
        .and_then(|v| v.as_str())
        .unwrap_or("No response received")
        .to_string();
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    let analysis_response = AnalysisResponse {
        result,
        success: true,
        model_used: Some(model),
        processing_time_ms: processing_time,
        tokens_used: json.get("total_tokens").and_then(|v| v.as_u64()).map(|v| v as u32),
        confidence_score: Some(0.95), // Mock confidence score
    };
    
    // Cache the result
    if state.config.read().await.enable_caching {
        let mut cache = state.analysis_cache.write().await;
        cache.insert(cache_key, (analysis_response.clone(), Instant::now()));
        
        // Clean old cache entries (simple cleanup)
        if cache.len() > 100 {
            cache.retain(|_, (_, time)| time.elapsed() < Duration::from_secs(3600));
        }
    }
    
    Ok(analysis_response)
}

// Enhanced connection check with health monitoring
#[tauri::command]
pub async fn check_ai_connection_enhanced(
    state: State<'_, AppState>,
) -> Result<HashMap<String, serde_json::Value>, ErrorDetails> {
    let config = state.config.read().await;
    let ai_url = format!("{}:{}/api/tags", config.ai_container_url, config.ai_container_port);
    
    let start_time = Instant::now();
    
    let response = HTTP_CLIENT
        .get(&ai_url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| ErrorDetails {
            code: "CONNECTION_ERROR".to_string(),
            message: format!("Failed to connect: {e}"),
            details: None,
            retry_after: Some(5),
        })?;
    
    let response_time = start_time.elapsed().as_millis() as u64;
    let is_connected = response.status().is_success();
    
    // Update connection status
    *state.connection_status.write().await = is_connected;
    
    let mut health_info = HashMap::new();
    health_info.insert("connected".to_string(), serde_json::Value::Bool(is_connected));
    health_info.insert("response_time_ms".to_string(), serde_json::Value::Number(response_time.into()));
    health_info.insert("status_code".to_string(), serde_json::Value::Number(response.status().as_u16().into()));
    
    if is_connected {
        if let Ok(models_json) = response.json::<serde_json::Value>().await {
            health_info.insert("models_available".to_string(), models_json);
        }
    }
    
    Ok(health_info)
}

// Enhanced model listing with metadata
#[tauri::command]
pub async fn get_available_models_enhanced(
    state: State<'_, AppState>,
) -> Result<Vec<HashMap<String, serde_json::Value>>, ErrorDetails> {
    let config = state.config.read().await;
    let ai_url = format!("{}:{}/api/tags", config.ai_container_url, config.ai_container_port);
    
    let response = HTTP_CLIENT
        .get(&ai_url)
        .send()
        .await
        .map_err(|e| ErrorDetails {
            code: "CONNECTION_ERROR".to_string(),
            message: format!("Failed to connect: {e}"),
            details: None,
            retry_after: Some(5),
        })?;
    
    if !response.status().is_success() {
        return Err(ErrorDetails {
            code: "AI_ERROR".to_string(),
            message: "Failed to fetch models".to_string(),
            details: None,
            retry_after: Some(10),
        });
    }
    
    let json: HashMap<String, serde_json::Value> = response
        .json()
        .await
        .map_err(|e| ErrorDetails {
            code: "PARSE_ERROR".to_string(),
            message: format!("Failed to parse response: {e}"),
            details: None,
            retry_after: None,
        })?;
    
    let models = json
        .get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|model| {
                    let mut model_info = HashMap::new();
                    if let Some(name) = model.get("name").and_then(|n| n.as_str()) {
                        model_info.insert("name".to_string(), serde_json::Value::String(name.to_string()));
                        model_info.insert("size".to_string(), model.get("size").cloned().unwrap_or(serde_json::Value::Null));
                        model_info.insert("modified".to_string(), model.get("modified_at").cloned().unwrap_or(serde_json::Value::Null));
                        model_info.insert("digest".to_string(), model.get("digest").cloned().unwrap_or(serde_json::Value::Null));
                        
                        // Add recommended usage info
                        let recommended_for = match name {
                            n if n.contains("codellama") => vec!["general", "rust", "java"],
                            n if n.contains("qwen") => vec!["python", "complex_analysis"],
                            n if n.contains("codegemma") => vec!["javascript", "typescript", "go"],
                            n if n.contains("magicoder") => vec!["c++", "c", "optimization"],
                            _ => vec!["general"],
                        };
                        model_info.insert("recommended_for".to_string(), 
                            serde_json::Value::Array(recommended_for.into_iter().map(|s| serde_json::Value::String(s.to_string())).collect()));
                        
                        Some(model_info)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_else(|| {
            vec![{
                let mut default_model = HashMap::new();
                default_model.insert("name".to_string(), serde_json::Value::String("codellama:7b".to_string()));
                default_model.insert("recommended_for".to_string(), 
                    serde_json::Value::Array(vec![serde_json::Value::String("general".to_string())]));
                default_model
            }]
        });
    
    Ok(models)
}

// Configuration management
#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    new_config: AppConfig,
) -> Result<(), ErrorDetails> {
    let mut config = state.config.write().await;
    *config = new_config;
    Ok(())
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, ErrorDetails> {
    let config = state.config.read().await;
    Ok(config.clone())
}

// Cache management
#[tauri::command]
pub async fn clear_analysis_cache(state: State<'_, AppState>) -> Result<(), ErrorDetails> {
    let mut cache = state.analysis_cache.write().await;
    cache.clear();
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            analyze_code_optimized,
            check_ai_connection_enhanced,
            get_available_models_enhanced,
            update_config,
            get_config,
            clear_analysis_cache
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
