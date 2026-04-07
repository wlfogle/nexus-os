use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};
use std::sync::Arc;

use crate::ai_optimized::{OptimizedAIService, AIRequest, RequestPriority};
use crate::local_recall::LocalRecallClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub ollama_url: String,
    pub default_model: String,
    pub timeout_seconds: u64,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for AIConfig {
    fn default() -> Self {
        let ollama_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let ollama_port = std::env::var("OLLAMA_PORT").unwrap_or_else(|_| "11434".to_string());
        let ollama_url = format!("http://{}:{}", ollama_host, ollama_port);
        
        Self {
            ollama_url,
            default_model: std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2:1b".to_string()),
            timeout_seconds: std::env::var("AI_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            temperature: std::env::var("AI_TEMPERATURE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.7),
            max_tokens: std::env::var("AI_MAX_TOKENS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4096),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}

#[derive(Debug, Clone)]
pub struct AIService {
    pub client: Client,
    pub config: AIConfig,
    pub optimized_service: Option<Arc<OptimizedAIService>>,
}

impl AIService {
    pub async fn new(config: &AIConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .context("Failed to create HTTP client")?;

        // Initialize optimized AI service
        let optimized_service = match OptimizedAIService::new(config).await {
            Ok(service) => Some(Arc::new(service)),
            Err(e) => {
                debug!("Failed to initialize OptimizedAIService: {}", e);
                None
            }
        };

        let mut service = Self {
            client,
            config: config.clone(),
            optimized_service,
        };

        // Auto-initialize Ollama service if needed
        service.ensure_ollama_running().await?;
        
        // Automatically detect and set the best available model
        service.auto_detect_and_set_model().await?;
        
        Ok(service)
    }

    async fn test_connection(&self) -> Result<()> {
        let url = format!("{}/api/tags", self.config.ollama_url);
        let response = self.client.get(&url).send().await
            .context("Failed to connect to Ollama")?;

        if response.status().is_success() {
            info!("Successfully connected to Ollama at {}", self.config.ollama_url);
            Ok(())
        } else {
            error!("Failed to connect to Ollama: {}", response.status());
            Err(anyhow::anyhow!("Ollama connection failed: {}", response.status()))
        }
    }

    async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<String> {
        let model = model.unwrap_or(&self.config.default_model);
        let url = format!("{}/api/generate", self.config.ollama_url);
        
        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                temperature: self.config.temperature,
                num_predict: self.config.max_tokens,
            },
        };

        debug!("Sending request to Ollama: {:?}", request);

        info!("Sending request to Ollama model '{}' with timeout {}s", model, self.config.timeout_seconds);
        
        let response = match self.client
            .post(&url)
            .json(&request)
            .send()
            .await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to send request to Ollama: {}", e);
                return Err(anyhow::anyhow!("Network error connecting to Ollama: {}", e));
            }
        };

        info!("Received response from Ollama with status: {}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown HTTP error".to_string());
            error!("Ollama HTTP request failed with status {}: {}", status, error_text);
            return Err(anyhow::anyhow!("Ollama HTTP error {}: {}", status, error_text));
        }

        let ollama_response: OllamaResponse = match response.json().await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to parse Ollama JSON response: {}", e);
                return Err(anyhow::anyhow!("Invalid JSON response from Ollama: {}", e));
            }
        };

        info!("Successfully received response from Ollama model '{}': {} characters", model, ollama_response.response.len());
        debug!("Ollama response content: {:?}", ollama_response);
        Ok(ollama_response.response)
    }

    pub async fn chat(&self, message: &str, context: Option<&str>) -> Result<String> {
        // Use optimized AI service if available
        if let Some(ref optimized_service) = self.optimized_service {
            let ai_request = AIRequest::new(message.to_string())
                .with_priority(RequestPriority::Normal)
                .with_model(self.config.default_model.clone());
            
            let ai_request = if let Some(ctx) = context {
                ai_request.with_context(ctx.to_string())
            } else {
                ai_request
            };
            
            match optimized_service.chat_async(&ai_request.prompt, ai_request.context.as_deref()).await {
                Ok(response) => return Ok(response.content),
                Err(e) => {
                    debug!("OptimizedAIService failed, falling back to standard AI: {}", e);
                }
            }
        }
        
        // Build context-aware prompt with RAG integration
        let contextual_prompt = self.build_contextual_prompt(message, context).await?;
        
        // Generate response using AI model
        self.generate(&contextual_prompt, None).await
    }
    
    /// Build a context-aware prompt that incorporates RAG results, system context, and conversation history
    async fn build_contextual_prompt(&self, message: &str, context: Option<&str>) -> Result<String> {
        let mut prompt_parts = Vec::new();
        
        // System prompt - Define the AI's role and capabilities
        prompt_parts.push(format!(
            "You are NexusTerminal AI, an advanced terminal assistant with deep knowledge of Linux systems, programming, and development workflows.
            
**Your Capabilities:**
• Expert knowledge of terminal commands, system administration, and troubleshooting
• Understanding of programming languages, frameworks, and development tools  
• File system operations, process management, and system optimization
• Git workflows, package management, and service administration
• Network diagnostics, security best practices, and automation
• Context-aware assistance based on current directory and recent commands
            
**Response Guidelines:**
• Provide specific, executable commands when appropriate
• Include brief explanations of what commands do
• Suggest alternatives and best practices
• Use markdown formatting for better readability
• Be concise but comprehensive
• Prioritize practical, actionable advice
            
**Current Context:**"
        ));
        
        // Add system context if available
        if let Some(ctx) = context {
            prompt_parts.push(format!("System Context: {}", ctx));
        }
        
        // Try to get RAG context (implement basic RAG lookup)
        match self.get_rag_context(message).await {
            Ok(rag_context) if !rag_context.is_empty() => {
                prompt_parts.push(format!("**Relevant Knowledge:**\n{}", rag_context));
            },
            _ => {}
        }
        
        // Add the user's question
        prompt_parts.push(format!("**User Question:** {}", message));
        
        // Add instructions for response format
        prompt_parts.push(
            "**Instructions:** Provide a helpful, context-aware response. If the question relates to terminal commands, include specific commands with explanations. Use markdown formatting and structure your response clearly.".to_string()
        );
        
        Ok(prompt_parts.join("\n\n"))
    }
    
    /// Get relevant context from LocalRecall RAG system
    async fn get_rag_context(&self, query: &str) -> Result<String> {
        // Initialize LocalRecall client
        let recall_client = LocalRecallClient::default();
        
        // Try to get context from LocalRecall
        match recall_client.get_context_for_prompt(query, Some(5)).await {
            Ok(context) if !context.is_empty() => {
                debug!("Retrieved RAG context for query: {}", query);
                Ok(context)
            }
            Ok(_) => {
                debug!("No relevant context found for query: {}", query);
                Ok(String::new())
            }
            Err(e) => {
                warn!("Failed to retrieve RAG context: {}", e);
                // Don't fail the whole request if RAG is unavailable
                Ok(String::new())
            }
        }
    }
    
    /// Enhanced chat with memory and learning capabilities
    pub async fn chat_with_memory(&self, message: &str, conversation_id: &str, context: Option<&str>) -> Result<String> {
        // Build conversation history prompt
        let mut conversation_prompt = format!(
            "Conversation ID: {}\nPrevious context and memory would be loaded here.\n\n",
            conversation_id
        );
        
        // Add current message context
        conversation_prompt.push_str(&self.build_contextual_prompt(message, context).await?);
        
        // Generate response
        let response = self.generate(&conversation_prompt, None).await?;
        
        // Store conversation in RAG system for future context
        let recall_client = LocalRecallClient::default();
        let messages = vec![("user", message), ("assistant", response.as_str())];
        let _ = recall_client.index_conversation(&messages, context).await;
        
        Ok(response)
    }

    pub async fn complete_command(&self, partial_command: &str, context: &str) -> Result<Vec<String>> {
        let prompt = format!(
            "Given the following terminal context and partial command, suggest 3-5 possible completions:\n\nContext: {}\nPartial command: {}\n\nProvide only the completions, one per line, without explanations:",
            context, partial_command
        );

        let response = self.generate(&prompt, None).await?;
        
        let completions: Vec<String> = response
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .take(5)
            .collect();

        Ok(completions)
    }

    pub async fn explain_error(&self, error_output: &str, command: &str) -> Result<String> {
        let prompt = format!(
            "Analyze this command error and provide a clear explanation and solution:\n\nCommand: {}\nError output: {}\n\nPlease explain:\n1. What went wrong\n2. Why it happened\n3. How to fix it\n4. Alternative approaches if applicable",
            command, error_output
        );

        self.generate(&prompt, None).await
    }

    pub async fn generate_code(&self, description: &str, language: &str) -> Result<String> {
        let prompt = format!(
            "Generate {} code for the following requirement:\n\n{}\n\nProvide clean, well-commented code with proper error handling where appropriate:",
            language, description
        );

        self.generate(&prompt, None).await
    }

    pub async fn generate_commit_message(&self, diff: &str) -> Result<String> {
        let prompt = format!(
            "Generate a concise, descriptive git commit message for these changes:\n\n{}\n\nFollow conventional commit format (type: description). Be specific but concise:",
            diff
        );

        self.generate(&prompt, None).await
    }

    pub async fn analyze_repository(&self, file_tree: &str, readme_content: Option<&str>) -> Result<String> {
        let prompt = if let Some(readme) = readme_content {
            format!(
                "Analyze this repository structure and README:\n\nFile tree:\n{}\n\nREADME:\n{}\n\nProvide insights about:\n1. Project type and technology stack\n2. Architecture and structure\n3. Potential areas for improvement\n4. Development workflow suggestions",
                file_tree, readme
            )
        } else {
            format!(
                "Analyze this repository structure:\n\n{}\n\nProvide insights about:\n1. Project type and technology stack\n2. Architecture and structure\n3. Potential areas for improvement\n4. Development workflow suggestions",
                file_tree
            )
        };

        self.generate(&prompt, Some("codellama:7b")).await
    }

    pub async fn suggest_improvements(&self, code: &str, language: &str) -> Result<String> {
        let prompt = format!(
            "Review this {} code and suggest improvements:\n\n{}\n\nFocus on:\n1. Code quality and best practices\n2. Performance optimizations\n3. Security considerations\n4. Maintainability improvements\n5. Bug prevention",
            language, code
        );

        self.generate(&prompt, Some("codellama:7b")).await
    }

    pub async fn explain_concept(&self, concept: &str, context: &str) -> Result<String> {
        let prompt = format!(
            "Explain the concept '{}' in the context of '{}':\n\nProvide:\n1. A clear definition\n2. How it relates to the context\n3. Practical examples\n4. Common use cases or applications",
            concept, context
        );

        self.generate(&prompt, None).await
    }

    pub async fn get_available_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.config.ollama_url);
        
        let response = self.client.get(&url).send().await
            .context("Failed to fetch available models")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch models: {}", response.status()));
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            models: Vec<Model>,
        }

        #[derive(Deserialize)]
        struct Model {
            name: String,
        }

        let models_response: ModelsResponse = response.json().await
            .context("Failed to parse models response")?;

        Ok(models_response.models.into_iter().map(|m| m.name).collect())
    }

    /// System diagnostic and repair capabilities
    pub async fn diagnose_system_issue(&self, issue_description: &str, system_info: &str) -> Result<String> {
        let prompt = format!(
            "System Issue Diagnosis\n\nUser Report: {}\n\nSystem Information:\n{}\n\nAs a system administrator AI, provide:\n1. Problem analysis and root cause\n2. Step-by-step diagnostic commands to run\n3. Specific fix commands\n4. Verification steps\n5. Prevention measures\n\nBe specific to the Linux distribution and provide actual commands.",
            issue_description, system_info
        );
        
        self.generate(&prompt, Some("codellama:7b")).await
    }

    pub async fn fix_compilation_errors(&self, error_output: &str, project_context: &str) -> Result<String> {
        let prompt = format!(
            "Compilation Error Analysis and Fix\n\nError Output:\n{}\n\nProject Context:\n{}\n\nProvide:\n1. Error analysis\n2. Missing dependencies to install\n3. Configuration changes needed\n4. File modifications required\n5. Complete fix commands\n\nGenerate actual commands that can be executed.",
            error_output, project_context
        );
        
        self.generate(&prompt, Some("codellama:7b")).await
    }

    pub async fn fix_package_issues(&self, package_manager: &str, error_output: &str) -> Result<String> {
        let prompt = format!(
            "Package Management Issue Resolution\n\nPackage Manager: {}\nError Output:\n{}\n\nProvide specific commands to:\n1. Diagnose the package issue\n2. Fix dependency conflicts\n3. Repair package databases\n4. Install missing packages\n5. Verify the fix\n\nInclude actual {} commands.",
            package_manager, error_output, package_manager
        );
        
        self.generate(&prompt, Some("codellama:7b")).await
    }

    pub async fn fix_service_issues(&self, service_name: &str, service_status: &str, logs: &str) -> Result<String> {
        let prompt = format!(
            "Service Issue Diagnosis and Repair\n\nService: {}\nStatus: {}\nLogs:\n{}\n\nProvide:\n1. Issue identification\n2. Configuration file checks\n3. Dependency verification\n4. Repair commands\n5. Service restart sequence\n6. Monitoring commands\n\nInclude systemctl and configuration commands.",
            service_name, service_status, logs
        );
        
        self.generate(&prompt, Some("codellama:7b")).await
    }

    pub async fn fix_environment_setup(&self, tool_name: &str, installation_context: &str, error: &str) -> Result<String> {
        let prompt = format!(
            "Environment Setup and Tool Installation Fix\n\nTool: {}\nContext: {}\nError: {}\n\nProvide complete setup instructions:\n1. Prerequisites installation\n2. Environment variable setup\n3. Path configuration\n4. Tool installation commands\n5. Verification commands\n6. Common troubleshooting\n\nInclude shell configuration and export commands.",
            tool_name, installation_context, error
        );
        
        self.generate(&prompt, Some("codellama:7b")).await
    }

    pub async fn fix_display_issues(&self, display_error: &str, desktop_environment: &str) -> Result<String> {
        let prompt = format!(
            "Display and Desktop Environment Fix\n\nError: {}\nDesktop Environment: {}\n\nProvide solutions for:\n1. X11/Wayland configuration\n2. Display driver issues\n3. Resolution problems\n4. Multi-monitor setup\n5. Desktop environment restart\n6. Configuration file fixes\n\nInclude xrandr, systemctl, and config file commands.",
            display_error, desktop_environment
        );
        
        self.generate(&prompt, Some("codellama:7b")).await
    }

    pub async fn fix_network_issues(&self, network_problem: &str, network_config: &str) -> Result<String> {
        let prompt = format!(
            "Network Issue Diagnosis and Repair\n\nProblem: {}\nNetwork Config: {}\n\nProvide commands for:\n1. Network interface diagnosis\n2. DNS resolution fixes\n3. Firewall configuration\n4. Network service restart\n5. Connection testing\n6. Routing table fixes\n\nInclude ip, systemctl, and network manager commands.",
            network_problem, network_config
        );
        
        self.generate(&prompt, Some("codellama:7b")).await
    }

    pub async fn fix_permission_issues(&self, permission_error: &str, file_context: &str) -> Result<String> {
        let prompt = format!(
            "File Permission and Access Issue Resolution\n\nError: {}\nFile Context: {}\n\nProvide commands for:\n1. Permission analysis\n2. Ownership verification\n3. Group membership checks\n4. Permission fixes\n5. SELinux/AppArmor considerations\n6. Security implications\n\nInclude chmod, chown, ls, and security commands.",
            permission_error, file_context
        );
        
        self.generate(&prompt, Some("codellama:7b")).await
    }

    pub async fn auto_fix_system(&self, issue_type: &str, context: &str) -> Result<Vec<String>> {
        let prompt = format!(
            "Automated System Repair\n\nIssue Type: {}\nContext: {}\n\nGenerate an ordered sequence of shell commands to automatically fix this issue. Each command should be on its own line. Include only executable commands, no explanations.\n\nCommands:",
            issue_type, context
        );
        
        let response = self.generate(&prompt, Some("codellama:7b")).await?;
        
        let commands: Vec<String> = response
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.starts_with('#') || line.is_empty() {
                    None
                } else {
                    Some(line.to_string())
                }
            })
            .collect();
        
        Ok(commands)
    }

    /// Automatically detect and set the best available model
    async fn auto_detect_and_set_model(&mut self) -> Result<()> {
        info!("Auto-detecting best available AI model...");
        
        // Get all available models
        let available_models = match self.get_available_models().await {
            Ok(models) => models,
            Err(e) => {
                warn!("Failed to get available models: {}. Using configured default.", e);
                return Ok(());
            }
        };
        
        if available_models.is_empty() {
            return Err(anyhow::anyhow!("No AI models available! Please ensure Ollama has models installed."));
        }
        
        info!("Found {} available models: {:?}", available_models.len(), available_models);
        
        // Check if the configured model is available
        let configured_model = &self.config.default_model;
        if available_models.iter().any(|m| m == configured_model) {
            info!("Configured model '{}' is available, using it", configured_model);
            return Ok(());
        }
        
        // Smart model selection priority list
        let preferred_models = vec![
            // General purpose models (good balance of performance and size)
            "llama3.2:3b",
            "llama3.1:8b", 
            "qwen2.5:7b",
            "mistral:7b",
            "gemma2:9b",
            
            // Code-specific models
            "codellama:7b",
            "qwen2.5-coder:7b",
            "codeqwen:7b",
            "deepseek-coder:6.7b",
            "magicoder:7b",
            "starcoder2:7b",
            "codegemma:7b",
            
            // Smaller but capable models
            "phi3.5:3.8b",
            "llama3.2:1b",
            "tinyllama:1.1b",
            "tinydolphin:1.1b",
            "stablelm2:1.6b",
            "orca-mini:3b",
            
            // Vision models (if needed)
            "llava:7b",
            "llava:13b",
            "moondream:latest",
        ];
        
        // Find the best available model from our preferred list
        for preferred in &preferred_models {
            if available_models.iter().any(|m| m.starts_with(preferred) || m.contains(preferred.split(':').next().unwrap_or(preferred))) {
                // Find the exact match
                if let Some(exact_model) = available_models.iter().find(|m| m.starts_with(preferred) || m.contains(preferred.split(':').next().unwrap_or(preferred))) {
                    info!("Auto-selected model: '{}' (matched preference: '{}')", exact_model, preferred);
                    self.config.default_model = exact_model.clone();
                    return Ok(());
                }
            }
        }
        
        // If no preferred model found, use the first available model
        let fallback_model = &available_models[0];
        warn!("No preferred models found, using fallback model: '{}'", fallback_model);
        self.config.default_model = fallback_model.clone();
        
        info!("Successfully auto-configured AI model: '{}'", self.config.default_model);
        Ok(())
    }

    /// Automatically ensure Ollama is running and properly configured
    async fn ensure_ollama_running(&self) -> Result<()> {
        info!("Initializing Ollama AI service...");
        
        // Kill any existing Ollama processes first
        let _ = self.kill_existing_ollama().await;
        
        // Install Ollama if not present
        if let Err(e) = self.ensure_ollama_installed().await {
            error!("Failed to install Ollama: {}", e);
            return Err(anyhow::anyhow!("Ollama installation failed: {}", e));
        }
        
        // Start Ollama service with proper configuration
        if let Err(e) = self.start_ollama_service().await {
            error!("Failed to start Ollama service: {}", e);
            return Err(anyhow::anyhow!("Could not start Ollama service: {}", e));
        }
        
        // Wait for service to be ready
        let mut attempts = 0;
        let max_attempts = 30;
        
        while attempts < max_attempts {
            tokio::time::sleep(Duration::from_secs(2)).await;
            if self.test_connection().await.is_ok() {
                info!("Ollama service is ready after {} attempts", attempts + 1);
                break;
            }
            attempts += 1;
            info!("Waiting for Ollama to start... attempt {}/{}", attempts, max_attempts);
        }
        
        if attempts >= max_attempts {
            return Err(anyhow::anyhow!("Ollama failed to start after {} attempts", max_attempts));
        }
        
        // Ensure default model is available
        self.ensure_default_model_available().await?;
        
        info!("Ollama AI service fully initialized and ready");
        Ok(())
    }

    /// Kill any existing Ollama processes
    async fn kill_existing_ollama(&self) -> Result<()> {
        use tokio::process::Command;
        
        info!("Cleaning up any existing Ollama processes...");
        
        let kill_commands = [
            ("pkill", vec!["-f", "ollama"]),
            ("killall", vec!["ollama"]),
            ("sudo", vec!["pkill", "-f", "ollama"]),
            ("sudo", vec!["killall", "ollama"]),
        ];
        
        for (cmd, args) in &kill_commands {
            let _ = Command::new(cmd)
                .args(args)
                .output()
                .await;
        }
        
        // Wait a moment for processes to terminate
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }
    
    /// Ensure Ollama is installed
    async fn ensure_ollama_installed(&self) -> Result<()> {
        use tokio::process::Command;
        use std::path::Path;
        
        info!("Checking Ollama installation...");
        
        let ollama_paths = [
            "/usr/local/bin/ollama",
            "/usr/bin/ollama",
            "/opt/ollama/bin/ollama",
            "./bin/ollama",
            "../bin/ollama",
        ];
        
        // Check if Ollama is already installed
        for path in &ollama_paths {
            if Path::new(path).exists() {
                info!("Found Ollama at: {}", path);
                return Ok(());
            }
        }
        
        // Try to install Ollama if not found
        info!("Ollama not found, attempting to install...");
        
        let install_commands = [
            // Standard installation
            ("curl", vec!["-fsSL", "https://ollama.ai/install.sh", "|", "sh"]),
            // Alternative installation methods
            ("sudo", vec!["pacman", "-S", "--noconfirm", "ollama"]),
            ("yay", vec!["-S", "--noconfirm", "ollama"]),
            ("sudo", vec!["apt", "install", "-y", "ollama"]),
            ("sudo", vec!["dnf", "install", "-y", "ollama"]),
        ];
        
        for (cmd, args) in &install_commands {
            info!("Trying to install Ollama with: {} {}", cmd, args.join(" "));
            
            let mut command = Command::new(cmd);
            command.args(args);
            
            match command.output().await {
                Ok(output) if output.status.success() => {
                    info!("Successfully installed Ollama");
                    // Verify installation
                    for path in &ollama_paths {
                        if Path::new(path).exists() {
                            return Ok(());
                        }
                    }
                }
                Ok(output) => {
                    let error = String::from_utf8_lossy(&output.stderr);
                    debug!("Installation attempt failed: {}", error);
                }
                Err(e) => {
                    debug!("Failed to execute install command: {}", e);
                }
            }
        }
        
        Err(anyhow::anyhow!("Could not install Ollama. Please install it manually."))
    }
    
    /// Start Ollama service using system commands
    async fn start_ollama_service(&self) -> Result<()> {
        use tokio::process::Command;
        use std::path::Path;
        
        info!("Starting Ollama service...");
        
        // Use configurable models directory with fallback
        let models_dir = std::env::var("OLLAMA_MODELS")
            .or_else(|_| std::env::var("OLLAMA_MODELS_PATH"))
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                format!("{}/.ollama/models", home)
            });
        
        if !Path::new(&models_dir).exists() {
            return Err(anyhow::anyhow!("Models directory {} does not exist! Set OLLAMA_MODELS environment variable.", models_dir));
        }
        
        info!("Using models directory: {}", models_dir);
        
        let ollama_paths = [
            "/usr/local/bin/ollama",
            "/usr/bin/ollama",
            "/opt/ollama/bin/ollama",
            "ollama",
        ];
        
        let mut ollama_cmd = None;
        for path in &ollama_paths {
            if Path::new(path).exists() || *path == "ollama" {
                ollama_cmd = Some(*path);
                break;
            }
        }
        
        let ollama_binary = ollama_cmd.ok_or_else(|| anyhow::anyhow!("Ollama binary not found"))?;
        
        info!("Starting Ollama with binary: {}", ollama_binary);
        
        // Start Ollama server in background
        let mut command = Command::new(ollama_binary);
        command.arg("serve");
        command.env("OLLAMA_HOST", "0.0.0.0");
        command.env("OLLAMA_PORT", "11434");
        command.env("OLLAMA_MODELS", models_dir);
        command.env("OLLAMA_KEEP_ALIVE", "24h");
        command.env("OLLAMA_MAX_LOADED_MODELS", "1");
        
        // Redirect output to log files
        let log_file = std::fs::File::create("/tmp/ollama.log")
            .map_err(|e| anyhow::anyhow!("Failed to create log file: {}", e))?;
        command.stdout(log_file.try_clone().unwrap());
        command.stderr(log_file);
        
        match command.spawn() {
            Ok(mut child) => {
                // Don't wait for the serve command, let it run in background
                tokio::spawn(async move {
                    let _ = child.wait().await;
                });
                info!("Ollama service started successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to start Ollama service: {}", e);
                Err(anyhow::anyhow!("Could not start Ollama: {}", e))
            }
        }
    }

    /// Ensure the default model is available
    async fn ensure_default_model_available(&self) -> Result<()> {
        info!("Ensuring default model '{}' is available...", self.config.default_model);
        
        // Check what models are already available
        if let Ok(available_models) = self.get_available_models().await {
            info!("Available models: {:?}", available_models);
            
            // Check if our default model is already available
            if available_models.iter().any(|m| m.contains(self.config.default_model.split(':').next().unwrap_or(&self.config.default_model))) {
                info!("Default model '{}' is already available", self.config.default_model);
                return Ok(());
            }
        }
        
        // First, try to pull the model
        if let Err(e) = self.pull_default_model().await {
            info!("Could not pull model '{}': {}. Model might already be available locally.", self.config.default_model, e);
            
            // Try alternative lightweight models
            let alternative_models = [
                "llama3.2:1b",
                "phi3:mini", 
                "tinyllama:1.1b",
                "qwen2.5:0.5b",
                "gemma2:2b",
            ];
            
            for alt_model in &alternative_models {
                info!("Trying alternative model: {}", alt_model);
                if self.pull_model(alt_model).await.is_ok() {
                    info!("Successfully pulled alternative model: {}", alt_model);
                    // Update config to use this model
                    return Ok(());
                }
            }
            
            return Err(anyhow::anyhow!("Could not pull any suitable AI model. Please check your internet connection."));
        }
        
        info!("Default model '{}' is ready", self.config.default_model);
        Ok(())
    }

    /// Pull the default model if not available
    async fn pull_default_model(&self) -> Result<()> {
        self.pull_model(&self.config.default_model).await
    }
    
    /// Pull a specific model
    async fn pull_model(&self, model: &str) -> Result<()> {
        use tokio::process::Command;
        use std::path::Path;
        
        info!("Attempting to pull model: {}", model);
        
        let ollama_paths = [
            "/usr/local/bin/ollama",
            "/usr/bin/ollama",
            "ollama",
        ];
        
        for ollama_cmd in &ollama_paths {
            if Path::new(ollama_cmd).exists() || *ollama_cmd == "ollama" {
                info!("Pulling model '{}' with: {}", model, ollama_cmd);
                
                let mut command = Command::new(ollama_cmd);
                command.args(["pull", model]);
                command.env("OLLAMA_HOST", "127.0.0.1:11434");
                
                match tokio::time::timeout(Duration::from_secs(300), command.output()).await {
                    Ok(Ok(output)) => {
                        if output.status.success() {
                            info!("Successfully pulled model '{}'", model);
                            return Ok(());
                        } else {
                            let error = String::from_utf8_lossy(&output.stderr);
                            info!("Model pull failed for '{}': {}", model, error);
                        }
                    }
                    Ok(Err(e)) => {
                        info!("Command execution failed for '{}': {}", model, e);
                    }
                    Err(_) => {
                        info!("Model pull timed out for '{}'", model);
                    }
                }
            }
        }
        
        Err(anyhow::anyhow!("Could not pull model '{}'", model))
    }

    /// Submit a high-priority request through the optimized service
    pub async fn submit_priority_request(&self, prompt: String, priority: RequestPriority) -> Result<String> {
        if let Some(optimized) = &self.optimized_service {
            let request = AIRequest::new_with_options(
                prompt,
                self.config.default_model.clone(),
                priority,
                self.config.max_tokens,
                self.config.temperature,
            );
            
            let mut rx = optimized.submit_request(request).await?;
            match rx.recv().await {
                Some(response) => {
                    if response.success {
                        Ok(response.content)
                    } else {
                        Err(anyhow::anyhow!(response.error.unwrap_or("Unknown error".to_string())))
                    }
                }
                None => Err(anyhow::anyhow!("No response received"))
            }
        } else {
            // Fallback to direct generation
            self.generate(&prompt, None).await
        }
    }
    
    /// Process multiple requests with intelligent batching and prioritization
    pub async fn batch_process_requests(&self, requests: Vec<(String, RequestPriority)>) -> Result<Vec<String>> {
        if let Some(optimized) = &self.optimized_service {
            let mut request_receivers = Vec::new();
            
            // Submit all requests
            for (prompt, priority) in requests {
                let request = AIRequest::new_with_options(
                    prompt,
                    self.config.default_model.clone(),
                    priority,
                    self.config.max_tokens,
                    self.config.temperature,
                );
                
                let rx = optimized.submit_request(request).await?;
                request_receivers.push(rx);
            }
            
            // Collect all responses
            let mut responses = Vec::new();
            for mut rx in request_receivers {
                match rx.recv().await {
                    Some(response) => {
                        if response.success {
                            responses.push(response.content);
                        } else {
                            responses.push(format!("Error: {}", response.error.unwrap_or("Unknown error".to_string())));
                        }
                    }
                    None => {
                        responses.push("Error: No response received".to_string());
                    }
                }
            }
            
            Ok(responses)
        } else {
            // Fallback to sequential processing
            let mut responses = Vec::new();
            for (prompt, _priority) in requests {
                let response = self.generate(&prompt, None).await?;
                responses.push(response);
            }
            Ok(responses)
        }
    }
    
    /// Get service statistics and performance metrics
    pub async fn get_service_stats(&self) -> Result<String> {
        if let Some(optimized) = &self.optimized_service {
            Ok(optimized.get_stats().await)
        } else {
            Ok("Optimized service not available".to_string())
        }
    }
    
    /// Clear completed requests from the optimized service
    pub async fn clear_completed_requests(&self) -> Result<()> {
        if let Some(optimized) = &self.optimized_service {
            optimized.clear_completed().await;
        }
        Ok(())
    }
    
    /// Smart error analysis using optimized service for critical fixes
    pub async fn analyze_critical_error(&self, error_output: &str, command: &str, context: &str) -> Result<String> {
        let prompt = format!(
            "CRITICAL ERROR ANALYSIS\n\nCommand: {}\nError: {}\nContext: {}\n\nThis is a high-priority error analysis. Provide:\n1. Immediate impact assessment\n2. Rapid diagnostic steps\n3. Emergency fix commands\n4. Risk mitigation\n5. Recovery procedures\n\nPrioritize speed and accuracy for production systems.",
            command, error_output, context
        );
        
        self.submit_priority_request(prompt, RequestPriority::Critical).await
    }
}

impl Default for AIService {
    fn default() -> Self {
        let config = AIConfig::default();
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self {
            client,
            config,
            optimized_service: None, // Can't create OptimizedAIService without async context
        }
    }
}
