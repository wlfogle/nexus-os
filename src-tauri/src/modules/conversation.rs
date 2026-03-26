use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub token_count: Option<u32>,
    pub model: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub system_prompt: Option<String>,
    pub context_window: u32,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub message_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub model: String,
    pub tags: Vec<String>,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
    pub system_prompt: Option<String>,
    pub context_window: u32,
    pub auto_title: bool,
    pub save_history: bool,
    pub stream_response: bool,
}

impl Default for ConversationConfig {
    fn default() -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            temperature: 0.7,
            max_tokens: None,
            system_prompt: Some("You are a helpful AI assistant with access to various tools and capabilities. You can help with coding, analysis, file operations, and many other tasks.".to_string()),
            context_window: 4096,
            auto_title: true,
            save_history: true,
            stream_response: true,
        }
    }
}

pub struct ConversationManager {
    conversations: Arc<RwLock<HashMap<String, Conversation>>>,
    active_conversation: Arc<Mutex<Option<String>>>,
    config: Arc<RwLock<ConversationConfig>>,
    ai_module: Arc<crate::modules::ai_module::AIModule>,
    database: Arc<crate::modules::database::Database>,
}

impl ConversationManager {
    pub async fn new(
        ai_module: Arc<crate::modules::ai_module::AIModule>,
        database: Arc<crate::modules::database::Database>,
    ) -> Result<Self, AppError> {
        let manager = Self {
            conversations: Arc::new(RwLock::new(HashMap::new())),
            active_conversation: Arc::new(Mutex::new(None)),
            config: Arc::new(RwLock::new(ConversationConfig::default())),
            ai_module,
            database,
        };

        // Load conversations from database
        manager.load_conversations().await?;

        Ok(manager)
    }

    pub async fn create_conversation(&self, title: Option<String>) -> Result<String, AppError> {
        let id = Uuid::new_v4().to_string();
        let config = self.config.read().await;
        
        let conversation = Conversation {
            id: id.clone(),
            title: title.unwrap_or_else(|| "New Conversation".to_string()),
            messages: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            model: config.model.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            system_prompt: config.system_prompt.clone(),
            context_window: config.context_window,
            tags: Vec::new(),
            metadata: HashMap::new(),
        };

        // Save to database
        self.save_conversation(&conversation).await?;

        // Add to memory
        self.conversations.write().await.insert(id.clone(), conversation);

        // Set as active conversation
        *self.active_conversation.lock().await = Some(id.clone());

        Ok(id)
    }

    pub async fn send_message(&self, conversation_id: &str, content: &str) -> Result<String, AppError> {
        let mut conversations = self.conversations.write().await;
        let conversation = conversations.get_mut(conversation_id)
            .ok_or_else(|| AppError::NotFound(format!("Conversation {} not found", conversation_id)))?;

        // Add user message
        let user_message = Message {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: content.to_string(),
            timestamp: Utc::now(),
            token_count: None,
            model: None,
            metadata: HashMap::new(),
        };

        conversation.messages.push(user_message);
        conversation.updated_at = Utc::now();

        // Prepare context for AI
        let context = self.build_context(conversation).await?;
        
        // Get AI response
        let ai_response = self.ai_module.process_request(&context).await?;

        // Create assistant message
        let assistant_message = Message {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::Assistant,
            content: ai_response.content.clone(),
            timestamp: Utc::now(),
            token_count: ai_response.token_count,
            model: Some(ai_response.model.clone()),
            metadata: ai_response.metadata.clone(),
        };

        conversation.messages.push(assistant_message);
        conversation.updated_at = Utc::now();

        // Auto-generate title if this is the first exchange
        if conversation.messages.len() == 2 && self.config.read().await.auto_title {
            let new_title = self.generate_title(&conversation.messages[0].content).await?;
            conversation.title = new_title;
        }

        // Save updated conversation
        self.save_conversation(conversation).await?;

        Ok(ai_response.content)
    }

    pub async fn stream_message(&self, conversation_id: &str, content: &str) -> Result<tokio::sync::mpsc::Receiver<String>, AppError> {
        let mut conversations = self.conversations.write().await;
        let conversation = conversations.get_mut(conversation_id)
            .ok_or_else(|| AppError::NotFound(format!("Conversation {} not found", conversation_id)))?;

        // Add user message
        let user_message = Message {
            id: Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content: content.to_string(),
            timestamp: Utc::now(),
            token_count: None,
            model: None,
            metadata: HashMap::new(),
        };

        conversation.messages.push(user_message);
        conversation.updated_at = Utc::now();

        // Prepare context for AI
        let context = self.build_context(conversation).await?;
        
        // Stream AI response
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let ai_module = Arc::clone(&self.ai_module);
        let conversation_id = conversation_id.to_string();
        let conversations_clone = Arc::clone(&self.conversations);
        
        tokio::spawn(async move {
            let mut full_response = String::new();
            
            match ai_module.stream_request(&context).await {
                Ok(mut stream_rx) => {
                    while let Some(chunk) = stream_rx.recv().await {
                        full_response.push_str(&chunk);
                        if tx.send(chunk).await.is_err() {
                            break;
                        }
                    }

                    // Save complete response
                    if let Some(conversation) = conversations_clone.write().await.get_mut(&conversation_id) {
                        let assistant_message = Message {
                            id: Uuid::new_v4().to_string(),
                            role: MessageRole::Assistant,
                            content: full_response,
                            timestamp: Utc::now(),
                            token_count: None,
                            model: Some("streaming".to_string()),
                            metadata: HashMap::new(),
                        };
                        conversation.messages.push(assistant_message);
                        conversation.updated_at = Utc::now();
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("Error: {}", e)).await;
                }
            }
        });

        Ok(rx)
    }

    pub async fn list_conversations(&self) -> Result<Vec<ConversationSummary>, AppError> {
        let conversations = self.conversations.read().await;
        let mut summaries: Vec<ConversationSummary> = conversations
            .values()
            .map(|conv| ConversationSummary {
                id: conv.id.clone(),
                title: conv.title.clone(),
                message_count: conv.messages.len(),
                created_at: conv.created_at,
                updated_at: conv.updated_at,
                model: conv.model.clone(),
                tags: conv.tags.clone(),
                preview: conv.messages.first()
                    .map(|m| {
                        if m.content.len() > 100 {
                            format!("{}...", &m.content[..100])
                        } else {
                            m.content.clone()
                        }
                    })
                    .unwrap_or_else(|| "Empty conversation".to_string()),
            })
            .collect();

        summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(summaries)
    }

    pub async fn get_conversation(&self, id: &str) -> Result<Conversation, AppError> {
        let conversations = self.conversations.read().await;
        conversations.get(id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Conversation {} not found", id)))
    }

    pub async fn delete_conversation(&self, id: &str) -> Result<(), AppError> {
        // Remove from database
        self.database.execute(
            "DELETE FROM conversations WHERE id = ?",
            &[id]
        ).await?;

        // Remove from memory
        self.conversations.write().await.remove(id);

        // Clear active conversation if it was deleted
        let mut active = self.active_conversation.lock().await;
        if active.as_ref() == Some(&id.to_string()) {
            *active = None;
        }

        Ok(())
    }

    pub async fn search_conversations(&self, query: &str) -> Result<Vec<ConversationSummary>, AppError> {
        let conversations = self.conversations.read().await;
        let mut matches: Vec<ConversationSummary> = conversations
            .values()
            .filter(|conv| {
                conv.title.to_lowercase().contains(&query.to_lowercase()) ||
                conv.messages.iter().any(|msg| 
                    msg.content.to_lowercase().contains(&query.to_lowercase())
                )
            })
            .map(|conv| ConversationSummary {
                id: conv.id.clone(),
                title: conv.title.clone(),
                message_count: conv.messages.len(),
                created_at: conv.created_at,
                updated_at: conv.updated_at,
                model: conv.model.clone(),
                tags: conv.tags.clone(),
                preview: conv.messages.first()
                    .map(|m| {
                        if m.content.len() > 100 {
                            format!("{}...", &m.content[..100])
                        } else {
                            m.content.clone()
                        }
                    })
                    .unwrap_or_else(|| "Empty conversation".to_string()),
            })
            .collect();

        matches.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(matches)
    }

    pub async fn export_conversation(&self, id: &str, format: &str) -> Result<String, AppError> {
        let conversation = self.get_conversation(id).await?;
        
        match format.to_lowercase().as_str() {
            "json" => Ok(serde_json::to_string_pretty(&conversation)?),
            "markdown" => {
                let mut markdown = format!("# {}\n\n", conversation.title);
                markdown.push_str(&format!("**Created:** {}\n", conversation.created_at.format("%Y-%m-%d %H:%M:%S")));
                markdown.push_str(&format!("**Model:** {}\n\n", conversation.model));
                
                for message in &conversation.messages {
                    let role = match message.role {
                        MessageRole::User => "👤 User",
                        MessageRole::Assistant => "🤖 Assistant",
                        MessageRole::System => "⚙️ System",
                        MessageRole::Tool => "🔧 Tool",
                    };
                    markdown.push_str(&format!("## {}\n\n{}\n\n", role, message.content));
                }
                
                Ok(markdown)
            }
            "txt" => {
                let mut text = format!("Conversation: {}\n", conversation.title);
                text.push_str(&format!("Created: {}\n", conversation.created_at.format("%Y-%m-%d %H:%M:%S")));
                text.push_str(&format!("Model: {}\n\n", conversation.model));
                text.push_str(&"=".repeat(50));
                text.push_str("\n\n");
                
                for message in &conversation.messages {
                    let role = match message.role {
                        MessageRole::User => "USER",
                        MessageRole::Assistant => "ASSISTANT",
                        MessageRole::System => "SYSTEM",
                        MessageRole::Tool => "TOOL",
                    };
                    text.push_str(&format!("[{}]: {}\n\n", role, message.content));
                }
                
                Ok(text)
            }
            _ => Err(AppError::Validation(format!("Unsupported export format: {}", format)))
        }
    }

    async fn build_context(&self, conversation: &Conversation) -> Result<String, AppError> {
        let mut context = String::new();
        
        // Add system prompt if present
        if let Some(system_prompt) = &conversation.system_prompt {
            context.push_str(&format!("System: {}\n\n", system_prompt));
        }

        // Add conversation history (limit to context window)
        let messages_to_include = if conversation.messages.len() > 20 {
            &conversation.messages[conversation.messages.len() - 20..]
        } else {
            &conversation.messages
        };

        for message in messages_to_include {
            let role = match message.role {
                MessageRole::User => "Human",
                MessageRole::Assistant => "Assistant",
                MessageRole::System => "System",
                MessageRole::Tool => "Tool",
            };
            context.push_str(&format!("{}: {}\n\n", role, message.content));
        }

        Ok(context)
    }

    async fn generate_title(&self, first_message: &str) -> Result<String, AppError> {
        let prompt = format!(
            "Generate a short, descriptive title (max 50 characters) for a conversation that starts with: \"{}\"",
            &first_message[..first_message.len().min(200)]
        );

        let response = self.ai_module.process_request(&prompt).await?;
        Ok(response.content.trim().to_string())
    }

    async fn save_conversation(&self, conversation: &Conversation) -> Result<(), AppError> {
        let conversation_json = serde_json::to_string(conversation)?;
        
        self.database.execute(
            "INSERT OR REPLACE INTO conversations (id, title, data, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
            &[
                &conversation.id,
                &conversation.title,
                &conversation_json,
                &conversation.created_at.to_rfc3339(),
                &conversation.updated_at.to_rfc3339(),
            ]
        ).await?;

        Ok(())
    }

    async fn load_conversations(&self) -> Result<(), AppError> {
        let rows = self.database.query_all(
            "SELECT id, data FROM conversations ORDER BY updated_at DESC"
        ).await?;

        let mut conversations = self.conversations.write().await;
        
        for row in rows {
            if let (Some(id), Some(data)) = (row.get("id"), row.get("data")) {
                if let Ok(conversation) = serde_json::from_str::<Conversation>(data) {
                    conversations.insert(id.to_string(), conversation);
                }
            }
        }

        Ok(())
    }

    pub async fn set_active_conversation(&self, id: Option<String>) -> Result<(), AppError> {
        *self.active_conversation.lock().await = id;
        Ok(())
    }

    pub async fn get_active_conversation(&self) -> Option<String> {
        self.active_conversation.lock().await.clone()
    }

    pub async fn update_config(&self, config: ConversationConfig) -> Result<(), AppError> {
        *self.config.write().await = config;
        Ok(())
    }

    pub async fn get_config(&self) -> ConversationConfig {
        self.config.read().await.clone()
    }
}
