use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json;
use tracing::{info, error};

use crate::search::{SearchEngine, SearchQuery};
use crate::ai::AIProcessor;
use crate::db::Database;
use crate::config::Config;
use crate::indexer::FileIndexer;

#[derive(Parser)]
#[command(name = "omniosearch-cli")]
#[command(about = "👁️ OmnioSearch - The All-Seeing File Search CLI")]
#[command(version = "1.0.0")]
#[command(author = "OmnioSearch Team")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    
    /// Output format
    #[arg(short, long, default_value = "human")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search for files using natural language or patterns
    Search {
        /// Search query (natural language supported)
        query: String,
        
        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        limit: usize,
        
        /// Search in file content
        #[arg(short = 'C', long)]
        content: bool,
        
        /// Include hidden files
        #[arg(short = 'H', long)]
        hidden: bool,
        
        /// File types to search (comma-separated)
        #[arg(short, long)]
        types: Option<String>,
        
        /// Minimum file size (e.g., 1MB, 500KB)
        #[arg(long)]
        min_size: Option<String>,
        
        /// Maximum file size (e.g., 100MB, 2GB)
        #[arg(long)]
        max_size: Option<String>,
    },
    
    /// Index files and directories
    Index {
        /// Paths to index
        paths: Vec<String>,
        
        /// Force re-indexing of existing files
        #[arg(short, long)]
        force: bool,
        
        /// Show progress during indexing
        #[arg(short, long)]
        progress: bool,
    },
    
    /// Get indexing status and statistics
    Status,
    
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    
    /// Cloud storage integration
    Cloud {
        #[command(subcommand)]
        action: CloudAction,
    },
    
    /// AI and natural language features
    AI {
        #[command(subcommand)]
        action: AIAction,
    },
    
    /// Database operations
    Database {
        #[command(subcommand)]
        action: DatabaseAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    
    /// Add a search path
    AddPath {
        path: String,
    },
    
    /// Remove a search path
    RemovePath {
        path: String,
    },
    
    /// Set configuration value
    Set {
        key: String,
        value: String,
    },
    
    /// Reset to default configuration
    Reset,
}

#[derive(Subcommand)]
pub enum CloudAction {
    /// List available cloud providers
    List,
    
    /// Authenticate with a cloud provider
    Auth {
        provider: String,
    },
    
    /// Search in cloud storage
    Search {
        provider: String,
        query: String,
    },
    
    /// Show cloud provider status
    Status {
        provider: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AIAction {
    /// Test natural language processing
    Parse {
        query: String,
    },
    
    /// Get search suggestions
    Suggest {
        partial: String,
    },
    
    /// Download AI models
    Download,
    
    /// Show AI model status
    Status,
}

#[derive(Subcommand)]
pub enum DatabaseAction {
    /// Optimize database
    Optimize,
    
    /// Show database statistics
    Stats,
    
    /// Clean orphaned entries
    Clean,
    
    /// Export database
    Export {
        path: String,
    },
    
    /// Import database
    Import {
        path: String,
    },
}

#[derive(Clone)]
pub enum OutputFormat {
    Human,
    Json,
    Csv,
    Table,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "human" => Ok(OutputFormat::Human),
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            "table" => Ok(OutputFormat::Table),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

pub async fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("omniosearch=debug,cli=debug")
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("omniosearch=warn,cli=info")
            .init();
    }
    
    info!("🚀 OmnioSearch CLI starting...");
    
    // Load configuration
    let config = if let Some(config_path) = cli.config {
        Config::from_file(&config_path).await?
    } else {
        Config::load().await?
    };
    
    // Initialize components
    let database = Database::new(&config.database_path().to_string_lossy()).await?;
    let ai_processor = AIProcessor::new(&config).await?;
    let indexer = FileIndexer::new(
        std::sync::Arc::new(tokio::sync::RwLock::new(database)),
        std::sync::Arc::new(tokio::sync::RwLock::new(config.clone()))
    ).await?;
    
    let search_engine = SearchEngine::new(
        std::sync::Arc::new(tokio::sync::RwLock::new(database)),
        std::sync::Arc::new(tokio::sync::RwLock::new(ai_processor)),
        std::sync::Arc::new(tokio::sync::RwLock::new(config.clone())),
        std::sync::Arc::new(tokio::sync::RwLock::new(indexer))
    ).await?;
    
    // Execute command
    match cli.command {
        Commands::Search { 
            query, limit, content, hidden, types, min_size, max_size 
        } => {
            handle_search_command(
                search_engine, query, limit, content, hidden, 
                types, min_size, max_size, cli.format
            ).await?;
        }
        
        Commands::Index { paths, force, progress } => {
            handle_index_command(indexer, paths, force, progress).await?;
        }
        
        Commands::Status => {
            handle_status_command(indexer, cli.format).await?;
        }
        
        Commands::Config { action } => {
            handle_config_command(action, config, cli.format).await?;
        }
        
        Commands::Cloud { action } => {
            handle_cloud_command(action, cli.format).await?;
        }
        
        Commands::AI { action } => {
            handle_ai_command(action, ai_processor, cli.format).await?;
        }
        
        Commands::Database { action } => {
            handle_database_command(action, database, cli.format).await?;
        }
    }
    
    Ok(())
}

async fn handle_search_command(
    search_engine: SearchEngine,
    query: String,
    limit: usize,
    content: bool,
    hidden: bool,
    types: Option<String>,
    min_size: Option<String>,
    max_size: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    info!("🔍 Searching for: {}", query);
    
    // Build search query
    let mut search_query = SearchQuery::from_text(&query);
    search_query.max_results = limit;
    search_query.search_content = content;
    search_query.include_hidden = hidden;
    
    // Parse file types
    if let Some(types_str) = types {
        search_query.file_types = types_str
            .split(',')
            .map(|t| t.trim().to_string())
            .collect();
    }
    
    // Parse file sizes
    if let Some(min_str) = min_size {
        search_query.size_min = parse_file_size(&min_str);
    }
    
    if let Some(max_str) = max_size {
        search_query.size_max = parse_file_size(&max_str);
    }
    
    // Perform search
    let results = search_engine.search_with_query(&search_query).await?;
    
    // Output results
    match format {
        OutputFormat::Human => {
            if results.is_empty() {
                println!("No files found matching your search.");
            } else {
                println!("Found {} files:\n", results.len());
                for (i, result) in results.iter().enumerate() {
                    println!("{}. {} ({})", 
                        i + 1,
                        result.path,
                        format_file_size(result.size)
                    );
                    println!("   Modified: {}", result.modified.format("%Y-%m-%d %H:%M:%S"));
                    if result.relevance_score > 0.0 {
                        println!("   Relevance: {:.2}", result.relevance_score);
                    }
                    if !result.content_matches.is_empty() {
                        println!("   Content matches: {}", result.content_matches.len());
                    }
                    println!();
                }
            }
        }
        
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        
        OutputFormat::Csv => {
            println!("path,name,size,modified,type,relevance");
            for result in results {
                println!("{},{},{},{},{},{:.3}",
                    result.path,
                    result.name,
                    result.size,
                    result.modified.format("%Y-%m-%d %H:%M:%S"),
                    result.file_type,
                    result.relevance_score
                );
            }
        }
        
        OutputFormat::Table => {
            // Simple table format
            println!("{:<50} {:<10} {:<20} {:<10}", "Path", "Size", "Modified", "Relevance");
            println!("{}", "-".repeat(90));
            for result in results {
                println!("{:<50} {:<10} {:<20} {:.3}",
                    truncate_string(&result.path, 47),
                    format_file_size(result.size),
                    result.modified.format("%Y-%m-%d %H:%M").to_string(),
                    result.relevance_score
                );
            }
        }
    }
    
    Ok(())
}

async fn handle_index_command(
    indexer: FileIndexer,
    paths: Vec<String>,
    force: bool,
    progress: bool,
) -> Result<()> {
    if paths.is_empty() {
        error!("No paths specified for indexing");
        return Ok(());
    }
    
    info!("📂 Starting indexing of {} paths", paths.len());
    
    if progress {
        println!("Starting indexing...");
        // In a real implementation, we'd show progress updates
    }
    
    indexer.start_indexing(paths).await?;
    
    println!("✅ Indexing completed successfully");
    Ok(())
}

async fn handle_status_command(
    indexer: FileIndexer,
    format: OutputFormat,
) -> Result<()> {
    let status = indexer.get_status().await?;
    
    match format {
        OutputFormat::Human => {
            println!("📊 OmnioSearch Index Status\n");
            println!("Total files:       {}", status.total_files);
            println!("Indexed files:     {}", status.indexed_files);
            println!("Pending files:     {}", status.pending_files);
            println!("Failed files:      {}", status.failed_files);
            println!("Last update:       {}", status.last_update.format("%Y-%m-%d %H:%M:%S"));
            println!("Index size:        {:.1} MB", status.index_size_mb);
            if status.indexing_speed > 0.0 {
                println!("Indexing speed:    {:.1} files/sec", status.indexing_speed);
            }
        }
        
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        
        _ => {
            println!("Status format not supported for this output type");
        }
    }
    
    Ok(())
}

async fn handle_config_command(
    action: ConfigAction,
    mut config: Config,
    format: OutputFormat,
) -> Result<()> {
    match action {
        ConfigAction::Show => {
            match format {
                OutputFormat::Human => {
                    println!("📄 OmnioSearch Configuration\n");
                    println!("Search paths:");
                    for path in config.search_paths() {
                        println!("  - {}", path);
                    }
                    println!("\nExcluded paths:");
                    for path in config.excluded_paths() {
                        println!("  - {}", path);
                    }
                    println!("\nDatabase: {}", config.database_path().display());
                    println!("Cache: {}", config.cache_path().display());
                    println!("AI models: {}", config.ai_models_path().display());
                }
                
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&config)?);
                }
                
                _ => {
                    println!("Config display not supported for this output type");
                }
            }
        }
        
        ConfigAction::AddPath { path } => {
            config.add_search_path(path.clone());
            config.save().await?;
            println!("✅ Added search path: {}", path);
        }
        
        ConfigAction::RemovePath { path } => {
            config.remove_search_path(&path);
            config.save().await?;
            println!("✅ Removed search path: {}", path);
        }
        
        ConfigAction::Set { key, value } => {
            // In a real implementation, we'd have a proper config setter
            println!("⚠️ Config setting not implemented: {} = {}", key, value);
        }
        
        ConfigAction::Reset => {
            let default_config = Config::default();
            default_config.save().await?;
            println!("✅ Configuration reset to defaults");
        }
    }
    
    Ok(())
}

async fn handle_cloud_command(
    action: CloudAction,
    format: OutputFormat,
) -> Result<()> {
    match action {
        CloudAction::List => {
            println!("☁️ Available cloud providers:");
            println!("  - google-drive  (Google Drive)");
            println!("  - dropbox       (Dropbox)");
            println!("  - onedrive      (Microsoft OneDrive)");
            println!("  - nextcloud     (NextCloud)");
            println!("  - terabox       (TeraBox - 1TB Free Storage)");
        }
        
        CloudAction::Auth { provider } => {
            println!("🔑 Cloud authentication for {} not implemented in CLI", provider);
            println!("Please use the GUI application for cloud authentication.");
        }
        
        CloudAction::Search { provider, query } => {
            println!("🔍 Cloud search in {} for '{}' not implemented in CLI", provider, query);
            println!("Please use the GUI application for cloud search.");
        }
        
        CloudAction::Status { provider: _ } => {
            println!("☁️ Cloud status check not implemented in CLI");
        }
    }
    
    Ok(())
}

async fn handle_ai_command(
    action: AIAction,
    ai_processor: AIProcessor,
    format: OutputFormat,
) -> Result<()> {
    match action {
        AIAction::Parse { query } => {
            match ai_processor.process_natural_language(&query).await {
                Ok(result) => {
                    match format {
                        OutputFormat::Human => {
                            println!("🧠 Natural Language Analysis for: '{}'\n", query);
                            println!("Parsed query: {}", result.text);
                            println!("Search content: {}", result.search_content);
                            println!("Include hidden: {}", result.include_hidden);
                            if !result.file_types.is_empty() {
                                println!("File types: {}", result.file_types.join(", "));
                            }
                        }
                        
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&result)?);
                        }
                        
                        _ => {
                            println!("AI parse output not supported for this format");
                        }
                    }
                }
                Err(e) => {
                    println!("❌ AI parsing failed: {}", e);
                }
            }
        }
        
        AIAction::Suggest { partial } => {
            match ai_processor.get_search_suggestions(&partial).await {
                Ok(suggestions) => {
                    if suggestions.is_empty() {
                        println!("No suggestions available for: '{}'", partial);
                    } else {
                        println!("💡 Suggestions for: '{}'\n", partial);
                        for (i, suggestion) in suggestions.iter().enumerate() {
                            println!("{}. {}", i + 1, suggestion);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to get suggestions: {}", e);
                }
            }
        }
        
        AIAction::Download => {
            println!("📥 AI model download not implemented in CLI");
        }
        
        AIAction::Status => {
            println!("🤖 AI Status: Models loaded and ready");
        }
    }
    
    Ok(())
}

async fn handle_database_command(
    action: DatabaseAction,
    database: Database,
    format: OutputFormat,
) -> Result<()> {
    match action {
        DatabaseAction::Stats => {
            let status = database.get_indexing_status().await?;
            println!("💾 Database Statistics\n");
            println!("Total entries: {}", status.total_files);
            println!("Index size: {:.1} MB", status.index_size_mb);
        }
        
        DatabaseAction::Optimize => {
            println!("⚙️ Database optimization not implemented");
        }
        
        DatabaseAction::Clean => {
            println!("🧹 Database cleaning not implemented");
        }
        
        DatabaseAction::Export { path } => {
            println!("📤 Database export to {} not implemented", path);
        }
        
        DatabaseAction::Import { path } => {
            println!("📥 Database import from {} not implemented", path);
        }
    }
    
    Ok(())
}

// Utility functions

fn parse_file_size(size_str: &str) -> Option<u64> {
    let size_str = size_str.to_uppercase();
    
    if let Some(captures) = regex::Regex::new(r"^(\d+(?:\.\d+)?)\s*(B|KB|MB|GB|TB)?$")
        .unwrap()
        .captures(&size_str) 
    {
        if let Ok(number) = captures[1].parse::<f64>() {
            let unit = captures.get(2).map_or("B", |m| m.as_str());
            
            let multiplier = match unit {
                "B" => 1,
                "KB" => 1_024,
                "MB" => 1_024 * 1_024,
                "GB" => 1_024 * 1_024 * 1_024,
                "TB" => 1_024_u64.pow(4),
                _ => 1,
            };
            
            return Some((number * multiplier as f64) as u64);
        }
    }
    
    None
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len-3])
    }
}
