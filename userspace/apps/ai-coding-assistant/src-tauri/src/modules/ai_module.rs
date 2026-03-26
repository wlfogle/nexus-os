use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;
use reqwest::Client;
use tracing::{info, warn, error};

// Optimized: Single HTTP client instance with connection pooling
static HTTP_CLIENT: Lazy<Arc<Client>> = Lazy::new(|| {
    Arc::new(
        Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new())
    )
});

// Optimized: Response caching
static RESPONSE_CACHE: Lazy<Arc<RwLock<HashMap<String, CachedResponse>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

#[derive(Clone, Debug)]
struct CachedResponse {
    response: String,
    timestamp: std::time::SystemTime,
    ttl: std::time::Duration,
}

#[derive(Debug, Clone)]
pub enum AIOperation {
    Analyze,
    FixBugs,
    Optimize,
    Document,
    Test,
    ExplainCode,
    DebugError,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub context_window: Option<u32>,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            model: "codellama:7b".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(2048),
            context_window: Some(4096),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnalysisRequest {
    pub code: String,
    pub language: String,
    pub operation: String,
    pub config: Option<AIConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnalysisResponse {
    pub result: String,
    pub success: bool,
    pub model_used: Option<String>,
    pub processing_time: Option<u64>,
    pub confidence: Option<f32>,
}

// Optimized: Generic AI processing function
pub async fn process_with_ai(
    content: &str,
    language: &str,
    operation: AIOperation,
    config: Option<AIConfig>,
) -> Result<AnalysisResponse, crate::errors::AppError> {
    let start_time = std::time::Instant::now();
    let config = config.unwrap_or_default();
    
    // Create cache key
    let cache_key = create_cache_key(content, language, &operation, &config);
    
    // Check cache first
    if let Some(cached) = get_cached_response(&cache_key).await {
        info!("Cache hit for AI request");
        return Ok(AnalysisResponse {
            result: cached,
            success: true,
            model_used: Some(config.model),
            processing_time: Some(start_time.elapsed().as_millis() as u64),
            confidence: Some(0.95), // High confidence for cached responses
        });
    }
    
    let prompt = create_optimized_prompt(content, language, &operation, &config);
    
    let mut request_body = HashMap::new();
    request_body.insert("model", serde_json::Value::String(config.model.clone()));
    request_body.insert("prompt", serde_json::Value::String(prompt));
    request_body.insert("stream", serde_json::Value::Bool(false));
    
    if let Some(temp) = config.temperature {
        request_body.insert("temperature", serde_json::Value::Number(
            serde_json::Number::from_f64(temp as f64).unwrap_or_else(|| serde_json::Number::from(0))
        ));
    }
    
    match send_ai_request(&request_body).await {
        Ok(response) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            
            // Cache successful response
            cache_response(&cache_key, &response).await;
            
            Ok(AnalysisResponse {
                result: response,
                success: true,
                model_used: Some(config.model),
                processing_time: Some(processing_time),
                confidence: Some(calculate_confidence(&operation, content)),
            })
        }
        Err(e) => {
            error!("AI processing failed: {:?}", e);
            Err(e)
        }
    }
}

// Optimized: Template-based prompt creation
fn create_optimized_prompt(
    content: &str,
    language: &str,
    operation: &AIOperation,
    config: &AIConfig,
) -> String {
    let base_context = format!(
        "Language: {}\nContext Window: {}\nContent Length: {} chars",
        language,
        config.context_window.unwrap_or(4096),
        content.len()
    );
    
    let instruction = match operation {
        AIOperation::Analyze => format!(
            "Analyze this {} code and provide detailed feedback about potential issues, improvements, and optimizations. Be specific with line numbers where applicable.",
            language
        ),
        AIOperation::FixBugs => format!(
            "Find and fix bugs in this {} code. Provide the corrected code with explanations.",
            language
        ),
        AIOperation::Optimize => format!(
            "Optimize this {} code for better performance and readability. Show before and after examples.",
            language
        ),
        AIOperation::Document => format!(
            "Generate comprehensive documentation for this {} code including function descriptions, parameters, and usage examples.",
            language
        ),
        AIOperation::Test => format!(
            "Generate comprehensive unit tests for this {} code.",
            language
        ),
        AIOperation::ExplainCode => format!(
            "Explain what this {} code does in clear, detailed terms. Break down complex parts.",
            language
        ),
        AIOperation::DebugError => format!(
            "Debug this error message and provide solutions. Explain what caused it and how to fix it."
        ),
        AIOperation::Custom(instruction) => instruction.clone(),
    };
    
    format!("{}\n\n{}\n\nCode:\n```{}\n{}\n```", base_context, instruction, language, content)
}

async fn send_ai_request(
    request_body: &HashMap<&str, serde_json::Value>,
) -> Result<String, crate::errors::AppError> {
    let ai_container_url = "http://192.168.122.172:11434/api/generate";
    
    let response = HTTP_CLIENT
        .post(ai_container_url)
        .json(request_body)
        .send()
        .await
        .map_err(|e| crate::errors::AppError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(crate::errors::AppError::AIService(
            format!("AI container returned error: {}", response.status())
        ));
    }
    
    let json: HashMap<String, serde_json::Value> = response
        .json()
        .await
        .map_err(|e| crate::errors::AppError::Validation(format!("JSON error: {}", e)))?;
    
    let result = json
        .get("response")
        .and_then(|v| v.as_str())
        .ok_or_else(|| crate::errors::AppError::AIService("No response from AI".to_string()))?;
    
    Ok(result.to_string())
}

fn create_cache_key(
    content: &str,
    language: &str,
    operation: &AIOperation,
    config: &AIConfig,
) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    language.hash(&mut hasher);
    format!("{:?}", operation).hash(&mut hasher);
    config.model.hash(&mut hasher);
    
    format!("{:x}", hasher.finish())
}

async fn get_cached_response(cache_key: &str) -> Option<String> {
    let cache = RESPONSE_CACHE.read().await;
    
    if let Some(cached) = cache.get(cache_key) {
        if cached.timestamp.elapsed().unwrap_or_default() < cached.ttl {
            return Some(cached.response.clone());
        }
    }
    
    None
}

async fn cache_response(cache_key: &str, response: &str) {
    let mut cache = RESPONSE_CACHE.write().await;
    
    // Limit cache size to prevent memory issues
    if cache.len() >= 1000 {
        // Collect keys to remove first
        let mut entries_to_sort: Vec<(String, std::time::SystemTime)> = cache.iter()
            .map(|(k, v)| (k.clone(), v.timestamp))
            .collect();
        
        // Sort by timestamp (oldest first)
        entries_to_sort.sort_by_key(|(_, timestamp)| *timestamp);
        
        // Take oldest 200 keys
        let keys_to_remove: Vec<String> = entries_to_sort
            .into_iter()
            .take(200)
            .map(|(k, _)| k)
            .collect();
        
        // Now remove them
        for key in keys_to_remove {
            cache.remove(&key);
        }
    }
    
    cache.insert(cache_key.to_string(), CachedResponse {
        response: response.to_string(),
        timestamp: std::time::SystemTime::now(),
        ttl: std::time::Duration::from_secs(3600), // 1 hour TTL
    });
}

fn calculate_confidence(operation: &AIOperation, content: &str) -> f32 {
    let mut confidence: f32 = 0.7; // Base confidence
    
    // Adjust based on content characteristics
    if content.len() > 100 && content.len() < 5000 {
        confidence += 0.1;
    }
    
    // Adjust based on operation type
    match operation {
        AIOperation::ExplainCode | AIOperation::Document => confidence += 0.1,
        AIOperation::FixBugs | AIOperation::Optimize => confidence += 0.05,
        AIOperation::DebugError => confidence += 0.15,
        _ => {}
    }
    
    confidence.min(1.0)
}

// Optimized: Check AI service availability with circuit breaker pattern
pub async fn check_ai_service_health() -> Result<bool, crate::errors::AppError> {
    let ai_container_url = "http://192.168.122.172:11434/api/tags";
    
    match HTTP_CLIENT.get(ai_container_url).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(e) => {
            warn!("AI service health check failed: {}", e);
            Ok(false)
        }
    }
}

// Get available models with caching
pub async fn get_available_models() -> Result<Vec<String>, crate::errors::AppError> {
    static MODEL_CACHE: Lazy<Arc<RwLock<Option<(Vec<String>, std::time::SystemTime)>>>> = 
        Lazy::new(|| Arc::new(RwLock::new(None)));
    
    // Check cache first (5 minute TTL)
    {
        let cache = MODEL_CACHE.read().await;
        if let Some((models, timestamp)) = cache.as_ref() {
            if timestamp.elapsed().unwrap_or_default() < std::time::Duration::from_secs(300) {
                return Ok(models.clone());
            }
        }
    }
    
    let ai_container_url = "http://192.168.122.172:11434/api/tags";
    
    let response = HTTP_CLIENT
        .get(ai_container_url)
        .send()
        .await
        .map_err(|e| crate::errors::AppError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(crate::errors::AppError::AIService(
            "Failed to get models".to_string()
        ));
    }
    
    let json: HashMap<String, serde_json::Value> = response
        .json()
        .await
        .map_err(|e| crate::errors::AppError::Validation(format!("JSON error: {}", e)))?;
    
    let models = json
        .get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|model| model.get("name").and_then(|n| n.as_str()))
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_else(|| vec!["codellama:7b".to_string()]);
    
    // Update cache
    {
        let mut cache = MODEL_CACHE.write().await;
        *cache = Some((models.clone(), std::time::SystemTime::now()));
    }
    
    Ok(models)
}
