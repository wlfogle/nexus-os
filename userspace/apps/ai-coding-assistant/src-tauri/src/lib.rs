pub mod optimized_lib;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;

#[derive(Serialize, Deserialize)]
struct AnalysisResponse {
    result: String,
    success: bool,
    model_used: Option<String>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[tauri::command]
async fn analyze_code(code: String, language: String, operation: String) -> Result<AnalysisResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;
    
    let ai_container_url = "http://192.168.122.172:11434/api/generate";
    
    let prompt = match operation.as_str() {
        "analyze" => format!("Analyze this {language} code and provide brief feedback: {code}"),
        "fix_bugs" => format!("Find and fix bugs in this {language} code: {code}"),
        "optimize" => format!("Optimize this {language} code for better performance: {code}"),
        _ => format!("Review this {language} code: {code}"),
    };
    
    let mut request_body = HashMap::new();
    // Use available codellama model
    request_body.insert("model".to_string(), serde_json::Value::String("codellama:7b".to_string()));
    request_body.insert("prompt".to_string(), serde_json::Value::String(prompt));
    request_body.insert("stream".to_string(), serde_json::Value::Bool(false));
    request_body.insert("options".to_string(), serde_json::json!({
        "temperature": 0.1,
        "top_p": 0.9,
        "max_tokens": 512
    }));
    
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
                            model_used: Some("codellama:7b".to_string()),
                        })
                    },
                    Err(e) => Err(format!("Failed to parse response: {e}")),
                }
            } else {
                Err(format!("AI container returned error: {}", response.status()))
            }
        },
        Err(e) => Err(format!("Failed to connect to AI container: {e}")),
    }
}

#[tauri::command]
async fn check_ai_connection() -> Result<bool, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Client build error: {e}"))?;
    
    let ai_container_url = "http://192.168.122.172:11434/api/tags";
    
    match client.get(ai_container_url).send().await {
        Ok(response) => {
            println!("AI connection check: Status {}", response.status());
            Ok(response.status().is_success())
        },
        Err(e) => {
            println!("AI connection error: {e}");
            Ok(false)
        },
    }
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
async fn execute_command(command: String) -> Result<String, String> {
    let safe_commands = [
        "ls", "pwd", "whoami", "date", "uptime", "free", "df", "ps",
        "systemctl", "journalctl", "lscpu", "lsblk", "lsusb", "lspci",
        "uname", "hostnamectl", "cat", "head", "tail", "grep", "find",
        "neofetch", "fastfetch", "screenfetch", "inxi", "hwinfo"
    ];
    
    let cmd_parts: Vec<&str> = command.split_whitespace().collect();
    if cmd_parts.is_empty() {
        return Err("Empty command".to_string());
    }
    
    let cmd_name = cmd_parts[0];
    if !safe_commands.contains(&cmd_name) {
        return Err(format!("Command '{cmd_name}' is not allowed for security reasons"));
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
        Err(e) => Err(format!("Failed to execute command: {e}")),
    }
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            analyze_code,
            check_ai_connection,
            read_file,
            write_file,
            execute_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
