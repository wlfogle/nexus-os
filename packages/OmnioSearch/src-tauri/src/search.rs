use std::sync::Arc;
use std::path::Path;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};
use chrono::{DateTime, Utc};
use fuzzy_matcher::{FuzzyMatcher, SkimMatcher};
use rayon::prelude::*;

use crate::ai::AIProcessor;
use crate::db::Database;
use crate::config::Config;
use crate::indexer::FileIndexer;
use crate::cloud;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub file_types: Vec<String>,
    pub size_min: Option<u64>,
    pub size_max: Option<u64>,
    pub modified_after: Option<DateTime<Utc>>,
    pub modified_before: Option<DateTime<Utc>>,
    pub search_content: bool,
    pub include_hidden: bool,
    pub max_results: usize,
    pub fuzzy_threshold: f64,
}

impl SearchQuery {
    pub fn from_text(text: &str) -> Self {
        Self {
            text: text.to_string(),
            file_types: vec![],
            size_min: None,
            size_max: None,
            modified_after: None,
            modified_before: None,
            search_content: false,
            include_hidden: false,
            max_results: 1000,
            fuzzy_threshold: 0.6,
        }
    }

    pub fn natural_language(text: &str) -> Self {
        let mut query = Self::from_text(text);
        query.search_content = true;
        query.include_hidden = true;
        query.fuzzy_threshold = 0.4; // More lenient for NL queries
        query
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub file_type: String,
    pub mime_type: String,
    pub relevance_score: f64,
    pub content_matches: Vec<ContentMatch>,
    pub is_directory: bool,
    pub permissions: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMatch {
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
}

pub struct SearchEngine {
    database: Arc<RwLock<Database>>,
    ai_processor: Arc<RwLock<AIProcessor>>,
    config: Arc<RwLock<Config>>,
    indexer: Arc<RwLock<FileIndexer>>,
    fuzzy_matcher: SkimMatcher<'static>,
}

impl SearchEngine {
    pub async fn new(
        database: Arc<RwLock<Database>>,
        ai_processor: Arc<RwLock<AIProcessor>>,
        config: Arc<RwLock<Config>>,
        indexer: Arc<RwLock<FileIndexer>>,
    ) -> Result<Self> {
        info!("🔍 Initializing SearchEngine with AI integration...");

        let fuzzy_matcher = SkimMatcher::default();

        Ok(Self {
            database,
            ai_processor,
            config,
            indexer,
            fuzzy_matcher,
        })
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        debug!("🔍 Starting search for: {}", query);

        // First, try AI-enhanced natural language processing
        let structured_query = {
            let ai = self.ai_processor.read().await;
            match ai.process_natural_language(query).await {
                Ok(ai_query) => {
                    info!("🧠 AI processed query successfully");
                    ai_query
                }
                Err(e) => {
                    warn!("⚠️ AI processing failed, using fallback: {}", e);
                    SearchQuery::natural_language(query)
                }
            }
        };

        self.search_with_query(&structured_query).await
    }

    pub async fn search_with_query(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        debug!("🔍 Executing structured search: {:?}", query);

        let mut results = Vec::new();

        // 1. Database search (indexed files)
        let db_results = self.search_database(query).await?;
        results.extend(db_results);

        // 2. Real-time file system search (for new/unindexed files)
        if results.len() < query.max_results {
            let fs_results = self.search_filesystem(query).await?;
            results.extend(fs_results);
        }

        // 3. Content search (if enabled)
        if query.search_content && results.len() < query.max_results {
            let content_results = self.search_content(query).await?;
            results.extend(content_results);
        }

        // 4. Cloud search (if configured)
        let cloud_results = self.search_cloud(query).await?;
        results.extend(cloud_results);

        // Deduplicate and sort by relevance
        self.deduplicate_and_rank(results, query).await
    }

    async fn search_database(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        debug!("💾 Searching database index...");

        let db = self.database.read().await;
        let mut results = Vec::new();

        // Full-text search using SQLite FTS5
        let fts_results = db.fts_search(&query.text, query.max_results).await?;
        
        for file_entry in fts_results {
            if self.matches_filters(&file_entry, query) {
                let result = SearchResult {
                    path: file_entry.path.clone(),
                    name: Path::new(&file_entry.path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    size: file_entry.size,
                    modified: file_entry.modified,
                    file_type: file_entry.file_type,
                    mime_type: file_entry.mime_type,
                    relevance_score: self.calculate_relevance(&file_entry.path, &query.text),
                    content_matches: vec![],
                    is_directory: file_entry.is_directory,
                    permissions: file_entry.permissions,
                    icon: self.get_file_icon(&file_entry.mime_type).await,
                };
                results.push(result);
            }
        }

        debug!("💾 Database search found {} results", results.len());
        Ok(results)
    }

    async fn search_filesystem(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        debug!("📂 Searching filesystem with fd...");

        let config = self.config.read().await;
        let search_paths = config.search_paths();
        
        let mut results = Vec::new();

        // Use fd (find) for fast file discovery
        for search_path in search_paths {
            let fd_results = self.fd_search(&search_path, query).await?;
            results.extend(fd_results);
        }

        debug!("📂 Filesystem search found {} results", results.len());
        Ok(results)
    }

    async fn fd_search(&self, path: &str, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        use std::process::Command;

        let mut args = vec![
            "--type", "f",
            "--follow",
            "--absolute-path",
            "--max-results", &query.max_results.to_string(),
        ];

        if query.include_hidden {
            args.push("--hidden");
        }

        // Add file type filters
        for file_type in &query.file_types {
            args.extend(["--extension", file_type]);
        }

        args.push(&query.text);
        args.push(path);

        let output = Command::new("fd")
            .args(&args)
            .output()?;

        let mut results = Vec::new();

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Ok(metadata) = std::fs::metadata(line) {
                    let result = SearchResult {
                        path: line.to_string(),
                        name: Path::new(line)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        size: metadata.len(),
                        modified: metadata.modified()
                            .unwrap_or(std::time::UNIX_EPOCH)
                            .into(),
                        file_type: self.get_file_type(line).await,
                        mime_type: mime_guess::from_path(line)
                            .first_or_octet_stream()
                            .to_string(),
                        relevance_score: self.calculate_relevance(line, &query.text),
                        content_matches: vec![],
                        is_directory: metadata.is_dir(),
                        permissions: format!("{:o}", metadata.permissions().mode()),
                        icon: None,
                    };
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    async fn search_content(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        debug!("📄 Searching file content with ripgrep...");

        use std::process::Command;

        let config = self.config.read().await;
        let search_paths = config.search_paths();
        
        let mut results = Vec::new();

        for search_path in search_paths {
            let rg_results = self.ripgrep_search(&search_path, query).await?;
            results.extend(rg_results);
        }

        debug!("📄 Content search found {} results", results.len());
        Ok(results)
    }

    async fn ripgrep_search(&self, path: &str, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        use std::process::Command;

        let mut args = vec![
            "--json",
            "--follow",
            "--max-count", "10", // Max matches per file
            "--max-filesize", "10M", // Skip large files
        ];

        if query.include_hidden {
            args.push("--hidden");
        }

        // Add file type filters
        if !query.file_types.is_empty() {
            for file_type in &query.file_types {
                args.extend(["--type-add", &format!("custom:*.{}", file_type)]);
                args.extend(["--type", "custom"]);
            }
        }

        args.push(&query.text);
        args.push(path);

        let output = Command::new("rg")
            .args(&args)
            .output()?;

        let mut results = Vec::new();

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            for line in stdout.lines() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    if json["type"] == "match" {
                        if let (Some(path), Some(line_num), Some(line_content)) = (
                            json["data"]["path"]["text"].as_str(),
                            json["data"]["line_number"].as_u64(),
                            json["data"]["lines"]["text"].as_str(),
                        ) {
                            let matches = json["data"]["submatches"]
                                .as_array()
                                .unwrap_or(&vec![])
                                .iter()
                                .map(|m| ContentMatch {
                                    line_number: line_num as usize,
                                    line_content: line_content.to_string(),
                                    match_start: m["start"].as_u64().unwrap_or(0) as usize,
                                    match_end: m["end"].as_u64().unwrap_or(0) as usize,
                                })
                                .collect();

                            if let Ok(metadata) = std::fs::metadata(path) {
                                let result = SearchResult {
                                    path: path.to_string(),
                                    name: Path::new(path)
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string(),
                                    size: metadata.len(),
                                    modified: metadata.modified()
                                        .unwrap_or(std::time::UNIX_EPOCH)
                                        .into(),
                                    file_type: self.get_file_type(path).await,
                                    mime_type: mime_guess::from_path(path)
                                        .first_or_octet_stream()
                                        .to_string(),
                                    relevance_score: self.calculate_content_relevance(line_content, &query.text),
                                    content_matches: matches,
                                    is_directory: false,
                                    permissions: format!("{:o}", metadata.permissions().mode()),
                                    icon: None,
                                };
                                results.push(result);
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    async fn search_cloud(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        debug!("☁️ Searching cloud storage...");
        // Cloud search implementation would go here
        // For now, return empty results
        Ok(vec![])
    }

    pub async fn add_cloud_provider(&self, provider: &str) -> Result<()> {
        info!("☁️ Adding cloud provider: {}", provider);
        // Cloud provider integration would go here
        Ok(())
    }

    async fn deduplicate_and_rank(&self, mut results: Vec<SearchResult>, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        debug!("🎯 Deduplicating and ranking {} results", results.len());

        // Remove duplicates by path
        results.sort_by(|a, b| a.path.cmp(&b.path));
        results.dedup_by(|a, b| a.path == b.path);

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.truncate(query.max_results);

        info!("✅ Final results: {} files", results.len());
        Ok(results)
    }

    fn calculate_relevance(&self, path: &str, query: &str) -> f64 {
        let filename = Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        // Fuzzy match on filename
        let fuzzy_score = self.fuzzy_matcher.fuzzy_match(&filename, query)
            .map(|score| score as f64 / 100.0)
            .unwrap_or(0.0);

        // Exact match bonus
        let exact_bonus = if filename.to_lowercase().contains(&query.to_lowercase()) {
            0.5
        } else {
            0.0
        };

        // Recent file bonus
        let recency_bonus = if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                let age_days = modified.elapsed().unwrap_or_default().as_secs() / (24 * 3600);
                (30.0 - age_days.min(30) as f64) / 30.0 * 0.2
            } else {
                0.0
            }
        } else {
            0.0
        };

        fuzzy_score + exact_bonus + recency_bonus
    }

    fn calculate_content_relevance(&self, content: &str, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();

        let exact_matches = content_lower.matches(&query_lower).count() as f64;
        let content_length = content.len() as f64;

        // Base score from match frequency
        let frequency_score = (exact_matches * query.len() as f64) / content_length.max(1.0);

        // Context relevance (matches at word boundaries are better)
        let word_boundary_bonus = if content_lower.split_whitespace().any(|word| word.contains(&query_lower)) {
            0.3
        } else {
            0.0
        };

        frequency_score + word_boundary_bonus
    }

    fn matches_filters(&self, file_entry: &crate::db::FileEntry, query: &SearchQuery) -> bool {
        // Size filters
        if let Some(min_size) = query.size_min {
            if file_entry.size < min_size {
                return false;
            }
        }
        if let Some(max_size) = query.size_max {
            if file_entry.size > max_size {
                return false;
            }
        }

        // Date filters
        if let Some(after) = query.modified_after {
            if file_entry.modified < after {
                return false;
            }
        }
        if let Some(before) = query.modified_before {
            if file_entry.modified > before {
                return false;
            }
        }

        // File type filters
        if !query.file_types.is_empty() {
            let extension = Path::new(&file_entry.path)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            
            if !query.file_types.iter().any(|ft| ft == extension) {
                return false;
            }
        }

        true
    }

    async fn get_file_type(&self, path: &str) -> String {
        Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    async fn get_file_icon(&self, mime_type: &str) -> Option<String> {
        // Icon mapping logic would go here
        // For now, return None
        None
    }
}

use std::os::unix::fs::PermissionsExt;
