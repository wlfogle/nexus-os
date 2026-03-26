// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};
use tauri::{Manager, State};

mod search;
mod ai;
mod db;
mod cloud;
mod config;
mod indexer;

use search::{SearchEngine, SearchQuery, SearchResult};
use ai::AIProcessor;
use db::Database;
use config::Config;
use indexer::FileIndexer;

#[derive(Clone)]
pub struct AppState {
    pub search_engine: Arc<RwLock<SearchEngine>>,
    pub ai_processor: Arc<RwLock<AIProcessor>>,
    pub database: Arc<RwLock<Database>>,
    pub config: Arc<RwLock<Config>>,
    pub indexer: Arc<RwLock<FileIndexer>>,
}

impl AppState {
    async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔍 Initializing OmnioSearch - The All-Seeing File Search Engine...");

        // Load configuration
        let config = Config::load().await?;
        info!("✅ Configuration loaded");

        // Initialize database
        let database = Database::new(&config.database_path()).await?;
        info!("✅ Database initialized");

        // Initialize AI processor
        let ai_processor = AIProcessor::new(&config).await?;
        info!("✅ AI processor initialized with local models");

        // Initialize file indexer
        let indexer = FileIndexer::new(database.clone(), config.clone()).await?;
        info!("✅ File indexer initialized");

        // Initialize search engine
        let search_engine = SearchEngine::new(
            database.clone(),
            ai_processor.clone(),
            config.clone(),
            indexer.clone()
        ).await?;
        info!("✅ Search engine initialized");

        Ok(Self {
            search_engine: Arc::new(RwLock::new(search_engine)),
            ai_processor: Arc::new(RwLock::new(ai_processor)),
            database: Arc::new(RwLock::new(database)),
            config: Arc::new(RwLock::new(config)),
            indexer: Arc::new(RwLock::new(indexer)),
        })
    }
}

#[tauri::command]
async fn search_files(
    query: String,
    app_state: State<'_, AppState>
) -> Result<Vec<SearchResult>, String> {
    info!("🔍 Searching for: {}", query);
    
    let search_engine = app_state.search_engine.read().await;
    match search_engine.search(&query).await {
        Ok(results) => {
            info!("✅ Found {} results", results.len());
            Ok(results)
        }
        Err(e) => {
            error!("❌ Search error: {}", e);
            Err(format!("Search failed: {}", e))
        }
    }
}

#[tauri::command]
async fn natural_language_search(
    query: String,
    app_state: State<'_, AppState>
) -> Result<Vec<SearchResult>, String> {
    info!("🧠 Natural language search: {}", query);
    
    let ai_processor = app_state.ai_processor.read().await;
    let structured_query = match ai_processor.process_natural_language(&query).await {
        Ok(query) => query,
        Err(e) => {
            warn!("⚠️ AI processing failed, falling back to regular search: {}", e);
            SearchQuery::from_text(&query)
        }
    };
    
    let search_engine = app_state.search_engine.read().await;
    match search_engine.search_with_query(&structured_query).await {
        Ok(results) => {
            info!("✅ NL search found {} results", results.len());
            Ok(results)
        }
        Err(e) => {
            error!("❌ Natural language search error: {}", e);
            Err(format!("Search failed: {}", e))
        }
    }
}

#[tauri::command]
async fn start_indexing(
    paths: Vec<String>,
    app_state: State<'_, AppState>
) -> Result<(), String> {
    info!("📂 Starting indexing for paths: {:?}", paths);
    
    let indexer = app_state.indexer.clone();
    tokio::spawn(async move {
        let indexer_guard = indexer.read().await;
        if let Err(e) = indexer_guard.start_indexing(paths).await {
            error!("❌ Indexing error: {}", e);
        }
    });
    
    Ok(())
}

#[tauri::command]
async fn get_indexing_status(
    app_state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    let indexer = app_state.indexer.read().await;
    match indexer.get_status().await {
        Ok(status) => Ok(serde_json::to_value(status).unwrap()),
        Err(e) => Err(format!("Failed to get status: {}", e))
    }
}

#[tauri::command]
async fn get_search_suggestions(
    query: String,
    app_state: State<'_, AppState>
) -> Result<Vec<String>, String> {
    let ai_processor = app_state.ai_processor.read().await;
    match ai_processor.get_search_suggestions(&query).await {
        Ok(suggestions) => Ok(suggestions),
        Err(e) => {
            warn!("⚠️ Failed to get AI suggestions: {}", e);
            Ok(vec![]) // Return empty suggestions on failure
        }
    }
}

#[tauri::command]
async fn add_to_cloud_search(
    provider: String,
    app_state: State<'_, AppState>
) -> Result<(), String> {
    info!("☁️ Adding cloud provider: {}", provider);
    
    let search_engine = app_state.search_engine.write().await;
    match search_engine.add_cloud_provider(&provider).await {
        Ok(_) => {
            info!("✅ Cloud provider added successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to add cloud provider: {}", e);
            Err(format!("Cloud integration failed: {}", e))
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter("omniosearch=info,tauri=info")
        .init();

    info!("🚀 OmnioSearch starting up...");

    // Initialize app state
    let app_state = match AppState::new().await {
        Ok(state) => {
            info!("👁️ OmnioSearch initialized successfully - The Omnio Vision is active!");
            state
        }
        Err(e) => {
            error!("💥 Failed to initialize OmnioSearch: {}", e);
            panic!("Initialization failed: {}", e);
        }
    };

    // Build and run Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            search_files,
            natural_language_search,
            start_indexing,
            get_indexing_status,
            get_search_suggestions,
            add_to_cloud_search
        ])
        .setup(|app| {
            info!("🎨 Setting up Tauri frontend...");
            
            // Start background indexing of common directories
            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                if let Ok(app_state) = app_handle.state::<AppState>().try_lock() {
                    let home_dir = dirs::home_dir().unwrap_or_else(|| "/home".into());
                    let default_paths = vec![
                        home_dir.to_string_lossy().to_string(),
                        "/usr/share/applications".to_string(),
                        "/opt".to_string()
                    ];
                    
                    let indexer = app_state.indexer.clone();
                    tokio::spawn(async move {
                        let indexer_guard = indexer.read().await;
                        if let Err(e) = indexer_guard.start_indexing(default_paths).await {
                            error!("❌ Background indexing error: {}", e);
                        }
                    });
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
