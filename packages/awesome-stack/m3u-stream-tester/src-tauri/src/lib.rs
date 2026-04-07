use std::fs;
use std::path::Path;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Deserialize, Serialize};
use regex::Regex;
use futures::future::join_all;
use tokio::time::timeout;
use tauri::Emitter;

static SHOULD_STOP: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StreamInfo {
    name: String,
    url: String,
    group_title: Option<String>,
    tvg_id: Option<String>,
    tvg_name: Option<String>,
    tvg_logo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestResult {
    stream: StreamInfo,
    working: bool,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestProgress {
    tested: usize,
    total: usize,
    working: usize,
    current_stream: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CategoryStats {
    name: String,
    total: usize,
    working: usize,
    failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestSummary {
    total_streams: usize,
    working_streams: usize,
    failed_streams: usize,
    categories: Vec<CategoryStats>,
}

#[tauri::command]
async fn test_streams_from_folder(
    folder_path: String,
    output_path: String,
    timeout_seconds: u64,
    max_concurrent: usize,
    window: tauri::Window,
) -> Result<Vec<TestResult>, String> {
    let folder = Path::new(&folder_path);
    
    if !folder.exists() || !folder.is_dir() {
        return Err("Invalid folder path".to_string());
    }

    // Find all M3U files
    let mut m3u_files = Vec::new();
    for entry in fs::read_dir(folder).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "m3u" || extension == "m3u8" {
                m3u_files.push(path);
            }
        }
    }

    if m3u_files.is_empty() {
        return Err("No M3U files found in the folder".to_string());
    }

    // Parse all M3U files
    let mut all_streams = Vec::new();
    for m3u_file in &m3u_files {
        let streams = parse_m3u_file(m3u_file).map_err(|e| e.to_string())?;
        all_streams.extend(streams);
    }

    // Remove duplicates based on URL
    let mut unique_streams = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();
    for stream in all_streams {
        if !seen_urls.contains(&stream.url) {
            seen_urls.insert(stream.url.clone());
            unique_streams.push(stream);
        }
    }

    // Emit initial progress
    let _ = window.emit("progress", TestProgress {
        tested: 0,
        total: unique_streams.len(),
        working: 0,
        current_stream: Some("Starting tests...".to_string()),
    });

    // Reset stop flag and test streams
    SHOULD_STOP.store(false, Ordering::Relaxed);
    let results = test_streams_batch(unique_streams, timeout_seconds, max_concurrent, window.clone()).await;
    
// Group by categories
    let mut category_map: std::collections::HashMap<String, CategoryStats> = std::collections::HashMap::new();
    println!("Processing {} results for categories", results.len());
    
    for result in &results {
        let category_name = categorize_stream(&result.stream);
        println!("Stream '{}' has category: '{}'", result.stream.name, category_name);
        
        let stats = category_map.entry(category_name.clone()).or_insert(CategoryStats {
            name: category_name.clone(),
            total: 0,
            working: 0,
            failed: 0,
        });
        stats.total += 1;
        if result.working {
            stats.working += 1;
        } else {
            stats.failed += 1;
        }
    }

    let mut categories: Vec<CategoryStats> = category_map.into_iter().map(|(_, v)| v).collect();
    categories.sort_by(|a, b| a.name.cmp(&b.name));
    
    println!("Found {} categories: {:?}", categories.len(), categories.iter().map(|c| &c.name).collect::<Vec<_>>());

    // Save working streams
    let working_streams: Vec<_> = results.iter()
        .filter(|r| r.working)
        .map(|r| r.stream.clone())
        .collect();
    
    save_working_streams(&working_streams, &output_path).map_err(|e| e.to_string())?;
    
    // Emit category statistics
    let _ = window.emit("categories", TestSummary {
        total_streams: results.len(),
        working_streams: working_streams.len(),
        failed_streams: results.len() - working_streams.len(),
        categories: categories.clone(),
    });
    
    Ok(results)
}

#[tauri::command]
async fn test_single_stream(url: String, timeout_seconds: u64) -> Result<bool, String> {
    test_stream_url(&url, timeout_seconds).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_testing() -> Result<(), String> {
    SHOULD_STOP.store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
async fn save_streams_by_category(
    streams: Vec<TestResult>,
    output_folder: String,
    only_working: bool,
) -> Result<Vec<String>, String> {
    let mut saved_files = Vec::new();
    let mut category_map: std::collections::HashMap<String, Vec<StreamInfo>> = std::collections::HashMap::new();
    
    for result in streams {
        if only_working && !result.working {
            continue;
        }
        
        let category_name = categorize_stream(&result.stream);
        category_map.entry(category_name).or_insert_with(Vec::new).push(result.stream);
    }
    
    for (category, streams) in category_map {
        let filename = format!("{}/{}.m3u8", output_folder, sanitize_filename(&category));
        save_working_streams(&streams, &filename).map_err(|e| e.to_string())?;
        saved_files.push(filename);
    }
    
    Ok(saved_files)
}

fn is_placeholder_title(title: &str) -> bool {
    let title_lower = title.to_lowercase();
    title_lower == "(no name)" || 
    title_lower == "no name" ||
    title_lower == "unknown" ||
    title_lower == "untitled" ||
    title_lower == "no title" ||
    title_lower == "no group" ||
    title_lower == "default" ||
    title_lower == "misc" ||
    title_lower == "other" ||
    title_lower == "n/a" ||
    title_lower == "na" ||
    title_lower.is_empty()
}

fn categorize_stream(stream: &StreamInfo) -> String {
    // If group-title exists and is meaningful, use it
    if let Some(ref group_title) = stream.group_title {
        if !group_title.is_empty() && !is_placeholder_title(group_title) {
            return group_title.clone();
        }
    }
    
    // Smart categorization based on name and URL
    let name_lower = stream.name.to_lowercase();
    let url_lower = stream.url.to_lowercase();
    
    // News channels
    if name_lower.contains("news") || name_lower.contains("cnn") || name_lower.contains("fox") 
        || name_lower.contains("bbc") || name_lower.contains("msnbc") || name_lower.contains("abc") 
        || name_lower.contains("cbs") || name_lower.contains("nbc") || name_lower.contains("al jazeera")
        || name_lower.contains("reuters") || name_lower.contains("sky news") {
        return "News".to_string();
    }
    
    // Sports channels
    if name_lower.contains("sport") || name_lower.contains("espn") || name_lower.contains("fox sports")
        || name_lower.contains("nfl") || name_lower.contains("nba") || name_lower.contains("mlb")
        || name_lower.contains("nhl") || name_lower.contains("football") || name_lower.contains("soccer")
        || name_lower.contains("tennis") || name_lower.contains("golf") || name_lower.contains("racing")
        || name_lower.contains("boxing") || name_lower.contains("mma") || name_lower.contains("ufc") {
        return "Sports".to_string();
    }
    
    // Entertainment channels
    if name_lower.contains("entertainment") || name_lower.contains("e!") || name_lower.contains("bravo")
        || name_lower.contains("tlc") || name_lower.contains("lifetime") || name_lower.contains("vh1")
        || name_lower.contains("mtv") || name_lower.contains("comedy") || name_lower.contains("drama") {
        return "Entertainment".to_string();
    }
    
    // Movie channels
    if name_lower.contains("movie") || name_lower.contains("cinema") || name_lower.contains("film")
        || name_lower.contains("hbo") || name_lower.contains("showtime") || name_lower.contains("starz")
        || name_lower.contains("cinemax") || name_lower.contains("premium") {
        return "Movies".to_string();
    }
    
    // Kids channels
    if name_lower.contains("kids") || name_lower.contains("cartoon") || name_lower.contains("disney")
        || name_lower.contains("nickelodeon") || name_lower.contains("nick") || name_lower.contains("pbskids")
        || name_lower.contains("children") || name_lower.contains("family") {
        return "Kids".to_string();
    }
    
    // Music channels
    if name_lower.contains("music") || name_lower.contains("radio") || name_lower.contains("fm ")
        || name_lower.contains("am ") || name_lower.contains("hits") || name_lower.contains("rock")
        || name_lower.contains("pop") || name_lower.contains("jazz") || name_lower.contains("classical") {
        return "Music".to_string();
    }
    
    // Documentary/Educational
    if name_lower.contains("discovery") || name_lower.contains("national geographic")
        || name_lower.contains("history") || name_lower.contains("documentary") || name_lower.contains("science")
        || name_lower.contains("nature") || name_lower.contains("animal") || name_lower.contains("education") {
        return "Documentary".to_string();
    }
    
    // International/Regional
    if name_lower.contains("spanish") || name_lower.contains("latino") || name_lower.contains("french")
        || name_lower.contains("german") || name_lower.contains("italian") || name_lower.contains("chinese")
        || name_lower.contains("japanese") || name_lower.contains("korean") || name_lower.contains("arabic")
        || name_lower.contains("hindi") || name_lower.contains("international") {
        return "International".to_string();
    }
    
    // Local/Regional US
    if name_lower.contains(" tv") || name_lower.contains("local") || name_lower.contains("regional")
        || name_lower.contains("channel") || name_lower.contains("broadcast") {
        return "Local".to_string();
    }
    
    // Check URL for hints
    if url_lower.contains("news") {
        return "News".to_string();
    } else if url_lower.contains("sport") {
        return "Sports".to_string();
    } else if url_lower.contains("movie") || url_lower.contains("film") {
        return "Movies".to_string();
    } else if url_lower.contains("music") || url_lower.contains("radio") {
        return "Music".to_string();
    }
    
    // Default category
    "General".to_string()
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

fn parse_m3u_file(file_path: &Path) -> Result<Vec<StreamInfo>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    let mut streams = Vec::new();
    let mut current_info = StreamInfo {
        name: String::new(),
        url: String::new(),
        group_title: None,
        tvg_id: None,
        tvg_name: None,
        tvg_logo: None,
    };
    
    let extinf_regex = Regex::new(r"#EXTINF:.*?,(.*)")?;
    let attr_regex = Regex::new(r#"([\w-]+)="([^"]*)"|([\w-]+)=([^\s,]+)"#)?;
    
    for line in lines {
        let line = line.trim();
        
        if line.starts_with("#EXTINF:") {
            // Extract channel name
            if let Some(caps) = extinf_regex.captures(line) {
                current_info.name = caps.get(1).unwrap().as_str().trim().to_string();
            }
            
            // Extract attributes
            for caps in attr_regex.captures_iter(line) {
                let (attr_name, attr_value) = if let (Some(name), Some(value)) = (caps.get(1), caps.get(2)) {
                    (name.as_str(), value.as_str())
                } else if let (Some(name), Some(value)) = (caps.get(3), caps.get(4)) {
                    (name.as_str(), value.as_str())
                } else {
                    continue;
                };
                
                println!("Found attribute: {}='{}'", attr_name, attr_value);
                
                match attr_name {
                    "group-title" => current_info.group_title = Some(attr_value.to_string()),
                    "tvg-id" => current_info.tvg_id = Some(attr_value.to_string()),
                    "tvg-name" => current_info.tvg_name = Some(attr_value.to_string()),
                    "tvg-logo" => current_info.tvg_logo = Some(attr_value.to_string()),
                    _ => {}
                }
            }
        } else if !line.is_empty() && !line.starts_with('#') && line.starts_with("http") {
            // This is a stream URL
            current_info.url = line.to_string();
            
            if current_info.name.is_empty() {
                current_info.name = format!("Stream {}", streams.len() + 1);
            }
            
            println!("Adding stream: '{}' with group_title: {:?}", current_info.name, current_info.group_title);
            streams.push(current_info.clone());
            
            // Reset for next stream
            current_info = StreamInfo {
                name: String::new(),
                url: String::new(),
                group_title: None,
                tvg_id: None,
                tvg_name: None,
                tvg_logo: None,
            };
        }
    }
    
    Ok(streams)
}

async fn test_single_stream_task(
    stream: StreamInfo,
    timeout_seconds: u64,
    total_streams: usize,
    tested_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    working_counter: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    window: tauri::Window,
) -> TestResult {
    // Check if we should stop
    if SHOULD_STOP.load(Ordering::Relaxed) {
        return TestResult {
            stream,
            working: false,
            error: Some("Cancelled".to_string()),
        };
    }
    
    let working = match test_stream_url(&stream.url, timeout_seconds).await {
        Ok(result) => result,
        Err(_) => false,
    };
    
    let tested = tested_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
    let working_count = if working {
        working_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
    } else {
        working_counter.load(std::sync::atomic::Ordering::SeqCst)
    };
    
    // Emit progress update
    let _ = window.emit("progress", TestProgress {
        tested,
        total: total_streams,
        working: working_count,
        current_stream: Some(stream.name.clone()),
    });
    
    TestResult {
        stream,
        working,
        error: None,
    }
}

async fn test_streams_batch(
    streams: Vec<StreamInfo>,
    timeout_seconds: u64,
    max_concurrent: usize,
    window: tauri::Window,
) -> Vec<TestResult> {
    use futures::stream::{FuturesUnordered, StreamExt};
    
    let total_streams = streams.len();
    let tested_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let working_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    
    let mut results = Vec::with_capacity(total_streams);
    let mut futures = FuturesUnordered::new();
    let mut stream_iter = streams.into_iter();
    
    // Start initial batch of tasks
    for _ in 0..max_concurrent.min(total_streams) {
        if let Some(stream) = stream_iter.next() {
            let future = test_single_stream_task(
                stream,
                timeout_seconds,
                total_streams,
                tested_counter.clone(),
                working_counter.clone(),
                window.clone(),
            );
            futures.push(future);
        }
    }
    
    // Process results as they complete and spawn new tasks
    while let Some(result) = futures.next().await {
        results.push(result);
        
        // Start next task if available
        if let Some(stream) = stream_iter.next() {
            let future = test_single_stream_task(
                stream,
                timeout_seconds,
                total_streams,
                tested_counter.clone(),
                working_counter.clone(),
                window.clone(),
            );
            futures.push(future);
        }
    }
    
    results
}

async fn test_stream_url(url: &str, timeout_seconds: u64) -> Result<bool, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .user_agent("Mozilla/5.0 (Linux x86_64) AppleWebKit/537.36")
        .build()?;
    
    // Try HEAD request first with shorter timeout
    let head_timeout = Duration::from_secs(timeout_seconds.min(5));
    
    match timeout(head_timeout, client.head(url).send()).await {
        Ok(Ok(response)) => {
            if response.status().is_success() {
                // Check content type
                if let Some(content_type) = response.headers().get("content-type") {
                    let content_type = content_type.to_str().unwrap_or("").to_lowercase();
                    if content_type.contains("video") || 
                       content_type.contains("audio") ||
                       content_type.contains("application/vnd.apple.mpegurl") ||
                       content_type.contains("application/x-mpegurl") ||
                       content_type.contains("text/plain") {
                        return Ok(true);
                    }
                }
                return Ok(true); // Valid response, assume it's working
            }
        }
        _ => {
            // HEAD failed, try GET with range
            match timeout(
                head_timeout,
                client.get(url).header("Range", "bytes=0-512").send()
            ).await {
                Ok(Ok(response)) => {
                    return Ok(response.status().is_success() || response.status().as_u16() == 206);
                }
                _ => {}
            }
        }
    }
    
    Ok(false)
}

fn save_working_streams(streams: &[StreamInfo], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut content = String::from("#EXTM3U\n");
    
    for stream in streams {
        let mut extinf_line = String::from("#EXTINF:-1");
        
        if let Some(ref tvg_id) = stream.tvg_id {
            extinf_line.push_str(&format!(" tvg-id=\"{}\"", tvg_id));
        }
        if let Some(ref tvg_name) = stream.tvg_name {
            extinf_line.push_str(&format!(" tvg-name=\"{}\"", tvg_name));
        }
        if let Some(ref tvg_logo) = stream.tvg_logo {
            extinf_line.push_str(&format!(" tvg-logo=\"{}\"", tvg_logo));
        }
        if let Some(ref group_title) = stream.group_title {
            extinf_line.push_str(&format!(" group-title=\"{}\"", group_title));
        }
        
        extinf_line.push_str(&format!(",{}\n", stream.name));
        content.push_str(&extinf_line);
        content.push_str(&format!("{}\n", stream.url));
    }
    
    fs::write(output_path, content)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![test_streams_from_folder, test_single_stream, stop_testing, save_streams_by_category])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
