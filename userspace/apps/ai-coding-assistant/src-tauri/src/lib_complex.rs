// Module declarations
pub mod errors;
pub mod modules {
    pub mod conversation;
    pub mod ollama;
    pub mod tools;
    pub mod ai_module;
    pub mod config;
    pub mod database;
    pub mod file_operations;
    pub mod plugin_system;
    pub mod project_context;
    pub mod telemetry;
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::path::Path;
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;
use regex::Regex;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use semver::Version;
use std::fs::File;

#[derive(Serialize, Deserialize)]
struct AnalysisRequest {
    code: String,
    language: String,
    operation: String,
}

#[derive(Serialize, Deserialize)]
struct AnalysisResponse {
    result: String,
    success: bool,
    model_used: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct FileInfo {
    name: String,
    path: String,
    is_dir: bool,
    size: Option<u64>,
    modified: Option<u64>,
    extension: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct SystemInfo {
    cpu_usage: f32,
    memory_usage: f32,
    disk_usage: f32,
    uptime: String,
    processes: Vec<ProcessInfo>,
}

#[derive(Serialize, Deserialize)]
struct ProcessInfo {
    pid: String,
    name: String,
    cpu: String,
    memory: String,
}

#[derive(Serialize, Deserialize)]
struct SearchResult {
    file_path: String,
    line_number: usize,
    content: String,
    match_text: String,
}

#[derive(Serialize, Deserialize)]
struct TextSelection {
    text: String,
    context: Option<String>,
    file_path: Option<String>,
    language: Option<String>,
    selection_type: String, // "code", "text", "error", "command", etc.
}

#[derive(Serialize, Deserialize)]
struct ActionResult {
    original_text: String,
    result: String,
    action_performed: String,
    suggestions: Vec<String>,
    confidence: f32,
}

#[derive(Serialize, Deserialize)]
struct SmartSuggestion {
    action: String,
    description: String,
    priority: u8,
    icon: String,
}

#[derive(Serialize, Deserialize)]
struct CodeDiff {
    original: String,
    modified: String,
    changes: Vec<DiffLine>,
}

#[derive(Serialize, Deserialize)]
struct DiffLine {
    line_number: usize,
    change_type: String, // "added", "removed", "modified", "unchanged"
    content: String,
}

#[derive(Serialize, Deserialize)]
struct UpdateInfo {
    current_version: String,
    latest_version: String,
    update_available: bool,
    release_notes: String,
    download_url: Option<String>,
    security_update: bool,
    update_size: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct SystemUpdateInfo {
    packages_available: Vec<PackageUpdate>,
    security_updates: usize,
    total_updates: usize,
    last_updated: String,
}

#[derive(Serialize, Deserialize)]
struct PackageUpdate {
    name: String,
    current_version: String,
    new_version: String,
    description: String,
    is_security: bool,
    size: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct ModelUpdateInfo {
    model_name: String,
    current_version: Option<String>,
    latest_version: String,
    model_size: u64,
    download_url: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct BackupInfo {
    backup_id: String,
    backup_path: String,
    created_at: String,
    size: u64,
    description: String,
}

#[tauri::command]
fn read_file(file_path: String) -> Result<String, String> {
    fs::read_to_string(&file_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_file(file_path: String, content: String) -> Result<(), String> {
    fs::write(&file_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_files(dir_path: String) -> Result<Vec<String>, String> {
    let entries: Result<Vec<_>, _> = fs::read_dir(&dir_path)
        .map_err(|e| e.to_string())?
        .collect();
    
    let mut file_names = Vec::new();
    for entry in entries.map_err(|e| e.to_string())? {
        if let Ok(name) = entry.file_name().into_string() {
            file_names.push(name);
        }
    }
    Ok(file_names)
}

#[tauri::command]
fn run_git_command(command: String) -> Result<String, String> {
    Command::new("git")
        .args(command.split_whitespace())
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .map_err(|e| e.to_string())
}
// Command to analyze code using the AI LXC container
#[tauri::command]
async fn analyze_code(code: String, language: String, operation: String) -> Result<AnalysisResponse, String> {
    let client = reqwest::Client::new();
    let ai_container_url = "http://192.168.122.172:11434/api/generate";
    
    // Create the prompt based on operation
    let prompt = create_prompt(&code, &language, &operation);
    
    let mut request_body = HashMap::new();
    request_body.insert("model".to_string(), serde_json::Value::String(select_model(&language)));
    request_body.insert("prompt".to_string(), serde_json::Value::String(prompt));
    request_body.insert("stream".to_string(), serde_json::Value::Bool(false));
    
    match client.post(ai_container_url)
        .json(&request_body)
        .send()
        .await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<HashMap<String, serde_json::Value>>().await {
                    Ok(json) => {
                        let result = json.get("response")
                            .unwrap_or(&serde_json::Value::String("No response".to_string()))
                            .as_str()
                            .unwrap_or("Error parsing response")
                            .to_string();
                        Ok(AnalysisResponse {
                            result,
                            success: true,
                            model_used: Some(select_model(&language)),
                        })
                    },
                    Err(e) => Err(format!("Failed to parse response: {}", e)),
                }
            } else {
                Err(format!("AI container returned error: {}", response.status()))
            }
        },
        Err(e) => Err(format!("Failed to connect to AI container: {}", e)),
    }
}

// Command to check connection to AI container
#[tauri::command]
async fn check_ai_connection() -> Result<bool, String> {
    let client = reqwest::Client::new();
    let ai_container_url = "http://192.168.122.172:11434/api/tags";
    
    match client.get(ai_container_url).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

// Command to execute terminal commands
#[tauri::command]
async fn execute_command(command: String) -> Result<String, String> {
    use std::process::Command;
    
    // Security: Allow only safe commands for system optimization
    let safe_commands = [
        "ls", "pwd", "whoami", "date", "uptime", "free", "df", "ps", "top", "htop",
        "systemctl", "journalctl", "lscpu", "lsblk", "lsusb", "lspci", "lsmod",
        "uname", "hostnamectl", "timedatectl", "localectl", "cat", "head", "tail",
        "grep", "find", "which", "whereis", "man", "info", "help", "history",
        "neofetch", "fastfetch", "screenfetch", "inxi", "hwinfo"
    ];
    
    let cmd_parts: Vec<&str> = command.split_whitespace().collect();
    if cmd_parts.is_empty() {
        return Err("Empty command".to_string());
    }
    
    let cmd_name = cmd_parts[0];
    if !safe_commands.contains(&cmd_name) {
        return Err(format!("Command '{}' is not allowed for security reasons. Allowed commands: {}", cmd_name, safe_commands.join(", ")));
    }
    
    match Command::new(cmd_name)
        .args(&cmd_parts[1..])
        .output() {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

// Command for general AI queries
#[tauri::command]
async fn general_ai_query(query: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let ai_container_url = "http://192.168.122.172:11434/api/generate";
    
    let mut request_body = HashMap::new();
    request_body.insert("model".to_string(), serde_json::Value::String("codellama:7b".to_string()));
    request_body.insert("prompt".to_string(), serde_json::Value::String(query));
    request_body.insert("stream".to_string(), serde_json::Value::Bool(false));
    
    match client.post(ai_container_url)
        .json(&request_body)
        .send()
        .await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<HashMap<String, serde_json::Value>>().await {
                    Ok(json) => {
                        let result = json.get("response")
                            .unwrap_or(&serde_json::Value::String("No response".to_string()))
                            .as_str()
                            .unwrap_or("Error parsing response")
                            .to_string();
                        Ok(result)
                    },
                    Err(e) => Err(format!("Failed to parse response: {}", e)),
                }
            } else {
                Err(format!("AI container returned error: {}", response.status()))
            }
        },
        Err(e) => Err(format!("Failed to connect to AI container: {}", e)),
    }
}

// Command to get available models
#[tauri::command]
async fn get_available_models() -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    let ai_container_url = "http://192.168.122.172:11434/api/tags";
    
    match client.get(ai_container_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<HashMap<String, serde_json::Value>>().await {
                    Ok(json) => {
                        let models = json.get("models")
                            .and_then(|m| m.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|model| model.get("name").and_then(|n| n.as_str()))
                                    .map(|s| s.to_string())
                                    .collect()
                            })
                            .unwrap_or_else(|| vec!["codellama".to_string()]);
                        Ok(models)
                    },
                    Err(e) => Err(format!("Failed to parse models: {}", e)),
                }
            } else {
                Err("Failed to get models".to_string())
            }
        },
        Err(e) => Err(format!("Failed to connect: {}", e)),
    }
}

fn create_prompt(code: &str, language: &str, operation: &str) -> String {
    match operation {
        "analyze" => format!(
            "Analyze this {} code and provide detailed feedback about potential issues, improvements, and optimizations. Be specific with line numbers where applicable:\n\n{}", 
            language, code
        ),
        "fix_bugs" => format!(
            "Find and fix bugs in this {} code. Provide the corrected code with explanations:\n\n{}", 
            language, code
        ),
        "optimize" => format!(
            "Optimize this {} code for better performance and readability. Show before and after examples:\n\n{}", 
            language, code
        ),
        "document" => format!(
            "Generate comprehensive documentation for this {} code including function descriptions, parameters, and usage examples:\n\n{}", 
            language, code
        ),
        "test" => format!(
            "Generate comprehensive unit tests for this {} code:\n\n{}", 
            language, code
        ),
        _ => format!("Review this {} code:\n\n{}", language, code),
    }
}

fn select_model(language: &str) -> String {
    match language.to_lowercase().as_str() {
        "rust" => "codellama:7b".to_string(),
        "python" => "codellama:7b".to_string(),
        "javascript" | "typescript" => "codellama:7b".to_string(),
        "c++" | "cpp" | "c" => "codellama:7b".to_string(),
        "go" => "codellama:7b".to_string(),
        "java" => "codellama:7b".to_string(),
        _ => "codellama:7b".to_string(),
    }
}

// Enhanced file operations
#[tauri::command]
fn get_file_info(file_path: String) -> Result<FileInfo, String> {
    let path = Path::new(&file_path);
    let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
    
    let modified = metadata.modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());
    
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_string());
    
    Ok(FileInfo {
        name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
        path: file_path,
        is_dir: metadata.is_dir(),
        size: if metadata.is_file() { Some(metadata.len()) } else { None },
        modified,
        extension,
    })
}

// Advanced file search
#[tauri::command]
fn search_in_files(pattern: String, dir_path: String, file_extensions: Option<Vec<String>>) -> Result<Vec<SearchResult>, String> {
    let regex = Regex::new(&pattern).map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    for entry in WalkDir::new(dir_path).max_depth(10) {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().is_file() {
            // Filter by file extensions if provided
            if let Some(ref extensions) = file_extensions {
                if let Some(file_ext) = entry.path().extension().and_then(|s| s.to_str()) {
                    if !extensions.contains(&file_ext.to_string()) {
                        continue;
                    }
                }
            }
            
            if let Ok(content) = fs::read_to_string(entry.path()) {
                for (index, line) in content.lines().enumerate() {
                    if regex.is_match(line) {
                        if let Some(match_result) = regex.find(line) {
                            results.push(SearchResult {
                                file_path: entry.path().display().to_string(),
                                line_number: index + 1,
                                content: line.to_string(),
                                match_text: match_result.as_str().to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(results)
}

// System monitoring
#[tauri::command]
fn get_system_info() -> Result<SystemInfo, String> {
    let mut cpu_usage = 0.0;
    let mut memory_usage = 0.0;
    let mut disk_usage = 0.0;
    let mut uptime = "Unknown".to_string();
    let mut processes = Vec::new();
    
    // Get CPU info
    if let Ok(output) = Command::new("top").arg("-bn1").output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("%Cpu(s):") {
                // Parse CPU usage from top output
                if let Some(cpu_str) = line.split_whitespace().nth(1) {
                    cpu_usage = cpu_str.trim_end_matches('%').parse().unwrap_or(0.0);
                }
                break;
            }
        }
    }
    
    // Get memory info
    if let Ok(output) = Command::new("free").arg("-m").output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let total: f32 = parts[1].parse().unwrap_or(1.0);
                    let used: f32 = parts[2].parse().unwrap_or(0.0);
                    memory_usage = (used / total) * 100.0;
                }
                break;
            }
        }
    }
    
    // Get disk usage
    if let Ok(output) = Command::new("df").arg("-h").arg("/").output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let usage_str = parts[4].trim_end_matches('%');
                disk_usage = usage_str.parse().unwrap_or(0.0);
            }
            break;
        }
    }
    
    // Get uptime
    if let Ok(output) = Command::new("uptime").arg("-p").output() {
        uptime = String::from_utf8_lossy(&output.stdout).trim().to_string();
    }
    
    // Get top processes
    if let Ok(output) = Command::new("ps").args(["aux", "--sort=-%cpu"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for (i, line) in output_str.lines().skip(1).enumerate() {
            if i >= 10 { break; } // Top 10 processes
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                processes.push(ProcessInfo {
                    pid: parts[1].to_string(),
                    name: parts[10].to_string(),
                    cpu: parts[2].to_string(),
                    memory: parts[3].to_string(),
                });
            }
        }
    }
    
    Ok(SystemInfo {
        cpu_usage,
        memory_usage,
        disk_usage,
        uptime,
        processes,
    })
}

// Enhanced command execution with more safety checks
#[tauri::command]
async fn execute_safe_command(command: String, args: Vec<String>) -> Result<String, String> {
    let safe_commands = [
        "ls", "pwd", "whoami", "date", "uptime", "free", "df", "ps", "top", "htop",
        "systemctl", "journalctl", "lscpu", "lsblk", "lsusb", "lspci", "lsmod",
        "uname", "hostnamectl", "timedatectl", "localectl", "cat", "head", "tail",
        "grep", "find", "which", "whereis", "man", "info", "help", "history",
        "neofetch", "fastfetch", "screenfetch", "inxi", "hwinfo", "git", "curl",
        "wget", "ping", "traceroute", "netstat", "ss", "lsof", "du", "tree"
    ];
    
    if !safe_commands.contains(&command.as_str()) {
        return Err(format!("Command '{}' is not allowed for security reasons", command));
    }
    
    match Command::new(&command).args(&args).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

// AI-powered file analysis
#[tauri::command]
async fn analyze_file_with_ai(file_path: String, analysis_type: String) -> Result<String, String> {
    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let file_extension = Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("text");
    
    let prompt = match analysis_type.as_str() {
        "security" => format!("Analyze this {} file for security vulnerabilities and potential issues:\n\n{}", file_extension, content),
        "performance" => format!("Analyze this {} file for performance improvements and optimizations:\n\n{}", file_extension, content),
        "style" => format!("Analyze this {} file for code style and best practices:\n\n{}", file_extension, content),
        "documentation" => format!("Generate comprehensive documentation for this {} file:\n\n{}", file_extension, content),
        _ => format!("Analyze this {} file and provide general feedback:\n\n{}", file_extension, content),
    };
    
    general_ai_query(prompt).await
}

// Project-wide analysis
#[tauri::command]
async fn analyze_project(project_path: String) -> Result<String, String> {
    let mut file_count = 0;
    let mut total_lines = 0;
    let mut languages = std::collections::HashMap::new();
    
    for entry in WalkDir::new(&project_path).max_depth(5) {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                *languages.entry(ext.to_string()).or_insert(0) += 1;
            }
            
            if let Ok(content) = fs::read_to_string(entry.path()) {
                total_lines += content.lines().count();
                file_count += 1;
            }
        }
    }
    
    let analysis_prompt = format!(
        "Analyze this project structure:\n\nTotal files: {}\nTotal lines of code: {}\nLanguages found: {:?}\n\nProject path: {}\n\nProvide insights about the project structure, technologies used, and recommendations for improvement.",
        file_count, total_lines, languages, project_path
    );
    
    general_ai_query(analysis_prompt).await
}

// Git integration with AI analysis
#[tauri::command]
async fn analyze_git_changes() -> Result<String, String> {
    let git_status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| e.to_string())?;
    
    let git_diff = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .map_err(|e| e.to_string())?;
    
    let status_output = String::from_utf8_lossy(&git_status.stdout);
    let diff_output = String::from_utf8_lossy(&git_diff.stdout);
    
    let analysis_prompt = format!(
        "Analyze these Git changes:\n\nStatus:\n{}\n\nDiff:\n{}\n\nProvide feedback on the changes, potential issues, and suggestions for commit messages.",
        status_output, diff_output
    );
    
    general_ai_query(analysis_prompt).await
}

// Smart text selection analysis - detects what type of content is selected
#[tauri::command]
fn analyze_text_selection(selection: TextSelection) -> Result<Vec<SmartSuggestion>, String> {
    let mut suggestions = Vec::new();
    let text = &selection.text;
    
    // Detect if it's code
    if is_code_snippet(text) {
        suggestions.push(SmartSuggestion {
            action: "fix_code".to_string(),
            description: "Fix bugs and issues in this code".to_string(),
            priority: 9,
            icon: "🐛".to_string(),
        });
        suggestions.push(SmartSuggestion {
            action: "optimize_code".to_string(),
            description: "Optimize code for better performance".to_string(),
            priority: 8,
            icon: "⚡".to_string(),
        });
        suggestions.push(SmartSuggestion {
            action: "explain_code".to_string(),
            description: "Explain what this code does".to_string(),
            priority: 7,
            icon: "📖".to_string(),
        });
        suggestions.push(SmartSuggestion {
            action: "add_comments".to_string(),
            description: "Add detailed comments to code".to_string(),
            priority: 6,
            icon: "💬".to_string(),
        });
    }
    
    // Detect if it's an error message
    if is_error_message(text) {
        suggestions.push(SmartSuggestion {
            action: "debug_error".to_string(),
            description: "Debug this error and provide solutions".to_string(),
            priority: 10,
            icon: "🔧".to_string(),
        });
        suggestions.push(SmartSuggestion {
            action: "explain_error".to_string(),
            description: "Explain what this error means".to_string(),
            priority: 9,
            icon: "❓".to_string(),
        });
    }
    
    // Detect if it's a command or shell script
    if is_command_or_script(text) {
        suggestions.push(SmartSuggestion {
            action: "explain_command".to_string(),
            description: "Explain what this command does".to_string(),
            priority: 8,
            icon: "💻".to_string(),
        });
        suggestions.push(SmartSuggestion {
            action: "improve_command".to_string(),
            description: "Suggest improvements for this command".to_string(),
            priority: 7,
            icon: "✨".to_string(),
        });
    }
    
    // Always available actions
    suggestions.push(SmartSuggestion {
        action: "rewrite".to_string(),
        description: "Rewrite this text to be clearer".to_string(),
        priority: 5,
        icon: "✏️".to_string(),
    });
    
    suggestions.push(SmartSuggestion {
        action: "translate".to_string(),
        description: "Translate or convert this text".to_string(),
        priority: 4,
        icon: "🌐".to_string(),
    });
    
    suggestions.push(SmartSuggestion {
        action: "summarize".to_string(),
        description: "Create a summary of this content".to_string(),
        priority: 3,
        icon: "📋".to_string(),
    });
    
    // Sort by priority (highest first)
    suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));
    
    Ok(suggestions)
}

// Context-aware AI action performer - like highlighting text and asking me to do something
#[tauri::command]
async fn perform_smart_action(selection: TextSelection, action: String, custom_instruction: Option<String>) -> Result<ActionResult, String> {
    let prompt = create_smart_action_prompt(&selection, &action, custom_instruction.as_deref());
    
    match general_ai_query(prompt).await {
        Ok(result) => {
            let confidence = calculate_confidence(&action, &selection.text);
            let suggestions = generate_follow_up_suggestions(&action, &selection.text);
            
            Ok(ActionResult {
                original_text: selection.text.clone(),
                result,
                action_performed: action,
                suggestions,
                confidence,
            })
        },
        Err(e) => Err(e),
    }
}

// Generate diff for code changes
#[tauri::command]
fn generate_code_diff(original: String, modified: String) -> Result<CodeDiff, String> {
    let original_lines: Vec<&str> = original.lines().collect();
    let modified_lines: Vec<&str> = modified.lines().collect();
    let mut changes = Vec::new();
    
    // Simple diff algorithm - can be enhanced with proper diff library
    let max_len = original_lines.len().max(modified_lines.len());
    
    for i in 0..max_len {
        let original_line = original_lines.get(i);
        let modified_line = modified_lines.get(i);
        
        match (original_line, modified_line) {
            (Some(orig), Some(modif)) => {
                if orig != modif {
                    changes.push(DiffLine {
                        line_number: i + 1,
                        change_type: "modified".to_string(),
                        content: modif.to_string(),
                    });
                } else {
                    changes.push(DiffLine {
                        line_number: i + 1,
                        change_type: "unchanged".to_string(),
                        content: orig.to_string(),
                    });
                }
            },
            (None, Some(modif)) => {
                changes.push(DiffLine {
                    line_number: i + 1,
                    change_type: "added".to_string(),
                    content: modif.to_string(),
                });
            },
            (Some(orig), None) => {
                changes.push(DiffLine {
                    line_number: i + 1,
                    change_type: "removed".to_string(),
                    content: orig.to_string(),
                });
            },
            (None, None) => break,
        }
    }
    
    Ok(CodeDiff {
        original,
        modified,
        changes,
    })
}

// Auto-apply AI suggestions with backup
#[tauri::command]
async fn apply_ai_suggestion(file_path: String, original_text: String, suggested_text: String, create_backup: bool) -> Result<String, String> {
    if create_backup {
        let backup_path = format!("{}.backup.{}", file_path, chrono::Utc::now().timestamp());
        fs::copy(&file_path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;
    }
    
    let current_content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let new_content = current_content.replace(&original_text, &suggested_text);
    
    fs::write(&file_path, &new_content).map_err(|e| e.to_string())?;
    
    Ok(format!("Successfully applied changes to {}", file_path))
}

// Helper functions for text analysis
fn is_code_snippet(text: &str) -> bool {
    let code_indicators = [
        "function", "def ", "class ", "import ", "#include", "using ", "package ",
        "{", "}", "(", ")", ";", "==", "!=", "&&", "||", "++", "--",
        "console.log", "print(", "println!", "fmt.Println", "std::", "fn ",
        "let ", "const ", "var ", "int ", "string ", "bool ", "float"
    ];
    
    let line_count = text.lines().count();
    let has_code_patterns = code_indicators.iter().any(|&indicator| text.contains(indicator));
    let has_indentation = text.lines().any(|line| line.starts_with("    ") || line.starts_with("\t"));
    
    (has_code_patterns || has_indentation) && line_count > 1
}

fn is_error_message(text: &str) -> bool {
    let error_indicators = [
        "Error:", "error:", "ERROR:", "Exception:", "exception:", "EXCEPTION:",
        "Warning:", "warning:", "WARNING:", "Fatal:", "fatal:", "FATAL:",
        "Traceback", "Stack trace", "stack trace", "at line", "line ",
        "failed", "Failed", "FAILED", "cannot", "Cannot", "CANNOT",
        "undefined", "Undefined", "null pointer", "segmentation fault",
        "permission denied", "access denied", "not found", "Not found"
    ];
    
    error_indicators.iter().any(|&indicator| text.contains(indicator))
}

fn is_command_or_script(text: &str) -> bool {
    let command_indicators = [
        "sudo ", "cd ", "ls ", "cp ", "mv ", "rm ", "mkdir ", "chmod ", "chown",
        "git ", "npm ", "pip ", "cargo ", "docker ", "kubectl ",
        "systemctl ", "service ", "crontab ", "ssh ", "scp ", "rsync",
        "#!/", "bash", "sh", "zsh", "fish", "powershell", "cmd"
    ];
    
    let starts_with_command = text.lines().next()
        .map(|first_line| command_indicators.iter().any(|&cmd| first_line.contains(cmd)))
        .unwrap_or(false);
    
    let has_pipe_or_redirect = text.contains("|") || text.contains(">") || text.contains("<");
    
    starts_with_command || has_pipe_or_redirect
}

fn create_smart_action_prompt(selection: &TextSelection, action: &str, custom_instruction: Option<&str>) -> String {
    let base_context = if let Some(context) = &selection.context {
        format!("\n\nContext: {}", context)
    } else {
        String::new()
    };
    
    let language_context = if let Some(lang) = &selection.language {
        format!("\n\nThis is {} code.", lang)
    } else {
        String::new()
    };
    
    let custom_context = if let Some(instruction) = custom_instruction {
        format!("\n\nSpecial instructions: {}", instruction)
    } else {
        String::new()
    };
    
    match action {
        "fix_code" => format!(
            "Fix any bugs, errors, or issues in this code. Provide the corrected version with explanations of what was wrong:\n\n{}{}{}{}\n\nProvide both the fixed code and explanation.",
            selection.text, base_context, language_context, custom_context
        ),
        "optimize_code" => format!(
            "Optimize this code for better performance, readability, and maintainability. Show the improved version:\n\n{}{}{}{}\n\nExplain the optimizations made.",
            selection.text, base_context, language_context, custom_context
        ),
        "explain_code" => format!(
            "Explain what this code does in clear, detailed terms. Break down complex parts:\n\n{}{}{}{}\n\nProvide a comprehensive explanation.",
            selection.text, base_context, language_context, custom_context
        ),
        "add_comments" => format!(
            "Add detailed, helpful comments to this code. Explain what each section does:\n\n{}{}{}{}\n\nReturn the code with comprehensive comments.",
            selection.text, base_context, language_context, custom_context
        ),
        "debug_error" => format!(
            "Debug this error message and provide solutions. Explain what caused it and how to fix it:\n\n{}{}{}{}\n\nProvide debugging steps and solutions.",
            selection.text, base_context, language_context, custom_context
        ),
        "explain_error" => format!(
            "Explain what this error means in simple terms. What causes it and how can it be prevented?\n\n{}{}{}{}\n\nProvide a clear explanation.",
            selection.text, base_context, language_context, custom_context
        ),
        "explain_command" => format!(
            "Explain what this command or script does. Break down each part and its purpose:\n\n{}{}{}{}\n\nProvide a detailed explanation.",
            selection.text, base_context, language_context, custom_context
        ),
        "improve_command" => format!(
            "Suggest improvements for this command or script. Make it more efficient, safer, or more readable:\n\n{}{}{}{}\n\nProvide improved version with explanations.",
            selection.text, base_context, language_context, custom_context
        ),
        "rewrite" => format!(
            "Rewrite this text to be clearer, more professional, and better structured:\n\n{}{}{}{}\n\nProvide the improved version.",
            selection.text, base_context, language_context, custom_context
        ),
        "translate" => format!(
            "Translate or convert this content as appropriate. If it's code, convert to a different language or format:\n\n{}{}{}{}\n\nProvide the translated/converted version.",
            selection.text, base_context, language_context, custom_context
        ),
        "summarize" => format!(
            "Create a concise summary of this content. Highlight the key points:\n\n{}{}{}{}\n\nProvide a clear summary.",
            selection.text, base_context, language_context, custom_context
        ),
        _ => format!(
            "Analyze and improve this content according to the action '{}': \n\n{}{}{}{}\n\nProvide your analysis and improvements.",
            action, selection.text, base_context, language_context, custom_context
        ),
    }
}

fn calculate_confidence(action: &str, text: &str) -> f32 {
    let mut confidence: f32 = 0.7; // Base confidence
    
    // Increase confidence based on text characteristics
    if is_code_snippet(text) && (action == "fix_code" || action == "optimize_code" || action == "explain_code") {
        confidence += 0.2;
    }
    
    if is_error_message(text) && (action == "debug_error" || action == "explain_error") {
        confidence += 0.25;
    }
    
    if is_command_or_script(text) && (action == "explain_command" || action == "improve_command") {
        confidence += 0.2;
    }
    
    // Text length factor
    if text.len() > 50 && text.len() < 5000 {
        confidence += 0.05;
    }
    
    confidence.min(1.0)
}

fn generate_follow_up_suggestions(action: &str, _text: &str) -> Vec<String> {
    match action {
        "fix_code" => vec![
            "Optimize the fixed code further".to_string(),
            "Add unit tests for this code".to_string(),
            "Document the code with comments".to_string(),
        ],
        "optimize_code" => vec![
            "Add error handling".to_string(),
            "Create unit tests".to_string(),
            "Review security aspects".to_string(),
        ],
        "explain_code" => vec![
            "Optimize this code".to_string(),
            "Find potential bugs".to_string(),
            "Add documentation".to_string(),
        ],
        "debug_error" => vec![
            "Prevent similar errors".to_string(),
            "Add better error handling".to_string(),
            "Create monitoring for this issue".to_string(),
        ],
        _ => vec![
            "Ask a follow-up question".to_string(),
            "Request more details".to_string(),
            "Apply changes to file".to_string(),
        ],
    }
}

// Advanced update functionality
#[tauri::command]
async fn check_for_updates() -> Result<UpdateInfo, String> {
    let client = reqwest::Client::new();
    let current_version = "0.1.0";
    
    // Check for updates from a hypothetical GitHub releases API
    let update_url = "https://api.github.com/repos/youruser/your-ai-app/releases/latest";
    
    match client.get(update_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                // In a real implementation, you'd parse the GitHub API response
                let latest_version = "0.1.1"; // Mock data
                let update_available = Version::parse(latest_version).unwrap() > Version::parse(current_version).unwrap();
                
                Ok(UpdateInfo {
                    current_version: current_version.to_string(),
                    latest_version: latest_version.to_string(),
                    update_available,
                    release_notes: "Bug fixes and performance improvements".to_string(),
                    download_url: Some("https://github.com/youruser/your-ai-app/releases/latest".to_string()),
                    security_update: false,
                    update_size: Some(15_000_000), // 15MB
                })
            } else {
                Err("Failed to check for updates".to_string())
            }
        },
        Err(_) => {
            // Fallback to local version comparison
            Ok(UpdateInfo {
                current_version: current_version.to_string(),
                latest_version: current_version.to_string(),
                update_available: false,
                release_notes: "Unable to check for updates - offline mode".to_string(),
                download_url: None,
                security_update: false,
                update_size: None,
            })
        }
    }
}

#[tauri::command]
async fn check_system_updates() -> Result<SystemUpdateInfo, String> {
    let mut packages_available = Vec::new();
    let mut security_updates = 0;
    let mut total_updates = 0;
    
    // Check for system updates based on the package manager
    if let Ok(output) = Command::new("apt").args(["list", "--upgradable"]).output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines().skip(1) { // Skip header
            if !line.is_empty() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0].split('/').next().unwrap_or(parts[0]).to_string();
                    let versions = parts[1].split(' ').collect::<Vec<&str>>();
                    
                    let new_version = versions.get(0).unwrap_or(&"unknown").to_string();
                    let current_version = versions.get(1).unwrap_or(&"unknown").to_string();
                    
                    let is_security = line.contains("security") || line.contains("Security");
                    if is_security {
                        security_updates += 1;
                    }
                    
                    packages_available.push(PackageUpdate {
                        name,
                        current_version,
                        new_version,
                        description: "System package update".to_string(),
                        is_security,
                        size: None,
                    });
                    
                    total_updates += 1;
                }
            }
        }
    }
    
    // Get last update time
    let last_updated = if let Ok(output) = Command::new("stat").args(["-c", "%Y", "/var/lib/apt/lists"]).output() {
        let timestamp = String::from_utf8_lossy(&output.stdout).trim().parse::<i64>().unwrap_or(0);
        let datetime = DateTime::from_timestamp(timestamp, 0)
            .unwrap_or_else(|| Utc::now())
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        datetime
    } else {
        "Unknown".to_string()
    };
    
    Ok(SystemUpdateInfo {
        packages_available,
        security_updates,
        total_updates,
        last_updated,
    })
}

#[tauri::command]
async fn perform_system_update(update_type: String) -> Result<String, String> {
    match update_type.as_str() {
        "security_only" => {
            let output = Command::new("sudo")
                .args(["apt", "upgrade", "-y", "--only-upgrade", "$(apt list --upgradable 2>/dev/null | grep -i security | cut -d/ -f1)"])
                .output()
                .map_err(|e| format!("Failed to perform security updates: {}", e))?;
            
            if output.status.success() {
                Ok("Security updates completed successfully".to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
        "all" => {
            let output = Command::new("sudo")
                .args(["apt", "upgrade", "-y"])
                .output()
                .map_err(|e| format!("Failed to perform system update: {}", e))?;
            
            if output.status.success() {
                Ok("All system updates completed successfully".to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
        "refresh" => {
            let output = Command::new("sudo")
                .args(["apt", "update"])
                .output()
                .map_err(|e| format!("Failed to refresh package lists: {}", e))?;
            
            if output.status.success() {
                Ok("Package lists refreshed successfully".to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
        _ => Err("Invalid update type. Use 'security_only', 'all', or 'refresh'".to_string())
    }
}

#[tauri::command]
async fn check_ai_model_updates() -> Result<Vec<ModelUpdateInfo>, String> {
    let _client = reqwest::Client::new();
    let mut models = Vec::new();
    
    // Check what models are currently available
    let current_models = get_available_models().await.unwrap_or_default();
    
    // Mock model update information (in a real implementation, you'd check Ollama's model registry)
    let available_models = vec![
        ("codellama:7b", "Latest CodeLlama 7B model with improved performance", 3_800_000_000),
        ("codellama:13b", "Larger CodeLlama model with better accuracy", 7_300_000_000),
        ("llama2:7b", "General purpose Llama2 7B model", 3_800_000_000),
        ("mistral:7b", "Fast and efficient Mistral 7B model", 4_100_000_000),
        ("neural-chat:7b", "Optimized for conversational AI", 4_200_000_000),
    ];
    
    for (model_name, description, size) in available_models {
        let is_installed = current_models.iter().any(|m| m.contains(model_name));
        
        models.push(ModelUpdateInfo {
            model_name: model_name.to_string(),
            current_version: if is_installed { Some("installed".to_string()) } else { None },
            latest_version: "latest".to_string(),
            model_size: size,
            download_url: format!("ollama://pull/{}", model_name),
            description: description.to_string(),
        });
    }
    
    Ok(models)
}

#[tauri::command]
async fn update_ai_model(model_name: String) -> Result<String, String> {
    // Use Ollama to pull/update the model
    let output = Command::new("ollama")
        .args(["pull", &model_name])
        .output()
        .map_err(|e| format!("Failed to update AI model: {}", e))?;
    
    if output.status.success() {
        Ok(format!("Successfully updated AI model: {}", model_name))
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to update model {}: {}", model_name, error))
    }
}

#[tauri::command]
async fn remove_ai_model(model_name: String) -> Result<String, String> {
    // Use Ollama to remove the model
    let output = Command::new("ollama")
        .args(["rm", &model_name])
        .output()
        .map_err(|e| format!("Failed to remove AI model: {}", e))?;
    
    if output.status.success() {
        Ok(format!("Successfully removed AI model: {}", model_name))
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to remove model {}: {}", model_name, error))
    }
}

#[tauri::command]
fn create_backup(backup_type: String, items: Vec<String>) -> Result<BackupInfo, String> {
    let backup_id = Uuid::new_v4().to_string();
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_dir = format!("/tmp/ai_app_backups/{}", timestamp);
    
    // Create backup directory
    fs::create_dir_all(&backup_dir).map_err(|e| format!("Failed to create backup directory: {}", e))?;
    
    let mut total_size = 0u64;
    let description = match backup_type.as_str() {
        "config" => {
            // Backup configuration files
            for item in &items {
                if let Ok(metadata) = fs::metadata(item) {
                    total_size += metadata.len();
                    let filename = Path::new(item).file_name().unwrap().to_str().unwrap();
                    let backup_path = format!("{}/{}", backup_dir, filename);
                    fs::copy(item, backup_path).map_err(|e| format!("Failed to backup {}: {}", item, e))?;
                }
            }
            "Configuration files backup".to_string()
        },
        "models" => {
            // Backup AI models (this would be model metadata, not the full models)
            "AI models metadata backup".to_string()
        },
        "full" => {
            // Full application backup
            "Full application backup".to_string()
        },
        _ => return Err("Invalid backup type".to_string())
    };
    
    Ok(BackupInfo {
        backup_id,
        backup_path: backup_dir,
        created_at: Utc::now().to_rfc3339(),
        size: total_size,
        description,
    })
}

#[tauri::command]
fn restore_backup(backup_id: String) -> Result<String, String> {
    // Find backup by ID and restore it
    let backup_base = "/tmp/ai_app_backups";
    
    // In a real implementation, you'd have a database of backups
    // For now, we'll simulate restoration
    if Path::new(backup_base).exists() {
        Ok(format!("Backup {} restored successfully", backup_id))
    } else {
        Err(format!("Backup {} not found", backup_id))
    }
}

#[tauri::command]
fn list_backups() -> Result<Vec<BackupInfo>, String> {
    let mut backups = Vec::new();
    let backup_base = "/tmp/ai_app_backups";
    
    if let Ok(entries) = fs::read_dir(backup_base) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        let backup_name = entry.file_name().to_string_lossy().to_string();
                        backups.push(BackupInfo {
                            backup_id: Uuid::new_v4().to_string(),
                            backup_path: entry.path().to_string_lossy().to_string(),
                            created_at: Utc::now().to_rfc3339(),
                            size: 0, // Would calculate actual size
                            description: format!("Backup from {}", backup_name),
                        });
                    }
                }
            }
        }
    }
    
    Ok(backups)
}

#[tauri::command]
async fn download_update(download_url: String, destination: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    
    let response = client.get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download update: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }
    
    let mut file = File::create(&destination)
        .map_err(|e| format!("Failed to create destination file: {}", e))?;
    
    let content = response.bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    std::io::Write::write_all(&mut file, &content)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(format!("Update downloaded successfully to {}", destination))
}

#[tauri::command]
fn verify_update_integrity(file_path: String, expected_hash: String) -> Result<bool, String> {
    use sha2::{Sha256, Digest};
    
    let contents = fs::read(&file_path)
        .map_err(|e| format!("Failed to read update file: {}", e))?;
    
    let mut hasher = Sha256::new();
    hasher.update(contents);
    let result = hasher.finalize();
    let actual_hash = format!("{:x}", result);
    
    Ok(actual_hash == expected_hash)
}

#[tauri::command]
fn get_update_history() -> Result<Vec<HashMap<String, String>>, String> {
    // Return update history (mock data)
    let mut history = Vec::new();
    
    let mut update1 = HashMap::new();
    update1.insert("version".to_string(), "0.1.0".to_string());
    update1.insert("date".to_string(), "2024-01-15".to_string());
    update1.insert("description".to_string(), "Initial release".to_string());
    history.push(update1);
    
    let mut update2 = HashMap::new();
    update2.insert("version".to_string(), "0.0.9".to_string());
    update2.insert("date".to_string(), "2024-01-10".to_string());
    update2.insert("description".to_string(), "Beta release with AI improvements".to_string());
    history.push(update2);
    
    Ok(history)
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            analyze_code,
            check_ai_connection,
            get_available_models,
            general_ai_query,
            execute_command,
            execute_safe_command,
            read_file,
            write_file,
            list_files,
            run_git_command,
            get_file_info,
            search_in_files,
            get_system_info,
            analyze_file_with_ai,
            analyze_project,
            analyze_git_changes,
            analyze_text_selection,
            perform_smart_action,
            generate_code_diff,
            apply_ai_suggestion,
            check_for_updates,
            check_system_updates,
            perform_system_update,
            check_ai_model_updates,
            update_ai_model,
            remove_ai_model,
            create_backup,
            restore_backup,
            list_backups,
            download_update,
            verify_update_integrity,
            get_update_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
