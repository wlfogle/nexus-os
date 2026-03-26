use crate::errors::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,
    pub timeout: u64,
    pub max_tokens: Option<u32>,
    pub temperature: f32,
    pub top_p: f32,
    pub repeat_penalty: f32,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            timeout: 120,
            max_tokens: Some(4096),
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    pub details: ModelDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetails {
    pub format: String,
    pub family: String,
    pub families: Option<Vec<String>>,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: Option<bool>,
    pub options: Option<ChatOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatOptions {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub num_predict: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub model: String,
    pub created_at: String,
    pub message: ChatMessage,
    pub done: bool,
    pub total_duration: Option<u64>,
    pub load_duration: Option<u64>,
    pub prompt_eval_count: Option<u32>,
    pub prompt_eval_duration: Option<u64>,
    pub eval_count: Option<u32>,
    pub eval_duration: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ListModelsResponse {
    pub models: Vec<OllamaModel>,
}

pub struct OllamaClient {
    client: Client,
    config: OllamaConfig,
}

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, AppError> {
        let url = format!("{}/api/tags", self.config.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to fetch models: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!("HTTP error: {}", response.status())));
        }

        let models_response: ListModelsResponse = response
            .json()
            .await
            .map_err(|e| AppError::Parsing(format!("Failed to parse models response: {}", e)))?;

        Ok(models_response.models)
    }

    pub async fn chat(&self, model: &str, messages: Vec<ChatMessage>) -> Result<String, AppError> {
        let url = format!("{}/api/chat", self.config.base_url);
        
        let request = ChatRequest {
            model: model.to_string(),
            messages,
            stream: Some(false),
            options: Some(ChatOptions {
                temperature: Some(self.config.temperature),
                top_p: Some(self.config.top_p),
                repeat_penalty: Some(self.config.repeat_penalty),
                num_predict: self.config.max_tokens,
            }),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to send chat request: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!("HTTP error: {}", response.status())));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::Parsing(format!("Failed to parse chat response: {}", e)))?;

        Ok(chat_response.message.content)
    }

    pub async fn chat_stream(&self, model: &str, messages: Vec<ChatMessage>) -> Result<mpsc::Receiver<String>, AppError> {
        let url = format!("{}/api/chat", self.config.base_url);
        
        let request = ChatRequest {
            model: model.to_string(),
            messages,
            stream: Some(true),
            options: Some(ChatOptions {
                temperature: Some(self.config.temperature),
                top_p: Some(self.config.top_p),
                repeat_penalty: Some(self.config.repeat_penalty),
                num_predict: self.config.max_tokens,
            }),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to send chat request: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!("HTTP error: {}", response.status())));
        }

        let (tx, rx) = mpsc::channel(100);
        
        tokio::spawn(async move {
            let mut stream = response.bytes_stream();
            use futures::StreamExt;
            
            while let Some(item) = stream.next().await {
                match item {
                    Ok(chunk) => {
                        if let Ok(text) = String::from_utf8(chunk.to_vec()) {
                            for line in text.lines() {
                                if line.trim().is_empty() {
                                    continue;
                                }
                                
                                if let Ok(chat_response) = serde_json::from_str::<ChatResponse>(line) {
                                    if !chat_response.message.content.is_empty() {
                                        if tx.send(chat_response.message.content).await.is_err() {
                                            break;
                                        }
                                    }
                                    
                                    if chat_response.done {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(rx)
    }

    pub async fn pull_model(&self, model: &str) -> Result<(), AppError> {
        let url = format!("{}/api/pull", self.config.base_url);
        
        let request = serde_json::json!({
            "name": model,
            "stream": false
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to pull model: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!("HTTP error: {}", response.status())));
        }

        Ok(())
    }

    pub async fn delete_model(&self, model: &str) -> Result<(), AppError> {
        let url = format!("{}/api/delete", self.config.base_url);
        
        let request = serde_json::json!({
            "name": model
        });

        let response = self.client
            .delete(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to delete model: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!("HTTP error: {}", response.status())));
        }

        Ok(())
    }

    pub async fn show_model(&self, model: &str) -> Result<serde_json::Value, AppError> {
        let url = format!("{}/api/show", self.config.base_url);
        
        let request = serde_json::json!({
            "name": model
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to show model: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!("HTTP error: {}", response.status())));
        }

        let model_info: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Parsing(format!("Failed to parse model info: {}", e)))?;

        Ok(model_info)
    }

    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String, AppError> {
        let url = format!("{}/api/generate", self.config.base_url);
        
        let request = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "top_p": self.config.top_p,
                "repeat_penalty": self.config.repeat_penalty,
                "num_predict": self.config.max_tokens
            }
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to generate: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Network(format!("HTTP error: {}", response.status())));
        }

        let generate_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Parsing(format!("Failed to parse generate response: {}", e)))?;

        Ok(generate_response["response"].as_str().unwrap_or("").to_string())
    }

    pub async fn check_health(&self) -> Result<bool, AppError> {
        let url = format!("{}/api/tags", self.config.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

// Utility functions for common model operations
pub struct ModelManager {
    ollama: OllamaClient,
}

impl ModelManager {
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            ollama: OllamaClient::new(config),
        }
    }

    pub async fn ensure_model_available(&self, model: &str) -> Result<(), AppError> {
        let models = self.ollama.list_models().await?;
        
        if !models.iter().any(|m| m.name == model) {
            tracing::info!("Model {} not found locally, pulling...", model);
            self.ollama.pull_model(model).await?;
            tracing::info!("Successfully pulled model {}", model);
        }

        Ok(())
    }

    pub async fn get_recommended_models(&self) -> Vec<&'static str> {
        vec![
            "llama3.1:8b",
            "llama3.1:70b", 
            "codellama:7b",
            "codellama:13b",
            "mistral:7b",
            "phi3:mini",
            "qwen2:7b",
            "gemma2:9b",
            "llava:7b",
            "nomic-embed-text",
        ]
    }

    pub async fn get_best_model_for_task(&self, task: &str) -> &'static str {
        match task.to_lowercase().as_str() {
            task if task.contains("code") => "codellama:13b",
            task if task.contains("math") => "llama3.1:70b",
            task if task.contains("creative") => "mistral:7b",
            task if task.contains("vision") => "llava:7b",
            task if task.contains("embed") => "nomic-embed-text",
            task if task.contains("fast") => "phi3:mini",
            _ => "llama3.1:8b",
        }
    }
}
