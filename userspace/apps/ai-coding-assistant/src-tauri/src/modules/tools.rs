use crate::errors::AppError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tokio::fs;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub returns: String,
    pub category: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,       // Read-only operations
    Low,        // Safe write operations
    Medium,     // System operations with limited scope
    High,       // Operations that can modify system state
    Critical,   // Operations that can cause damage
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResult, AppError>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        
        // Register built-in tools
        registry.register_builtin_tools();
        registry
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.definition().name.clone();
        self.tools.insert(name, tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|tool| tool.definition()).collect()
    }

    pub async fn execute_tool(&self, name: &str, params: HashMap<String, serde_json::Value>) -> Result<ToolResult, AppError> {
        let tool = self.tools.get(name)
            .ok_or_else(|| AppError::NotFound(format!("Tool '{}' not found", name)))?;
        
        tool.execute(params).await
    }

    fn register_builtin_tools(&mut self) {
        // File operations
        self.register(Arc::new(ReadFileTool));
        self.register(Arc::new(WriteFileTool));
        self.register(Arc::new(ListDirectoryTool));
        self.register(Arc::new(CreateDirectoryTool));
        self.register(Arc::new(DeleteFileTool));
        self.register(Arc::new(SearchFilesTool));
        
        // Code execution
        self.register(Arc::new(ExecuteShellTool));
        self.register(Arc::new(ExecutePythonTool));
        self.register(Arc::new(ExecuteNodeTool));
        self.register(Arc::new(ExecuteRustTool));
        
        // System operations
        self.register(Arc::new(ProcessListTool));
        self.register(Arc::new(SystemInfoTool));
        self.register(Arc::new(NetworkInfoTool));
        
        // Git operations
        self.register(Arc::new(GitStatusTool));
        self.register(Arc::new(GitLogTool));
        self.register(Arc::new(GitDiffTool));
        
        // Analysis tools
        self.register(Arc::new(AnalyzeCodeTool));
        self.register(Arc::new(FindDuplicatesTool));
        self.register(Arc::new(CodeMetricsTool));
    }
}

// File Operations Tools

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    param_type: "string".to_string(),
                    description: "Path to the file to read".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            returns: "File contents as string".to_string(),
            category: "file_operations".to_string(),
            risk_level: RiskLevel::Safe,
        }
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResult, AppError> {
        let path = params.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Missing or invalid 'path' parameter".to_string()))?;

        match fs::read_to_string(path).await {
            Ok(content) => Ok(ToolResult {
                success: true,
                output: content,
                error: None,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("path".to_string(), serde_json::Value::String(path.to_string()));
                    meta
                },
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to read file: {}", e)),
                metadata: HashMap::new(),
            }),
        }
    }
}

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "write_file".to_string(),
            description: "Write content to a file".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    param_type: "string".to_string(),
                    description: "Path to the file to write".to_string(),
                    required: true,
                    default_value: None,
                },
                ToolParameter {
                    name: "content".to_string(),
                    param_type: "string".to_string(),
                    description: "Content to write to the file".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            returns: "Success message".to_string(),
            category: "file_operations".to_string(),
            risk_level: RiskLevel::Medium,
        }
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResult, AppError> {
        let path = params.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Missing or invalid 'path' parameter".to_string()))?;
        
        let content = params.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Missing or invalid 'content' parameter".to_string()))?;

        // Create parent directories if they don't exist
        if let Some(parent) = Path::new(path).parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to create parent directories: {}", e)),
                    metadata: HashMap::new(),
                });
            }
        }

        match fs::write(path, content).await {
            Ok(_) => Ok(ToolResult {
                success: true,
                output: format!("Successfully wrote {} bytes to {}", content.len(), path),
                error: None,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("path".to_string(), serde_json::Value::String(path.to_string()));
                    meta.insert("bytes_written".to_string(), serde_json::Value::Number(content.len().into()));
                    meta
                },
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to write file: {}", e)),
                metadata: HashMap::new(),
            }),
        }
    }
}

pub struct ListDirectoryTool;

#[async_trait]
impl Tool for ListDirectoryTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "list_directory".to_string(),
            description: "List contents of a directory".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    param_type: "string".to_string(),
                    description: "Path to the directory to list".to_string(),
                    required: true,
                    default_value: Some(serde_json::Value::String(".".to_string())),
                },
            ],
            returns: "List of files and directories".to_string(),
            category: "file_operations".to_string(),
            risk_level: RiskLevel::Safe,
        }
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResult, AppError> {
        let path = params.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        match fs::read_dir(path).await {
            Ok(mut entries) => {
                let mut files = Vec::new();
                while let Some(entry) = entries.next_entry().await.unwrap_or(None) {
                    if let Ok(metadata) = entry.metadata().await {
                        let file_info = serde_json::json!({
                            "name": entry.file_name().to_string_lossy(),
                            "path": entry.path().to_string_lossy(),
                            "is_dir": metadata.is_dir(),
                            "size": metadata.len(),
                            "modified": metadata.modified().ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs())
                        });
                        files.push(file_info);
                    }
                }

                Ok(ToolResult {
                    success: true,
                    output: serde_json::to_string_pretty(&files).unwrap_or_default(),
                    error: None,
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("path".to_string(), serde_json::Value::String(path.to_string()));
                        meta.insert("count".to_string(), serde_json::Value::Number(files.len().into()));
                        meta
                    },
                })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to read directory: {}", e)),
                metadata: HashMap::new(),
            }),
        }
    }
}

pub struct ExecuteShellTool;

#[async_trait]
impl Tool for ExecuteShellTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "execute_shell".to_string(),
            description: "Execute a shell command".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "command".to_string(),
                    param_type: "string".to_string(),
                    description: "Shell command to execute".to_string(),
                    required: true,
                    default_value: None,
                },
                ToolParameter {
                    name: "working_dir".to_string(),
                    param_type: "string".to_string(),
                    description: "Working directory for the command".to_string(),
                    required: false,
                    default_value: None,
                },
            ],
            returns: "Command output".to_string(),
            category: "execution".to_string(),
            risk_level: RiskLevel::High,
        }
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResult, AppError> {
        let command = params.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Missing or invalid 'command' parameter".to_string()))?;

        let working_dir = params.get("working_dir")
            .and_then(|v| v.as_str());

        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", command]);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", command]);
            cmd
        };

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        match cmd.output().await {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                Ok(ToolResult {
                    success: output.status.success(),
                    output: stdout.to_string(),
                    error: if !stderr.is_empty() { Some(stderr.to_string()) } else { None },
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("command".to_string(), serde_json::Value::String(command.to_string()));
                        meta.insert("exit_code".to_string(), serde_json::Value::Number(
                            output.status.code().unwrap_or(-1).into()
                        ));
                        if let Some(dir) = working_dir {
                            meta.insert("working_dir".to_string(), serde_json::Value::String(dir.to_string()));
                        }
                        meta
                    },
                })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to execute command: {}", e)),
                metadata: HashMap::new(),
            }),
        }
    }
}

pub struct ExecutePythonTool;

#[async_trait]
impl Tool for ExecutePythonTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "execute_python".to_string(),
            description: "Execute Python code".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "code".to_string(),
                    param_type: "string".to_string(),
                    description: "Python code to execute".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            returns: "Python execution output".to_string(),
            category: "execution".to_string(),
            risk_level: RiskLevel::High,
        }
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResult, AppError> {
        let code = params.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Validation("Missing or invalid 'code' parameter".to_string()))?;

        let mut cmd = Command::new("python3");
        cmd.arg("-c").arg(code);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        match cmd.output().await {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                Ok(ToolResult {
                    success: output.status.success(),
                    output: stdout.to_string(),
                    error: if !stderr.is_empty() { Some(stderr.to_string()) } else { None },
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("language".to_string(), serde_json::Value::String("python".to_string()));
                        meta.insert("exit_code".to_string(), serde_json::Value::Number(
                            output.status.code().unwrap_or(-1).into()
                        ));
                        meta
                    },
                })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to execute Python code: {}", e)),
                metadata: HashMap::new(),
            }),
        }
    }
}

// Placeholder implementations for other tools...

pub struct CreateDirectoryTool;
pub struct DeleteFileTool;
pub struct SearchFilesTool;
pub struct ExecuteNodeTool;
pub struct ExecuteRustTool;
pub struct ProcessListTool;
pub struct SystemInfoTool;
pub struct NetworkInfoTool;
pub struct GitStatusTool;
pub struct GitLogTool;
pub struct GitDiffTool;
pub struct AnalyzeCodeTool;
pub struct FindDuplicatesTool;
pub struct CodeMetricsTool;

// Implement basic stubs for the remaining tools to avoid compilation errors
macro_rules! impl_placeholder_tool {
    ($tool:ident, $name:expr, $desc:expr, $category:expr, $risk:expr) => {
        #[async_trait]
        impl Tool for $tool {
            fn definition(&self) -> ToolDefinition {
                ToolDefinition {
                    name: $name.to_string(),
                    description: $desc.to_string(),
                    parameters: vec![],
                    returns: "Not implemented".to_string(),
                    category: $category.to_string(),
                    risk_level: $risk,
                }
            }

            async fn execute(&self, _params: HashMap<String, serde_json::Value>) -> Result<ToolResult, AppError> {
                Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Tool not yet implemented".to_string()),
                    metadata: HashMap::new(),
                })
            }
        }
    };
}

impl_placeholder_tool!(CreateDirectoryTool, "create_directory", "Create a directory", "file_operations", RiskLevel::Medium);
impl_placeholder_tool!(DeleteFileTool, "delete_file", "Delete a file", "file_operations", RiskLevel::High);
impl_placeholder_tool!(SearchFilesTool, "search_files", "Search for files", "file_operations", RiskLevel::Safe);
impl_placeholder_tool!(ExecuteNodeTool, "execute_node", "Execute Node.js code", "execution", RiskLevel::High);
impl_placeholder_tool!(ExecuteRustTool, "execute_rust", "Execute Rust code", "execution", RiskLevel::High);
impl_placeholder_tool!(ProcessListTool, "process_list", "List running processes", "system", RiskLevel::Safe);
impl_placeholder_tool!(SystemInfoTool, "system_info", "Get system information", "system", RiskLevel::Safe);
impl_placeholder_tool!(NetworkInfoTool, "network_info", "Get network information", "system", RiskLevel::Safe);
impl_placeholder_tool!(GitStatusTool, "git_status", "Get git status", "git", RiskLevel::Safe);
impl_placeholder_tool!(GitLogTool, "git_log", "Get git log", "git", RiskLevel::Safe);
impl_placeholder_tool!(GitDiffTool, "git_diff", "Get git diff", "git", RiskLevel::Safe);
impl_placeholder_tool!(AnalyzeCodeTool, "analyze_code", "Analyze code quality", "analysis", RiskLevel::Safe);
impl_placeholder_tool!(FindDuplicatesTool, "find_duplicates", "Find duplicate code", "analysis", RiskLevel::Safe);
impl_placeholder_tool!(CodeMetricsTool, "code_metrics", "Calculate code metrics", "analysis", RiskLevel::Safe);
