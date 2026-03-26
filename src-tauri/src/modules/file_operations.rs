use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncReadExt;
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;
use regex::Regex;
use crate::errors::{AppError, AppResult};
use tracing::{info, warn, error};

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<u64>,
    pub extension: Option<String>,
    pub permissions: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub content: String,
    pub match_text: String,
    pub context_before: Option<String>,
    pub context_after: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex: bool,
    pub include_binary: bool,
    pub max_results: Option<usize>,
    pub context_lines: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            regex: false,
            include_binary: false,
            max_results: Some(1000),
            context_lines: 2,
        }
    }
}

// Optimized: Async file reading with error handling
pub async fn read_file_async(file_path: &str) -> AppResult<String> {
    info!("Reading file: {}", file_path);
    
    // Validate file path
    validate_file_path(file_path)?;
    
    // Check file size to prevent reading extremely large files
    let metadata = fs::metadata(file_path).await?;
    if metadata.len() > 100_000_000 { // 100MB limit
        return Err(AppError::Validation(
            format!("File too large: {} bytes. Maximum allowed: 100MB", metadata.len())
        ));
    }
    
    match fs::read_to_string(file_path).await {
        Ok(content) => {
            info!("Successfully read file: {} ({} bytes)", file_path, content.len());
            Ok(content)
        }
        Err(e) => {
            error!("Failed to read file {}: {}", file_path, e);
            Err(AppError::FileSystem(format!("Failed to read file {}: {}", file_path, e)))
        }
    }
}

// Optimized: Async file writing with backup
pub async fn write_file_async(file_path: &str, content: &str, create_backup: bool) -> AppResult<()> {
    info!("Writing file: {} ({} bytes)", file_path, content.len());
    
    validate_file_path(file_path)?;
    
    // Create backup if requested and file exists
    if create_backup && Path::new(file_path).exists() {
        let backup_path = format!("{}.backup.{}", file_path, chrono::Utc::now().timestamp());
        if let Err(e) = fs::copy(file_path, backup_path).await {
            warn!("Failed to create backup: {}", e);
        }
    }
    
    // Write atomically using temporary file
    let temp_path = format!("{}.tmp.{}", file_path, uuid::Uuid::new_v4());
    
    match fs::write(&temp_path, content).await {
        Ok(_) => {
            // Atomic rename
            if let Err(e) = fs::rename(&temp_path, file_path).await {
                // Cleanup temp file
                let _ = fs::remove_file(&temp_path).await;
                return Err(AppError::FileSystem(format!("Failed to write file {}: {}", file_path, e)));
            }
            info!("Successfully wrote file: {}", file_path);
            Ok(())
        }
        Err(e) => {
            error!("Failed to write file {}: {}", file_path, e);
            // Cleanup temp file
            let _ = fs::remove_file(&temp_path).await;
            Err(AppError::FileSystem(format!("Failed to write file {}: {}", file_path, e)))
        }
    }
}

// Optimized: List files with detailed information
pub async fn list_files_detailed(dir_path: &str) -> AppResult<Vec<FileInfo>> {
    info!("Listing files in directory: {}", dir_path);
    
    validate_file_path(dir_path)?;
    
    let mut entries = fs::read_dir(dir_path).await
        .map_err(|e| AppError::FileSystem(format!("Failed to read directory {}: {}", dir_path, e)))?;
    
    let mut file_infos = Vec::new();
    
    while let Some(entry) = entries.next_entry().await
        .map_err(|e| AppError::FileSystem(format!("Failed to read directory entry: {}", e)))? {
        
        let path = entry.path();
        let metadata = entry.metadata().await
            .map_err(|e| AppError::FileSystem(format!("Failed to get metadata: {}", e)))?;
        
        let modified = metadata.modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());
        
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string());
        
        let name = entry.file_name().to_string_lossy().to_string();
        let path_str = path.to_string_lossy().to_string();
        
        // Get MIME type for files
        let mime_type = if metadata.is_file() {
            infer_mime_type(&path_str)
        } else {
            None
        };
        
        file_infos.push(FileInfo {
            name,
            path: path_str,
            is_dir: metadata.is_dir(),
            size: if metadata.is_file() { Some(metadata.len()) } else { None },
            modified,
            extension,
            permissions: format_permissions(&metadata),
            mime_type,
        });
    }
    
    // Sort by name
    file_infos.sort_by(|a, b| a.name.cmp(&b.name));
    
    info!("Listed {} items in directory: {}", file_infos.len(), dir_path);
    Ok(file_infos)
}

// Optimized: Advanced file search with context
pub async fn search_in_files_advanced(
    pattern: &str,
    dir_path: &str,
    file_extensions: Option<Vec<String>>,
    options: SearchOptions,
) -> AppResult<Vec<SearchResult>> {
    info!("Searching for pattern '{}' in directory: {}", pattern, dir_path);
    
    validate_file_path(dir_path)?;
    
    let regex = if options.regex {
        if options.case_sensitive {
            Regex::new(pattern)
        } else {
            Regex::new(&format!("(?i){}", pattern))
        }
    } else {
        let escaped_pattern = regex::escape(pattern);
        let word_pattern = if options.whole_word {
            format!(r"\b{}\b", escaped_pattern)
        } else {
            escaped_pattern
        };
        
        if options.case_sensitive {
            Regex::new(&word_pattern)
        } else {
            Regex::new(&format!("(?i){}", word_pattern))
        }
    }.map_err(|e| AppError::Validation(format!("Invalid regex pattern: {}", e)))?;
    
    let mut results = Vec::new();
    let mut processed_files = 0;
    
    for entry in WalkDir::new(dir_path).max_depth(10) {
        let entry = entry.map_err(|e| AppError::FileSystem(e.to_string()))?;
        
        if !entry.file_type().is_file() {
            continue;
        }
        
        // Filter by file extensions if provided
        if let Some(ref extensions) = file_extensions {
            if let Some(file_ext) = entry.path().extension().and_then(|s| s.to_str()) {
                if !extensions.contains(&file_ext.to_string()) {
                    continue;
                }
            }
        }
        
        // Skip binary files unless explicitly included
        if !options.include_binary && is_binary_file(entry.path()).await? {
            continue;
        }
        
        match search_in_file(entry.path(), &regex, &options).await {
            Ok(mut file_results) => {
                results.append(&mut file_results);
                processed_files += 1;
                
                // Check max results limit
                if let Some(max) = options.max_results {
                    if results.len() >= max {
                        break;
                    }
                }
            }
            Err(e) => {
                warn!("Failed to search in file {:?}: {}", entry.path(), e);
                continue;
            }
        }
    }
    
    info!("Search completed. Found {} results in {} files", results.len(), processed_files);
    Ok(results)
}

async fn search_in_file(
    file_path: &Path,
    regex: &Regex,
    options: &SearchOptions,
) -> AppResult<Vec<SearchResult>> {
    let content = fs::read_to_string(file_path).await?;
    let lines: Vec<&str> = content.lines().collect();
    let mut results = Vec::new();
    
    for (index, line) in lines.iter().enumerate() {
        if regex.is_match(line) {
            if let Some(match_result) = regex.find(line) {
                let context_before = if options.context_lines > 0 && index > 0 {
                    let start = index.saturating_sub(options.context_lines);
                    Some(lines[start..index].join("\n"))
                } else {
                    None
                };
                
                let context_after = if options.context_lines > 0 && index < lines.len() - 1 {
                    let end = std::cmp::min(index + options.context_lines + 1, lines.len());
                    Some(lines[index + 1..end].join("\n"))
                } else {
                    None
                };
                
                results.push(SearchResult {
                    file_path: file_path.display().to_string(),
                    line_number: index + 1,
                    content: line.to_string(),
                    match_text: match_result.as_str().to_string(),
                    context_before,
                    context_after,
                });
            }
        }
    }
    
    Ok(results)
}

async fn is_binary_file(file_path: &Path) -> AppResult<bool> {
    let mut file = fs::File::open(file_path).await?;
    let mut buffer = [0; 1024];
    let bytes_read = file.read(&mut buffer).await?;
    
    // Check for null bytes (common in binary files)
    Ok(buffer[..bytes_read].contains(&0))
}

fn validate_file_path(path: &str) -> AppResult<()> {
    if path.is_empty() {
        return Err(AppError::Validation("File path cannot be empty".to_string()));
    }
    
    // Basic security check - prevent path traversal
    if path.contains("..") {
        return Err(AppError::Security("Path traversal detected".to_string()));
    }
    
    // Check for null bytes
    if path.contains('\0') {
        return Err(AppError::Security("Null byte in path".to_string()));
    }
    
    Ok(())
}

fn infer_mime_type(file_path: &str) -> Option<String> {
    infer::get_from_path(file_path)
        .ok()
        .flatten()
        .map(|kind| kind.mime_type().to_string())
}

fn format_permissions(metadata: &std::fs::Metadata) -> Option<String> {
    use std::os::unix::fs::PermissionsExt;
    
    let mode = metadata.permissions().mode();
    Some(format!("{:o}", mode & 0o777))
}

// File watcher functionality for real-time updates
pub struct FileWatcher {
    // Implementation would use notify crate for file system events
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {}
    }
    
    pub async fn watch_directory(&self, _path: &str) -> AppResult<()> {
        // Implementation would set up file system watching
        // This is a placeholder for future implementation
        Ok(())
    }
}
