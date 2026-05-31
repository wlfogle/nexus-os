use alloc::string::{String, ToString};
use alloc::format;
use crate::ollama_client::OllamaClient;

/// Process incoming AI request and return response
pub fn handle_request(payload: &[u8]) -> String {
    // Parse payload as UTF-8 request string
    let request = core::str::from_utf8(payload)
        .unwrap_or("[nexus-ai] Request contains invalid UTF-8");
    
    // Phase 5.0: Fixed model name; Phase 5.2 will support model selection
    let client = OllamaClient::new("http://localhost:11434");
    
    // In Phase 5.1, this will parse JSON and extract model/params
    // For now: send raw request as prompt
    match client.generate("mistral", request) {
        Ok(response) => response,
        Err(e) => format!("[nexus-ai] Error: {}", e),
    }
}