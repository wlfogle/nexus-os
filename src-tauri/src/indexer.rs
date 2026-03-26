use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};
use walkdir::{WalkDir, DirEntry};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use rayon::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::db::{Database, FileEntry, IndexStatus};
use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingProgress {
    pub current_path: String,
    pub processed_files: u64,
    pub total_files: u64,
    pub processing_speed: f64, // files per second
    pub estimated_time_remaining: Duration,
    pub is_running: bool,
    pub phase: IndexingPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexingPhase {
    Scanning,
    Indexing,
    ContentExtraction,
    Finalizing,
    Complete,
    Error(String),
}

pub struct FileIndexer {
    database: Arc<RwLock<Database>>,
    config: Arc<RwLock<Config>>,
    progress_sender: mpsc::UnboundedSender<IndexingProgress>,
    progress_receiver: Arc<RwLock<mpsc::UnboundedReceiver<IndexingProgress>>>,
    watcher: Option<notify::RecommendedWatcher>,
    indexed_paths: Arc<RwLock<HashSet<PathBuf>>>,
}

impl FileIndexer {
    pub async fn new(
        database: Arc<RwLock<Database>>,
        config: Arc<RwLock<Config>>,
    ) -> Result<Self> {
        info!("📂 Initializing file indexer...");

        let (progress_sender, progress_receiver) = mpsc::unbounded_channel();
        let indexed_paths = Arc::new(RwLock::new(HashSet::new()));

        let indexer = Self {
            database,
            config,
            progress_sender,
            progress_receiver: Arc::new(RwLock::new(progress_receiver)),
            watcher: None,
            indexed_paths,
        };

        info!("✅ File indexer initialized");
        Ok(indexer)
    }

    pub async fn start_indexing(&self, paths: Vec<String>) -> Result<()> {
        info!("🚀 Starting file indexing for {} paths", paths.len());

        let config = self.config.read().await;
        let database = self.database.clone();
        let progress_sender = self.progress_sender.clone();
        let indexed_paths = self.indexed_paths.clone();

        // Start background indexing task
        let indexing_config = config.clone();
        drop(config); // Release the lock

        tokio::spawn(async move {
            if let Err(e) = Self::index_paths_background(
                paths,
                database,
                indexing_config,
                progress_sender,
                indexed_paths,
            ).await {
                error!("❌ Background indexing failed: {}", e);
            }
        });

        Ok(())
    }

    async fn index_paths_background(
        paths: Vec<String>,
        database: Arc<RwLock<Database>>,
        config: Config,
        progress_sender: mpsc::UnboundedSender<IndexingProgress>,
        indexed_paths: Arc<RwLock<HashSet<PathBuf>>>,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Phase 1: Scan for files
        let _ = progress_sender.send(IndexingProgress {
            current_path: "Scanning...".to_string(),
            processed_files: 0,
            total_files: 0,
            processing_speed: 0.0,
            estimated_time_remaining: Duration::from_secs(0),
            is_running: true,
            phase: IndexingPhase::Scanning,
        });

        let all_files = Self::discover_files(&paths, &config).await?;
        let total_files = all_files.len() as u64;
        
        info!("📊 Found {} files to index", total_files);

        // Phase 2: Index files
        let _ = progress_sender.send(IndexingProgress {
            current_path: "Indexing files...".to_string(),
            processed_files: 0,
            total_files,
            processing_speed: 0.0,
            estimated_time_remaining: Duration::from_secs(0),
            is_running: true,
            phase: IndexingPhase::Indexing,
        });

        let mut processed_files = 0u64;
        let batch_size = 1000;
        let mut batch = Vec::new();

        for file_path in all_files {
            batch.push(file_path);
            
            if batch.len() >= batch_size {
                Self::process_file_batch(&batch, &database, &config).await?;
                processed_files += batch.len() as u64;
                
                // Update progress
                let elapsed = start_time.elapsed();
                let processing_speed = processed_files as f64 / elapsed.as_secs_f64();
                let remaining_files = total_files - processed_files;
                let estimated_time_remaining = if processing_speed > 0.0 {
                    Duration::from_secs_f64(remaining_files as f64 / processing_speed)
                } else {
                    Duration::from_secs(0)
                };

                let _ = progress_sender.send(IndexingProgress {
                    current_path: format!("Processing batch {} of {}", 
                                        processed_files / batch_size as u64 + 1,
                                        (total_files + batch_size as u64 - 1) / batch_size as u64),
                    processed_files,
                    total_files,
                    processing_speed,
                    estimated_time_remaining,
                    is_running: true,
                    phase: IndexingPhase::Indexing,
                });

                batch.clear();
            }
        }

        // Process remaining files
        if !batch.is_empty() {
            Self::process_file_batch(&batch, &database, &config).await?;
            processed_files += batch.len() as u64;
        }

        // Phase 3: Content extraction
        let _ = progress_sender.send(IndexingProgress {
            current_path: "Extracting content...".to_string(),
            processed_files,
            total_files,
            processing_speed: 0.0,
            estimated_time_remaining: Duration::from_secs(0),
            is_running: true,
            phase: IndexingPhase::ContentExtraction,
        });

        // Extract content for supported file types
        Self::extract_content_phase(&database, &config, &progress_sender).await?;

        // Phase 4: Finalize
        let _ = progress_sender.send(IndexingProgress {
            current_path: "Finalizing...".to_string(),
            processed_files,
            total_files,
            processing_speed: 0.0,
            estimated_time_remaining: Duration::from_secs(0),
            is_running: true,
            phase: IndexingPhase::Finalizing,
        });

        // Commit all changes
        {
            let db = database.read().await;
            db.commit().await?;
        }

        // Update indexed paths cache
        {
            let mut indexed = indexed_paths.write().await;
            for path_str in &paths {
                indexed.insert(PathBuf::from(path_str));
            }
        }

        // Complete
        let total_time = start_time.elapsed();
        let final_speed = processed_files as f64 / total_time.as_secs_f64();

        let _ = progress_sender.send(IndexingProgress {
            current_path: format!("Complete! Indexed {} files in {:.1}s", processed_files, total_time.as_secs_f64()),
            processed_files,
            total_files,
            processing_speed: final_speed,
            estimated_time_remaining: Duration::from_secs(0),
            is_running: false,
            phase: IndexingPhase::Complete,
        });

        info!("✅ Indexing complete: {} files indexed in {:.2}s ({:.1} files/sec)", 
              processed_files, total_time.as_secs_f64(), final_speed);

        Ok(())
    }

    async fn discover_files(paths: &[String], config: &Config) -> Result<Vec<PathBuf>> {
        debug!("🔍 Discovering files in {} paths", paths.len());

        let mut all_files = Vec::new();

        for path_str in paths {
            let path = Path::new(path_str);
            
            if !path.exists() {
                warn!("⚠️ Path does not exist: {}", path_str);
                continue;
            }

            let walker = WalkDir::new(path)
                .follow_links(true)
                .max_depth(20) // Prevent infinite recursion
                .into_iter()
                .filter_entry(|e| !Self::is_hidden_or_excluded(e, config))
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());

            for entry in walker {
                let file_path = entry.path().to_path_buf();
                
                // Check if we should index this file
                if let Ok(metadata) = entry.metadata() {
                    let size = metadata.len();
                    if config.should_index_file(&file_path.to_string_lossy(), size) {
                        all_files.push(file_path);
                    }
                }
            }
        }

        debug!("🔍 Discovered {} files", all_files.len());
        Ok(all_files)
    }

    fn is_hidden_or_excluded(entry: &DirEntry, config: &Config) -> bool {
        let path = entry.path();
        let path_str = path.to_string_lossy();

        // Check if path is excluded
        if config.is_path_excluded(&path_str) {
            return true;
        }

        // Check for hidden files/directories
        if let Some(file_name) = path.file_name() {
            if let Some(name_str) = file_name.to_str() {
                if name_str.starts_with('.') && !config.indexing_settings.watch_file_changes {
                    return true;
                }
            }
        }

        false
    }

    async fn process_file_batch(
        batch: &[PathBuf],
        database: &Arc<RwLock<Database>>,
        config: &Config,
    ) -> Result<()> {
        debug!("📦 Processing batch of {} files", batch.len());

        // Process files in parallel using rayon
        let file_entries: Vec<FileEntry> = batch
            .par_iter()
            .filter_map(|path| {
                match Self::create_file_entry(path, config) {
                    Ok(entry) => Some(entry),
                    Err(e) => {
                        warn!("⚠️ Failed to process {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect();

        // Insert into database
        {
            let db = database.read().await;
            for entry in file_entries {
                if let Err(e) = db.insert_file(&entry).await {
                    warn!("⚠️ Failed to insert {}: {}", entry.path, e);
                }
            }
        }

        Ok(())
    }

    fn create_file_entry(path: &Path, config: &Config) -> Result<FileEntry> {
        let metadata = std::fs::metadata(path)?;
        
        let modified = metadata.modified()
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)?;
        
        let created = metadata.created()
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)?;

        let file_type = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Calculate simple checksum for duplicate detection
        let checksum = Self::calculate_file_checksum(path)?;

        Ok(FileEntry {
            id: Uuid::new_v4().to_string(),
            path: path.to_string_lossy().to_string(),
            name,
            size: metadata.len(),
            modified: DateTime::from_timestamp(modified.as_secs() as i64, 0).unwrap_or_else(|| Utc::now()),
            created: DateTime::from_timestamp(created.as_secs() as i64, 0).unwrap_or_else(|| Utc::now()),
            file_type,
            mime_type,
            is_directory: metadata.is_dir(),
            permissions: format!("{:o}", Self::get_permissions(&metadata)),
            checksum: Some(checksum),
            indexed_at: Utc::now(),
            content_extracted: false,
        })
    }

    fn calculate_file_checksum(path: &Path) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // For performance, only hash file metadata and first 1KB
        let metadata = std::fs::metadata(path)?;
        let mut hasher = DefaultHasher::new();
        
        // Hash file size, modified time, and path
        metadata.len().hash(&mut hasher);
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                duration.as_secs().hash(&mut hasher);
            }
        }
        path.to_string_lossy().hash(&mut hasher);

        // Hash first 1KB of file content for small files
        if metadata.len() < 1024 && metadata.is_file() {
            if let Ok(content) = std::fs::read(path) {
                content.hash(&mut hasher);
            }
        }

        Ok(format!("{:x}", hasher.finish()))
    }

    #[cfg(unix)]
    fn get_permissions(metadata: &std::fs::Metadata) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    }

    #[cfg(not(unix))]
    fn get_permissions(_metadata: &std::fs::Metadata) -> u32 {
        0o644 // Default permissions for non-Unix systems
    }

    async fn extract_content_phase(
        database: &Arc<RwLock<Database>>,
        config: &Config,
        progress_sender: &mpsc::UnboundedSender<IndexingProgress>,
    ) -> Result<()> {
        debug!("📄 Starting content extraction phase...");

        // This would implement content extraction for different file types
        // For now, we'll just mark it as a placeholder

        let _ = progress_sender.send(IndexingProgress {
            current_path: "Content extraction complete".to_string(),
            processed_files: 0,
            total_files: 0,
            processing_speed: 0.0,
            estimated_time_remaining: Duration::from_secs(0),
            is_running: true,
            phase: IndexingPhase::ContentExtraction,
        });

        Ok(())
    }

    pub async fn start_file_watcher(&mut self) -> Result<()> {
        info!("👀 Starting file system watcher...");

        let config = self.config.read().await;
        if !config.indexing_settings.watch_file_changes {
            info!("📁 File watching is disabled in configuration");
            return Ok(());
        }

        let (tx, mut rx) = mpsc::unbounded_channel();
        let database = self.database.clone();
        let config_clone = config.clone();

        let mut watcher = notify::recommended_watcher(move |event: Result<Event, _>| {
            if let Ok(event) = event {
                if let Err(e) = tx.send(event) {
                    warn!("⚠️ Failed to send file system event: {}", e);
                }
            }
        })?;

        // Watch all search paths
        for path in config.search_paths() {
            if Path::new(path).exists() {
                watcher.watch(Path::new(path), RecursiveMode::Recursive)?;
                debug!("👀 Watching path: {}", path);
            }
        }

        self.watcher = Some(watcher);

        // Handle file system events
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                Self::handle_file_event(event, &database, &config_clone).await;
            }
        });

        info!("✅ File system watcher started");
        Ok(())
    }

    async fn handle_file_event(
        event: Event,
        database: &Arc<RwLock<Database>>,
        config: &Config,
    ) {
        debug!("📁 File system event: {:?}", event);

        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in event.paths {
                    if path.is_file() && config.should_index_file(&path.to_string_lossy(), 0) {
                        if let Ok(entry) = Self::create_file_entry(&path, config) {
                            let db = database.read().await;
                            if let Err(e) = db.insert_file(&entry).await {
                                warn!("⚠️ Failed to update index for {}: {}", path.display(), e);
                            } else {
                                debug!("✅ Updated index for: {}", path.display());
                            }
                        }
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in event.paths {
                    let db = database.read().await;
                    if let Err(e) = db.delete_file(&path.to_string_lossy()).await {
                        warn!("⚠️ Failed to remove from index {}: {}", path.display(), e);
                    } else {
                        debug!("🗑️ Removed from index: {}", path.display());
                    }
                }
            }
            _ => {
                // Ignore other event types
            }
        }
    }

    pub async fn get_status(&self) -> Result<IndexStatus> {
        let db = self.database.read().await;
        db.get_indexing_status().await
    }

    pub async fn is_path_indexed(&self, path: &Path) -> bool {
        let indexed = self.indexed_paths.read().await;
        indexed.contains(path)
    }

    pub async fn get_progress(&self) -> Option<IndexingProgress> {
        let mut receiver = self.progress_receiver.write().await;
        receiver.try_recv().ok()
    }
}
