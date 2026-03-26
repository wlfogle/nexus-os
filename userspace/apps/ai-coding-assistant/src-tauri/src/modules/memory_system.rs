use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub importance: f32,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
    pub tags: Vec<String>,
    pub connections: Vec<String>, // Connected memory IDs
    pub embedding: Option<Vec<f32>>, // Vector embedding for similarity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryType {
    Episodic,    // Specific events/conversations
    Semantic,    // Facts and knowledge
    Procedural,  // How-to knowledge
    Working,     // Temporary context
    Emotional,   // Emotional associations
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningPattern {
    pub pattern_id: String,
    pub pattern_type: String,
    pub frequency: u32,
    pub success_rate: f32,
    pub last_used: DateTime<Utc>,
    pub context: String,
}

pub struct AdvancedMemorySystem {
    memories: Arc<RwLock<HashMap<String, Memory>>>,
    patterns: Arc<RwLock<HashMap<String, LearningPattern>>>,
    user_preferences: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    conversation_history: Arc<RwLock<Vec<ConversationMemory>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMemory {
    pub id: String,
    pub user_input: String,
    pub ai_response: String,
    pub context: String,
    pub satisfaction_score: Option<f32>,
    pub timestamp: DateTime<Utc>,
    pub tools_used: Vec<String>,
    pub outcome: String,
}

impl AdvancedMemorySystem {
    pub fn new() -> Self {
        Self {
            memories: Arc::new(RwLock::new(HashMap::new())),
            patterns: Arc::new(RwLock::new(HashMap::new())),
            user_preferences: Arc::new(RwLock::new(HashMap::new())),
            conversation_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // Long-term memory formation
    pub async fn store_memory(&self, content: String, memory_type: MemoryType, importance: f32) -> Result<String, AppError> {
        let memory_id = Uuid::new_v4().to_string();
        let memory = Memory {
            id: memory_id.clone(),
            content,
            memory_type,
            importance,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            tags: Vec::new(),
            connections: Vec::new(),
            embedding: None,
        };

        let mut memories = self.memories.write().await;
        memories.insert(memory_id.clone(), memory);
        Ok(memory_id)
    }

    // Retrieve memories with relevance scoring
    pub async fn recall_memories(&self, query: &str, limit: usize) -> Result<Vec<Memory>, AppError> {
        let memories = self.memories.read().await;
        let mut relevant_memories: Vec<Memory> = memories
            .values()
            .filter(|memory| {
                memory.content.to_lowercase().contains(&query.to_lowercase()) ||
                memory.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
            })
            .cloned()
            .collect();

        // Sort by relevance score (importance + recency + access frequency)
        relevant_memories.sort_by(|a, b| {
            let score_a = a.importance + (1.0 / (Utc::now() - a.last_accessed).num_hours() as f32.max(1.0)) + (a.access_count as f32 * 0.1);
            let score_b = b.importance + (1.0 / (Utc::now() - b.last_accessed).num_hours() as f32.max(1.0)) + (b.access_count as f32 * 0.1);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        relevant_memories.truncate(limit);
        Ok(relevant_memories)
    }

    // Learn from user interactions
    pub async fn learn_from_interaction(&self, user_input: &str, ai_response: &str, tools_used: Vec<String>, satisfaction: Option<f32>) -> Result<(), AppError> {
        let conversation_memory = ConversationMemory {
            id: Uuid::new_v4().to_string(),
            user_input: user_input.to_string(),
            ai_response: ai_response.to_string(),
            context: "conversation".to_string(),
            satisfaction_score: satisfaction,
            timestamp: Utc::now(),
            tools_used,
            outcome: "completed".to_string(),
        };

        let mut history = self.conversation_history.write().await;
        history.push(conversation_memory);

        // Analyze patterns
        self.analyze_patterns().await?;
        Ok(())
    }

    // Pattern recognition and learning
    async fn analyze_patterns(&self) -> Result<(), AppError> {
        let history = self.conversation_history.read().await;
        let mut patterns = self.patterns.write().await;

        // Analyze recent interactions for patterns
        for conversation in history.iter().rev().take(50) {
            for tool in &conversation.tools_used {
                let pattern_key = format!("tool_usage_{}", tool);
                if let Some(pattern) = patterns.get_mut(&pattern_key) {
                    pattern.frequency += 1;
                    pattern.last_used = conversation.timestamp;
                    if let Some(satisfaction) = conversation.satisfaction_score {
                        pattern.success_rate = (pattern.success_rate + satisfaction) / 2.0;
                    }
                } else {
                    patterns.insert(pattern_key.clone(), LearningPattern {
                        pattern_id: pattern_key,
                        pattern_type: "tool_usage".to_string(),
                        frequency: 1,
                        success_rate: conversation.satisfaction_score.unwrap_or(0.5),
                        last_used: conversation.timestamp,
                        context: tool.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    // Adaptive response generation based on learned patterns
    pub async fn get_adaptive_context(&self, user_input: &str) -> Result<String, AppError> {
        let relevant_memories = self.recall_memories(user_input, 5).await?;
        let patterns = self.patterns.read().await;
        let preferences = self.user_preferences.read().await;

        let mut context = String::new();
        
        // Add relevant memories
        if !relevant_memories.is_empty() {
            context.push_str("Relevant memories:\n");
            for memory in relevant_memories {
                context.push_str(&format!("- {} (importance: {})\n", memory.content, memory.importance));
            }
        }

        // Add successful patterns
        context.push_str("\nSuccessful patterns:\n");
        for pattern in patterns.values().filter(|p| p.success_rate > 0.7) {
            context.push_str(&format!("- {} (success: {:.1}%)\n", pattern.context, pattern.success_rate * 100.0));
        }

        // Add user preferences
        if !preferences.is_empty() {
            context.push_str("\nUser preferences:\n");
            for (key, value) in preferences.iter() {
                context.push_str(&format!("- {}: {}\n", key, value));
            }
        }

        Ok(context)
    }

    // Update user preferences based on behavior
    pub async fn update_preferences(&self, key: String, value: serde_json::Value) -> Result<(), AppError> {
        let mut preferences = self.user_preferences.write().await;
        preferences.insert(key, value);
        Ok(())
    }

    // Memory consolidation - move important working memory to long-term
    pub async fn consolidate_memories(&self) -> Result<(), AppError> {
        let mut memories = self.memories.write().await;
        let mut to_promote = Vec::new();

        for (id, memory) in memories.iter() {
            if matches!(memory.memory_type, MemoryType::Working) && 
               memory.access_count > 5 && 
               memory.importance > 0.7 {
                to_promote.push(id.clone());
            }
        }

        for id in to_promote {
            if let Some(memory) = memories.get_mut(&id) {
                memory.memory_type = MemoryType::Semantic;
                memory.importance += 0.1; // Boost importance
            }
        }

        Ok(())
    }

    // Get memory statistics
    pub async fn get_memory_stats(&self) -> Result<serde_json::Value, AppError> {
        let memories = self.memories.read().await;
        let patterns = self.patterns.read().await;
        let history = self.conversation_history.read().await;

        let stats = serde_json::json!({
            "total_memories": memories.len(),
            "memory_types": {
                "episodic": memories.values().filter(|m| matches!(m.memory_type, MemoryType::Episodic)).count(),
                "semantic": memories.values().filter(|m| matches!(m.memory_type, MemoryType::Semantic)).count(),
                "procedural": memories.values().filter(|m| matches!(m.memory_type, MemoryType::Procedural)).count(),
                "working": memories.values().filter(|m| matches!(m.memory_type, MemoryType::Working)).count(),
                "emotional": memories.values().filter(|m| matches!(m.memory_type, MemoryType::Emotional)).count(),
            },
            "total_patterns": patterns.len(),
            "conversation_history": history.len(),
            "average_satisfaction": history.iter()
                .filter_map(|c| c.satisfaction_score)
                .sum::<f32>() / history.len().max(1) as f32
        });

        Ok(stats)
    }
}
