use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};
use crate::ai::{SystemState, WorkloadType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub category: IntentCategory,
    pub action: String,
    pub parameters: HashMap<String, String>,
    pub confidence: f64,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntentCategory {
    SystemOptimization,
    FileManagement,
    PackageManagement,
    Monitoring,
    Backup,
    Hardware,
    Maintenance,
    Query,
    Conversation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: EntityType,
    pub value: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Number,
    Percentage,
    Duration,
    Process,
    Package,
    Path,
    SystemComponent,
    Action,
}

pub struct NLPProcessor {
    intent_patterns: HashMap<IntentCategory, Vec<IntentPattern>>,
    entity_extractors: HashMap<EntityType, Regex>,
    response_templates: HashMap<String, Vec<String>>,
    conversation_context: ConversationContext,
    system_vocabulary: SystemVocabulary,
}

#[derive(Debug, Clone)]
struct IntentPattern {
    pattern: Regex,
    action: String,
    confidence: f64,
    parameter_extractors: HashMap<String, Regex>,
}

#[derive(Debug, Clone)]
struct ConversationContext {
    last_intent: Option<Intent>,
    last_system_state: Option<SystemState>,
    conversation_history: Vec<(String, String)>, // (user_input, ai_response)
    user_preferences: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct SystemVocabulary {
    processes: Vec<String>,
    packages: Vec<String>,
    system_components: Vec<String>,
    actions: Vec<String>,
    synonyms: HashMap<String, Vec<String>>,
}

impl NLPProcessor {
    pub async fn new_with_sysadmin_vocab() -> Result<Self, Box<dyn std::error::Error>> {
        info!("🗣️ Initializing NLP Processor with system administration vocabulary...");
        
        let mut processor = Self {
            intent_patterns: HashMap::new(),
            entity_extractors: HashMap::new(),
            response_templates: HashMap::new(),
            conversation_context: ConversationContext {
                last_intent: None,
                last_system_state: None,
                conversation_history: Vec::new(),
                user_preferences: HashMap::new(),
            },
            system_vocabulary: SystemVocabulary {
                processes: vec![
                    "firefox".to_string(), "chrome".to_string(), "code".to_string(),
                    "steam".to_string(), "docker".to_string(), "nginx".to_string(),
                    "postgres".to_string(), "redis".to_string(), "node".to_string(),
                ],
                packages: vec![
                    "firefox".to_string(), "chromium".to_string(), "code".to_string(),
                    "docker".to_string(), "nginx".to_string(), "postgres".to_string(),
                    "rust".to_string(), "python".to_string(), "nodejs".to_string(),
                ],
                system_components: vec![
                    "cpu".to_string(), "memory".to_string(), "ram".to_string(),
                    "disk".to_string(), "gpu".to_string(), "temperature".to_string(),
                    "fans".to_string(), "kernel".to_string(),
                ],
                actions: vec![
                    "optimize".to_string(), "clean".to_string(), "update".to_string(),
                    "backup".to_string(), "monitor".to_string(), "install".to_string(),
                    "remove".to_string(), "restart".to_string(),
                ],
                synonyms: HashMap::new(),
            },
        };
        
        processor.initialize_intent_patterns().await?;
        processor.initialize_entity_extractors().await?;
        processor.initialize_response_templates().await?;
        processor.initialize_synonyms().await?;
        
        Ok(processor)
    }
    
    async fn initialize_intent_patterns(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // System Optimization patterns
        let mut optimization_patterns = Vec::new();
        
        optimization_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(optimize|improve|speed up|boost).*?(cpu|processor|performance)")?,
            action: "optimize_cpu".to_string(),
            confidence: 0.9,
            parameter_extractors: HashMap::new(),
        });
        
        optimization_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(optimize|clean|free up|clear).*?(memory|ram)")?,
            action: "optimize_memory".to_string(),
            confidence: 0.9,
            parameter_extractors: HashMap::new(),
        });
        
        optimization_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(optimize|improve).*?(system|performance)")?,
            action: "optimize_system".to_string(),
            confidence: 0.8,
            parameter_extractors: HashMap::new(),
        });
        
        optimization_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(set|change).*?cpu.*?(governor|frequency|performance)")?,
            action: "set_cpu_governor".to_string(),
            confidence: 0.95,
            parameter_extractors: {
                let mut extractors = HashMap::new();
                extractors.insert("governor".to_string(), 
                    Regex::new(r"(?i)(performance|powersave|ondemand|conservative|userspace|schedutil)").unwrap());
                extractors
            },
        });
        
        optimization_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(reduce|lower|decrease).*?(temperature|temp|heat)")?,
            action: "reduce_temperature".to_string(),
            confidence: 0.85,
            parameter_extractors: HashMap::new(),
        });
        
        self.intent_patterns.insert(IntentCategory::SystemOptimization, optimization_patterns);
        
        // File Management patterns
        let mut file_patterns = Vec::new();
        
        file_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(clean|delete|remove).*?(temp|temporary|cache).*?(files?)")?,
            action: "clean_temp_files".to_string(),
            confidence: 0.9,
            parameter_extractors: HashMap::new(),
        });
        
        file_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(organize|sort|arrange).*?(files?)")?,
            action: "organize_files".to_string(),
            confidence: 0.85,
            parameter_extractors: HashMap::new(),
        });
        
        file_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(find|search for).*?(duplicate|duplicated).*?(files?)")?,
            action: "find_duplicates".to_string(),
            confidence: 0.9,
            parameter_extractors: HashMap::new(),
        });
        
        file_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(analyze|check).*?(disk|storage).*?(usage|space)")?,
            action: "analyze_disk_usage".to_string(),
            confidence: 0.9,
            parameter_extractors: HashMap::new(),
        });
        
        self.intent_patterns.insert(IntentCategory::FileManagement, file_patterns);
        
        // Package Management patterns
        let mut package_patterns = Vec::new();
        
        package_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(update|upgrade).*?(system|packages?)")?,
            action: "update_packages".to_string(),
            confidence: 0.95,
            parameter_extractors: HashMap::new(),
        });
        
        package_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(install|add).*?package")?,
            action: "install_package".to_string(),
            confidence: 0.9,
            parameter_extractors: {
                let mut extractors = HashMap::new();
                extractors.insert("package".to_string(), 
                    Regex::new(r"(?i)(?:install|add)\s+(?:package\s+)?([a-zA-Z0-9\-_]+)").unwrap());
                extractors
            },
        });
        
        package_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(remove|uninstall|delete).*?package")?,
            action: "remove_package".to_string(),
            confidence: 0.9,
            parameter_extractors: {
                let mut extractors = HashMap::new();
                extractors.insert("package".to_string(), 
                    Regex::new(r"(?i)(?:remove|uninstall|delete)\s+(?:package\s+)?([a-zA-Z0-9\-_]+)").unwrap());
                extractors
            },
        });
        
        package_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(search|find).*?package")?,
            action: "search_packages".to_string(),
            confidence: 0.85,
            parameter_extractors: {
                let mut extractors = HashMap::new();
                extractors.insert("query".to_string(), 
                    Regex::new(r"(?i)(?:search|find).*?package.*?([a-zA-Z0-9\-_]+)").unwrap());
                extractors
            },
        });
        
        self.intent_patterns.insert(IntentCategory::PackageManagement, package_patterns);
        
        // Monitoring patterns
        let mut monitoring_patterns = Vec::new();
        
        monitoring_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(show|display|check).*?(system|hardware).*?(status|info)")?,
            action: "show_system_info".to_string(),
            confidence: 0.9,
            parameter_extractors: HashMap::new(),
        });
        
        monitoring_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(monitor|watch|track).*?(cpu|memory|temperature|performance)")?,
            action: "start_monitoring".to_string(),
            confidence: 0.85,
            parameter_extractors: HashMap::new(),
        });
        
        monitoring_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(what.*?is.*?|show.*?|check.*?)(cpu|memory|disk|temperature)")?,
            action: "get_system_metric".to_string(),
            confidence: 0.9,
            parameter_extractors: {
                let mut extractors = HashMap::new();
                extractors.insert("metric".to_string(), 
                    Regex::new(r"(?i)(cpu|memory|disk|temperature|ram|gpu)").unwrap());
                extractors
            },
        });
        
        self.intent_patterns.insert(IntentCategory::Monitoring, monitoring_patterns);
        
        // Backup patterns
        let mut backup_patterns = Vec::new();
        
        backup_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(backup|save).*?(system|data|files?)")?,
            action: "backup_system".to_string(),
            confidence: 0.9,
            parameter_extractors: HashMap::new(),
        });
        
        backup_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(restore|recover).*?(system|data|files?)")?,
            action: "restore_system".to_string(),
            confidence: 0.9,
            parameter_extractors: HashMap::new(),
        });
        
        self.intent_patterns.insert(IntentCategory::Backup, backup_patterns);
        
        // Maintenance patterns
        let mut maintenance_patterns = Vec::new();
        
        maintenance_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(clean|cleanup).*?(system)")?,
            action: "clean_system".to_string(),
            confidence: 0.9,
            parameter_extractors: HashMap::new(),
        });
        
        maintenance_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(restart|reboot).*?(system)")?,
            action: "restart_system".to_string(),
            confidence: 0.95,
            parameter_extractors: HashMap::new(),
        });
        
        self.intent_patterns.insert(IntentCategory::Maintenance, maintenance_patterns);
        
        // Query patterns
        let mut query_patterns = Vec::new();
        
        query_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(how.*?do.*?i|how.*?can.*?i|help.*?me)")?,
            action: "provide_help".to_string(),
            confidence: 0.8,
            parameter_extractors: HashMap::new(),
        });
        
        query_patterns.push(IntentPattern {
            pattern: Regex::new(r"(?i)(what.*?should.*?i|recommend|suggest)")?,
            action: "provide_recommendation".to_string(),
            confidence: 0.8,
            parameter_extractors: HashMap::new(),
        });
        
        self.intent_patterns.insert(IntentCategory::Query, query_patterns);
        
        Ok(())
    }
    
    async fn initialize_entity_extractors(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.entity_extractors.insert(EntityType::Number, 
            Regex::new(r"\b(\d+(?:\.\d+)?)\b")?);
        
        self.entity_extractors.insert(EntityType::Percentage, 
            Regex::new(r"\b(\d+(?:\.\d+)?)%\b")?);
        
        self.entity_extractors.insert(EntityType::Duration, 
            Regex::new(r"\b(\d+)\s*(second|minute|hour|day|week|month)s?\b")?);
        
        self.entity_extractors.insert(EntityType::Process, 
            Regex::new(r"\b(firefox|chrome|code|steam|docker|nginx|postgres|redis|node)\b")?);
        
        self.entity_extractors.insert(EntityType::Package, 
            Regex::new(r"\b([a-zA-Z0-9\-_]+)\b")?);
        
        self.entity_extractors.insert(EntityType::Path, 
            Regex::new(r"(/[^\s]*|~[^\s]*|[A-Za-z]:[^\s]*)")?);
        
        self.entity_extractors.insert(EntityType::SystemComponent, 
            Regex::new(r"\b(cpu|memory|ram|disk|gpu|temperature|temp|fan|kernel)\b")?);
        
        Ok(())
    }
    
    async fn initialize_response_templates(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.response_templates.insert("optimize_cpu".to_string(), vec![
            "🚀 Optimizing CPU performance for your i9-13900HX...".to_string(),
            "⚡ Applying CPU optimizations to boost performance...".to_string(),
            "🔧 Tuning CPU settings for maximum efficiency...".to_string(),
        ]);
        
        self.response_templates.insert("optimize_memory".to_string(), vec![
            "🧠 Freeing up memory to improve system responsiveness...".to_string(),
            "💾 Optimizing RAM usage across your 64GB system...".to_string(),
            "🗑️ Clearing unnecessary memory usage...".to_string(),
        ]);
        
        self.response_templates.insert("clean_system".to_string(), vec![
            "🧹 Starting comprehensive system cleanup...".to_string(),
            "🗑️ Removing temporary files and clearing caches...".to_string(),
            "✨ Cleaning up your Garuda Linux system...".to_string(),
        ]);
        
        self.response_templates.insert("update_packages".to_string(), vec![
            "📦 Updating system packages with pacman...".to_string(),
            "⬆️ Checking for package updates on Garuda Linux...".to_string(),
            "🔄 Synchronizing package databases and updating...".to_string(),
        ]);
        
        self.response_templates.insert("show_system_info".to_string(), vec![
            "📊 Here's your current system status:".to_string(),
            "💻 System information for your i9-13900HX setup:".to_string(),
            "🖥️ Current hardware status:".to_string(),
        ]);
        
        self.response_templates.insert("error".to_string(), vec![
            "❌ I encountered an error while processing your request.".to_string(),
            "⚠️ Something went wrong. Let me try a different approach.".to_string(),
            "🔧 I'm having trouble with that request. Can you rephrase it?".to_string(),
        ]);
        
        self.response_templates.insert("success".to_string(), vec![
            "✅ Task completed successfully!".to_string(),
            "🎉 Done! Your system has been optimized.".to_string(),
            "✨ Operation completed successfully.".to_string(),
        ]);
        
        self.response_templates.insert("help".to_string(), vec![
            "🤖 I can help you with system optimization, file management, package updates, monitoring, and more!".to_string(),
            "💡 Try asking me to 'optimize CPU', 'clean system', 'update packages', or 'show system status'.".to_string(),
            "🔍 I'm here to help manage your Garuda Linux system. What would you like me to do?".to_string(),
        ]);
        
        Ok(())
    }
    
    async fn initialize_synonyms(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut synonyms = HashMap::new();
        
        synonyms.insert("optimize".to_string(), vec![
            "improve".to_string(), "boost".to_string(), "enhance".to_string(), 
            "speed up".to_string(), "tune".to_string()
        ]);
        
        synonyms.insert("clean".to_string(), vec![
            "cleanup".to_string(), "clear".to_string(), "remove".to_string(), 
            "delete".to_string(), "purge".to_string()
        ]);
        
        synonyms.insert("monitor".to_string(), vec![
            "watch".to_string(), "track".to_string(), "observe".to_string(), 
            "check".to_string(), "view".to_string()
        ]);
        
        synonyms.insert("memory".to_string(), vec![
            "ram".to_string(), "memory".to_string()
        ]);
        
        synonyms.insert("cpu".to_string(), vec![
            "processor".to_string(), "cpu".to_string()
        ]);
        
        self.system_vocabulary.synonyms = synonyms;
        
        Ok(())
    }
    
    pub async fn parse_intent(&mut self, input: &str) -> Result<Intent, Box<dyn std::error::Error>> {
        debug!("🔍 Parsing intent from input: {}", input);
        
        let normalized_input = self.normalize_input(input);
        let mut best_intent: Option<Intent> = None;
        let mut best_confidence = 0.0;
        
        // Try to match against all intent patterns
        for (category, patterns) in &self.intent_patterns {
            for pattern in patterns {
                if let Some(matches) = pattern.pattern.captures(&normalized_input) {
                    let mut parameters = HashMap::new();
                    
                    // Extract parameters using parameter extractors
                    for (param_name, extractor) in &pattern.parameter_extractors {
                        if let Some(param_match) = extractor.captures(&normalized_input) {
                            if let Some(value) = param_match.get(1) {
                                parameters.insert(param_name.clone(), value.as_str().to_string());
                            }
                        }
                    }
                    
                    // Extract entities
                    let entities = self.extract_entities(&normalized_input)?;
                    
                    let intent = Intent {
                        category: category.clone(),
                        action: pattern.action.clone(),
                        parameters,
                        confidence: pattern.confidence,
                        entities,
                    };
                    
                    if pattern.confidence > best_confidence {
                        best_confidence = pattern.confidence;
                        best_intent = Some(intent);
                    }
                }
            }
        }
        
        // If no specific intent found, try to extract a general query intent
        if best_intent.is_none() {
            let entities = self.extract_entities(&normalized_input)?;
            best_intent = Some(Intent {
                category: IntentCategory::Conversation,
                action: "general_query".to_string(),
                parameters: HashMap::new(),
                confidence: 0.3,
                entities,
            });
        }
        
        let intent = best_intent.unwrap();
        
        // Update conversation context
        self.conversation_context.last_intent = Some(intent.clone());
        
        Ok(intent)
    }
    
    fn normalize_input(&self, input: &str) -> String {
        let mut normalized = input.to_lowercase();
        
        // Apply synonym replacements
        for (canonical, synonyms) in &self.system_vocabulary.synonyms {
            for synonym in synonyms {
                if normalized.contains(synonym) {
                    normalized = normalized.replace(synonym, canonical);
                }
            }
        }
        
        normalized
    }
    
    fn extract_entities(&self, text: &str) -> Result<Vec<Entity>, Box<dyn std::error::Error>> {
        let mut entities = Vec::new();
        
        for (entity_type, regex) in &self.entity_extractors {
            for captures in regex.captures_iter(text) {
                if let Some(matched) = captures.get(1) {
                    entities.push(Entity {
                        entity_type: entity_type.clone(),
                        value: matched.as_str().to_string(),
                        start_pos: matched.start(),
                        end_pos: matched.end(),
                    });
                }
            }
        }
        
        Ok(entities)
    }
    
    pub async fn generate_response(&mut self, action: &str, system_state: &SystemState) -> Result<String, Box<dyn std::error::Error>> {
        debug!("💬 Generating response for action: {}", action);
        
        // Update conversation context
        self.conversation_context.last_system_state = Some(system_state.clone());
        
        let base_response = self.get_response_template(action);
        let contextual_response = self.add_context_to_response(&base_response, system_state).await?;
        
        // Add the conversation to history
        if let Some(last_user_input) = self.conversation_context.conversation_history.last() {
            self.conversation_context.conversation_history.push((
                last_user_input.0.clone(),
                contextual_response.clone()
            ));
        }
        
        Ok(contextual_response)
    }
    
    fn get_response_template(&self, action: &str) -> String {
        if let Some(templates) = self.response_templates.get(action) {
            // Select a random template for variety
            let index = rand::random::<usize>() % templates.len();
            templates[index].clone()
        } else {
            "I'll help you with that.".to_string()
        }
    }
    
    async fn add_context_to_response(&self, base_response: &str, system_state: &SystemState) -> Result<String, Box<dyn std::error::Error>> {
        let mut response = base_response.to_string();
        
        // Add system-specific context
        match system_state.current_workload {
            WorkloadType::Gaming => {
                response += " I notice you're in gaming mode, so I'll prioritize performance optimizations.";
            },
            WorkloadType::Development => {
                response += " Since you're developing, I'll make sure to preserve your workflow.";
            },
            WorkloadType::Media => {
                response += " I'll optimize for media processing while you're working with media files.";
            },
            WorkloadType::SystemMaintenance => {
                response += " Perfect timing for system maintenance tasks!";
            },
            WorkloadType::Idle => {
                response += " Great time to run some maintenance tasks while the system is idle.";
            },
        }
        
        // Add specific metrics if relevant
        if system_state.cpu_usage > 80.0 {
            response += &format!(" I see your CPU usage is high at {:.1}%, so this should help.", system_state.cpu_usage);
        }
        
        if system_state.memory_usage > 80.0 {
            response += &format!(" With memory usage at {:.1}%, this optimization is definitely needed.", system_state.memory_usage);
        }
        
        if system_state.temperature > 80.0 {
            response += &format!(" Your CPU temperature is {:.1}°C, so I'll focus on cooling optimizations.", system_state.temperature);
        }
        
        Ok(response)
    }
    
    pub async fn handle_conversational_response(&mut self, user_input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let normalized_input = user_input.to_lowercase();
        
        // Handle common conversational patterns
        if normalized_input.contains("thank") {
            return Ok("🙂 You're welcome! I'm here whenever you need system assistance.".to_string());
        }
        
        if normalized_input.contains("hello") || normalized_input.contains("hi") {
            return Ok("👋 Hello! I'm your AI System Administrator. How can I help optimize your Garuda Linux system today?".to_string());
        }
        
        if normalized_input.contains("goodbye") || normalized_input.contains("bye") {
            return Ok("👋 Goodbye! I'll keep monitoring your system in the background.".to_string());
        }
        
        if normalized_input.contains("how are you") {
            return Ok("🤖 I'm running perfectly! Your system is looking good - how can I help you today?".to_string());
        }
        
        // If no conversational pattern matches, provide help
        Ok("🤖 I'm here to help manage your i9-13900HX Garuda Linux system. Try asking me to optimize performance, clean up files, update packages, or monitor system status!".to_string())
    }
    
    pub fn get_conversation_context(&self) -> &ConversationContext {
        &self.conversation_context
    }
    
    pub fn update_user_preference(&mut self, key: String, value: String) {
        self.conversation_context.user_preferences.insert(key, value);
    }
    
    pub fn get_nlp_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        stats.insert("total_intent_patterns".to_string(), 
            self.intent_patterns.values().map(|v| v.len()).sum::<usize>() as f64);
        stats.insert("conversation_history_length".to_string(), 
            self.conversation_context.conversation_history.len() as f64);
        stats.insert("user_preferences_count".to_string(), 
            self.conversation_context.user_preferences.len() as f64);
        stats.insert("vocabulary_size".to_string(), 
            self.system_vocabulary.processes.len() + 
            self.system_vocabulary.packages.len() + 
            self.system_vocabulary.system_components.len() + 
            self.system_vocabulary.actions.len());
        
        stats
    }
}
