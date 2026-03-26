use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;
use tracing::{info, error, debug};
use uuid::Uuid;

/// Inspired by agent-protocol and agenticSeek from your starred repos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProtocolMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Inspired by AGiXT and SuperCoder - Dynamic AI Agent with capabilities
#[derive(Debug, Clone)]
pub struct IntelligentAgent {
    pub agent_id: String,
    pub capabilities: Vec<AgentCapability>,
    pub memory: AgentMemory,
    pub ollama_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentCapability {
    CodeGeneration,
    SystemDiagnostics,
    FileManagement,
    NetworkOperations,
    ProcessControl,
    GitOperations,
    TerminalAutomation,
    Learning,
    WebBrowsing,
}

/// Inspired by anything-llm and localGPT for memory management
#[derive(Debug, Clone)]
pub struct AgentMemory {
    pub conversation_history: Vec<AgentProtocolMessage>,
    pub learned_patterns: HashMap<String, f64>,
    pub context_embeddings: Vec<ContextEmbedding>,
    pub working_directory: String,
    pub session_context: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ContextEmbedding {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub relevance_score: f64,
}

/// Inspired by open-interpreter for natural language processing
impl IntelligentAgent {
    pub fn new(ollama_url: String, model: String) -> Self {
        Self {
            agent_id: Uuid::new_v4().to_string(),
            capabilities: vec![
                AgentCapability::CodeGeneration,
                AgentCapability::SystemDiagnostics,
                AgentCapability::FileManagement,
                AgentCapability::NetworkOperations,
                AgentCapability::ProcessControl,
                AgentCapability::GitOperations,
                AgentCapability::TerminalAutomation,
                AgentCapability::Learning,
                AgentCapability::WebBrowsing,
            ],
            memory: AgentMemory {
                conversation_history: Vec::new(),
                learned_patterns: HashMap::new(),
                context_embeddings: Vec::new(),
                working_directory: std::env::current_dir()
                    .unwrap_or_else(|_| "/tmp".into())
                    .to_string_lossy()
                    .to_string(),
                session_context: HashMap::new(),
            },
            ollama_url,
            model,
        }
    }

    /// Inspired by aider - AI pair programming capabilities
    pub async fn process_intelligent_request(&mut self, user_input: &str) -> Result<String> {
        info!("Processing intelligent request: {}", user_input);

        // Add to conversation history
        let message = AgentProtocolMessage {
            id: Uuid::new_v4().to_string(),
            role: "user".to_string(),
            content: user_input.to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        self.memory.conversation_history.push(message);

        // Analyze intent using multiple approaches
        let intent = self.analyze_intent(user_input).await?;
        
        // Route to appropriate capability
        match intent {
            AgentIntent::CodeGeneration => self.generate_code(user_input).await,
            AgentIntent::SystemOperation => self.execute_system_operation(user_input).await,
            AgentIntent::FileOperation => self.handle_file_operation(user_input).await,
            AgentIntent::GitOperation => self.handle_git_operation(user_input).await,
            AgentIntent::Learning => self.learn_from_interaction(user_input).await,
            AgentIntent::Conversation => self.generate_conversational_response(user_input).await,
            AgentIntent::WebBrowsing => self.browse_web(user_input).await,
        }
    }

    /// Inspired by AutoNode for cognitive automation
    async fn analyze_intent(&self, input: &str) -> Result<AgentIntent> {
        let input_lower = input.to_lowercase();
        
        // Pattern matching with machine learning influence
        if input_lower.contains("code") || input_lower.contains("implement") || input_lower.contains("function") {
            return Ok(AgentIntent::CodeGeneration);
        }
        
        if input_lower.contains("ps") || input_lower.contains("kill") || input_lower.contains("service") {
            return Ok(AgentIntent::SystemOperation);
        }
        
        if input_lower.contains("file") || input_lower.contains("directory") || input_lower.contains("ls") {
            return Ok(AgentIntent::FileOperation);
        }
        
        if input_lower.contains("git") || input_lower.contains("commit") || input_lower.contains("branch") {
            return Ok(AgentIntent::GitOperation);
        }
        
        if input_lower.contains("learn") || input_lower.contains("remember") || input_lower.contains("pattern") {
            return Ok(AgentIntent::Learning);
        }
        
        if input_lower.contains("search") || input_lower.contains("browse") || input_lower.contains("web") {
            return Ok(AgentIntent::WebBrowsing);
        }

        Ok(AgentIntent::Conversation)
    }

    /// Inspired by gpt-pilot and gpt-engineer for code generation
    async fn generate_code(&mut self, request: &str) -> Result<String> {
        info!("Generating code for: {}", request);
        
        let prompt = format!(
            "You are an expert software engineer. Generate high-quality, production-ready code for:\n\n{}\n\nProvide:\n1. Clean, well-documented code\n2. Error handling\n3. Best practices\n4. Explanation of the approach\n\nCode:",
            request
        );

        self.call_ollama(&prompt).await
    }

    /// Inspired by LocalAI and open-interpreter for system operations
    async fn execute_system_operation(&mut self, request: &str) -> Result<String> {
        info!("Executing system operation: {}", request);
        
        // Safety check - never execute dangerous commands
        if self.is_dangerous_command(request) {
            return Ok("❌ Cannot execute potentially dangerous system commands for security reasons.".to_string());
        }

        // Extract command from natural language
        let command = self.extract_command_from_request(request).await?;
        
        // Execute safely
        match Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()
            .await
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                if output.status.success() {
                    Ok(format!("✅ Command executed successfully:\n\n```\n{}\n```", stdout))
                } else {
                    Ok(format!("❌ Command failed:\n\nError:\n```\n{}\n```", stderr))
                }
            }
            Err(e) => Ok(format!("❌ Failed to execute command: {}", e))
        }
    }

    /// Inspired by repomix and repo-wizard for file operations
    async fn handle_file_operation(&mut self, request: &str) -> Result<String> {
        info!("Handling file operation: {}", request);
        
        let prompt = format!(
            "You are a file system expert. For the request: '{}'\n\nProvide the exact shell commands to:\n1. Safely perform the operation\n2. Include proper error checking\n3. Show verification steps\n\nCommands:",
            request
        );

        self.call_ollama(&prompt).await
    }

    /// Inspired by gitnow and git-auto-deploy for git operations
    async fn handle_git_operation(&mut self, request: &str) -> Result<String> {
        info!("Handling git operation: {}", request);
        
        let prompt = format!(
            "You are a Git expert. For the request: '{}'\n\nProvide:\n1. The correct Git commands\n2. Explanation of what each command does\n3. Any prerequisites or warnings\n4. Best practices\n\nGit commands:",
            request
        );

        self.call_ollama(&prompt).await
    }

    /// Inspired by awesome-ai-coding for learning capabilities
    async fn learn_from_interaction(&mut self, input: &str) -> Result<String> {
        info!("Learning from interaction: {}", input);
        
        // Extract patterns and update memory
        let pattern_key = self.extract_pattern_key(input);
        let current_score = self.memory.learned_patterns.get(&pattern_key).unwrap_or(&0.0);
        self.memory.learned_patterns.insert(pattern_key.clone(), current_score + 1.0);
        
        Ok(format!("📚 Learned new pattern: '{}' (confidence: {:.2})", pattern_key, current_score + 1.0))
    }

    /// Inspired by cognito-ai-search for web browsing
    async fn browse_web(&mut self, request: &str) -> Result<String> {
        info!("Browsing web for: {}", request);
        
        // Use curl to search safely
        match Command::new("curl")
            .arg("-s")
            .arg(&format!("https://api.duckduckgo.com/?q={}&format=json&no_html=1", urlencoding::encode(request)))
            .output()
            .await
        {
            Ok(output) => {
                let response = String::from_utf8_lossy(&output.stdout);
                Ok(format!("🌐 Web search results:\n\n```json\n{}\n```", response))
            }
            Err(e) => Ok(format!("❌ Web search failed: {}", e))
        }
    }

    /// Core AI interaction using local Ollama
    async fn call_ollama(&self, prompt: &str) -> Result<String> {
        let client = reqwest::Client::new();
        
        let request_body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.7,
                "num_predict": 4096
            }
        });

        match client
            .post(&format!("{}/api/generate", self.ollama_url))
            .json(&request_body)
            .send()
            .await
        {
            Ok(response) => {
                let ollama_response: serde_json::Value = response.json().await?;
                Ok(ollama_response["response"].as_str().unwrap_or("No response").to_string())
            }
            Err(e) => {
                error!("Ollama request failed: {}", e);
                Err(anyhow::anyhow!("AI service unavailable: {}", e))
            }
        }
    }

    async fn generate_conversational_response(&mut self, input: &str) -> Result<String> {
        let context = self.build_context();
        let prompt = format!(
            "You are an intelligent terminal AI assistant with advanced capabilities. Context:\n{}\n\nUser: {}\n\nProvide a helpful, intelligent response:",
            context, input
        );

        self.call_ollama(&prompt).await
    }

    fn build_context(&self) -> String {
        format!(
            "Working Directory: {}\nCapabilities: {:?}\nRecent Patterns: {:?}",
            self.memory.working_directory,
            self.capabilities,
            self.memory.learned_patterns.keys().take(5).collect::<Vec<_>>()
        )
    }

    fn is_dangerous_command(&self, command: &str) -> bool {
        let dangerous_patterns = ["rm -rf", "dd if=", ":(){ :|:&};:", "mkfs", "fdisk", "format"];
        dangerous_patterns.iter().any(|pattern| command.contains(pattern))
    }

    async fn extract_command_from_request(&self, request: &str) -> Result<String> {
        let prompt = format!(
            "Extract the exact shell command from this natural language request: '{}'\n\nRespond with ONLY the command, no explanation:",
            request
        );
        
        self.call_ollama(&prompt).await
    }

    fn extract_pattern_key(&self, input: &str) -> String {
        input.split_whitespace().take(3).collect::<Vec<_>>().join(" ")
    }
}

#[derive(Debug, Clone)]
pub enum AgentIntent {
    CodeGeneration,
    SystemOperation,
    FileOperation,
    GitOperation,
    Learning,
    Conversation,
    WebBrowsing,
}

/// Inspired by agents.md for agent configuration
pub struct AgentConfig {
    pub name: String,
    pub description: String,
    pub capabilities: Vec<AgentCapability>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "Nexus Terminal Agent".to_string(),
            description: "Intelligent terminal AI assistant with autonomous capabilities".to_string(),
            capabilities: vec![
                AgentCapability::CodeGeneration,
                AgentCapability::SystemDiagnostics,
                AgentCapability::FileManagement,
                AgentCapability::NetworkOperations,
                AgentCapability::ProcessControl,
                AgentCapability::GitOperations,
                AgentCapability::TerminalAutomation,
                AgentCapability::Learning,
                AgentCapability::WebBrowsing,
            ],
            model: std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:7b".to_string()),
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}
