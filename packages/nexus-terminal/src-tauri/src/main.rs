// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use anyhow::Result;
use chrono::Timelike;
use tracing::info;

mod ai;
mod git;
mod git_advanced;
mod terminal;
mod ai_optimized;
mod vision_commands;
mod config;
mod utils;
mod broadcast;
mod web_scraper;
mod vision;
mod security_scanner;
mod command_flow;
mod plugin_system;
mod collaboration;
mod workflow_automation;
mod analytics;
mod cloud_integration;
mod ecosystem_awareness;
mod local_recall;
mod ollama_config;

use ai::AIService;
use ai_optimized::RequestPriority;
use terminal::TerminalManager;
use config::AppConfig;
use vision::VisionService;
use ai_optimized::OptimizedAIService;

#[derive(Debug)]
struct AppState {
    terminal_manager: Arc<RwLock<TerminalManager>>,
    ai_service: Arc<RwLock<AIService>>,
    optimized_ai_service: Arc<RwLock<OptimizedAIService>>,
    vision_service: Arc<RwLock<VisionService>>,
    config: Arc<RwLock<AppConfig>>,
    security_scanner: Arc<RwLock<security_scanner::SecurityScanner>>,
    command_flow_engine: Arc<RwLock<command_flow::CommandFlowEngine>>,
    plugin_system: Arc<RwLock<plugin_system::PluginSystem>>,
    collaboration_manager: Arc<RwLock<collaboration::CollaborationManager>>,
    workflow_engine: Arc<RwLock<workflow_automation::WorkflowEngine>>,
    analytics_engine: Arc<RwLock<analytics::AnalyticsEngine>>,
    cloud_manager: Arc<RwLock<cloud_integration::CloudIntegrationManager>>,
    ecosystem_awareness: Arc<RwLock<ecosystem_awareness::EcosystemAwareness>>,
}

// AI-related commands
#[tauri::command]
async fn ai_chat(
    message: String,
    context: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let ai_service = state.ai_service.read().await;
    ai_service
        .chat(&message, context.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_complete_command(
    partial_command: String,
    context: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let ai_service = state.ai_service.read().await;
    ai_service
        .complete_command(&partial_command, &context)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_explain_error(
    error_output: String,
    command: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let ai_service = state.ai_service.read().await;
    ai_service
        .explain_error(&error_output, &command)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_generate_code(
    description: String,
    language: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let ai_service = state.ai_service.read().await;
    ai_service
        .generate_code(&description, &language)
        .await
        .map_err(|e| e.to_string())
}

// Terminal-related commands
#[tauri::command]
async fn create_terminal(
    shell: Option<String>,
    args: Option<Vec<String>>,
    cwd: Option<String>,
    env: Option<std::collections::HashMap<String, String>>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut terminal_manager = state.terminal_manager.write().await;
    terminal_manager
        .create_terminal_with_config(shell, args, cwd, env)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_simple_terminal(
    shell: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut terminal_manager = state.terminal_manager.write().await;
    terminal_manager
        .create_terminal(shell)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn write_to_terminal(
    terminal_id: String,
    data: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let terminal_manager = state.terminal_manager.read().await;
    terminal_manager
        .write_to_terminal(&terminal_id, &data)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn resize_terminal(
    terminal_id: String,
    cols: u16,
    rows: u16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let terminal_manager = state.terminal_manager.read().await;
    terminal_manager
        .resize_terminal(&terminal_id, cols, rows)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn kill_terminal(
    terminal_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut terminal_manager = state.terminal_manager.write().await;
    terminal_manager
        .kill_terminal(&terminal_id)
        .await
        .map_err(|e| e.to_string())
}

// Git integration commands
#[tauri::command]
async fn git_status(path: String) -> Result<String, String> {
    git::get_status(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_generate_commit(
    path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let changes = git::get_diff(&path).map_err(|e| e.to_string())?;
    let ai_service = state.ai_service.read().await;
    ai_service
        .generate_commit_message(&changes)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_get_branch_name(path: String) -> Result<String, String> {
    git::get_branch_name(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_is_repo(path: String) -> Result<bool, String> {
    Ok(git::is_repo(&path))
}

#[tauri::command]
async fn git_get_recent_commits(path: String, limit: usize) -> Result<Vec<String>, String> {
    git::get_recent_commits(&path, limit).map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_get_remote_url(path: String) -> Result<Option<String>, String> {
    git::get_remote_url(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_get_ahead_behind(path: String) -> Result<(usize, usize), String> {
    git::get_ahead_behind_count(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_get_branch_info(path: String) -> Result<git::BranchInfo, String> {
    git::get_branch_info(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_get_all_branches(path: String) -> Result<git::BranchList, String> {
    git::get_all_branches(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_get_stash_list(path: String) -> Result<Vec<git::StashEntry>, String> {
    git::get_stash_list(&path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_get_commit_changes(path: String, commit_hash: String) -> Result<Vec<git::FileChange>, String> {
    git::get_commit_changes(&path, &commit_hash).map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_get_repository_stats(path: String) -> Result<git::RepositoryStats, String> {
    git::get_repository_stats(&path).map_err(|e| e.to_string())
}

// Advanced Git Integration commands
#[tauri::command]
async fn git_generate_visual_graph(
    path: String,
    max_commits: Option<u32>,
) -> Result<git_advanced::GitGraph, String> {
    let git_advanced = git_advanced::GitAdvanced::new(&path);
    git_advanced.generate_visual_graph(max_commits).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_generate_time_travel(
    path: String,
    commit: String,
) -> Result<Vec<git_advanced::GitTimeTravel>, String> {
    let git_advanced = git_advanced::GitAdvanced::new(&path);
    git_advanced.generate_time_travel(Some(commit)).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_generate_statistics(
    path: String,
) -> Result<git_advanced::GitStatistics, String> {
    let git_advanced = git_advanced::GitAdvanced::new(&path);
    git_advanced.generate_statistics().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_generate_visualization(
    path: String,
    max_commits: Option<u32>,
) -> Result<git_advanced::GitVisualization, String> {
    let git_advanced = git_advanced::GitAdvanced::new(&path);
    git_advanced.generate_visualization(max_commits).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_get_advanced_branch_info(
    path: String,
) -> Result<Vec<git_advanced::BranchInfo>, String> {
    let git_advanced = git_advanced::GitAdvanced::new(&path);
    git_advanced.get_branch_info().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_time_travel_to_commit(
    path: String,
    commit: String,
) -> Result<String, String> {
    let git_advanced = git_advanced::GitAdvanced::new(&path);
    git_advanced.time_travel_to_commit(&commit).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn git_create_branch_from_commit(
    path: String,
    branch_name: String,
    commit: String,
) -> Result<String, String> {
    let git_advanced = git_advanced::GitAdvanced::new(&path);
    git_advanced.create_branch_from_commit(&branch_name, &commit).await.map_err(|e| e.to_string())
}

// Contextual suggestions commands
#[tauri::command]
async fn get_contextual_suggestions(
    partial_command: String,
    context: serde_json::Value,
    _filter: Option<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let ai_service = state.ai_service.read().await;
    
    // Convert context to a structured prompt for AI
    let context_str = format!(
        "Current directory: {}\nPartial command: {}\nContext: {}", 
        context.get("currentDirectory").and_then(|v| v.as_str()).unwrap_or("unknown"),
        partial_command,
        serde_json::to_string(&context).unwrap_or_default()
    );
    
    let prompt = format!(
        "Based on the following context, suggest relevant shell commands:\n{}\n\nProvide suggestions as a JSON array with command, description, confidence (0-1), and category fields.",
        context_str
    );
    
    let response = ai_service.chat(&prompt, Some(&context_str)).await.map_err(|e| e.to_string())?;
    
    // Parse AI response and return structured suggestions
    // For now, return a fallback response if parsing fails
    match serde_json::from_str::<Vec<serde_json::Value>>(&response) {
        Ok(suggestions) => Ok(suggestions),
        Err(_) => {
            // Fallback: generate basic suggestions based on partial command
            let mut suggestions = Vec::new();
            
            if partial_command.starts_with("git") {
                suggestions.push(serde_json::json!({
                    "command": "git status",
                    "description": "Show working tree status",
                    "confidence": 0.8,
                    "category": "git",
                    "context": ["git-command"]
                }));
            } else if partial_command.starts_with("ls") {
                suggestions.push(serde_json::json!({
                    "command": "ls -la",
                    "description": "List files with details",
                    "confidence": 0.9,
                    "category": "navigation",
                    "context": ["file-listing"]
                }));
            }
            
            Ok(suggestions)
        }
    }
}

#[tauri::command]
async fn get_current_context() -> Result<serde_json::Value, String> {
    use std::env;
    // Process commands would be used for system integration
    
    let current_dir = env::current_dir().map_err(|e| e.to_string())?
        .to_string_lossy().to_string();
    
    // Get directory contents
    let dir_contents: Vec<String> = std::fs::read_dir(&current_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| {
            entry.ok().and_then(|e| e.file_name().to_str().map(|s| s.to_string()))
        })
        .collect();
    
    // Check if it's a git repository
    let git_info = if git::is_repo(&current_dir) {
        let status = git::get_status(&current_dir).unwrap_or_default();
        let branch = git::get_branch_name(&current_dir).unwrap_or("unknown".to_string());
        let remote_url = git::get_remote_url(&current_dir).unwrap_or(None);
        
        serde_json::json!({
            "branch": branch,
            "status": if status.contains("nothing to commit") { "clean" } else { "dirty" },
            "remote": remote_url
        })
    } else {
        serde_json::Value::Null
    };
    
    // Determine project type
    let project_type = if dir_contents.contains(&"package.json".to_string()) {
        "nodejs"
    } else if dir_contents.contains(&"Cargo.toml".to_string()) {
        "rust"
    } else if dir_contents.contains(&"requirements.txt".to_string()) || dir_contents.contains(&"setup.py".to_string()) {
        "python"
    } else if dir_contents.contains(&"go.mod".to_string()) {
        "go"
    } else {
        "other"
    };
    
    // Get time of day
    let hour = chrono::Local::now().hour();
    let time_of_day = match hour {
        6..=11 => "morning",
        12..=17 => "afternoon",
        18..=21 => "evening",
        _ => "night"
    };
    
    Ok(serde_json::json!({
        "currentDirectory": current_dir,
        "directoryContents": dir_contents,
        "gitRepository": git_info,
        "projectType": project_type,
        "timeOfDay": time_of_day,
        "dayOfWeek": chrono::Local::now().format("%A").to_string(),
        "recentCommands": Vec::<String>::new(), // Would need to be tracked separately
        "workingOnFiles": Vec::<String>::new(), // Would need file monitoring
        "activeProcesses": Vec::<serde_json::Value>::new(), // Would need process monitoring
        "environmentVars": std::env::vars().collect::<std::collections::HashMap<String, String>>(),
        "shellHistory": Vec::<String>::new() // Would need shell history integration
    }))
}

#[tauri::command]
async fn learn_from_command(
    command: String,
    successful: bool,
    context: serde_json::Value,
) -> Result<(), String> {
    // Log command learning for future analysis
    tracing::info!(
        "Learning from command: {} (success: {}) in context: {}",
        command,
        successful,
        context.get("currentDirectory").and_then(|v| v.as_str()).unwrap_or("unknown")
    );
    
    // In a real implementation, this would store learning data in a database
    // For now, we just log it
    Ok(())
}

// Config commands
#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.read().await;
    Ok(config.clone())
}

#[tauri::command]
async fn get_ai_models_cache_dir(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.read().await;
    Ok(config.ai_models_cache_dir().to_string_lossy().to_string())
}

#[tauri::command]
async fn get_screenshots_temp_dir(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.read().await;
    Ok(config.screenshots_temp_dir().to_string_lossy().to_string())
}

#[tauri::command]
async fn create_temp_screenshot_path(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.read().await;
    config.create_temp_screenshot_path()
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_config(
    new_config: AppConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.write().await;
    *config = new_config.clone();
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_temp_file_path(
    name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let config = state.config.read().await;
    Ok(config.temp_file_path(&name).to_string_lossy().to_string())
}

#[tauri::command]
async fn get_cache_file_path(
    name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let config = state.config.read().await;
    Ok(config.cache_file_path(&name).to_string_lossy().to_string())
}

// AI helper commands
#[tauri::command]
async fn check_ai_connection(state: State<'_, AppState>) -> Result<bool, String> {
    let ai_service = state.ai_service.read().await;
    // Test if we can get available models (indicates connection)
    match ai_service.get_available_models().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
async fn ai_analyze_repository(
    project_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let file_tree = utils::get_file_tree(&project_path, Some(3))
        .map_err(|e| e.to_string())?;
    
    let readme_content = {
        let readme_paths = ["README.md", "readme.md", "README.txt", "README"];
        let mut content = None;
        for readme_path in &readme_paths {
            let full_path = format!("{}/{}", project_path, readme_path);
            if let Ok(readme) = tokio::fs::read_to_string(&full_path).await {
                content = Some(readme);
                break;
            }
        }
        content
    };
    
    let ai_service = state.ai_service.read().await;
    ai_service
        .analyze_repository(&file_tree, readme_content.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_suggest_improvements(
    code: String,
    language: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let ai_service = state.ai_service.read().await;
    ai_service
        .suggest_improvements(&code, &language)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_explain_concept(
    concept: String,
    context: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let ai_service = state.ai_service.read().await;
    ai_service
        .explain_concept(&concept, &context)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_current_model(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.read().await;
    Ok(config.ai.default_model.clone())
}

#[tauri::command]
async fn set_ai_model(
    model: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Update the config
    {
        let mut config_guard = state.config.write().await;
        config_guard.ai.default_model = model.clone();
    }
    
    // Restart AI service with new model
    let config = {
        let config_guard = state.config.read().await;
        config_guard.ai.clone()
    };
    
    let new_ai_service = AIService::new(&config).await.map_err(|e| e.to_string())?;
    
    {
        let mut ai_service_guard = state.ai_service.write().await;
        *ai_service_guard = new_ai_service;
    }
    
    info!("AI model changed to: {}", model);
    Ok(())
}

#[tauri::command]
async fn get_available_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let ai_service = state.ai_service.read().await;
    ai_service.get_available_models().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn send_ai_message(
    message: String,
    context: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Extract context information
    let context_str = match context {
        serde_json::Value::Object(obj) => {
            let working_dir = obj.get("workingDirectory")
                .and_then(|v| v.as_str())
                .unwrap_or("~");
            let shell = obj.get("shell")
                .and_then(|v| v.as_str())
                .unwrap_or("bash");
            let source = obj.get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("terminal");
            
            format!("Working Directory: {}\nShell: {}\nSource: {}", working_dir, shell, source)
        },
        _ => "Basic terminal context".to_string()
    };
    
    // Use the memory-enabled AI chat with a unique conversation ID for the AI Assistant
    let ai_service = state.ai_service.read().await;
    ai_service
        .chat_with_memory(&message, "ai_assistant_main", Some(&context_str))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_terminal_context(state: State<'_, AppState>) -> Result<String, String> {
    let terminal_manager = state.terminal_manager.read().await;
    let terminals = terminal_manager.list_terminals();
    
    if terminals.is_empty() {
        Ok("No active terminals".to_string())
    } else {
        let mut context_parts = Vec::new();
        
        // Add system information
        context_parts.push(format!("System: {} on {}", 
            std::env::consts::OS, 
            std::env::consts::ARCH
        ));
        
        // Add current working directory
        if let Ok(cwd) = std::env::current_dir() {
            context_parts.push(format!("Current Directory: {}", cwd.display()));
        }
        
        // Add terminal information
        for terminal in terminals.iter() {
            context_parts.push(format!("Terminal {}: {} shell in {}", 
                terminal.id, 
                terminal.shell, 
                terminal.cwd
            ));
        }
        
        // Add environment context
        if let Ok(user) = std::env::var("USER") {
            context_parts.push(format!("User: {}", user));
        }
        
        if let Ok(shell) = std::env::var("SHELL") {
            context_parts.push(format!("Default Shell: {}", shell));
        }
        
        // Add helpful context about AI capabilities
        context_parts.push("AI Assistant Capabilities: I can help with terminal commands, code analysis, system diagnostics, git operations, package management, debugging, error analysis, code generation, and system optimization. Ask me specific questions for better assistance.".to_string());
        
        Ok(context_parts.join("\n"))
    }
}

// Window control commands
#[tauri::command]
async fn minimize_window(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_maximize(window: tauri::Window) -> Result<(), String> {
    if window.is_maximized().map_err(|e| e.to_string())? {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn close_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

// AI service management commands
#[tauri::command]
async fn restart_ai_service(state: State<'_, AppState>) -> Result<(), String> {
    let config = {
        let config_guard = state.config.read().await;
        config_guard.ai.clone()
    };
    
    // Recreate AI service
    let new_ai_service = AIService::new(&config).await.map_err(|e| e.to_string())?;
    
    // Replace the AI service in state
    {
        let mut ai_service_guard = state.ai_service.write().await;
        *ai_service_guard = new_ai_service;
    }
    
    Ok(())
}

// OptimizedAIService commands
#[tauri::command]
async fn optimized_ai_chat(
    message: String,
    context: Option<String>,
    priority: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use ai_optimized::{AIRequest, RequestPriority};
    
    let optimized_service = state.optimized_ai_service.read().await;
    
    let priority_level = match priority.as_deref() {
        Some("critical") => RequestPriority::Critical,
        Some("high") => RequestPriority::High,
        Some("background") => RequestPriority::Background,
        _ => RequestPriority::Normal,
    };
    
    let request = AIRequest::new(message)
        .with_priority(priority_level);
    
    let request = if let Some(ctx) = context {
        request.with_context(ctx)
    } else {
        request
    };
    
    let response = optimized_service.chat_async(&request.prompt, request.context.as_deref()).await
        .map_err(|e| e.to_string())?;
    
    Ok(response.content)
}

#[tauri::command]
async fn get_ai_service_stats(
    state: State<'_, AppState>,
) -> Result<ai_optimized::PoolStats, String> {
    let optimized_service = state.optimized_ai_service.read().await;
    let stats = optimized_service.get_pool_stats().await;
    Ok(stats)
}

#[tauri::command]
async fn force_ai_cleanup(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let optimized_service = state.optimized_ai_service.read().await;
    optimized_service.force_cleanup().await;
    Ok(())
}

#[tauri::command]
async fn submit_ai_request(
    message: String,
    priority: String,
    timeout_seconds: Option<u64>,
    context: Option<String>,
    model: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use ai_optimized::{AIRequest, RequestPriority};
    use std::time::Duration;
    
    let priority_level = match priority.as_str() {
        "critical" => RequestPriority::Critical,
        "high" => RequestPriority::High,
        "background" => RequestPriority::Background,
        _ => RequestPriority::Normal,
    };
    
    let mut request = AIRequest::new(message).with_priority(priority_level);
    
    if let Some(timeout) = timeout_seconds {
        request = request.with_timeout(Duration::from_secs(timeout));
    }
    
    if let Some(ctx) = context {
        request = request.with_context(ctx);
    }
    
    if let Some(mdl) = model {
        request = request.with_model(mdl);
    }
    
    let optimized_service = state.optimized_ai_service.read().await;
    let mut response_rx = optimized_service.submit_request(request).await.map_err(|e| e.to_string())?;
    
    // Wait for response
    match response_rx.recv().await {
        Some(response) => {
            if response.success {
                Ok(response.content)
            } else {
                Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
            }
        }
        None => Err("No response received".to_string()),
    }
}

// Vision service commands
#[tauri::command]
async fn analyze_screen_with_ai(
    image_data: Vec<u8>,
    _prompt: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let vision_service = state.vision_service.read().await;
    let capture_id = uuid::Uuid::new_v4().to_string();
    vision_service
        .analyze_screen_comprehensive(&capture_id, image_data)
        .await
        .map(|analysis| analysis.summary)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn capture_and_analyze_screen(
    state: State<'_, AppState>,
) -> Result<vision::ScreenAnalysis, String> {
    let vision_service = state.vision_service.read().await;
    let capture = vision_service.capture_full_screen().await.map_err(|e| e.to_string())?;
    let capture_id = uuid::Uuid::new_v4().to_string();
    vision_service
        .analyze_screen_comprehensive(&capture_id, capture.data)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_clear_completed_requests(state: State<'_, AppState>) -> Result<(), String> {
    let ai_service = state.ai_service.read().await;
    ai_service.clear_completed_requests().await.map_err(|e| e.to_string())
}

// Optimized AI service commands
#[tauri::command]
async fn ai_submit_priority_request(
    prompt: String,
    priority: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let ai_service = state.ai_service.read().await;
    
    let priority_level = match priority.as_str() {
        "high" => RequestPriority::High,
        "critical" => RequestPriority::Critical,
        "low" => RequestPriority::Low,
        _ => RequestPriority::Normal,
    };
    
    ai_service
        .submit_priority_request(prompt, priority_level)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_batch_process(
    requests: Vec<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let ai_service = state.ai_service.read().await;
    
    let processed_requests: Vec<(String, RequestPriority)> = requests
        .into_iter()
        .filter_map(|req| {
            let prompt = req.get("prompt").and_then(|p| p.as_str())?;
            let priority_str = req.get("priority").and_then(|p| p.as_str()).unwrap_or("normal");
            
            let priority = match priority_str {
                "high" => RequestPriority::High,
                "critical" => RequestPriority::Critical,
                "low" => RequestPriority::Low,
                _ => RequestPriority::Normal,
            };
            
            Some((prompt.to_string(), priority))
        })
        .collect();
    
    ai_service
        .batch_process_requests(processed_requests)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_get_service_stats(state: State<'_, AppState>) -> Result<String, String> {
    let ai_service = state.ai_service.read().await;
    ai_service
        .get_service_stats()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_clear_completed(state: State<'_, AppState>) -> Result<(), String> {
    let ai_service = state.optimized_ai_service.read().await;
    ai_service.clear_completed().await;
    Ok(())
}

#[tauri::command]
async fn ai_chat_async(
    message: String,
    context: Option<String>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let ai_service = state.optimized_ai_service.read().await;
    let response = ai_service.chat_async(&message, context.as_deref()).await.map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "id": response.id,
        "content": response.content,
        "model_used": response.model_used,
        "processing_time_ms": response.processing_time.as_millis(),
        "tokens_used": response.tokens_used,
        "success": response.success,
        "error": response.error
    }))
}

#[tauri::command]
async fn ai_submit_async_request(
    prompt: String,
    model: Option<String>,
    priority: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let ai_service = state.optimized_ai_service.read().await;
    
    let priority = match priority.as_str() {
        "critical" => ai_optimized::RequestPriority::Critical,
        "high" => ai_optimized::RequestPriority::High,
        "normal" => ai_optimized::RequestPriority::Normal,
        "low" => ai_optimized::RequestPriority::Low,
        "background" => ai_optimized::RequestPriority::Background,
        _ => ai_optimized::RequestPriority::Normal,
    };
    
    let request = ai_optimized::AIRequest::simple(prompt)
        .with_priority(priority);
    
    if let Some(model) = model {
        let request = request.with_model(model);
        let _rx = ai_service.submit_request_async(request).await.map_err(|e| e.to_string())?;
    } else {
        let _rx = ai_service.submit_request_async(request).await.map_err(|e| e.to_string())?;
    }
    
    Ok("Request submitted successfully".to_string())
}

#[tauri::command]
async fn ai_analyze_critical_error(
    error_output: String,
    command: String,
    context: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let ai_service = state.ai_service.read().await;
    ai_service
        .analyze_critical_error(&error_output, &command, &context)
        .await
        .map_err(|e| e.to_string())
}

// Advanced AI Request Builder commands
#[tauri::command]
async fn ai_create_simple_request(
    prompt: String,
) -> Result<String, String> {
    use crate::ai_optimized::AIRequest;
    let request = AIRequest::simple(prompt);
    Ok(format!("Created request {} with timeout: {:?}s", request.id, request.timeout.as_secs()))
}

#[tauri::command]
async fn ai_create_custom_request(
    prompt: String,
    priority: String,
    timeout_seconds: u64,
    context: Option<String>,
    model: Option<String>,
) -> Result<serde_json::Value, String> {
    use crate::ai_optimized::{AIRequest, RequestPriority};
    use std::time::Duration;
    
    let priority_level = match priority.as_str() {
        "critical" => RequestPriority::Critical,
        "high" => RequestPriority::High,
        "low" => RequestPriority::Low,
        "background" => RequestPriority::Background,
        _ => RequestPriority::Normal,
    };
    
    let mut request = AIRequest::simple(prompt)
        .with_priority(priority_level)
        .with_timeout(Duration::from_secs(timeout_seconds));
    
    if let Some(ctx) = context {
        request = request.with_context(ctx);
    }
    
    if let Some(mdl) = model {
        request = request.with_model(mdl);
    }
    
    let can_retry = request.can_retry();
    let created_at_secs = request.created_at.elapsed().as_secs();
    
    Ok(serde_json::json!({
        "id": request.id,
        "priority": format!("{:?}", request.priority),
        "timeout": request.timeout.as_secs(),
        "max_retries": request.max_retries,
        "retry_count": request.retry_count,
        "can_retry": can_retry,
        "created_ago_seconds": created_at_secs
    }))
}

#[tauri::command]
async fn ai_increment_request_retry(
    request_data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use crate::ai_optimized::{AIRequest, RequestPriority};
    use std::time::Duration;
    
    // Reconstruct AIRequest from JSON for retry logic
    let prompt = request_data.get("prompt")
        .and_then(|v| v.as_str())
        .ok_or("Missing prompt")?
        .to_string();
    
    let mut request = AIRequest::simple(prompt);
    
    // Restore request properties
    if let Some(priority_str) = request_data.get("priority").and_then(|v| v.as_str()) {
        let priority = match priority_str {
            "Critical" => RequestPriority::Critical,
            "High" => RequestPriority::High,
            "Low" => RequestPriority::Low,
            "Background" => RequestPriority::Background,
            _ => RequestPriority::Normal,
        };
        request = request.with_priority(priority);
    }
    
    if let Some(timeout) = request_data.get("timeout").and_then(|v| v.as_u64()) {
        request = request.with_timeout(Duration::from_secs(timeout));
    }
    
    if let Some(retries) = request_data.get("retry_count").and_then(|v| v.as_u64()) {
        request.retry_count = retries as u32;
    }
    
    // Increment retry count
    request.increment_retry();
    
    Ok(serde_json::json!({
        "id": request.id,
        "retry_count": request.retry_count,
        "max_retries": request.max_retries,
        "can_retry": request.can_retry(),
        "updated": true
    }))
}

// AI Response Analysis commands
#[tauri::command]
async fn ai_analyze_response(
    response_data: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use crate::ai_optimized::AIResponse;
    use std::time::Duration;
    
    // Extract response data for analysis
    let content = response_data.get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let processing_time_ms = response_data.get("processing_time_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    let success = response_data.get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    
    let tokens_used = response_data.get("tokens_used")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    
    // Create AIResponse for analysis
    let response = AIResponse {
        id: uuid::Uuid::new_v4().to_string(),
        request_id: response_data.get("request_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        content: content.to_string(),
        model_used: response_data.get("model_used")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        processing_time: Duration::from_millis(processing_time_ms),
        tokens_used,
        success,
        error: response_data.get("error")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    };
    
    Ok(serde_json::json!({
        "response_id": response.id,
        "request_id": response.request_id,
        "content_length": response.content.len(),
        "model_used": response.model_used,
        "processing_time_ms": response.processing_time.as_millis(),
        "tokens_used": response.tokens_used,
        "success": response.success,
        "has_error": response.error.is_some(),
        "word_count": response.content.split_whitespace().count(),
        "line_count": response.content.lines().count()
    }))
}

// Direct VisionService commands
#[tauri::command]
async fn vision_initialize_service() -> Result<(), String> {
    let vision_service = vision::get_vision_service();
    let mut service = vision_service.lock().await;
    service.initialize().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn vision_capture_full_screen() -> Result<vision::ScreenCapture, String> {
    let vision_service = vision::get_vision_service();
    let service = vision_service.lock().await;
    service.capture_full_screen().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn vision_capture_region(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<vision::ScreenCapture, String> {
    let vision_service = vision::get_vision_service();
    let service = vision_service.lock().await;
    service.capture_screen_region(x, y, width, height).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn vision_perform_ocr(
    image_path: String,
    engine: String,
) -> Result<Vec<vision::OCRResult>, String> {
    let vision_service = vision::get_vision_service();
    let service = vision_service.lock().await;
    service.perform_ocr(&image_path, &engine).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn vision_detect_ui_elements(
    image_path: String,
) -> Result<Vec<vision::VisualElement>, String> {
    let vision_service = vision::get_vision_service();
    let service = vision_service.lock().await;
    service.detect_ui_elements(&image_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn vision_analyze_with_ai(
    image_data: Vec<u8>,
    prompt: String,
    context: String,
    ollama_host: String,
    ollama_port: String,
) -> Result<String, String> {
    let vision_service = vision::get_vision_service();
    let service = vision_service.lock().await;
    service.analyze_screen_with_ai(image_data, prompt, context, ollama_host, ollama_port)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn vision_comprehensive_analysis(
    capture_id: String,
    image_data: Vec<u8>,
) -> Result<vision::ScreenAnalysis, String> {
    let vision_service = vision::get_vision_service();
    let service = vision_service.lock().await;
    service.analyze_screen_comprehensive(&capture_id, image_data)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn vision_check_dependencies() -> Result<(), String> {
    let vision_service = vision::get_vision_service();
    let service = vision_service.lock().await;
    service.check_vision_dependencies().await.map_err(|e| e.to_string())
}

// HTTP Client Pool Management commands
#[tauri::command]
async fn ai_create_optimized_service(
    state: State<'_, AppState>,
) -> Result<String, String> {
    use crate::ai_optimized::OptimizedAIService;
    
    let config = {
        let config_guard = state.config.read().await;
        config_guard.ai.clone()
    };
    
    match OptimizedAIService::new_with_config(&config).await {
        Ok(_service) => Ok("Optimized AI service created successfully".to_string()),
        Err(e) => Err(format!("Failed to create optimized service: {}", e))
    }
}

#[tauri::command]
async fn ai_get_pool_stats(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let ai_service = state.ai_service.read().await;
    
    // Access the optimized service if available
    if let Some(optimized) = &ai_service.optimized_service {
        let stats = optimized.get_pool_stats().await;
        Ok(serde_json::json!({
            "active_connections": stats.active_connections,
            "idle_connections": stats.idle_connections,
            "pending_requests": stats.pending_requests,
            "processed_requests": stats.processed_requests,
            "failed_requests": stats.failed_requests,
            "average_response_time": stats.average_response_time,
            "queue_by_priority": stats.queue_by_priority
        }))
    } else {
        Err("Optimized service not available".to_string())
    }
}

#[tauri::command]
async fn close_terminal(
    terminal_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut terminal_manager = state.terminal_manager.write().await;
    terminal_manager
        .kill_terminal(&terminal_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_terminal_info(
    terminal_id: String,
    state: State<'_, AppState>,
) -> Result<Option<terminal::TerminalInfo>, String> {
    let terminal_manager = state.terminal_manager.read().await;
    Ok(terminal_manager.get_terminal_info(&terminal_id))
}

#[tauri::command]
async fn list_terminals(
    state: State<'_, AppState>,
) -> Result<Vec<terminal::TerminalInfo>, String> {
    let terminal_manager = state.terminal_manager.read().await;
    Ok(terminal_manager.list_terminals())
}

#[tauri::command]
async fn get_terminal_count(
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let terminal_manager = state.terminal_manager.read().await;
    Ok(terminal_manager.get_terminal_count())
}

// System utilities
#[tauri::command]
async fn get_system_info() -> Result<HashMap<String, String>, String> {
    utils::get_system_info().map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_files(
    query: String,
    path: String,
    include_content: bool,
) -> Result<Vec<String>, String> {
    utils::search_files(&query, &path, include_content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn execute_safe_system_command(command: String) -> Result<String, String> {
    utils::execute_safe_command(&command).await.map_err(|e| e.to_string())
}

// AI System Diagnostic and Repair Commands
#[tauri::command]
async fn ai_diagnose_system(
    issue_description: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let system_info = utils::get_detailed_system_info().await.map_err(|e| e.to_string())?;
    let ai_service = state.ai_service.read().await;
    ai_service
        .diagnose_system_issue(&issue_description, &system_info)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_fix_compilation(
    error_output: String,
    project_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let project_context = utils::analyze_project_structure(&project_path).await.map_err(|e| e.to_string())?;
    let ai_service = state.ai_service.read().await;
    ai_service
        .fix_compilation_errors(&error_output, &project_context)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_fix_packages(
    package_manager: String,
    error_output: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let ai_service = state.ai_service.read().await;
    ai_service
        .fix_package_issues(&package_manager, &error_output)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_fix_service(
    service_name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let service_status = utils::get_service_status(&service_name).await.map_err(|e| e.to_string())?;
    let service_logs = utils::get_service_logs(&service_name).await.unwrap_or_default();
    let ai_service = state.ai_service.read().await;
    ai_service
        .fix_service_issues(&service_name, &service_status, &service_logs)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_fix_environment(
    tool_name: String,
    error: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let installation_context = utils::get_environment_info().await.map_err(|e| e.to_string())?;
    let ai_service = state.ai_service.read().await;
    ai_service
        .fix_environment_setup(&tool_name, &installation_context, &error)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_fix_display(
    display_error: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let desktop_env = utils::get_desktop_environment().await.unwrap_or("unknown".to_string());
    let ai_service = state.ai_service.read().await;
    ai_service
        .fix_display_issues(&display_error, &desktop_env)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_fix_network(
    network_problem: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let network_config = utils::get_network_config().await.map_err(|e| e.to_string())?;
    let ai_service = state.ai_service.read().await;
    ai_service
        .fix_network_issues(&network_problem, &network_config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_fix_permissions(
    permission_error: String,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let file_context = utils::analyze_file_permissions(&file_path).await.map_err(|e| e.to_string())?;
    let ai_service = state.ai_service.read().await;
    ai_service
        .fix_permission_issues(&permission_error, &file_context)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ai_auto_fix(
    issue_type: String,
    context: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let ai_service = state.ai_service.read().await;
    ai_service
        .auto_fix_system(&issue_type, &context)
        .await
        .map_err(|e| e.to_string())
}

// Template execution commands
#[tauri::command]
async fn execute_template_command(
    command: String,
    working_directory: Option<String>,
) -> Result<serde_json::Value, String> {
    use tokio::process::Command;
    
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", &command]);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c").arg(&command);
        c
    };
    
    if let Some(wd) = working_directory {
        cmd.current_dir(wd);
    }
    
    let output = cmd.output().await.map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "output": String::from_utf8_lossy(&output.stdout),
        "exitCode": output.status.code().unwrap_or(-1)
    }))
}

#[tauri::command]
async fn import_templates(file_path: String) -> Result<Vec<serde_json::Value>, String> {
    use tokio::fs;
    
    let content = fs::read_to_string(&file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let templates: Vec<serde_json::Value> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    Ok(templates)
}

#[tauri::command]
async fn export_templates(
    templates: Vec<serde_json::Value>,
    file_path: String,
) -> Result<(), String> {
    use tokio::fs;
    
    let json_content = serde_json::to_string_pretty(&templates)
        .map_err(|e| format!("Failed to serialize templates: {}", e))?;
    
    fs::write(&file_path, json_content)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}

// Web scraping commands - fixed thread safety
#[tauri::command]
async fn start_web_scraping(
    _job_id: String,
    options: web_scraper::ScrapingOptions,
) -> Result<String, String> {
    // Clone the scraper to avoid holding mutex across await
    let scraper = {
        let guard = web_scraper::get_web_scraper().lock().map_err(|e| e.to_string())?;
        guard.clone()
    };
    let mut scraper = scraper;
    scraper.start_scraping(options).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_scraping_progress(job_id: String) -> Result<web_scraper::ScrapingResult, String> {
    let scraper = web_scraper::get_web_scraper().lock().map_err(|e| e.to_string())?;
    scraper.get_scraping_progress(&job_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn scrape_single_page(
    url: String,
    output_path: Option<String>,
) -> Result<web_scraper::DownloadedFile, String> {
    let scraper = {
        let guard = web_scraper::get_web_scraper().lock().map_err(|e| e.to_string())?;
        guard.clone()
    };
    scraper.scrape_single_page(&url, output_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn extract_links(url: String) -> Result<Vec<String>, String> {
    let scraper = {
        let guard = web_scraper::get_web_scraper().lock().map_err(|e| e.to_string())?;
        guard.clone()
    };
    scraper.extract_links(&url).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn generate_site_map(
    url: String,
    max_depth: u32,
) -> Result<web_scraper::SiteMap, String> {
    let scraper = {
        let guard = web_scraper::get_web_scraper().lock().map_err(|e| e.to_string())?;
        guard.clone()
    };
    scraper.generate_site_map(&url, max_depth).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_robots_txt(url: String) -> Result<web_scraper::RobotsTxtInfo, String> {
    let scraper = {
        let guard = web_scraper::get_web_scraper().lock().map_err(|e| e.to_string())?;
        guard.clone()
    };
    scraper.check_robots_txt(&url).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_website_metadata(url: String) -> Result<web_scraper::WebsiteMetadata, String> {
    let scraper = {
        let guard = web_scraper::get_web_scraper().lock().map_err(|e| e.to_string())?;
        guard.clone()
    };
    scraper.get_website_metadata(&url).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn estimate_scraping(
    options: web_scraper::ScrapingOptions,
) -> Result<web_scraper::ScrapingEstimate, String> {
    let scraper = {
        let guard = web_scraper::get_web_scraper().lock().map_err(|e| e.to_string())?;
        guard.clone()
    };
    scraper.estimate_scraping(&options).await.map_err(|e| e.to_string())
}

// Terminal broadcasting commands
#[tauri::command]
async fn register_broadcast_session(
    session: broadcast::TerminalSession,
) -> Result<(), String> {
    let manager = broadcast::get_broadcast_manager();
    manager.register_session(session).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn unregister_broadcast_session(session_id: String) -> Result<(), String> {
    let manager = broadcast::get_broadcast_manager();
    manager.unregister_session(&session_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_broadcast_sessions() -> Result<Vec<broadcast::TerminalSession>, String> {
    let manager = broadcast::get_broadcast_manager();
    manager.get_sessions().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_broadcast_session(session_id: String) -> Result<Option<broadcast::TerminalSession>, String> {
    let manager = broadcast::get_broadcast_manager();
    manager.get_session(&session_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_session_status(
    session_id: String,
    status: broadcast::SessionStatus,
) -> Result<(), String> {
    let manager = broadcast::get_broadcast_manager();
    manager.update_session_status(&session_id, status).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn execute_on_session(
    session_id: String,
    command: String,
) -> Result<broadcast::SessionResult, String> {
    let manager = broadcast::get_broadcast_manager();
    manager.execute_on_session(&session_id, &command).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn broadcast_command(
    session_ids: Vec<String>,
    command: String,
) -> Result<broadcast::BroadcastResult, String> {
    let manager = broadcast::get_broadcast_manager();
    manager.broadcast_command(&session_ids, &command).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_broadcast_sessions(
    sessions: Vec<broadcast::TerminalSession>,
) -> Result<usize, String> {
    let manager = broadcast::get_broadcast_manager();
    manager.import_sessions(sessions).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn export_broadcast_sessions() -> Result<Vec<broadcast::TerminalSession>, String> {
    let manager = broadcast::get_broadcast_manager();
    manager.export_sessions().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_local_session(name: String) -> Result<broadcast::TerminalSession, String> {
    Ok(broadcast::BroadcastManager::create_local_session(name))
}

#[tauri::command]
async fn get_active_broadcasts() -> Result<Vec<String>, String> {
    let manager = broadcast::get_broadcast_manager();
    manager.get_active_broadcasts().await.map_err(|e| e.to_string())
}

// Security Scanner commands
#[tauri::command]
async fn security_scan_directory(
    path: String,
    scan_type: String,
    state: State<'_, AppState>,
) -> Result<security_scanner::ScanResult, String> {
    let security_scanner = state.security_scanner.read().await;
    let scan_type = match scan_type.as_str() {
        "vulnerabilities" => security_scanner::ScanType::Vulnerabilities,
        "malware" => security_scanner::ScanType::Malware,
        "secrets" => security_scanner::ScanType::Secrets,
        "dependencies" => security_scanner::ScanType::Dependencies,
        _ => security_scanner::ScanType::Comprehensive,
    };
    security_scanner.scan_directory(&path, scan_type).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn security_scan_real_time(
    enable: bool,
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut security_scanner = state.security_scanner.write().await;
    if enable {
        security_scanner.start_real_time_monitoring(paths).await.map_err(|e| e.to_string())
    } else {
        security_scanner.stop_real_time_monitoring().await.map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn security_get_scan_results(
    scan_id: String,
    state: State<'_, AppState>,
) -> Result<security_scanner::ScanResult, String> {
    let security_scanner = state.security_scanner.read().await;
    security_scanner.get_scan_results(&scan_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn security_set_scan_config(
    config: security_scanner::SecurityConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut security_scanner = state.security_scanner.write().await;
    security_scanner.update_config(config).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn security_update_rules(
    rules: Vec<security_scanner::SecurityRule>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut security_scanner = state.security_scanner.write().await;
    security_scanner.update_rules(rules).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn security_get_vulnerabilities(
    severity: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<security_scanner::Vulnerability>, String> {
    let security_scanner = state.security_scanner.read().await;
    security_scanner.get_vulnerabilities(severity).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn security_remediate_vulnerability(
    vulnerability_id: String,
    auto_fix: bool,
    state: State<'_, AppState>,
) -> Result<security_scanner::RemediationResult, String> {
    let mut security_scanner = state.security_scanner.write().await;
    security_scanner.remediate_vulnerability(&vulnerability_id, auto_fix).await.map_err(|e| e.to_string())
}

// Command Flow Visualization commands
#[tauri::command]
async fn command_flow_analyze(
    command: String,
    context: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<command_flow::FlowAnalysis, String> {
    let command_flow_engine = state.command_flow_engine.read().await;
    command_flow_engine.analyze_command(&command, &context).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn command_flow_create_graph(
    commands: Vec<String>,
    state: State<'_, AppState>,
) -> Result<command_flow::DependencyGraph, String> {
    let command_flow_engine = state.command_flow_engine.read().await;
    command_flow_engine.create_dependency_graph(&commands).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn command_flow_get_dependencies(
    command: String,
    state: State<'_, AppState>,
) -> Result<Vec<command_flow::CommandDependency>, String> {
    let command_flow_engine = state.command_flow_engine.read().await;
    command_flow_engine.get_command_dependencies(&command).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn command_flow_visualize_execution(
    execution_id: String,
    state: State<'_, AppState>,
) -> Result<command_flow::ExecutionVisualization, String> {
    let command_flow_engine = state.command_flow_engine.read().await;
    command_flow_engine.visualize_execution(&execution_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn command_flow_track_execution(
    command: String,
    start_time: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut command_flow_engine = state.command_flow_engine.write().await;
    command_flow_engine.track_command_execution(&command, &start_time).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn command_flow_get_execution_history(
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<command_flow::ExecutionRecord>, String> {
    let command_flow_engine = state.command_flow_engine.read().await;
    command_flow_engine.get_execution_history(limit).await.map_err(|e| e.to_string())
}

// Plugin System commands
#[tauri::command]
async fn plugin_install(
    plugin_path: String,
    state: State<'_, AppState>,
) -> Result<plugin_system::InstallResult, String> {
    let mut plugin_system = state.plugin_system.write().await;
    plugin_system.install_plugin(&plugin_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn plugin_uninstall(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut plugin_system = state.plugin_system.write().await;
    plugin_system.uninstall_plugin(&plugin_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn plugin_list(
    state: State<'_, AppState>,
) -> Result<Vec<plugin_system::PluginInfo>, String> {
    let plugin_system = state.plugin_system.read().await;
    plugin_system.list_plugins().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn plugin_enable(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut plugin_system = state.plugin_system.write().await;
    plugin_system.enable_plugin(&plugin_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn plugin_disable(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut plugin_system = state.plugin_system.write().await;
    plugin_system.disable_plugin(&plugin_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn plugin_get_marketplace(
    state: State<'_, AppState>,
) -> Result<Vec<plugin_system::MarketplacePlugin>, String> {
    let plugin_system = state.plugin_system.read().await;
    plugin_system.get_marketplace_plugins().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn plugin_execute_command(
    plugin_id: String,
    command: String,
    args: Vec<String>,
    state: State<'_, AppState>,
) -> Result<plugin_system::PluginExecutionResult, String> {
    let plugin_system = state.plugin_system.read().await;
    plugin_system.execute_plugin_command(&plugin_id, &command, &args).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn plugin_get_info(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<plugin_system::PluginInfo, String> {
    let plugin_system = state.plugin_system.read().await;
    plugin_system.get_plugin_info(&plugin_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn plugin_update(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<plugin_system::UpdateResult, String> {
    let mut plugin_system = state.plugin_system.write().await;
    plugin_system.update_plugin(&plugin_id).await.map_err(|e| e.to_string())
}

// Collaboration commands
#[tauri::command]
async fn collaboration_create_session(
    name: String,
    permissions: collaboration::SessionPermissions,
    state: State<'_, AppState>,
) -> Result<collaboration::CollaborationSession, String> {
    let collaboration_manager = state.collaboration_manager.write().await;
    collaboration_manager.create_session(&name, permissions).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn collaboration_join_session(
    session_id: String,
    user_id: String,
    state: State<'_, AppState>,
) -> Result<collaboration::JoinResult, String> {
    let collaboration_manager = state.collaboration_manager.write().await;
    collaboration_manager.join_session(&session_id, &user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn collaboration_leave_session(
    session_id: String,
    user_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let collaboration_manager = state.collaboration_manager.write().await;
    collaboration_manager.leave_session(&session_id, &user_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn collaboration_share_terminal(
    terminal_id: String,
    session_id: String,
    permissions: collaboration::TerminalPermissions,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let collaboration_manager = state.collaboration_manager.write().await;
    collaboration_manager.share_terminal(&terminal_id, &session_id, permissions).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn collaboration_get_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<collaboration::CollaborationSession>, String> {
    let collaboration_manager = state.collaboration_manager.read().await;
    collaboration_manager.get_sessions().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn collaboration_send_message(
    session_id: String,
    message: collaboration::ChatMessage,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let collaboration_manager = state.collaboration_manager.write().await;
    collaboration_manager.send_message(&session_id, message).await.map_err(|e| e.to_string())
}

// Workflow Automation commands
#[tauri::command]
async fn workflow_create(
    name: String,
    description: String,
    steps: Vec<workflow_automation::WorkflowStep>,
    state: State<'_, AppState>,
) -> Result<workflow_automation::Workflow, String> {
    let mut workflow_engine = state.workflow_engine.write().await;
    workflow_engine.create_workflow_with_steps(&name, &description, steps).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn workflow_execute(
    workflow_id: String,
    parameters: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<workflow_automation::ExecutionResult, String> {
    let workflow_engine = state.workflow_engine.write().await;
    workflow_engine.execute_workflow_with_params(&workflow_id, &parameters).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn workflow_list(
    state: State<'_, AppState>,
) -> Result<Vec<workflow_automation::WorkflowInfo>, String> {
    let workflow_engine = state.workflow_engine.read().await;
    workflow_engine.list_workflow_infos().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn workflow_delete(
    workflow_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut workflow_engine = state.workflow_engine.write().await;
    workflow_engine.delete_workflow_by_id(&workflow_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn workflow_record_macro(
    name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut workflow_engine = state.workflow_engine.write().await;
    workflow_engine.start_recording_macro(&name).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn workflow_stop_recording(
    recording_id: String,
    state: State<'_, AppState>,
) -> Result<workflow_automation::Workflow, String> {
    let mut workflow_engine = state.workflow_engine.write().await;
    workflow_engine.stop_recording_macro(&recording_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn workflow_get_execution_history(
    workflow_id: String,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<workflow_automation::ExecutionRecord>, String> {
    let workflow_engine = state.workflow_engine.read().await;
    workflow_engine.get_execution_history(&workflow_id, limit).await.map_err(|e| e.to_string())
}

// Analytics commands
#[tauri::command]
async fn analytics_get_performance(
    time_range: String,
    state: State<'_, AppState>,
) -> Result<analytics::PerformanceMetrics, String> {
    let analytics_engine = state.analytics_engine.read().await;
    analytics_engine.get_performance_metrics(&time_range).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn analytics_get_usage_stats(
    period: String,
    state: State<'_, AppState>,
) -> Result<analytics::UsageStatistics, String> {
    let analytics_engine = state.analytics_engine.read().await;
    analytics_engine.get_usage_statistics(&period).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn analytics_get_insights(
    state: State<'_, AppState>,
) -> Result<Vec<analytics::Insight>, String> {
    let analytics_engine = state.analytics_engine.read().await;
    analytics_engine.get_insights().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn analytics_track_command(
    command: String,
    execution_time: u64,
    success: bool,
    context: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut analytics_engine = state.analytics_engine.write().await;
    analytics_engine.track_command(&command, execution_time, success, &context).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn analytics_get_command_patterns(
    state: State<'_, AppState>,
) -> Result<Vec<analytics::CommandPattern>, String> {
    let analytics_engine = state.analytics_engine.read().await;
    analytics_engine.get_command_patterns().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn analytics_get_optimization_suggestions(
    state: State<'_, AppState>,
) -> Result<Vec<analytics::OptimizationSuggestion>, String> {
    let analytics_engine = state.analytics_engine.read().await;
    analytics_engine.get_optimization_suggestions().await.map_err(|e| e.to_string())
}

// Ecosystem Awareness commands
#[tauri::command]
async fn ecosystem_get_comprehensive_context(
    state: State<'_, AppState>,
) -> Result<ecosystem_awareness::ComprehensiveContext, String> {
    let ecosystem_awareness = state.ecosystem_awareness.read().await;
    ecosystem_awareness.get_comprehensive_context().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ecosystem_learn_from_interaction(
    command: String,
    success: bool,
    execution_time: u64,
    user_context: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let ecosystem_awareness = state.ecosystem_awareness.read().await;
    let interaction = ecosystem_awareness::UserInteraction {
        command,
        success,
        execution_time,
        user_context,
        timestamp: chrono::Utc::now(),
    };
    ecosystem_awareness.learn_from_interaction(interaction).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ecosystem_predict_intent(
    input: String,
    context: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<Vec<ecosystem_awareness::IntentPrediction>, String> {
    let ecosystem_awareness = state.ecosystem_awareness.read().await;
    ecosystem_awareness.predict_intent(&input, &context).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ecosystem_get_adaptive_suggestions(
    context: String,
    state: State<'_, AppState>,
) -> Result<Vec<ecosystem_awareness::AdaptiveSuggestion>, String> {
    let ecosystem_awareness = state.ecosystem_awareness.read().await;
    ecosystem_awareness.get_adaptive_suggestions(&context).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ecosystem_analyze_pattern(
    pattern_type: String,
    data: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<ecosystem_awareness::PatternAnalysis, String> {
    let ecosystem_awareness = state.ecosystem_awareness.read().await;
    ecosystem_awareness.analyze_pattern(&pattern_type, &data).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ecosystem_get_system_insights(
    state: State<'_, AppState>,
) -> Result<Vec<ecosystem_awareness::SystemInsight>, String> {
    let ecosystem_awareness = state.ecosystem_awareness.read().await;
    ecosystem_awareness.get_system_insights().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ecosystem_update_learning_preferences(
    preferences: ecosystem_awareness::LearningPreferences,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut ecosystem_awareness = state.ecosystem_awareness.write().await;
    ecosystem_awareness.update_learning_preferences(preferences).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ecosystem_get_context_correlation(
    context_a: String,
    context_b: String,
    state: State<'_, AppState>,
) -> Result<f64, String> {
    let ecosystem_awareness = state.ecosystem_awareness.read().await;
    ecosystem_awareness.get_context_correlation(&context_a, &context_b).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ecosystem_predict_user_intent(
    partial_input: String,
    state: State<'_, AppState>,
) -> Result<Vec<ecosystem_awareness::IntentPrediction>, String> {
    let ecosystem_awareness = state.ecosystem_awareness.read().await;
    ecosystem_awareness.predict_user_intent(&partial_input).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ecosystem_analyze_system_patterns(
    state: State<'_, AppState>,
) -> Result<ecosystem_awareness::SystemPatternAnalysis, String> {
    let ecosystem_awareness = state.ecosystem_awareness.read().await;
    ecosystem_awareness.analyze_system_patterns().await.map_err(|e| e.to_string())
}

// Cloud Integration commands
#[tauri::command]
async fn cloud_backup_config(
    provider: String,
    config: cloud_integration::BackupConfig,
    state: State<'_, AppState>,
) -> Result<cloud_integration::BackupResult, String> {
    let mut cloud_manager = state.cloud_manager.write().await;
    cloud_manager.backup_configuration(&provider, config).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cloud_sync_data(
    provider: String,
    data_types: Vec<String>,
    state: State<'_, AppState>,
) -> Result<cloud_integration::SyncResult, String> {
    let mut cloud_manager = state.cloud_manager.write().await;
    cloud_manager.sync_data(&provider, &data_types).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cloud_restore_backup(
    provider: String,
    backup_id: String,
    state: State<'_, AppState>,
) -> Result<cloud_integration::RestoreResult, String> {
    let mut cloud_manager = state.cloud_manager.write().await;
    cloud_manager.restore_backup(&provider, &backup_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cloud_get_status(
    state: State<'_, AppState>,
) -> Result<cloud_integration::CloudStatus, String> {
    let cloud_manager = state.cloud_manager.read().await;
    cloud_manager.get_status().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cloud_configure_provider(
    provider: String,
    config: cloud_integration::ProviderConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut cloud_manager = state.cloud_manager.write().await;
    cloud_manager.configure_provider(&provider, config).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cloud_list_backups(
    provider: String,
    state: State<'_, AppState>,
) -> Result<Vec<cloud_integration::BackupInfo>, String> {
    let cloud_manager = state.cloud_manager.read().await;
    cloud_manager.list_backups(&provider).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cloud_get_providers(
    state: State<'_, AppState>,
) -> Result<Vec<cloud_integration::CloudProvider>, String> {
    let cloud_manager = state.cloud_manager.read().await;
    cloud_manager.get_available_providers().await.map_err(|e| e.to_string())
}

// LocalRecall RAG commands
#[tauri::command]
async fn local_recall_health_check() -> Result<(), String> {
    let client = local_recall::LocalRecallClient::default();
    client.health_check().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_list_collections() -> Result<Vec<local_recall::Collection>, String> {
    let client = local_recall::LocalRecallClient::default();
    client.list_collections().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_create_collection(
    name: String,
    description: Option<String>,
) -> Result<(), String> {
    let client = local_recall::LocalRecallClient::default();
    client.create_collection(&name, description).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_add_text(
    collection: String,
    content: String,
    metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    source: Option<String>,
) -> Result<(), String> {
    let client = local_recall::LocalRecallClient::default();
    client.add_text(&collection, &content, metadata, source).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_search(
    collection: String,
    query: String,
    max_results: Option<u32>,
    threshold: Option<f32>,
) -> Result<local_recall::SearchResponse, String> {
    let client = local_recall::LocalRecallClient::default();
    client.search(&collection, &query, max_results, threshold).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_index_command(
    command: String,
    output: String,
    working_dir: String,
    exit_code: i32,
    duration_ms: u64,
) -> Result<(), String> {
    let client = local_recall::LocalRecallClient::default();
    client.index_command(&command, &output, &working_dir, exit_code, duration_ms)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_index_conversation(
    messages: Vec<(String, String)>,
    context: Option<String>,
) -> Result<(), String> {
    let client = local_recall::LocalRecallClient::default();
    let message_refs: Vec<(&str, &str)> = messages.iter()
        .map(|(role, content)| (role.as_str(), content.as_str()))
        .collect();
    client.index_conversation(&message_refs, context.as_deref())
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_index_codebase(
    project_path: String,
    files: Vec<String>,
) -> Result<(), String> {
    let client = local_recall::LocalRecallClient::default();
    client.index_codebase(&project_path, &files).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_get_stats() -> Result<std::collections::HashMap<String, serde_json::Value>, String> {
    let client = local_recall::LocalRecallClient::default();
    client.get_stats().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_ensure_running() -> Result<(), String> {
    local_recall::ensure_localrecall_running().await.map_err(|e| e.to_string())
}

// Additional LocalRecall RAG commands
#[tauri::command]
async fn local_recall_initialize() -> Result<(), String> {
    let client = local_recall::LocalRecallClient::default();
    client.initialize().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_add_external_source(
    collection: String,
    url: String,
    update_interval: Option<u32>,
    metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
) -> Result<(), String> {
    let client = local_recall::LocalRecallClient::default();
    client.add_external_source(&collection, &url, update_interval, metadata).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_upload_file(
    collection: String,
    file_path: String,
    metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
) -> Result<(), String> {
    let client = local_recall::LocalRecallClient::default();
    client.upload_file(&collection, &file_path, metadata).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_reset_collection(
    collection: String,
) -> Result<(), String> {
    let client = local_recall::LocalRecallClient::default();
    client.reset_collection(&collection).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_auto_index_project(
    working_dir: String,
) -> Result<(), String> {
    let client = local_recall::LocalRecallClient::default();
    client.auto_index_project(&working_dir).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_recall_get_default_collection() -> Result<String, String> {
    let client = local_recall::LocalRecallClient::default();
    Ok(client.get_default_collection().to_string())
}

#[tauri::command]
async fn local_recall_set_default_collection(
    collection: String,
) -> Result<(), String> {
    let mut client = local_recall::LocalRecallClient::default();
    client.set_default_collection(collection);
    Ok(())
}

#[tauri::command]
async fn local_recall_get_context_for_prompt(
    query: String,
    max_results: Option<u32>,
) -> Result<String, String> {
    let client = local_recall::LocalRecallClient::default();
    client.get_context_for_prompt(&query, max_results).await.map_err(|e| e.to_string())
}

// Enhanced AI chat with memory
#[tauri::command]
async fn ai_chat_with_memory(
    message: String,
    conversation_id: String,
    context: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let ai_service = state.ai_service.read().await;
    ai_service
        .chat_with_memory(&message, &conversation_id, context.as_deref())
        .await
        .map_err(|e| e.to_string())
}

// Ollama Configuration commands
#[tauri::command]
async fn ollama_check_installation() -> Result<bool, String> {
    ollama_config::check_ollama_installation().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ollama_check_external_models() -> Result<bool, String> {
    ollama_config::check_external_models().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ollama_configure_models_path() -> Result<(), String> {
    ollama_config::configure_ollama_models_path().await.map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn ollama_start_service() -> Result<(), String> {
    ollama_config::start_ollama_service().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ollama_is_running() -> Result<bool, String> {
    Ok(ollama_config::is_ollama_running().await)
}

#[tauri::command]
async fn ollama_get_available_models() -> Result<Vec<String>, String> {
    ollama_config::get_available_models_from_ollama().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ollama_initialize_config() -> Result<ollama_config::OllamaConfig, String> {
    ollama_config::initialize_ollama_config().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ollama_ensure_configured() -> Result<(), String> {
    ollama_config::ensure_ollama_configured().await.map_err(|e| e.to_string())
}



#[tokio::main]
async fn main() {
    // Load .env file first for environment configuration
    if let Err(e) = dotenv::dotenv() {
        // .env file loading is optional, just warn if it fails
        eprintln!("⚠️  Warning: Could not load .env file: {}. Using system environment variables.", e);
    } else {
        println!("✅ Loaded configuration from .env file");
    }
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Initialize Ollama configuration at startup
    println!("🔧 Configuring Ollama at startup...");
    if let Err(e) = ollama_config::ensure_ollama_configured().await {
        eprintln!("⚠️  Ollama configuration failed: {}", e);
        eprintln!("   The application will continue, but AI features may be limited.");
    }

    // Initialize application state
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config, using defaults: {}", e);
        AppConfig::default()
    });
    
    // Ensure all configured directories exist
    if let Err(e) = config.ensure_directories() {
        eprintln!("Warning: Failed to create directories: {}", e);
    }
    let terminal_manager = TerminalManager::new();
    let ai_service = match AIService::new(&config.ai).await {
        Ok(service) => {
            println!("✅ AI service initialized successfully");
            service
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize AI service: {}", e);
            eprintln!("🔧 Attempting fallback AI service...");
            AIService::default()
        }
    };
    
    let optimized_ai_service = match OptimizedAIService::new(&config.ai).await {
        Ok(service) => service,
        Err(e) => {
            eprintln!("Warning: Failed to initialize OptimizedAIService: {}", e);
            // Try creating a fallback service
            match OptimizedAIService::new(&config.ai).await {
                Ok(service) => service,
                Err(e2) => {
                    eprintln!("Fatal: Could not initialize fallback AI service: {}", e2);
                    std::process::exit(1);
                }
            }
        }
    };
    
    let mut vision_service = VisionService::new();
    if let Err(e) = vision_service.initialize().await {
        eprintln!("Warning: Failed to initialize vision service: {}", e);
    }

    // Initialize Phase 4 services
    let security_scanner = security_scanner::SecurityScanner::new(security_scanner::SecurityConfig::default());
    let command_flow_engine = command_flow::CommandFlowEngine::new();
    let plugin_system = plugin_system::PluginSystem::new(config.paths.data_dir.join("plugins"));
    let collaboration_manager = collaboration::CollaborationManager::new();
    let workflow_engine = workflow_automation::WorkflowEngine::new();
    let analytics_engine = analytics::AnalyticsEngine::new();
    let cloud_manager = cloud_integration::CloudIntegrationManager::new();
    
    // Initialize Ecosystem Awareness with Adaptive Learning
    let ecosystem_awareness = match ecosystem_awareness::EcosystemAwareness::new().await {
        Ok(awareness) => awareness,
        Err(e) => {
            eprintln!("Warning: Failed to initialize ecosystem awareness: {}", e);
            // Create a minimal fallback implementation
            ecosystem_awareness::EcosystemAwareness::default()
        }
    };

    let app_state = AppState {
        terminal_manager: Arc::new(RwLock::new(terminal_manager)),
        ai_service: Arc::new(RwLock::new(ai_service)),
        optimized_ai_service: Arc::new(RwLock::new(optimized_ai_service)),
        vision_service: Arc::new(RwLock::new(vision_service)),
        config: Arc::new(RwLock::new(config)),
        security_scanner: Arc::new(RwLock::new(security_scanner)),
        command_flow_engine: Arc::new(RwLock::new(command_flow_engine)),
        plugin_system: Arc::new(RwLock::new(plugin_system)),
        collaboration_manager: Arc::new(RwLock::new(collaboration_manager)),
        workflow_engine: Arc::new(RwLock::new(workflow_engine)),
        analytics_engine: Arc::new(RwLock::new(analytics_engine)),
        cloud_manager: Arc::new(RwLock::new(cloud_manager)),
        ecosystem_awareness: Arc::new(RwLock::new(ecosystem_awareness)),
    };

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            // Initialize terminal app handle for event emission
            terminal::init_app_handle(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // AI commands
            ai_chat,
            ai_complete_command,
            ai_explain_error,
            ai_generate_code,
            ai_analyze_repository,
            ai_suggest_improvements,
            ai_explain_concept,
            check_ai_connection,
            get_current_model,
            set_ai_model,
            get_available_models,
            send_ai_message,
            get_terminal_context,
            // AI System Diagnostic and Repair
            ai_diagnose_system,
            ai_fix_compilation,
            ai_fix_packages,
            ai_fix_service,
            ai_fix_environment,
            ai_fix_display,
            ai_fix_network,
            ai_fix_permissions,
            ai_auto_fix,
            // Computer Vision commands (from vision_commands module)
            vision_commands::capture_screen,
            vision_commands::capture_screen_region,
            vision_commands::perform_ocr,
            vision_commands::detect_ui_elements,
            vision_commands::query_vision_ai,
            vision_commands::check_vision_dependencies,
            // Vision service direct commands
            analyze_screen_with_ai,
            capture_and_analyze_screen,
            // Enhanced vision service commands
            vision_commands::capture_screen_enhanced,
            vision_commands::capture_region_enhanced,
            vision_commands::perform_ocr_enhanced,
            vision_commands::analyze_screenshot,
            vision_commands::get_vision_stats,
            vision_commands::check_vision_service_status,
            // Terminal commands
            create_terminal,
            create_simple_terminal,
            write_to_terminal,
            resize_terminal,
            kill_terminal,
            close_terminal,
            get_terminal_info,
            list_terminals,
            get_terminal_count,
            // Git commands
            git_status,
            git_generate_commit,
            git_get_branch_name,
            git_is_repo,
            git_get_recent_commits,
            git_get_remote_url,
            git_get_ahead_behind,
            git_get_branch_info,
            git_get_all_branches,
            git_get_stash_list,
            git_get_commit_changes,
            git_get_repository_stats,
            // Advanced Git Integration commands
            git_generate_visual_graph,
            git_generate_time_travel,
            git_generate_statistics,
            git_generate_visualization,
            git_get_advanced_branch_info,
            git_time_travel_to_commit,
            git_create_branch_from_commit,
            // Contextual suggestions commands
            get_contextual_suggestions,
            get_current_context,
            learn_from_command,
            // Config commands
            get_config,
            update_config,
            get_temp_file_path,
            get_cache_file_path,
            // AppConfig utility commands
            get_ai_models_cache_dir,
            get_screenshots_temp_dir,
            create_temp_screenshot_path,
            // System utilities
            get_system_info,
            search_files,
            execute_safe_system_command,
            // Window controls
            minimize_window,
            toggle_maximize,
            close_window,
            // Template commands
            execute_template_command,
            import_templates,
            export_templates,
            // Web scraping commands
            start_web_scraping,
            get_scraping_progress,
            scrape_single_page,
            extract_links,
            generate_site_map,
            check_robots_txt,
            get_website_metadata,
            estimate_scraping,
            // Terminal broadcasting commands
            register_broadcast_session,
            unregister_broadcast_session,
            get_broadcast_sessions,
            get_broadcast_session,
            update_session_status,
            execute_on_session,
            broadcast_command,
            import_broadcast_sessions,
            export_broadcast_sessions,
            create_local_session,
            get_active_broadcasts,
            // AI service management
            restart_ai_service,
            ai_clear_completed_requests,
            // Optimized AI service commands - missing functions
            optimized_ai_chat,
            get_ai_service_stats,
            force_ai_cleanup,
            submit_ai_request,
            // Optimized AI service commands
            ai_submit_priority_request,
            ai_batch_process,
            ai_get_service_stats,
            ai_clear_completed,
            ai_analyze_critical_error,
            ai_chat_async,
            ai_submit_async_request,
            // Advanced AI Request Builder commands
            ai_create_simple_request,
            ai_create_custom_request,
            ai_increment_request_retry,
            ai_analyze_response,
            // Direct Vision Service commands
            vision_initialize_service,
            vision_capture_full_screen,
            vision_capture_region,
            vision_perform_ocr,
            vision_detect_ui_elements,
            vision_analyze_with_ai,
            vision_comprehensive_analysis,
            vision_check_dependencies,
            // HTTP Client Pool Management
            ai_create_optimized_service,
            ai_get_pool_stats,
            // Security Scanner commands
            security_scan_directory,
            security_scan_real_time,
            security_get_scan_results,
            security_set_scan_config,
            security_update_rules,
            security_get_vulnerabilities,
            security_remediate_vulnerability,
            // Command Flow Visualization commands
            command_flow_analyze,
            command_flow_create_graph,
            command_flow_get_dependencies,
            command_flow_visualize_execution,
            command_flow_track_execution,
            command_flow_get_execution_history,
            // Plugin System commands
            plugin_install,
            plugin_uninstall,
            plugin_list,
            plugin_enable,
            plugin_disable,
            plugin_get_marketplace,
            plugin_execute_command,
            plugin_get_info,
            plugin_update,
            // Collaboration commands
            collaboration_create_session,
            collaboration_join_session,
            collaboration_leave_session,
            collaboration_share_terminal,
            collaboration_get_sessions,
            collaboration_send_message,
            // Workflow Automation commands
            workflow_create,
            workflow_execute,
            workflow_list,
            workflow_delete,
            workflow_record_macro,
            workflow_stop_recording,
            workflow_get_execution_history,
            // Analytics commands
            analytics_get_performance,
            analytics_get_usage_stats,
            analytics_get_insights,
            analytics_track_command,
            analytics_get_command_patterns,
            analytics_get_optimization_suggestions,
            // Ecosystem Awareness commands
            ecosystem_get_comprehensive_context,
            ecosystem_learn_from_interaction,
            ecosystem_predict_intent,
            ecosystem_get_adaptive_suggestions,
            ecosystem_analyze_pattern,
            ecosystem_get_system_insights,
            ecosystem_update_learning_preferences,
            ecosystem_get_context_correlation,
            ecosystem_predict_user_intent,
            ecosystem_analyze_system_patterns,
            // Cloud Integration commands
            cloud_backup_config,
            cloud_sync_data,
            cloud_restore_backup,
            cloud_get_status,
            cloud_configure_provider,
            cloud_list_backups,
            cloud_get_providers,
            // LocalRecall RAG commands
            local_recall_health_check,
            local_recall_list_collections,
            local_recall_create_collection,
            local_recall_add_text,
            local_recall_search,
            local_recall_index_command,
            local_recall_index_conversation,
            local_recall_index_codebase,
            local_recall_get_stats,
            local_recall_ensure_running,
            local_recall_initialize,
            local_recall_add_external_source,
            local_recall_upload_file,
            local_recall_reset_collection,
            local_recall_auto_index_project,
            local_recall_get_default_collection,
            local_recall_set_default_collection,
            local_recall_get_context_for_prompt,
            // Enhanced AI commands with memory
            ai_chat_with_memory,
            // Ollama Configuration commands
            ollama_check_installation,
            ollama_check_external_models,
            ollama_configure_models_path,
            ollama_start_service,
            ollama_is_running,
            ollama_get_available_models,
            ollama_initialize_config,
            ollama_ensure_configured,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| {
            eprintln!("Failed to run Tauri application: {}", e);
            std::process::exit(1);
        })
        .expect("Failed to run Tauri application");
}
