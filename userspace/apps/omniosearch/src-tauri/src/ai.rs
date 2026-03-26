use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};
use tokenizers::Tokenizer;
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};

use crate::search::SearchQuery;
use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalLanguageQuery {
    pub intent: Intent,
    pub entities: Vec<Entity>,
    pub original_text: String,
    pub processed_query: SearchQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    FindFiles,
    FindByType,
    FindByDate,
    FindBySize,
    FindByContent,
    FindRecent,
    FindLarge,
    FindDuplicate,
    OpenFile,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: EntityType,
    pub value: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    FileName,
    FileType,
    Date,
    Size,
    Content,
    Location,
}

pub struct AIProcessor {
    device: Device,
    tokenizer: Tokenizer,
    model: BertModel,
    intent_classifier: IntentClassifier,
    entity_extractor: EntityExtractor,
    config: Config,
}

impl AIProcessor {
    pub async fn new(config: &Config) -> Result<Self> {
        info!("🧠 Initializing AI processor with local models...");

        // Initialize device (prefer CUDA if available, otherwise CPU)
        let device = if candle_core::cuda_is_available() {
            info!("🚀 Using CUDA device for AI processing");
            Device::new_cuda(0)?
        } else {
            info!("🖥️ Using CPU for AI processing");
            Device::Cpu
        };

        // Load tokenizer
        let tokenizer_path = config.ai_models_path().join("tokenizer.json");
        let tokenizer = if tokenizer_path.exists() {
            Tokenizer::from_file(tokenizer_path)?
        } else {
            info!("📥 Downloading tokenizer model...");
            Self::download_tokenizer(config).await?
        };

        // Load BERT model for embeddings
        let model_path = config.ai_models_path().join("bert-model.safetensors");
        let model = if model_path.exists() {
            Self::load_bert_model(&model_path, &device)?
        } else {
            info!("📥 Downloading BERT model...");
            Self::download_and_load_bert(config, &device).await?
        };

        // Initialize intent classifier
        let intent_classifier = IntentClassifier::new()?;
        
        // Initialize entity extractor
        let entity_extractor = EntityExtractor::new()?;

        info!("✅ AI processor initialized successfully");

        Ok(Self {
            device,
            tokenizer,
            model,
            intent_classifier,
            entity_extractor,
            config: config.clone(),
        })
    }

    pub async fn process_natural_language(&self, query: &str) -> Result<SearchQuery> {
        debug!("🧠 Processing natural language query: {}", query);

        // 1. Classify intent
        let intent = self.intent_classifier.classify(query).await?;
        debug!("🎯 Detected intent: {:?}", intent);

        // 2. Extract entities
        let entities = self.entity_extractor.extract(query).await?;
        debug!("🔍 Extracted entities: {:?}", entities);

        // 3. Convert to structured search query
        let search_query = self.build_search_query(&intent, &entities, query)?;
        
        info!("✅ NL processing complete: {:?}", search_query.text);
        Ok(search_query)
    }

    pub async fn get_search_suggestions(&self, partial_query: &str) -> Result<Vec<String>> {
        debug!("💡 Generating suggestions for: {}", partial_query);

        let mut suggestions = Vec::new();

        // Rule-based suggestions for common patterns
        if partial_query.is_empty() {
            return Ok(vec![]);
        }

        let query_lower = partial_query.to_lowercase();

        // File type suggestions
        if query_lower.contains("pdf") || query_lower.contains("document") {
            suggestions.push("large PDF files".to_string());
            suggestions.push("PDF files modified today".to_string());
            suggestions.push("PDF files in Documents folder".to_string());
        }

        // Size-based suggestions
        if query_lower.contains("large") || query_lower.contains("big") {
            suggestions.push("large files over 100MB".to_string());
            suggestions.push("largest files in Downloads".to_string());
        }

        // Time-based suggestions
        if query_lower.contains("recent") || query_lower.contains("today") {
            suggestions.push("files modified today".to_string());
            suggestions.push("recent downloads".to_string());
            suggestions.push("files created this week".to_string());
        }

        // Content-based suggestions
        if query_lower.contains("code") || query_lower.contains("script") {
            suggestions.push("Python scripts".to_string());
            suggestions.push("JavaScript files".to_string());
            suggestions.push("source code files".to_string());
        }

        // Media suggestions
        if query_lower.contains("image") || query_lower.contains("photo") {
            suggestions.push("high resolution images".to_string());
            suggestions.push("photos from last month".to_string());
            suggestions.push("images larger than 5MB".to_string());
        }

        // Generic completion suggestions
        if suggestions.is_empty() {
            suggestions = vec![
                format!("{} files", partial_query),
                format!("{} in Documents", partial_query),
                format!("large {} files", partial_query),
                format!("recent {} files", partial_query),
            ];
        }

        // Limit suggestions
        suggestions.truncate(5);
        
        debug!("💡 Generated {} suggestions", suggestions.len());
        Ok(suggestions)
    }

    fn build_search_query(&self, intent: &Intent, entities: &[Entity], original: &str) -> Result<SearchQuery> {
        let mut query = SearchQuery::natural_language(original);

        // Apply intent-specific modifications
        match intent {
            Intent::FindByType => {
                // Extract file types from entities
                for entity in entities {
                    if let EntityType::FileType = entity.entity_type {
                        query.file_types.push(entity.value.clone());
                    }
                }
            }
            Intent::FindByDate => {
                // Extract date information
                for entity in entities {
                    if let EntityType::Date = entity.entity_type {
                        // Parse date entity (simplified)
                        if entity.value.contains("today") {
                            query.modified_after = Some(chrono::Utc::today().and_hms(0, 0, 0));
                        } else if entity.value.contains("week") {
                            query.modified_after = Some(chrono::Utc::now() - chrono::Duration::days(7));
                        }
                    }
                }
            }
            Intent::FindBySize => {
                // Extract size information
                for entity in entities {
                    if let EntityType::Size = entity.entity_type {
                        if entity.value.contains("large") || entity.value.contains("big") {
                            query.size_min = Some(100 * 1024 * 1024); // 100MB
                        } else if entity.value.contains("small") {
                            query.size_max = Some(1024 * 1024); // 1MB
                        }
                    }
                }
            }
            Intent::FindByContent => {
                query.search_content = true;
            }
            Intent::FindRecent => {
                query.modified_after = Some(chrono::Utc::now() - chrono::Duration::days(7));
            }
            Intent::FindLarge => {
                query.size_min = Some(50 * 1024 * 1024); // 50MB
            }
            _ => {}
        }

        // Extract filename from entities
        for entity in entities {
            if let EntityType::FileName = entity.entity_type {
                query.text = entity.value.clone();
                break;
            }
        }

        Ok(query)
    }

    async fn download_tokenizer(config: &Config) -> Result<Tokenizer> {
        // In a real implementation, this would download from HuggingFace Hub
        // For now, create a simple tokenizer
        let tokenizer = Tokenizer::from_pretrained("bert-base-uncased", None)
            .context("Failed to load tokenizer")?;
        
        // Save for future use
        let tokenizer_path = config.ai_models_path().join("tokenizer.json");
        std::fs::create_dir_all(config.ai_models_path())?;
        tokenizer.save(&tokenizer_path, false)?;
        
        Ok(tokenizer)
    }

    fn load_bert_model(model_path: &std::path::Path, device: &Device) -> Result<BertModel> {
        // Load pre-trained BERT model
        let config = BertConfig::base();
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[model_path], config.hidden_size, device)? };
        let model = BertModel::load(&vb, &config)?;
        Ok(model)
    }

    async fn download_and_load_bert(config: &Config, device: &Device) -> Result<BertModel> {
        // In a real implementation, this would download from HuggingFace Hub
        // For now, create a simplified model
        info!("⚠️ Using simplified BERT model for demo purposes");
        
        let bert_config = BertConfig::base();
        
        // Create model directory
        std::fs::create_dir_all(config.ai_models_path())?;
        
        // For demo, we'll create a minimal model structure
        // In production, this would be a proper pre-trained model
        let vb = VarBuilder::zeros(bert_config.hidden_size, device);
        let model = BertModel::load(&vb, &bert_config)?;
        
        Ok(model)
    }

    pub async fn get_file_embeddings(&self, file_path: &str) -> Result<Vec<f32>> {
        // Generate embeddings for file content
        // This would be used for semantic search
        debug!("🔢 Generating embeddings for: {}", file_path);
        
        // Simplified embedding generation
        // In practice, this would process file content through the model
        let dummy_embedding = vec![0.1; 768]; // BERT base dimension
        
        Ok(dummy_embedding)
    }
}

struct IntentClassifier {
    patterns: HashMap<String, Intent>,
}

impl IntentClassifier {
    fn new() -> Result<Self> {
        let mut patterns = HashMap::new();
        
        // Define patterns for intent classification
        patterns.insert("find".to_string(), Intent::FindFiles);
        patterns.insert("search".to_string(), Intent::FindFiles);
        patterns.insert("look for".to_string(), Intent::FindFiles);
        patterns.insert("pdf".to_string(), Intent::FindByType);
        patterns.insert("image".to_string(), Intent::FindByType);
        patterns.insert("video".to_string(), Intent::FindByType);
        patterns.insert("document".to_string(), Intent::FindByType);
        patterns.insert("today".to_string(), Intent::FindByDate);
        patterns.insert("yesterday".to_string(), Intent::FindByDate);
        patterns.insert("last week".to_string(), Intent::FindByDate);
        patterns.insert("large".to_string(), Intent::FindBySize);
        patterns.insert("big".to_string(), Intent::FindBySize);
        patterns.insert("small".to_string(), Intent::FindBySize);
        patterns.insert("content".to_string(), Intent::FindByContent);
        patterns.insert("contains".to_string(), Intent::FindByContent);
        patterns.insert("recent".to_string(), Intent::FindRecent);
        patterns.insert("open".to_string(), Intent::OpenFile);
        
        Ok(Self { patterns })
    }
    
    async fn classify(&self, query: &str) -> Result<Intent> {
        let query_lower = query.to_lowercase();
        
        // Simple pattern matching for intent classification
        for (pattern, intent) in &self.patterns {
            if query_lower.contains(pattern) {
                return Ok(intent.clone());
            }
        }
        
        Ok(Intent::FindFiles) // Default intent
    }
}

struct EntityExtractor {
    patterns: HashMap<EntityType, Vec<regex::Regex>>,
}

impl EntityExtractor {
    fn new() -> Result<Self> {
        let mut patterns: HashMap<EntityType, Vec<regex::Regex>> = HashMap::new();
        
        // File type patterns
        patterns.insert(EntityType::FileType, vec![
            regex::Regex::new(r"\.(\w+)(?:\s|$)")?,
            regex::Regex::new(r"(\w+)\s+files?")?,
        ]);
        
        // Date patterns
        patterns.insert(EntityType::Date, vec![
            regex::Regex::new(r"(today|yesterday|last\s+week|last\s+month)")?,
            regex::Regex::new(r"(\d{1,2}[-/]\d{1,2}[-/]\d{2,4})")?,
        ]);
        
        // Size patterns  
        patterns.insert(EntityType::Size, vec![
            regex::Regex::new(r"(large|big|small|tiny)")?,
            regex::Regex::new(r"(\d+(?:\.\d+)?)\s*(kb|mb|gb)")?,
        ]);
        
        // Content patterns
        patterns.insert(EntityType::Content, vec![
            regex::Regex::new(r"contains?\s+[\"']([^\"']+)[\"']")?,
            regex::Regex::new(r"with\s+(.+?)(?:\s+in|\s+from|$)")?,
        ]);
        
        Ok(Self { patterns })
    }
    
    async fn extract(&self, query: &str) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();
        
        for (entity_type, regexes) in &self.patterns {
            for regex in regexes {
                for captures in regex.captures_iter(query) {
                    if let Some(matched) = captures.get(1) {
                        entities.push(Entity {
                            entity_type: entity_type.clone(),
                            value: matched.as_str().to_string(),
                            confidence: 0.8, // Simplified confidence scoring
                        });
                    }
                }
            }
        }
        
        Ok(entities)
    }
}
