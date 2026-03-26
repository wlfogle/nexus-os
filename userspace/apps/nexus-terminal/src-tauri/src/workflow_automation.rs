use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tokio::process::Command;
use std::process::Stdio;

// Missing types expected by main.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub step_type: WorkflowStepType,
    pub command: Option<String>,
    pub script: Option<String>,
    pub condition: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timeout_seconds: Option<u64>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStepType {
    Command,
    Script,
    Condition,
    Parallel,
    Sequential,
    FileOperation,
    ApiCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub execution_id: String,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<f64>,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub steps_completed: u32,
    pub total_steps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: WorkflowCategory,
    pub steps_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_executed: Option<DateTime<Utc>>,
    pub execution_count: u64,
    pub status: WorkflowStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Active,
    Inactive,
    Draft,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub triggered_by: TriggerType,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<f64>,
    pub steps_executed: u32,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub category: WorkflowCategory,
    pub nodes: Vec<WorkflowNode>,
    pub connections: Vec<WorkflowConnection>,
    pub variables: HashMap<String, WorkflowVariable>,
    pub triggers: Vec<WorkflowTrigger>,
    pub settings: WorkflowSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_executed: Option<DateTime<Utc>>,
    pub execution_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowCategory {
    Development,
    DevOps,
    Testing,
    Deployment,
    Monitoring,
    Security,
    DataProcessing,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub node_type: NodeType,
    pub name: String,
    pub description: String,
    pub position: NodePosition,
    pub config: NodeConfig,
    pub input_ports: Vec<Port>,
    pub output_ports: Vec<Port>,
    pub status: NodeStatus,
    pub execution_time: Option<f64>,
    pub retry_count: u32,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Command,
    Script,
    Condition,
    Loop,
    Parallel,
    Delay,
    FileOperation,
    GitOperation,
    ApiCall,
    Variable,
    Trigger,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub command: Option<String>,
    pub script: Option<String>,
    pub condition: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub environment: HashMap<String, String>,
    pub working_directory: Option<String>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: String,
    pub name: String,
    pub port_type: PortType,
    pub data_type: DataType,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortType {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    File,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Waiting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConnection {
    pub id: String,
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
    pub condition: Option<String>,
    pub transform: Option<DataTransform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTransform {
    pub transform_type: TransformType,
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformType {
    JsonPath,
    Regex,
    JavaScript,
    Template,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVariable {
    pub name: String,
    pub variable_type: VariableType,
    pub value: serde_json::Value,
    pub description: String,
    pub is_secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    Input,
    Output,
    Internal,
    Environment,
    Secret,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTrigger {
    pub id: String,
    pub trigger_type: TriggerType,
    pub config: TriggerConfig,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    Manual,
    Schedule,
    FileWatch,
    GitHook,
    WebHook,
    Command,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    pub schedule: Option<String>, // Cron expression
    pub file_patterns: Vec<String>,
    pub git_events: Vec<String>,
    pub webhook_path: Option<String>,
    pub command_pattern: Option<String>,
    pub event_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSettings {
    pub concurrent_execution: bool,
    pub max_concurrent_runs: u32,
    pub auto_retry: bool,
    pub notification_on_failure: bool,
    pub notification_on_success: bool,
    pub log_level: LogLevel,
    pub timeout_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: String,
    pub workflow_id: String,
    pub triggered_by: TriggerType,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub node_executions: HashMap<String, NodeExecution>,
    pub variables: HashMap<String, serde_json::Value>,
    pub logs: Vec<ExecutionLog>,
    pub metrics: ExecutionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecution {
    pub node_id: String,
    pub status: NodeStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLog {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub node_id: Option<String>,
    pub message: String,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub total_duration: Option<f64>,
    pub node_durations: HashMap<String, f64>,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f32>,
    pub network_calls: u32,
    pub file_operations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: MacroCategory,
    pub commands: Vec<MacroCommand>,
    pub shortcuts: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroCategory {
    Git,
    Development,
    System,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroCommand {
    pub command: String,
    pub delay_ms: Option<u64>,
    pub condition: Option<String>,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroRecording {
    pub id: String,
    pub name: String,
    pub started_at: DateTime<Utc>,
    pub commands: Vec<RecordedCommand>,
    pub is_recording: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedCommand {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub working_directory: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct WorkflowEngine {
    workflows: HashMap<String, Workflow>,
    executions: HashMap<String, WorkflowExecution>,
    macros: HashMap<String, Macro>,
    active_recordings: HashMap<String, MacroRecording>,
}

#[allow(dead_code)]
impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            executions: HashMap::new(),
            macros: HashMap::new(),
            active_recordings: HashMap::new(),
        }
    }

    pub fn create_workflow(&mut self, name: String, description: String, author: String) -> String {
        let workflow_id = uuid::Uuid::new_v4().to_string();
        
        let workflow = Workflow {
            id: workflow_id.clone(),
            name,
            description,
            version: "1.0.0".to_string(),
            author,
            category: WorkflowCategory::Custom,
            nodes: vec![],
            connections: vec![],
            variables: HashMap::new(),
            triggers: vec![],
            settings: WorkflowSettings {
                concurrent_execution: false,
                max_concurrent_runs: 1,
                auto_retry: false,
                notification_on_failure: true,
                notification_on_success: false,
                log_level: LogLevel::Info,
                timeout_minutes: 30,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_executed: None,
            execution_count: 0,
        };

        self.workflows.insert(workflow_id.clone(), workflow);
        workflow_id
    }

    pub fn add_node(&mut self, workflow_id: &str, node: WorkflowNode) -> Result<()> {
        if let Some(workflow) = self.workflows.get_mut(workflow_id) {
            workflow.nodes.push(node);
            workflow.updated_at = Utc::now();
            Ok(())
        } else {
            Err(anyhow!("Workflow not found: {}", workflow_id))
        }
    }

    pub fn add_connection(&mut self, workflow_id: &str, connection: WorkflowConnection) -> Result<()> {
        if let Some(workflow) = self.workflows.get_mut(workflow_id) {
            // Validate connection
            let from_node_exists = workflow.nodes.iter().any(|n| n.id == connection.from_node);
            let to_node_exists = workflow.nodes.iter().any(|n| n.id == connection.to_node);
            
            if !from_node_exists || !to_node_exists {
                return Err(anyhow!("Invalid connection: one or more nodes don't exist"));
            }

            workflow.connections.push(connection);
            workflow.updated_at = Utc::now();
            Ok(())
        } else {
            Err(anyhow!("Workflow not found: {}", workflow_id))
        }
    }

    pub async fn execute_workflow(&mut self, workflow_id: &str) -> Result<String> {
        let execution_id = uuid::Uuid::new_v4().to_string();
        
        // Get execution order first before borrowing mutably
        let execution_order = {
            if let Some(workflow) = self.workflows.get(workflow_id) {
                self.get_execution_order(workflow)?
            } else {
                return Err(anyhow!("Workflow not found: {}", workflow_id));
            }
        };
        
        let execution = WorkflowExecution {
            id: execution_id.clone(),
            workflow_id: workflow_id.to_string(),
            triggered_by: TriggerType::Manual,
            status: ExecutionStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            node_executions: HashMap::new(),
            variables: HashMap::new(),
            logs: vec![],
            metrics: ExecutionMetrics {
                total_duration: None,
                node_durations: HashMap::new(),
                memory_usage: None,
                cpu_usage: None,
                network_calls: 0,
                file_operations: 0,
            },
        };

        self.executions.insert(execution_id.clone(), execution);
        
        for node_id in execution_order {
            if let Err(e) = self.execute_node(&execution_id, &node_id).await {
                self.log_execution(&execution_id, LogLevel::Error, Some(&node_id), &format!("Node execution failed: {}", e));
                if let Some(exec) = self.executions.get_mut(&execution_id) {
                    exec.status = ExecutionStatus::Failed;
                    exec.completed_at = Some(Utc::now());
                }
                return Err(e);
            }
        }

        // Mark execution as completed
        if let Some(exec) = self.executions.get_mut(&execution_id) {
            exec.status = ExecutionStatus::Completed;
            exec.completed_at = Some(Utc::now());
            
            if let Some(started) = exec.started_at.signed_duration_since(Utc::now()).num_milliseconds().checked_abs() {
                exec.metrics.total_duration = Some(started as f64 / 1000.0);
            }
        }

        // Update workflow stats separately
        if let Some(workflow) = self.workflows.get_mut(workflow_id) {
            workflow.last_executed = Some(Utc::now());
            workflow.execution_count += 1;
        }

        Ok(execution_id)
    }

    fn get_execution_order(&self, workflow: &Workflow) -> Result<Vec<String>> {
        // Simple topological sort for workflow execution order
        let mut visited = std::collections::HashSet::new();
        let mut temp_visited = std::collections::HashSet::new();
        let mut order = Vec::new();

        for node in &workflow.nodes {
            if !visited.contains(&node.id) {
                self.dfs_visit(&node.id, workflow, &mut visited, &mut temp_visited, &mut order)?;
            }
        }

        order.reverse();
        Ok(order)
    }

    fn dfs_visit(
        &self,
        node_id: &str,
        workflow: &Workflow,
        visited: &mut std::collections::HashSet<String>,
        temp_visited: &mut std::collections::HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if temp_visited.contains(node_id) {
            return Err(anyhow!("Circular dependency detected"));
        }

        if visited.contains(node_id) {
            return Ok(());
        }

        temp_visited.insert(node_id.to_string());

        // Visit all dependencies first
        for connection in &workflow.connections {
            if connection.to_node == node_id {
                self.dfs_visit(&connection.from_node, workflow, visited, temp_visited, order)?;
            }
        }

        temp_visited.remove(node_id);
        visited.insert(node_id.to_string());
        order.push(node_id.to_string());

        Ok(())
    }

    async fn execute_node(&mut self, execution_id: &str, node_id: &str) -> Result<()> {
        if let Some(execution) = self.executions.get(execution_id) {
            if let Some(workflow) = self.workflows.get(&execution.workflow_id) {
                if let Some(node) = workflow.nodes.iter().find(|n| n.id == node_id) {
                    let start_time = Utc::now();
                    
                    // Create node execution record
                    let node_exec = NodeExecution {
                        node_id: node_id.to_string(),
                        status: NodeStatus::Running,
                        started_at: Some(start_time),
                        completed_at: None,
                        output: None,
                        error: None,
                        retry_count: 0,
                    };

                    if let Some(exec) = self.executions.get_mut(execution_id) {
                        exec.node_executions.insert(node_id.to_string(), node_exec);
                    }

                    // Execute based on node type
                    let result = match node.node_type {
                        NodeType::Command => self.execute_command_node(node).await,
                        NodeType::Script => self.execute_script_node(node).await,
                        NodeType::Condition => self.execute_condition_node(node, execution_id).await,
                        NodeType::FileOperation => self.execute_file_operation_node(node).await,
                        NodeType::Delay => self.execute_delay_node(node).await,
                        _ => Ok(serde_json::Value::Null),
                    };

                    let end_time = Utc::now();
                    let duration = end_time.signed_duration_since(start_time).num_milliseconds() as f64 / 1000.0;

                    // Update node execution record
                    if let Some(exec) = self.executions.get_mut(execution_id) {
                        if let Some(node_exec) = exec.node_executions.get_mut(node_id) {
                            node_exec.completed_at = Some(end_time);
                            match &result {
                                Ok(output) => {
                                    node_exec.status = NodeStatus::Completed;
                                    node_exec.output = Some(output.clone());
                                }
                                Err(error) => {
                                    node_exec.status = NodeStatus::Failed;
                                    node_exec.error = Some(error.to_string());
                                }
                            }
                        }
                        exec.metrics.node_durations.insert(node_id.to_string(), duration);
                    }

                    result.map(|_| ())
                } else {
                    Err(anyhow!("Node not found: {}", node_id))
                }
            } else {
                Err(anyhow!("Workflow not found"))
            }
        } else {
            Err(anyhow!("Execution not found"))
        }
    }

    async fn execute_command_node(&self, node: &WorkflowNode) -> Result<serde_json::Value> {
        if let Some(command) = &node.config.command {
            let mut cmd = if cfg!(target_os = "windows") {
                let mut c = Command::new("cmd");
                c.args(["/C", command]);
                c
            } else {
                let mut c = Command::new("sh");
                c.arg("-c").arg(command);
                c
            };

            // Set working directory
            if let Some(wd) = &node.config.working_directory {
                cmd.current_dir(wd);
            }

            // Set environment variables
            for (key, value) in &node.config.environment {
                cmd.env(key, value);
            }

            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

            let output = cmd.output().await?;
            
            Ok(serde_json::json!({
                "stdout": String::from_utf8_lossy(&output.stdout),
                "stderr": String::from_utf8_lossy(&output.stderr),
                "exit_code": output.status.code().unwrap_or(-1)
            }))
        } else {
            Err(anyhow!("No command specified for command node"))
        }
    }

    async fn execute_script_node(&self, node: &WorkflowNode) -> Result<serde_json::Value> {
        if let Some(script) = &node.config.script {
            // For simplicity, treat script as a shell command
            // In a real implementation, this could support multiple script languages
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(script);

            if let Some(wd) = &node.config.working_directory {
                cmd.current_dir(wd);
            }

            for (key, value) in &node.config.environment {
                cmd.env(key, value);
            }

            let output = cmd.output().await?;
            
            Ok(serde_json::json!({
                "output": String::from_utf8_lossy(&output.stdout),
                "error": String::from_utf8_lossy(&output.stderr),
                "success": output.status.success()
            }))
        } else {
            Err(anyhow!("No script specified for script node"))
        }
    }

    async fn execute_condition_node(&self, node: &WorkflowNode, execution_id: &str) -> Result<serde_json::Value> {
        if let Some(condition) = &node.config.condition {
            // Simple condition evaluation
            // In a real implementation, this would support complex expressions
            let result = self.evaluate_condition(condition, execution_id)?;
            Ok(serde_json::json!({ "result": result }))
        } else {
            Err(anyhow!("No condition specified for condition node"))
        }
    }

    async fn execute_file_operation_node(&self, node: &WorkflowNode) -> Result<serde_json::Value> {
        // File operations like copy, move, delete, etc.
        if let Some(operation) = node.config.parameters.get("operation") {
            match operation.as_str() {
                Some("copy") => {
                    let from = node.config.parameters.get("from").and_then(|v| v.as_str()).ok_or(anyhow!("Missing 'from' parameter"))?;
                    let to = node.config.parameters.get("to").and_then(|v| v.as_str()).ok_or(anyhow!("Missing 'to' parameter"))?;
                    
                    tokio::fs::copy(from, to).await?;
                    Ok(serde_json::json!({ "operation": "copy", "success": true }))
                }
                Some("delete") => {
                    let path = node.config.parameters.get("path").and_then(|v| v.as_str()).ok_or(anyhow!("Missing 'path' parameter"))?;
                    tokio::fs::remove_file(path).await?;
                    Ok(serde_json::json!({ "operation": "delete", "success": true }))
                }
                _ => Err(anyhow!("Unknown file operation")),
            }
        } else {
            Err(anyhow!("No operation specified for file operation node"))
        }
    }

    async fn execute_delay_node(&self, node: &WorkflowNode) -> Result<serde_json::Value> {
        if let Some(delay) = node.config.parameters.get("delay_ms").and_then(|v| v.as_u64()) {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            Ok(serde_json::json!({ "delayed_ms": delay }))
        } else {
            Err(anyhow!("No delay specified for delay node"))
        }
    }

    fn evaluate_condition(&self, condition: &str, _execution_id: &str) -> Result<bool> {
        // Simple condition evaluation
        // In a real implementation, this would support complex expressions
        match condition {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => {
                // For now, just check if it's a simple comparison
                if condition.contains("==") {
                    let parts: Vec<&str> = condition.split("==").collect();
                    if parts.len() == 2 {
                        Ok(parts[0].trim() == parts[1].trim())
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }

    pub fn start_macro_recording(&mut self, name: String) -> String {
        let recording_id = uuid::Uuid::new_v4().to_string();
        
        let recording = MacroRecording {
            id: recording_id.clone(),
            name,
            started_at: Utc::now(),
            commands: vec![],
            is_recording: true,
        };

        self.active_recordings.insert(recording_id.clone(), recording);
        recording_id
    }

    pub fn record_command(&mut self, recording_id: &str, command: String, working_directory: String, exit_code: Option<i32>, duration_ms: u64) -> Result<()> {
        if let Some(recording) = self.active_recordings.get_mut(recording_id) {
            if recording.is_recording {
                recording.commands.push(RecordedCommand {
                    command,
                    timestamp: Utc::now(),
                    working_directory,
                    exit_code,
                    duration_ms,
                });
                Ok(())
            } else {
                Err(anyhow!("Recording is not active"))
            }
        } else {
            Err(anyhow!("Recording not found: {}", recording_id))
        }
    }

    pub fn stop_macro_recording(&mut self, recording_id: &str) -> Result<String> {
        if let Some(mut recording) = self.active_recordings.remove(recording_id) {
            recording.is_recording = false;
            
            // Convert to macro
            let macro_id = uuid::Uuid::new_v4().to_string();
            let macro_commands = recording.commands.into_iter().map(|cmd| {
                MacroCommand {
                    command: cmd.command,
                    delay_ms: Some(100), // Default delay between commands
                    condition: None,
                    variables: HashMap::new(),
                }
            }).collect();

            let macro_obj = Macro {
                id: macro_id.clone(),
                name: recording.name,
                description: "Recorded macro".to_string(),
                category: MacroCategory::Custom,
                commands: macro_commands,
                shortcuts: vec![],
                created_at: Utc::now(),
                last_used: None,
                usage_count: 0,
            };

            self.macros.insert(macro_id.clone(), macro_obj);
            Ok(macro_id)
        } else {
            Err(anyhow!("Recording not found: {}", recording_id))
        }
    }

    pub async fn execute_macro(&mut self, macro_id: &str) -> Result<Vec<serde_json::Value>> {
        if let Some(macro_obj) = self.macros.get_mut(macro_id) {
            let mut results = Vec::new();
            
            for command in &macro_obj.commands {
                // Apply delay if specified
                if let Some(delay_ms) = command.delay_ms {
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                }

                // Execute command
                let mut cmd = if cfg!(target_os = "windows") {
                    let mut c = Command::new("cmd");
                    c.args(["/C", &command.command]);
                    c
                } else {
                    let mut c = Command::new("sh");
                    c.arg("-c").arg(&command.command);
                    c
                };

                // Set environment variables
                for (key, value) in &command.variables {
                    cmd.env(key, value);
                }

                let output = cmd.output().await?;
                results.push(serde_json::json!({
                    "command": command.command,
                    "output": String::from_utf8_lossy(&output.stdout),
                    "error": String::from_utf8_lossy(&output.stderr),
                    "exit_code": output.status.code().unwrap_or(-1)
                }));
            }

            macro_obj.last_used = Some(Utc::now());
            macro_obj.usage_count += 1;

            Ok(results)
        } else {
            Err(anyhow!("Macro not found: {}", macro_id))
        }
    }

    fn log_execution(&mut self, execution_id: &str, level: LogLevel, node_id: Option<&str>, message: &str) {
        if let Some(execution) = self.executions.get_mut(execution_id) {
            execution.logs.push(ExecutionLog {
                timestamp: Utc::now(),
                level,
                node_id: node_id.map(|s| s.to_string()),
                message: message.to_string(),
                context: HashMap::new(),
            });
        }
    }

    pub fn get_workflow(&self, workflow_id: &str) -> Option<&Workflow> {
        self.workflows.get(workflow_id)
    }

    pub fn get_execution(&self, execution_id: &str) -> Option<&WorkflowExecution> {
        self.executions.get(execution_id)
    }

    pub fn list_workflows(&self) -> Vec<&Workflow> {
        self.workflows.values().collect()
    }

    pub fn list_macros(&self) -> Vec<&Macro> {
        self.macros.values().collect()
    }

    pub fn get_macro(&self, macro_id: &str) -> Option<&Macro> {
        self.macros.get(macro_id)
    }

    pub fn export_workflow(&self, workflow_id: &str) -> Result<String> {
        if let Some(workflow) = self.workflows.get(workflow_id) {
            serde_json::to_string_pretty(workflow).map_err(Into::into)
        } else {
            Err(anyhow!("Workflow not found: {}", workflow_id))
        }
    }

    pub fn import_workflow(&mut self, workflow_json: &str) -> Result<String> {
        let workflow: Workflow = serde_json::from_str(workflow_json)?;
        let workflow_id = workflow.id.clone();
        self.workflows.insert(workflow_id.clone(), workflow);
        Ok(workflow_id)
    }

    pub fn delete_workflow(&mut self, workflow_id: &str) -> Result<()> {
        if self.workflows.remove(workflow_id).is_some() {
            Ok(())
        } else {
            Err(anyhow!("Workflow not found: {}", workflow_id))
        }
    }

    pub fn delete_macro(&mut self, macro_id: &str) -> Result<()> {
        if self.macros.remove(macro_id).is_some() {
            Ok(())
        } else {
            Err(anyhow!("Macro not found: {}", macro_id))
        }
    }

    // Methods expected by main.rs - Refactored version
    pub async fn create_workflow_with_steps(&mut self, name: &str, description: &str, steps: Vec<WorkflowStep>) -> Result<Workflow> {
        let workflow_id = uuid::Uuid::new_v4().to_string();
        
        // Convert WorkflowSteps to WorkflowNodes
        let mut nodes = Vec::new();
        for (i, step) in steps.iter().enumerate() {
            let node = WorkflowNode {
                id: step.id.clone(),
                node_type: match step.step_type {
                    WorkflowStepType::Command => NodeType::Command,
                    WorkflowStepType::Script => NodeType::Script,
                    WorkflowStepType::Condition => NodeType::Condition,
                    WorkflowStepType::Parallel => NodeType::Parallel,
                    WorkflowStepType::Sequential => NodeType::Command,
                    WorkflowStepType::FileOperation => NodeType::FileOperation,
                    WorkflowStepType::ApiCall => NodeType::ApiCall,
                },
                name: step.name.clone(),
                description: format!("Step {}: {}", i + 1, step.name),
                position: NodePosition { x: i as f64 * 100.0, y: 0.0 },
                config: NodeConfig {
                    command: step.command.clone(),
                    script: step.script.clone(),
                    condition: step.condition.clone(),
                    parameters: step.parameters.clone(),
                    environment: HashMap::new(),
                    working_directory: None,
                    timeout_seconds: step.timeout_seconds,
                },
                input_ports: vec![],
                output_ports: vec![],
                status: NodeStatus::Pending,
                execution_time: None,
                retry_count: 0,
                max_retries: step.retry_count,
            };
            nodes.push(node);
        }
        
        let workflow = Workflow {
            id: workflow_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            version: "1.0.0".to_string(),
            author: "User".to_string(),
            category: WorkflowCategory::Custom,
            nodes,
            connections: vec![],
            variables: HashMap::new(),
            triggers: vec![],
            settings: WorkflowSettings {
                concurrent_execution: false,
                max_concurrent_runs: 1,
                auto_retry: false,
                notification_on_failure: true,
                notification_on_success: false,
                log_level: LogLevel::Info,
                timeout_minutes: 30,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_executed: None,
            execution_count: 0,
        };

        self.workflows.insert(workflow_id.clone(), workflow.clone());
        Ok(workflow)
    }

    pub async fn execute_workflow_with_params(&self, workflow_id: &str, _parameters: &serde_json::Value) -> Result<ExecutionResult> {
        if let Some(workflow) = self.workflows.get(workflow_id) {
            let execution_id = uuid::Uuid::new_v4().to_string();
            let start_time = Utc::now();
            
            // Simple execution for demo purposes
            let mut steps_completed = 0;
            let total_steps = workflow.nodes.len() as u32;
            let mut output = serde_json::json!({});
            let mut error = None;
            let mut success = true;

            // Execute each node
            for node in &workflow.nodes {
                match self.execute_command_node(node).await {
                    Ok(node_output) => {
                        output[&node.id] = node_output;
                        steps_completed += 1;
                    }
                    Err(e) => {
                        error = Some(e.to_string());
                        success = false;
                        break;
                    }
                }
            }

            let end_time = Utc::now();
            let duration = end_time.signed_duration_since(start_time).num_milliseconds() as f64 / 1000.0;
            
            Ok(ExecutionResult {
                execution_id,
                workflow_id: workflow_id.to_string(),
                status: if success { ExecutionStatus::Completed } else { ExecutionStatus::Failed },
                started_at: start_time,
                completed_at: Some(end_time),
                duration_seconds: Some(duration),
                success,
                output,
                error,
                steps_completed,
                total_steps,
            })
        } else {
            Err(anyhow!("Workflow not found: {}", workflow_id))
        }
    }

    pub async fn list_workflow_infos(&self) -> Result<Vec<WorkflowInfo>> {
        let mut workflow_infos = Vec::new();
        
        for workflow in self.workflows.values() {
            workflow_infos.push(WorkflowInfo {
                id: workflow.id.clone(),
                name: workflow.name.clone(),
                description: workflow.description.clone(),
                category: workflow.category.clone(),
                steps_count: workflow.nodes.len(),
                created_at: workflow.created_at,
                updated_at: workflow.updated_at,
                last_executed: workflow.last_executed,
                execution_count: workflow.execution_count,
                status: WorkflowStatus::Active, // Default status
            });
        }
        
        Ok(workflow_infos)
    }

    pub async fn delete_workflow_by_id(&mut self, workflow_id: &str) -> Result<()> {
        if self.workflows.remove(workflow_id).is_some() {
            // Also remove any related executions
            self.executions.retain(|_, exec| exec.workflow_id != workflow_id);
            Ok(())
        } else {
            Err(anyhow!("Workflow not found: {}", workflow_id))
        }
    }

    pub async fn start_recording_macro(&mut self, name: &str) -> Result<String> {
        let recording_id = uuid::Uuid::new_v4().to_string();
        
        let recording = MacroRecording {
            id: recording_id.clone(),
            name: name.to_string(),
            started_at: Utc::now(),
            commands: vec![],
            is_recording: true,
        };

        self.active_recordings.insert(recording_id.clone(), recording);
        Ok(recording_id)
    }

    pub async fn stop_recording_macro(&mut self, recording_id: &str) -> Result<Workflow> {
        if let Some(mut recording) = self.active_recordings.remove(recording_id) {
            recording.is_recording = false;
            
            // Convert recorded commands to workflow
            let workflow_id = uuid::Uuid::new_v4().to_string();
            let mut nodes = Vec::new();
            
            for (i, cmd) in recording.commands.iter().enumerate() {
                let node = WorkflowNode {
                    id: format!("node_{}", i),
                    node_type: NodeType::Command,
                    name: format!("Command {}", i + 1),
                    description: cmd.command.clone(),
                    position: NodePosition { x: i as f64 * 100.0, y: 0.0 },
                    config: NodeConfig {
                        command: Some(cmd.command.clone()),
                        script: None,
                        condition: None,
                        parameters: HashMap::new(),
                        environment: HashMap::new(),
                        working_directory: Some(cmd.working_directory.clone()),
                        timeout_seconds: Some(30),
                    },
                    input_ports: vec![],
                    output_ports: vec![],
                    status: NodeStatus::Pending,
                    execution_time: Some(cmd.duration_ms as f64 / 1000.0),
                    retry_count: 0,
                    max_retries: 3,
                };
                nodes.push(node);
            }
            
            let workflow = Workflow {
                id: workflow_id.clone(),
                name: recording.name,
                description: "Workflow created from macro recording".to_string(),
                version: "1.0.0".to_string(),
                author: "User".to_string(),
                category: WorkflowCategory::Custom,
                nodes,
                connections: vec![],
                variables: HashMap::new(),
                triggers: vec![],
                settings: WorkflowSettings {
                    concurrent_execution: false,
                    max_concurrent_runs: 1,
                    auto_retry: false,
                    notification_on_failure: true,
                    notification_on_success: false,
                    log_level: LogLevel::Info,
                    timeout_minutes: 30,
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_executed: None,
                execution_count: 0,
            };

            self.workflows.insert(workflow_id.clone(), workflow.clone());
            Ok(workflow)
        } else {
            Err(anyhow!("Recording not found: {}", recording_id))
        }
    }

    pub async fn get_execution_history(&self, workflow_id: &str, limit: Option<u32>) -> Result<Vec<ExecutionRecord>> {
        let mut records = Vec::new();
        let limit = limit.unwrap_or(100) as usize;
        
        for execution in self.executions.values().filter(|e| e.workflow_id == workflow_id).take(limit) {
            let workflow_name = self.workflows.get(&execution.workflow_id)
                .map(|w| w.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
                
            let duration = execution.completed_at
                .map(|end| (end - execution.started_at).num_milliseconds() as f64 / 1000.0);
            
            let steps_executed = execution.node_executions.values()
                .filter(|ne| matches!(ne.status, NodeStatus::Completed))
                .count() as u32;
                
            let error_message = execution.node_executions.values()
                .find(|ne| matches!(ne.status, NodeStatus::Failed))
                .and_then(|ne| ne.error.clone());
            
            records.push(ExecutionRecord {
                id: execution.id.clone(),
                workflow_id: execution.workflow_id.clone(),
                workflow_name,
                triggered_by: execution.triggered_by.clone(),
                status: execution.status.clone(),
                started_at: execution.started_at,
                completed_at: execution.completed_at,
                duration_seconds: duration,
                steps_executed,
                error_message,
            });
        }
        
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_engine_creation() {
        let engine = WorkflowEngine::new();
        assert!(engine.workflows.is_empty());
        assert!(engine.macros.is_empty());
    }

    #[test]
    fn test_create_workflow() {
        let mut engine = WorkflowEngine::new();
        let workflow_id = engine.create_workflow(
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
            "Test Author".to_string(),
        );
        
        assert!(engine.workflows.contains_key(&workflow_id));
        let workflow = engine.get_workflow(&workflow_id).unwrap();
        assert_eq!(workflow.name, "Test Workflow");
    }

    #[test]
    fn test_macro_recording() {
        let mut engine = WorkflowEngine::new();
        let recording_id = engine.start_macro_recording("Test Macro".to_string());
        
        assert!(engine.active_recordings.contains_key(&recording_id));
        
        engine.record_command(
            &recording_id,
            "echo test".to_string(),
            "/tmp".to_string(),
            Some(0),
            100,
        ).unwrap();
        
        let macro_id = engine.stop_macro_recording(&recording_id).unwrap();
        assert!(engine.macros.contains_key(&macro_id));
        
        let macro_obj = engine.get_macro(&macro_id).unwrap();
        assert_eq!(macro_obj.name, "Test Macro");
        assert_eq!(macro_obj.commands.len(), 1);
    }

    #[tokio::test]
    async fn test_workflow_execution_order() {
        let mut engine = WorkflowEngine::new();
        let workflow_id = engine.create_workflow(
            "Test Workflow".to_string(),
            "A test workflow".to_string(),
            "Test Author".to_string(),
        );

        // Add nodes
        let node1 = WorkflowNode {
            id: "node1".to_string(),
            node_type: NodeType::Command,
            name: "First Node".to_string(),
            description: "First node".to_string(),
            position: NodePosition { x: 0.0, y: 0.0 },
            config: NodeConfig {
                command: Some("echo first".to_string()),
                script: None,
                condition: None,
                parameters: HashMap::new(),
                environment: HashMap::new(),
                working_directory: None,
                timeout_seconds: None,
            },
            input_ports: vec![],
            output_ports: vec![],
            status: NodeStatus::Pending,
            execution_time: None,
            retry_count: 0,
            max_retries: 3,
        };

        let node2 = WorkflowNode {
            id: "node2".to_string(),
            node_type: NodeType::Command,
            name: "Second Node".to_string(),
            description: "Second node".to_string(),
            position: NodePosition { x: 100.0, y: 0.0 },
            config: NodeConfig {
                command: Some("echo second".to_string()),
                script: None,
                condition: None,
                parameters: HashMap::new(),
                environment: HashMap::new(),
                working_directory: None,
                timeout_seconds: None,
            },
            input_ports: vec![],
            output_ports: vec![],
            status: NodeStatus::Pending,
            execution_time: None,
            retry_count: 0,
            max_retries: 3,
        };

        engine.add_node(&workflow_id, node1).unwrap();
        engine.add_node(&workflow_id, node2).unwrap();

        // Add connection
        let connection = WorkflowConnection {
            id: "conn1".to_string(),
            from_node: "node1".to_string(),
            from_port: "output".to_string(),
            to_node: "node2".to_string(),
            to_port: "input".to_string(),
            condition: None,
            transform: None,
        };

        engine.add_connection(&workflow_id, connection).unwrap();

        // Test execution order
        let workflow = engine.get_workflow(&workflow_id).unwrap();
        let order = engine.get_execution_order(workflow).unwrap();
        assert_eq!(order, vec!["node1", "node2"]);
    }
}
