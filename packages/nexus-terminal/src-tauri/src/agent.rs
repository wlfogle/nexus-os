/// NexusTerminal Agent — autonomous AI with native Ollama function-calling.
/// Uses Ollama's OpenAI-compatible tool_calls API — no text parsing, no infinite loops.
use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, HashMap};
use std::time::Duration;
use tauri::Emitter;
use tracing::{debug, info, warn};

use crate::model_router::{select_model, TaskKind};

// ── ask_user: global map of pending question answers ─────────────────────────────
// When the agent calls ask_user, it stores a oneshot sender here keyed by
// session_id. The answer_agent_question Tauri command resolves it when the
// user clicks a button in the UI.
static PENDING_ANSWERS: std::sync::OnceLock<
    std::sync::Mutex<HashMap<String, tokio::sync::oneshot::Sender<String>>>
> = std::sync::OnceLock::new();

pub fn pending_answers()
    -> &'static std::sync::Mutex<HashMap<String, tokio::sync::oneshot::Sender<String>>>
{
    PENDING_ANSWERS.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// Deliver the user's answer to a waiting ask_user call.
/// Called by the answer_agent_question Tauri command.
pub fn deliver_answer(session_id: &str, answer: String) -> bool {
    let mut map = pending_answers().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(tx) = map.remove(session_id) {
        let _ = tx.send(answer);
        true
    } else {
        false
    }
}

const MAX_STEPS: usize = 50;  // More steps = more thorough fixes

const SYSTEM_PROMPT: &str = r#"You are NexusAI — the autonomous agent of NexusOS. You act immediately. No preamble, no explanations, no instructions to the user. Just run tools and fix things.

## CRITICAL: Act, don't explain
- WRONG: "Here's how you could fix this: 1. Run grep..."
- RIGHT: [call grep tool immediately]
- NEVER write steps for the user to follow. YOU execute the steps.
- After every tool result: ask yourself "Is the task 100% done?" If no, call another tool.
- NEVER produce a text response while there are still actions to take.
- When the user says 'scan for errors' (without 'fix'): scan, then call ask_user with the findings
  and options ['Fix all errors', 'Report only']. Wait for the user's choice before proceeding.
- When the user says 'fix errors' or 'scan and fix': scan then fix directly without asking.
- Only call ask_user when you need a decision; never ask for trivial or read-only operations.

## Tool selection — MOST IMPORTANT RULE
Choose the correct tool based on what the user is asking about:

### Code / file tasks (scan a directory, fix errors, check a project)
Use: run_cmd, read_file, edit_file, grep, file_tree, list_dir
- "scan /path for errors" → detect project type in that path, run the right checker:
  - Cargo.toml present → `cargo check --message-format=short 2>&1`
  - package.json present → `npx tsc --noEmit 2>&1`
  - Generic → `grep -r 'error\|ERROR\|FIXME\|TODO' /path --include='*.rs' --include='*.ts' -l`
- NEVER call hardware_info for code/file questions.

### System hardware / OS diagnostics (CPU, RAM, disk, services, processes)
Use: hardware_info, process_list, list_services, systemctl_cmd
- "scan system" / "system status" / "optimize memory" / "check performance" → hardware_info
- NEVER call hardware_info for code scanning tasks.

## Code fix workflow
1. list_dir or file_tree the target path to understand the project
2. Detect project type (Cargo.toml → Rust, package.json → Node/TS)
3. run_cmd: `cargo check 2>&1` or `npx tsc --noEmit 2>&1` to get compiler errors
4. For each error: read_file the failing file, edit_file minimal fix
5. run_cmd to verify fix. Iterate until all pass, then git_commit
6. For deep AI code review: analyze_code(path, type) — types: errors|style|security|performance|cleanup|all
7. For AI-powered autofix: autofix_code(path, dry_run=true) first to preview, then dry_run=false to apply
8. Report: 1 paragraph, what was broken, what you changed

## Rules
- Always use the injected "Current working directory" for paths
- Never use /home/user or guessed paths
- edit_file for existing files, write_file for new files only
- Never call the same tool twice with identical arguments
- No stubs. No TODOs. Complete working code only.

## Tools
read_file, read_files, write_file, edit_file, run_cmd, list_dir, file_tree,
grep, create_dir, search_codebase, git_status, git_diff, git_log, git_commit,
http_get, http_post, ssh_exec, systemctl_cmd, process_list, docker_cmd,
list_services, hardware_info, proxmox_list,
analyze_code(path, analysis_type) — AI code review: errors|style|security|performance|cleanup|all,
autofix_code(path, dry_run) — AI rewrites file with fixes; always dry_run=true first,
ask_user(question, options) — show clickable buttons to the user and WAIT for their answer,
screenshot — capture the screen and see it with llama3.2-vision:11b"#;

// ── Ollama native function-calling types ──────────────────────────────────────

/// Tool definition sent to Ollama in OpenAI function-calling format.
#[derive(Debug, Clone, Serialize)]
struct Tool {
    #[serde(rename = "type")]
    kind: &'static str,
    function: ToolFunction,
}

#[derive(Debug, Clone, Serialize)]
struct ToolFunction {
    name: &'static str,
    description: &'static str,
    parameters: serde_json::Value,
}

/// Chat message with optional native tool_calls support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tool_calls: Option<Vec<ToolCallResponse>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: serde_json::Value,
}

// ── Agent response types (public API) ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub kind: String, // "tool_call" | "tool_result" | "answer"
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub answer: String,
    pub steps: Vec<AgentStep>,
}

// ── Ollama request/response ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
    tools: &'a [Tool],
    options: ChatOptions,
}

#[derive(Debug, Serialize)]
struct ChatOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Debug, Deserialize)]
struct ChatResponseBody {
    message: ChatMessage,
}

// ── Tauri event types ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct AgentTokenEvent {
    pub session_id: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentToolEvent {
    pub session_id: String,
    pub tool: String,
    pub args: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentToolResultEvent {
    pub session_id: String,
    pub tool: String,
    pub result: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentDoneEvent {
    pub session_id: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentErrorEvent {
    pub session_id: String,
    pub error: String,
}

/// Emitted when the agent calls ask_user — frontend renders clickable buttons.
/// `data` carries extra context for the frontend (e.g. scan_path for fix_engine).
#[derive(Debug, Clone, Serialize)]
pub struct AgentQuestionEvent {
    pub session_id: String,
    pub question: String,
    pub options: Vec<String>,
    /// Optional JSON context for the frontend (e.g. {"kind":"scan_and_fix","scan_path":"/…"})
    pub data: Option<serde_json::Value>,
}

// ── Streaming chunk types ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct StreamChunk {
    message: Option<StreamMessage>,
    done: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct StreamMessage {
    #[serde(default)]
    content: String,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCallResponse>>,
}

// ── Tool definitions (OpenAI function-calling format) ────────────────────────

fn build_tools() -> Vec<Tool> {
    vec![
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "read_file",
                description: "Read a single file from disk. Truncates at 32KB.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Absolute path to the file"}
                    },
                    "required": ["path"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "read_files",
                description: "Read multiple files at once. Returns content with === headers.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "paths": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Array of absolute file paths to read"
                        }
                    },
                    "required": ["paths"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "write_file",
                description: "Write content to a file. Creates parent directories if needed.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Absolute path to write to"},
                        "content": {"type": "string", "description": "Complete file content to write"}
                    },
                    "required": ["path", "content"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "run_cmd",
                description: "Run any shell command. Required 'cmd' field is the shell string. Optional 'cwd' is the working directory. \
USE THIS for code checks — exact correct usage: \
cargo check: {\"cmd\": \"cargo check --message-format=short 2>&1\", \"cwd\": \"/path/to/src-tauri\"} \
TypeScript: {\"cmd\": \"npx tsc --noEmit 2>&1\", \"cwd\": \"/path/to/project\"} \
cargo check does NOT take a path argument — use cwd instead. \
NEVER call this with empty cmd.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "cmd": {"type": "string", "description": "Shell command string (e.g. 'cargo check --message-format=short 2>&1'). NEVER empty."},
                        "cwd": {"type": "string", "description": "Working directory — use this to specify which project to check (e.g. the directory containing Cargo.toml or package.json)"}
                    },
                    "required": ["cmd"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "list_dir",
                description: "List directory contents. Directories have / suffix.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Directory path to list"}
                    },
                    "required": ["path"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "file_tree",
                description: "Show directory tree view. Skips node_modules, target, .git.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Root directory for tree"},
                        "depth": {"type": "integer", "description": "Maximum depth to traverse (default 3)"}
                    },
                    "required": ["path"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "grep",
                description: "Search for a pattern in files. Returns matching lines with numbers.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": {"type": "string", "description": "Search pattern (regex)"},
                        "path": {"type": "string", "description": "File or directory to search"},
                        "recursive": {"type": "boolean", "description": "Search recursively in directories"}
                    },
                    "required": ["pattern", "path"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "create_dir",
                description: "Create a directory and all parent directories (mkdir -p).",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Directory path to create"}
                    },
                    "required": ["path"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "git_status",
                description: "Show git status (short format with branch info).",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Repository path"}
                    },
                    "required": ["path"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "git_diff",
                description: "Show git diff summary (stat format).",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Repository path"}
                    },
                    "required": ["path"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "search_codebase",
                description: "Search codebase for a query using recursive case-insensitive grep.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search query"},
                        "path": {"type": "string", "description": "Root directory to search"}
                    },
                    "required": ["query", "path"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "http_get",
                description: "Fetch a URL and return the response body. For querying APIs (Proxmox, Jellyfin, etc).",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {"type": "string", "description": "URL to fetch"}
                    },
                    "required": ["url"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "list_services",
                description: "List running services. Tries systemctl, falls back to docker ps.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host": {"type": "string", "description": "Host to query (default: localhost)"}
                    },
                    "required": []
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "hardware_info",
                description: "HARDWARE AND OS diagnostics ONLY: CPU load, memory usage, disk space, running OS processes, system services, network interfaces. Use ONLY when the user asks about system performance, memory, CPU, disk, or network — NOT for scanning source code or project directories for programming errors.",
                parameters: serde_json::json!({ "type": "object", "properties": {}, "required": [] }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "proxmox_list",
                description: "List all VMs and LXC containers on the Proxmox host, plus storage pool status.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host": {"type": "string", "description": "SSH host alias for the Proxmox server (default: tiamat)"}
                    },
                    "required": []
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "edit_file",
                description: "Edit a file with exact search/replace. Replaces the FIRST occurrence of `search` with `replace`. Use this for targeted changes to existing files — never rewrite whole files.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path":    {"type": "string", "description": "Absolute path to the file"},
                        "search":  {"type": "string", "description": "Exact string to find (must be unique in the file)"},
                        "replace": {"type": "string", "description": "String to replace it with"}
                    },
                    "required": ["path", "search", "replace"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "git_log",
                description: "Show recent git commits (oneline format).",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path":  {"type": "string", "description": "Repository path"},
                        "count": {"type": "integer", "description": "Number of commits to show (default 10)"}
                    },
                    "required": ["path"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "git_commit",
                description: "Stage all changes and create a git commit.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path":    {"type": "string", "description": "Repository path"},
                        "message": {"type": "string", "description": "Commit message"}
                    },
                    "required": ["path", "message"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "ssh_exec",
                description: "Run a shell command on a remote host over SSH.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "host": {"type": "string", "description": "SSH host alias or user@host"},
                        "cmd":  {"type": "string", "description": "Command to run on the remote host"}
                    },
                    "required": ["host", "cmd"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "http_post",
                description: "Send an HTTP POST request with a JSON body.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url":  {"type": "string", "description": "URL to POST to"},
                        "body": {"type": "string", "description": "JSON body as a string"}
                    },
                    "required": ["url", "body"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "systemctl_cmd",
                description: "Run a systemctl command (start, stop, restart, status, enable, disable).",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action":  {"type": "string", "description": "Action: start|stop|restart|status|enable|disable"},
                        "service": {"type": "string", "description": "Service name (e.g. 'nginx', 'docker')"}
                    },
                    "required": ["action", "service"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "process_list",
                description: "List running processes, optionally filtered by name.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "filter": {"type": "string", "description": "Optional process name filter"}
                    },
                    "required": []
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "docker_cmd",
                description: "Run a docker command (ps, logs, restart, exec, etc).",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "cmd": {"type": "string", "description": "Docker subcommand and args"}
                    },
                    "required": ["cmd"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "web_search",
                description: "Search the web for current information. Uses DuckDuckGo or a local SearXNG instance. Use this to answer questions about current events, docs, or anything not in the codebase.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search query"},
                        "num_results": {"type": "integer", "description": "Number of results to return (default 5)"}
                    },
                    "required": ["query"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "mcp_call",
                description: "Call any MCP (Model Context Protocol) server tool. Use this to interact with Nextcloud, Home Assistant, Obsidian, TrueNAS, n8n, or any other MCP-compatible service.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "server": {"type": "string", "description": "MCP server name or URL (e.g. 'home-assistant', 'nextcloud', 'obsidian')"},
                        "tool": {"type": "string", "description": "Tool name to call on the MCP server"},
                        "args": {"type": "object", "description": "Arguments to pass to the tool"}
                    },
                    "required": ["server", "tool"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "analyze_code",
                description: "Analyze a SINGLE source code FILE (not a directory) using AI for a specific issue type. Use this for deep AI-powered code review of one file. For scanning a directory or project for errors use run_cmd with cargo check or npx tsc instead.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Absolute path to the source file to analyze"},
                        "analysis_type": {
                            "type": "string",
                            "description": "What to look for: 'errors' (bugs/logic), 'style' (formatting/best practices), 'security' (vulnerabilities), 'performance' (bottlenecks), 'cleanup' (dead/stub/zombie code), 'all' (comprehensive)",
                            "enum": ["errors", "style", "security", "performance", "cleanup", "all"]
                        }
                    },
                    "required": ["path", "analysis_type"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "autofix_code",
                description: "Have AI automatically fix all issues in a source file. Creates a .bak backup then rewrites the file with fixes applied. Use dry_run=true first to preview changes.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Absolute path to the source file to fix"},
                        "dry_run": {"type": "boolean", "description": "If true, show what would change without writing the file (default false)"}
                    },
                    "required": ["path"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "ask_user",
                description: "Ask the user a question and wait for their answer before proceeding. \
Use this when user confirmation is needed (e.g. after finding errors: ask 'Fix all 3 errors?'). \
The agent loop PAUSES until the user clicks a button in the UI. \
Always call this BEFORE making destructive changes or when the user said 'scan' but not 'fix'.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "question": {"type": "string", "description": "The question to present to the user"},
                        "options": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Clickable options (e.g. ['Fix all errors', 'Report only', 'Cancel'])"
                        }
                    },
                    "required": ["question", "options"]
                }),
            },
        },
        Tool {
            kind: "function",
            function: ToolFunction {
                name: "screenshot",
                description: "Capture the screen and analyze it with llama3.2-vision:11b. Use this to see what is on the screen, diagnose visual errors, or understand the current UI state.",
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "prompt": {"type": "string", "description": "What to look for or ask about the screen (e.g. 'What errors are visible?', 'Describe the current terminal output')"}
                    },
                    "required": ["prompt"]
                }),
            },
        },
    ]
}

// ── Argument normalization ───────────────────────────────────────────────────

/// Ollama may return arguments as a JSON string or a JSON object.
/// This normalizes to always be a JSON object.
fn normalize_args(args: &serde_json::Value) -> serde_json::Value {
    match args {
        serde_json::Value::String(s) => {
            serde_json::from_str(s).unwrap_or_else(|_| serde_json::json!({}))
        }
        other => other.clone(),
    }
}

// ── Tool execution ───────────────────────────────────────────────────────────

async fn exec_tool(name: &str, args: &serde_json::Value, default_cwd: &str, session_id: &str) -> String {
    match name {
        "read_file" => {
            let path = args["path"].as_str().unwrap_or("");
            match tokio::fs::read_to_string(path).await {
                Ok(content) => {
                    if content.len() > 32_000 {
                        format!("{}\n\n[... truncated at 32KB ...]", &content[..32_000])
                    } else {
                        content
                    }
                }
                Err(e) => format!("ERROR: {}", e),
            }
        }

        "read_files" => {
            let paths = args["paths"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();
            let mut result = String::new();
            for p in paths {
                result.push_str(&format!("=== {} ===\n", p));
                match tokio::fs::read_to_string(p).await {
                    Ok(content) => {
                        if content.len() > 32_000 {
                            result.push_str(&content[..32_000]);
                            result.push_str("\n[... truncated at 32KB ...]\n");
                        } else {
                            result.push_str(&content);
                            result.push('\n');
                        }
                    }
                    Err(e) => result.push_str(&format!("ERROR: {}\n", e)),
                }
            }
            result
        }

        "write_file" => {
            let path = args["path"].as_str().unwrap_or("");
            let content = args["content"].as_str().unwrap_or("");
            if let Some(parent) = std::path::Path::new(path).parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
            match tokio::fs::write(path, content).await {
                Ok(_) => format!("OK: wrote {} bytes to {}", content.len(), path),
                Err(e) => format!("ERROR: {}", e),
            }
        }

        "run_cmd" => {
            let cmd = args["cmd"].as_str().unwrap_or("");
            let cwd = args["cwd"].as_str().unwrap_or(default_cwd);
            if cmd.is_empty() {
                return "ERROR: empty command".to_string();
            }
            match tokio::time::timeout(
                Duration::from_secs(30),
                tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .current_dir(cwd)
                    .output(),
            )
            .await
            {
                Ok(Ok(out)) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let exit = out.status.code().unwrap_or(-1);
                    let result = if stderr.is_empty() {
                        stdout.to_string()
                    } else if stdout.is_empty() {
                        stderr.to_string()
                    } else {
                        format!("{}\nSTDERR: {}", stdout, stderr)
                    };
                    let result = if result.len() > 8000 {
                        format!("{}\n[... truncated ...]", &result[..8000])
                    } else {
                        result
                    };
                    if exit != 0 {
                        format!("EXIT {}\n{}", exit, result)
                    } else {
                        result
                    }
                }
                Ok(Err(e)) => format!("ERROR: {}", e),
                Err(_) => "ERROR: command timed out after 30s".to_string(),
            }
        }

        "list_dir" => {
            let path = args["path"].as_str().unwrap_or(default_cwd);
            match tokio::fs::read_dir(path).await {
                Ok(mut dir) => {
                    let mut entries = Vec::new();
                    while let Ok(Some(entry)) = dir.next_entry().await {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let is_dir = entry
                            .file_type()
                            .await
                            .map(|t| t.is_dir())
                            .unwrap_or(false);
                        entries.push(if is_dir {
                            format!("{}/", name)
                        } else {
                            name
                        });
                    }
                    entries.sort();
                    entries.join("\n")
                }
                Err(e) => format!("ERROR: {}", e),
            }
        }

        "file_tree" => {
            let path = args["path"].as_str().unwrap_or(default_cwd);
            let max_depth = args["depth"].as_u64().unwrap_or(3) as usize;
            let skip: &[&str] = &[
                "node_modules",
                "target",
                ".git",
                "__pycache__",
                ".cache",
            ];
            let mut result = format!("{}/\n", path);
            for entry in walkdir::WalkDir::new(path)
                .max_depth(max_depth)
                .sort_by(|a, b| a.file_name().cmp(b.file_name()))
                .into_iter()
                .filter_entry(|e| {
                    let name = e.file_name().to_string_lossy();
                    !skip.iter().any(|s| *s == name.as_ref())
                })
            {
                if let Ok(entry) = entry {
                    if entry.depth() == 0 {
                        continue;
                    }
                    let indent = "  ".repeat(entry.depth());
                    let name = entry.file_name().to_string_lossy();
                    let suffix = if entry.file_type().is_dir() { "/" } else { "" };
                    result.push_str(&format!("{}{}{}\n", indent, name, suffix));
                }
            }
            if result.len() > 16_000 {
                format!("{}\n[... truncated at 16KB ...]", &result[..16_000])
            } else {
                result
            }
        }

        "grep" => {
            let pattern = args["pattern"].as_str().unwrap_or("");
            let path = args["path"].as_str().unwrap_or(default_cwd);
            let recursive = args["recursive"].as_bool().unwrap_or(true);
            let cmd = if recursive {
                format!("grep -rn '{}' '{}'", pattern, path)
            } else {
                format!("grep -n '{}' '{}'", pattern, path)
            };
            match tokio::time::timeout(
                Duration::from_secs(15),
                tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output(),
            )
            .await
            {
                Ok(Ok(out)) => {
                    let result = String::from_utf8_lossy(&out.stdout).to_string();
                    if result.is_empty() {
                        "no matches".to_string()
                    } else if result.len() > 8000 {
                        format!("{}\n[... truncated ...]", &result[..8000])
                    } else {
                        result
                    }
                }
                Ok(Err(e)) => format!("ERROR: {}", e),
                Err(_) => "ERROR: grep timed out".to_string(),
            }
        }

        "create_dir" => {
            let path = args["path"].as_str().unwrap_or("");
            match tokio::fs::create_dir_all(path).await {
                Ok(_) => format!("OK: created {}", path),
                Err(e) => format!("ERROR: {}", e),
            }
        }

        "git_status" => {
            let path = args["path"].as_str().unwrap_or(default_cwd);
            match tokio::process::Command::new("git")
                .args(["-C", path, "status", "--short", "--branch"])
                .output()
                .await
            {
                Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
                Err(e) => format!("ERROR: {}", e),
            }
        }

        "git_diff" => {
            let path = args["path"].as_str().unwrap_or(default_cwd);
            match tokio::process::Command::new("git")
                .args(["-C", path, "--no-pager", "diff", "--stat"])
                .output()
                .await
            {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    if stdout.is_empty() && !stderr.is_empty() {
                        format!("ERROR: {}", stderr)
                    } else if stdout.is_empty() {
                        "No changes".to_string()
                    } else {
                        stdout.to_string()
                    }
                }
                Err(e) => format!("ERROR: {}", e),
            }
        }

        "search_codebase" => {
            let query = args["query"].as_str().unwrap_or("");
            let path = args["path"].as_str().unwrap_or(default_cwd);
            let cmd = format!(
                "grep -ri '{}' '{}' --include='*.rs' --include='*.ts' \
                 --include='*.js' --include='*.py' --include='*.toml' \
                 --include='*.json' --include='*.md' -l",
                query, path
            );
            match tokio::time::timeout(
                Duration::from_secs(15),
                tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output(),
            )
            .await
            {
                Ok(Ok(out)) => {
                    let result = String::from_utf8_lossy(&out.stdout).to_string();
                    if result.is_empty() {
                        "no matches".to_string()
                    } else if result.len() > 8000 {
                        format!("{}\n[... truncated ...]", &result[..8000])
                    } else {
                        result
                    }
                }
                Ok(Err(e)) => format!("ERROR: {}", e),
                Err(_) => "ERROR: search timed out".to_string(),
            }
        }

        "http_get" => {
            let url = args["url"].as_str().unwrap_or("");
            if url.is_empty() {
                return "ERROR: empty URL".to_string();
            }
            let client = match Client::builder()
                .timeout(Duration::from_secs(15))
                .danger_accept_invalid_certs(true)
                .build()
            {
                Ok(c) => c,
                Err(e) => return format!("ERROR: {}", e),
            };
            match client.get(url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    match resp.text().await {
                        Ok(body) => {
                            let body = if body.len() > 16_000 {
                                format!("{}\n[... truncated at 16KB ...]", &body[..16_000])
                            } else {
                                body
                            };
                            if status.is_success() {
                                body
                            } else {
                                format!("HTTP {}\n{}", status, body)
                            }
                        }
                        Err(e) => format!("HTTP {} — body read error: {}", status, e),
                    }
                }
                Err(e) => format!("ERROR: {}", e),
            }
        }

        "list_services" => {
            // Try systemctl first, fall back to docker ps
            let systemctl = tokio::process::Command::new("systemctl")
                .args(["list-units", "--type=service", "--state=running", "--no-pager"])
                .output()
                .await;
            match systemctl {
                Ok(out) if out.status.success() => {
                    let result = String::from_utf8_lossy(&out.stdout).to_string();
                    if result.len() > 8000 {
                        format!("{}\n[... truncated ...]", &result[..8000])
                    } else {
                        result
                    }
                }
                _ => {
                    // Fall back to docker ps
                    match tokio::process::Command::new("docker")
                        .args(["ps", "--format", "table {{.Names}}\t{{.Status}}\t{{.Ports}}"])
                        .output()
                        .await
                    {
                        Ok(out) => {
                            let result = String::from_utf8_lossy(&out.stdout).to_string();
                            if result.is_empty() {
                                "No running services found (systemctl and docker both failed or empty)".to_string()
                            } else if result.len() > 8000 {
                                format!("{}\n[... truncated ...]", &result[..8000])
                            } else {
                                result
                            }
                        }
                        Err(e) => format!("ERROR: neither systemctl nor docker available: {}", e),
                    }
                }
            }
        }

        "docker_cmd" => {
            let cmd = args["cmd"].as_str().unwrap_or("");
            if cmd.is_empty() {
                return "ERROR: empty docker command".to_string();
            }
            let full_cmd = format!("docker {}", cmd);
            match tokio::time::timeout(
                Duration::from_secs(30),
                tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(&full_cmd)
                    .output(),
            )
            .await
            {
                Ok(Ok(out)) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let exit = out.status.code().unwrap_or(-1);
                    let result = if stderr.is_empty() {
                        stdout.to_string()
                    } else if stdout.is_empty() {
                        stderr.to_string()
                    } else {
                        format!("{}\nSTDERR: {}", stdout, stderr)
                    };
                    let result = if result.len() > 8000 {
                        format!("{}\n[... truncated ...]", &result[..8000])
                    } else {
                        result
                    };
                    if exit != 0 {
                        format!("EXIT {}\n{}", exit, result)
                    } else {
                        result
                    }
                }
                Ok(Err(e)) => format!("ERROR: {}", e),
                Err(_) => "ERROR: docker command timed out after 30s".to_string(),
            }
        }

        "hardware_info" => {
            // CPU, memory, disk, top USER processes (no kernel threads), services, network
            let cmd = [
                "echo '=== CPU ===' && grep -c ^processor /proc/cpuinfo && cat /proc/loadavg",
                "echo '=== MEMORY ===' && free -h",
                "echo '=== DISK ===' && df -h --output=source,size,used,avail,pcent,target | grep -vE '^tmpfs|^udev|^/dev/loop' | head -10",
                // Exclude kernel threads (names in brackets like [kworker]) — they flood the output
                "echo '=== TOP PROCESSES ===' && ps aux --sort=-%cpu --no-headers | grep -v ' \\[' | head -12",
                "echo '=== SERVICES ===' && systemctl list-units --type=service --state=running --no-pager --no-legend 2>/dev/null | head -20 || docker ps --format 'table {{.Names}}\t{{.Status}}' 2>/dev/null",
                "echo '=== NETWORK ===' && ip -brief addr 2>/dev/null | grep -v '^lo ' | head -8",
            ].join(" && ");
            match tokio::time::timeout(
                Duration::from_secs(30),
                tokio::process::Command::new("sh").arg("-c").arg(&cmd).output(),
            ).await {
                Ok(Ok(out)) => {
                    let r = String::from_utf8_lossy(&out.stdout).to_string();
                    if r.len() > 12_000 { format!("{}\n[truncated]", &r[..12_000]) } else { r }
                }
                Ok(Err(e)) => format!("ERROR: {}", e),
                Err(_) => "ERROR: scan timed out".to_string(),
            }
        }

        "proxmox_list" => {
            // List all VMs and LXC containers on Proxmox host
            let host = args["host"].as_str().unwrap_or("tiamat");
            let cmd = format!(
                "ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 {} \
                \"echo '=== VMs ===' && qm list 2>/dev/null; \
                  echo '=== LXC ===' && pct list 2>/dev/null; \
                  echo '=== STORAGE ===' && pvesm status 2>/dev/null\"",
                host
            );
            match tokio::time::timeout(
                Duration::from_secs(20),
                tokio::process::Command::new("sh").arg("-c").arg(&cmd).output(),
            ).await {
                Ok(Ok(out)) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    if stdout.is_empty() { format!("ERROR: {}", stderr) } else { stdout.to_string() }
                }
                Ok(Err(e)) => format!("ERROR: {}", e),
                Err(_) => "ERROR: proxmox query timed out".to_string(),
            }
        }

        "edit_file" => {
            let path = args["path"].as_str().unwrap_or("");
            let search = args["search"].as_str().unwrap_or("");
            let replace = args["replace"].as_str().unwrap_or("");
            if path.is_empty() || search.is_empty() {
                return "ERROR: path and search are required".to_string();
            }
            match tokio::fs::read_to_string(path).await {
                Ok(content) => {
                    if !content.contains(search) {
                        return format!("ERROR: search string not found in {}\nHint: read the file first and use an exact substring.", path);
                    }
                    let new_content = content.replacen(search, replace, 1);
                    match tokio::fs::write(path, &new_content).await {
                        Ok(_) => format!("OK: edited {} ({} bytes → {} bytes)", path, content.len(), new_content.len()),
                        Err(e) => format!("ERROR writing {}: {}", path, e),
                    }
                }
                Err(e) => format!("ERROR reading {}: {}", path, e),
            }
        }

        "git_log" => {
            let path = args["path"].as_str().unwrap_or(default_cwd);
            let count = args["count"].as_u64().unwrap_or(10);
            match tokio::process::Command::new("git")
                .args(["-C", path, "--no-pager", "log", "--oneline", &format!("-{}", count)])
                .output().await
            {
                Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
                Err(e) => format!("ERROR: {}", e),
            }
        }

        "git_commit" => {
            let path = args["path"].as_str().unwrap_or(default_cwd);
            let message = args["message"].as_str().unwrap_or("fix");
            let stage = tokio::process::Command::new("git")
                .args(["-C", path, "add", "-A"])
                .output().await;
            if let Err(e) = stage {
                return format!("ERROR staging: {}", e);
            }
            match tokio::process::Command::new("git")
                .args(["-C", path, "commit", "-m", message])
                .output().await
            {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    if out.status.success() { stdout.to_string() } else { format!("ERROR: {}", stderr) }
                }
                Err(e) => format!("ERROR: {}", e),
            }
        }

        "ssh_exec" => {
            let host = args["host"].as_str().unwrap_or("");
            let cmd = args["cmd"].as_str().unwrap_or("");
            if host.is_empty() || cmd.is_empty() {
                return "ERROR: host and cmd are required".to_string();
            }
            let full_cmd = format!("ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 {} '{}'", host, cmd.replace('\'', "'\"'\"'"));
            match tokio::time::timeout(
                Duration::from_secs(30),
                tokio::process::Command::new("sh").arg("-c").arg(&full_cmd).output(),
            ).await {
                Ok(Ok(out)) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let exit = out.status.code().unwrap_or(-1);
                    let result = if stderr.is_empty() { stdout.to_string() } else if stdout.is_empty() { stderr.to_string() } else { format!("{}\nSTDERR: {}", stdout, stderr) };
                    if exit != 0 { format!("EXIT {}\n{}", exit, result) } else { result }
                }
                Ok(Err(e)) => format!("ERROR: {}", e),
                Err(_) => "ERROR: ssh timed out after 30s".to_string(),
            }
        }

        "http_post" => {
            let url = args["url"].as_str().unwrap_or("");
            let body = args["body"].as_str().unwrap_or("");
            if url.is_empty() { return "ERROR: url is required".to_string(); }
            let client = match Client::builder().timeout(Duration::from_secs(15)).danger_accept_invalid_certs(true).build() {
                Ok(c) => c,
                Err(e) => return format!("ERROR: {}", e),
            };
            match client.post(url).header("Content-Type", "application/json").body(body.to_string()).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    match resp.text().await {
                        Ok(b) => { let b = if b.len() > 8000 { format!("{}\n[truncated]", &b[..8000]) } else { b }; if status.is_success() { b } else { format!("HTTP {}\n{}", status, b) } }
                        Err(e) => format!("HTTP {} — read error: {}", status, e),
                    }
                }
                Err(e) => format!("ERROR: {}", e),
            }
        }

        "systemctl_cmd" => {
            let action = args["action"].as_str().unwrap_or("");
            let service = args["service"].as_str().unwrap_or("");
            if action.is_empty() || service.is_empty() { return "ERROR: action and service required".to_string(); }
            let valid = ["start", "stop", "restart", "status", "enable", "disable"];
            if !valid.contains(&action) { return format!("ERROR: action must be one of {:?}", valid); }
            match tokio::process::Command::new("systemctl").args([action, service]).output().await {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    if stdout.is_empty() { stderr.to_string() } else { stdout.to_string() }
                }
                Err(e) => format!("ERROR: {}", e),
            }
        }

        "process_list" => {
            let filter = args["filter"].as_str().unwrap_or("");
            let cmd = if filter.is_empty() {
                "ps aux --sort=-%cpu | head -30".to_string()
            } else {
                format!("ps aux | grep -i '{}' | grep -v grep", filter)
            };
            match tokio::process::Command::new("sh").arg("-c").arg(&cmd).output().await {
                Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
                Err(e) => format!("ERROR: {}", e),
            }
        }

        "web_search" => {
            let query = args["query"].as_str().unwrap_or("");
            let num = args["num_results"].as_u64().unwrap_or(5);
            if query.is_empty() { return "ERROR: query is required".to_string(); }

            // Try local SearXNG first (configurable via SEARXNG_URL env), fall back to DuckDuckGo
            let searxng_url = std::env::var("SEARXNG_URL").unwrap_or_default();
            let client = reqwest::Client::builder().timeout(Duration::from_secs(15)).build()
                .unwrap_or_else(|_| reqwest::Client::new());

            if !searxng_url.is_empty() {
                let url = format!("{}/search?q={}&format=json&categories=general&language=en",
                    searxng_url.trim_end_matches('/'),
                    urlencoding::encode(query));
                if let Ok(resp) = client.get(&url).send().await {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        if let Some(results) = data["results"].as_array() {
                            let out: Vec<String> = results.iter().take(num as usize).map(|r| {
                                format!("**{}**\n{}\n{}",
                                    r["title"].as_str().unwrap_or("(no title)"),
                                    r["url"].as_str().unwrap_or(""),
                                    r["content"].as_str().unwrap_or("(no snippet)"))
                            }).collect();
                            return out.join("\n\n");
                        }
                    }
                }
            }

            // DuckDuckGo instant answer API (no key needed)
            let ddg_url = format!("https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
                urlencoding::encode(query));
            match client.get(&ddg_url).header("User-Agent", "NexusTerminal/1.0").send().await {
                Ok(resp) => {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        let mut results = Vec::new();
                        if let Some(abstract_text) = data["Abstract"].as_str() {
                            if !abstract_text.is_empty() {
                                results.push(format!("**{}**\n{}",
                                    data["Heading"].as_str().unwrap_or(query), abstract_text));
                            }
                        }
                        if let Some(related) = data["RelatedTopics"].as_array() {
                            for topic in related.iter().take(num as usize - results.len()) {
                                if let Some(text) = topic["Text"].as_str() {
                                    results.push(format!("- {}", text));
                                }
                            }
                        }
                        if results.is_empty() {
                            format!("No results found for '{}'. Set SEARXNG_URL in .env for better web search.", query)
                        } else {
                            results.join("\n\n")
                        }
                    } else {
                        format!("ERROR: failed to parse search results")
                    }
                }
                Err(e) => format!("ERROR: web search failed: {}", e),
            }
        }

        "mcp_call" => {
            let server = args["server"].as_str().unwrap_or("");
            let tool = args["tool"].as_str().unwrap_or("");
            let call_args = args.get("args").cloned().unwrap_or(serde_json::json!({}));
            if server.is_empty() || tool.is_empty() { return "ERROR: server and tool are required".to_string(); }

            // Check for configured MCP server URL
            let server_env_key = format!("MCP_{}_URL", server.to_uppercase().replace('-', "_"));
            let server_url = std::env::var(&server_env_key)
                .or_else(|_| std::env::var("MCP_DEFAULT_URL"))
                .unwrap_or_else(|_| format!("http://localhost:3000"));

            let client = reqwest::Client::builder().timeout(Duration::from_secs(30)).build()
                .unwrap_or_else(|_| reqwest::Client::new());

            // Standard MCP HTTP/SSE transport — POST /mcp with JSON-RPC
            let rpc = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": { "name": tool, "arguments": call_args }
            });

            match client.post(&format!("{}/mcp", server_url.trim_end_matches('/')))
                .json(&rpc).send().await
            {
                Ok(resp) => {
                    match resp.json::<serde_json::Value>().await {
                        Ok(data) => {
                            if let Some(err) = data.get("error") {
                                format!("MCP error: {}", err)
                            } else if let Some(content) = data["result"]["content"].as_array() {
                                content.iter().filter_map(|c| c["text"].as_str()).collect::<Vec<_>>().join("\n")
                            } else {
                                data["result"].to_string()
                            }
                        }
                        Err(e) => format!("ERROR: failed to parse MCP response: {}", e),
                    }
                }
                Err(e) => format!("ERROR: MCP call to {} failed: {}\nSet {}=<url> in .env to configure this server.", server, e, server_env_key),
            }
        }

        "ask_user" => {
            let question = args["question"].as_str().unwrap_or("Proceed?");
            let options: Vec<String> = args["options"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_else(|| vec!["Yes".to_string(), "No".to_string()]);

            // Emit agent-question event — frontend renders clickable buttons
            if let Some(app) = crate::terminal::APP_HANDLE.get() {
                let _ = app.emit("agent-question", AgentQuestionEvent {
                    session_id: session_id.to_string(),
                    question: question.to_string(),
                    options: options.clone(),
                    data: None,
                });
            }

            // Store oneshot sender; will be resolved by answer_agent_question command
            let (tx, rx) = tokio::sync::oneshot::channel::<String>();
            {
                let mut map = pending_answers().lock().unwrap_or_else(|e| e.into_inner());
                map.insert(session_id.to_string(), tx);
            }

            // Block agent loop until user answers (or 120s timeout)
            match tokio::time::timeout(Duration::from_secs(120), rx).await {
                Ok(Ok(answer)) => answer,
                Ok(Err(_)) => "cancelled".to_string(),
                Err(_) => {
                    // Clean up on timeout
                    let mut map = pending_answers().lock().unwrap_or_else(|e| e.into_inner());
                    map.remove(session_id);
                    "timeout — no response after 120s. Proceeding with report only.".to_string()
                }
            }
        }

        "screenshot" => {
            let prompt = args["prompt"].as_str().unwrap_or("Describe what you see on the screen");
            match crate::vision_commands::capture_and_ask(
                prompt.to_string(), None, None,
            ).await {
                Ok(description) => description,
                Err(e) => format!("ERROR: screenshot failed: {}", e),
            }
        }

        "analyze_code" => {
            let path = args["path"].as_str().unwrap_or("");
            let analysis_type = args["analysis_type"].as_str().unwrap_or("errors");
            if path.is_empty() { return "ERROR: path is required".to_string(); }

            // If path is a DIRECTORY: find all project roots and run real compiler checks.
            // Handles both single projects (has Cargo.toml/package.json at root) and
            // monorepos (recursively finds up to 8 project roots in subdirectories).
            if std::path::Path::new(path).is_dir() {
                let mut results: Vec<String> = Vec::new();

                // Helper: locate Cargo.toml dirs (excluding build outputs)
                let find_cargo_dirs = async |root: &str| -> Vec<String> {
                    let cmd = format!(
                        "find '{}' -name 'Cargo.toml' \
                         -not -path '*/target/*' -not -path '*/.git/*' \
                         -not -path '*/node_modules/*' 2>/dev/null | head -8",
                        root
                    );
                    match tokio::process::Command::new("sh").arg("-c").arg(&cmd).output().await {
                        Ok(out) => String::from_utf8_lossy(&out.stdout)
                            .lines()
                            .filter_map(|l| std::path::Path::new(l.trim()).parent()
                                .map(|p| p.to_string_lossy().to_string()))
                            .collect(),
                        Err(_) => vec![],
                    }
                };

                // Helper: locate package.json dirs (excluding node_modules)
                let find_package_dirs = async |root: &str| -> Vec<String> {
                    let cmd = format!(
                        "find '{}' -name 'package.json' \
                         -not -path '*/node_modules/*' -not -path '*/.git/*' \
                         2>/dev/null | head -5",
                        root
                    );
                    match tokio::process::Command::new("sh").arg("-c").arg(&cmd).output().await {
                        Ok(out) => String::from_utf8_lossy(&out.stdout)
                            .lines()
                            .filter_map(|l| std::path::Path::new(l.trim()).parent()
                                .map(|p| p.to_string_lossy().to_string()))
                            .collect(),
                        Err(_) => vec![],
                    }
                };

                let cargo_dirs = find_cargo_dirs(path).await;
                let package_dirs = find_package_dirs(path).await;

                // Run cargo check in each Rust project root
                for dir in &cargo_dirs {
                    let short = std::path::Path::new(dir)
                        .strip_prefix(path).unwrap_or(std::path::Path::new(dir))
                        .display().to_string();
                    let prefix = if short.is_empty() { String::new() } else { format!("[{}] ", short) };
                    match tokio::time::timeout(
                        Duration::from_secs(90),
                        tokio::process::Command::new("cargo")
                            .args(["check", "--message-format=short"])
                            .current_dir(dir)
                            .output()
                    ).await {
                        Ok(Ok(out)) => {
                            let combined = format!("{}{}",
                                String::from_utf8_lossy(&out.stdout),
                                String::from_utf8_lossy(&out.stderr));
                            let trimmed = combined.trim();
                            results.push(if out.status.success() {
                                format!("{}cargo check: ✅ PASSED", prefix)
                            } else {
                                let t = if trimmed.len() > 4000 { &trimmed[..4000] } else { trimmed };
                                format!("{}cargo check ❌ ERRORS:\n{}", prefix, t)
                            });
                        }
                        Ok(Err(e)) => results.push(format!("{}cargo check failed to run: {}", prefix, e)),
                        Err(_) => results.push(format!("{}cargo check timed out (90s)", prefix)),
                    }
                }

                // Run tsc in each TypeScript project root
                for dir in &package_dirs {
                    let short = std::path::Path::new(dir)
                        .strip_prefix(path).unwrap_or(std::path::Path::new(dir))
                        .display().to_string();
                    let prefix = if short.is_empty() { String::new() } else { format!("[{}] ", short) };
                    match tokio::time::timeout(
                        Duration::from_secs(60),
                        tokio::process::Command::new("sh")
                            .arg("-c").arg("npx tsc --noEmit 2>&1")
                            .current_dir(dir)
                            .output()
                    ).await {
                        Ok(Ok(out)) => {
                            let combined = String::from_utf8_lossy(&out.stdout).to_string();
                            let trimmed = combined.trim();
                            results.push(if out.status.success() {
                                format!("{}tsc: ✅ PASSED", prefix)
                            } else {
                                let t = if trimmed.len() > 4000 { &trimmed[..4000] } else { trimmed };
                                format!("{}tsc ❌ ERRORS:\n{}", prefix, t)
                            });
                        }
                        Ok(Err(e)) => results.push(format!("{}tsc failed: {}", prefix, e)),
                        Err(_) => results.push(format!("{}tsc timed out (60s)", prefix)),
                    }
                }

                if results.is_empty() {
                    results.push(
                        "No Rust (Cargo.toml) or TypeScript (package.json) projects found.\n\
                         Use run_cmd with a specific command to check other project types.".to_string()
                    );
                }

                let summary = format!("=== Directory scan: {} ===\nChecked: {} Rust + {} TS projects\n\n{}",
                    path, cargo_dirs.len(), package_dirs.len(),
                    results.join("\n\n"));

                // If any errors were found, automatically ask the user what to do.
                // This is unconditional — no model cooperation required.
                let has_errors = results.iter().any(|r| r.contains('❌') || r.contains("ERRORS"));
                if has_errors && !session_id.is_empty() {
                    // Count error lines for the question
                    let error_count = results.iter().filter(|r| r.contains('❌') || r.contains("ERRORS")).count();
                    let question = format!(
                        "Found errors in {} project{}. What would you like to do?",
                        error_count,
                        if error_count == 1 { "" } else { "s" }
                    );
                    let options = vec![
                        "Fix all errors".to_string(),
                        "Report only".to_string(),
                    ];

                    // Emit question event — frontend shows buttons immediately.
                    // Include scan_path so the frontend can call scan_and_fix directly
                    // without routing through the model (which ignores tool instructions).
                    info!("analyze_code: has_errors={} session_id={:?}", has_errors, session_id);
                    if let Some(app) = crate::terminal::APP_HANDLE.get() {
                        let _ = app.emit("agent-question", AgentQuestionEvent {
                            session_id: session_id.to_string(),
                            question: question.clone(),
                            options: options.clone(),
                            data: Some(serde_json::json!({
                                "kind": "scan_and_fix",
                                "scan_path": path,
                            })),
                        });
                    }

                    // Store oneshot and wait
                    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
                    {
                        let mut map = pending_answers().lock().unwrap_or_else(|e| e.into_inner());
                        map.insert(session_id.to_string(), tx);
                    }
                    let user_choice = match tokio::time::timeout(Duration::from_secs(120), rx).await {
                        Ok(Ok(answer)) => answer,
                        _ => {
                            let mut map = pending_answers().lock().unwrap_or_else(|e| e.into_inner());
                            map.remove(session_id);
                            "Report only".to_string()
                        }
                    };

                    // If user chose Fix: spawn the Rust fix engine directly.
                    // This bypasses the model entirely — small local models cannot
                    // reliably chain read_file → edit_file → cargo check loops.
                    if user_choice.to_lowercase().contains("fix") {
                        if let Some(app) = crate::terminal::APP_HANDLE.get() {
                            let app2 = app.clone();
                            let sid = session_id.to_string();
                            let sp = path.to_string();
                            let ollama_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
                            let ollama_port = std::env::var("OLLAMA_PORT").unwrap_or_else(|_| "11434".to_string());
                            let ollama_url = format!("http://{}:{}", ollama_host, ollama_port);
                            tokio::spawn(async move {
                                crate::fix_engine::scan_and_fix(app2, sid, sp, ollama_url).await;
                            });
                            return format!("{summary}\n\n[Fix engine launched — watch the AI panel for live progress.]");
                        }
                        // Fallback if APP_HANDLE not available
                        return format!("{summary}\n\n║ USER DECISION: Fix all errors ║\nNow fix every error listed above: read each failing file, apply the minimal fix with edit_file, run cargo check / tsc to verify, iterate until all pass, then git_commit.");
                    } else {
                        return format!("{summary}\n\n║ USER DECISION: Report only ║\nUser chose report only. Provide a concise summary of what was found without making any changes.");
                    }
                }

                return summary;
            }

            // Single FILE path: read and send to Ollama for AI analysis
            let content = match tokio::fs::read_to_string(path).await {
                Ok(c) => c,
                Err(e) => return format!("ERROR reading {}: {}", path, e),
            };
            let ext = std::path::Path::new(path).extension()
                .and_then(|e| e.to_str()).unwrap_or("text");
            let language = match ext {
                "rs" => "Rust", "ts" | "tsx" => "TypeScript", "js" | "jsx" => "JavaScript",
                "py" => "Python", "go" => "Go", "java" => "Java",
                "cpp" | "cc" | "cxx" => "C++", "c" => "C", _ => ext,
            };
            let code = if content.len() > 24_000 {
                format!("{}\n... [truncated at 24KB]", &content[..24_000])
            } else { content };
            // Forceful prompt prefix — prevents smaller models from refusing to analyze
            let base_prefix = format!(
                "ANALYZE THIS {} CODE:\n\n```{}\n{}\n```\n\nYou must analyze the code above. Do not ask for more information — the code is right there. Analyze it now.\n\n",
                language.to_uppercase(), ext, code
            );
            let task_suffix = match analysis_type {
                "errors" => "TASK: Identify all syntax errors, type errors, logic issues, and potential bugs.\nFor each finding: exact line numbers, description, suggested fix.",
                "style" => "TASK: Review code style, naming conventions, formatting, documentation quality, and best practice compliance.\nFor each finding: line numbers, description, recommended improvement.",
                "security" => "TASK: Identify security vulnerabilities, unsafe operations, input validation issues, authentication problems, data exposure risks.\nFor each finding: line numbers, severity (High/Medium/Low), description, recommended fix.",
                "performance" => "TASK: Identify performance bottlenecks, inefficient algorithms, memory issues, I/O optimization opportunities.\nFor each finding: line numbers, impact (High/Medium/Low), description, suggested optimization.",
                "cleanup" => "TASK: Identify dead code, stub functions, unused variables/imports, zombie code, commented-out code, duplicate code.\nFor each finding: exact line numbers, why it is unused/stub, whether it is safe to remove.",
                _ => "TASK: Comprehensive analysis — errors, security, performance, style, dead code.\nFor each finding: category, line numbers, description, priority (High/Medium/Low), recommended fix.",
            };
            let prompt = format!("{base_prefix}{task_suffix}");

            let ollama_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
            let ollama_port = std::env::var("OLLAMA_PORT").unwrap_or_else(|_| "11434".to_string());
            let ollama_base = format!("http://{}:{}", ollama_host, ollama_port);
            // Code analysis: use deepseek-coder / granite-code priority chain
            let model = select_model(TaskKind::CodeAnalysis, &ollama_base).await;
            let url = format!("{}/api/generate", ollama_base);
            let client = reqwest::Client::builder().timeout(Duration::from_secs(120))
                .build().unwrap_or_else(|_| reqwest::Client::new());

            // Helper to call Ollama generate
            let call_ollama = |p: String| {
                let client2 = client.clone();
                let url2 = url.clone();
                let model2 = model.clone();
                async move {
                    let body = serde_json::json!({
                        "model": model2, "prompt": p, "stream": false,
                        "options": { "temperature": 0.1, "num_predict": 2048 }
                    });
                    match client2.post(&url2).json(&body).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            match resp.json::<serde_json::Value>().await {
                                Ok(data) => Ok(data["response"].as_str().unwrap_or("").to_string()),
                                Err(e) => Err(format!("parse error: {}", e)),
                            }
                        }
                        Ok(resp) => Err(format!("Ollama returned {}", resp.status())),
                        Err(e) => Err(format!("{}", e)),
                    }
                }
            };

            // Unhelpful-response phrases that indicate the model didn't actually analyze the code
            let unhelpful = [
                "i don't have access", "i cannot access", "i'm unable to see",
                "i can't see", "no code provided", "code is not provided",
                "i need to see", "please provide", "provide me with",
                "i'd be happy to help", "sure, i can analyze",
                "please provide the file", "it appears to be incomplete",
                "cannot provide a detailed analysis",
            ];

            let analysis = match call_ollama(prompt).await {
                Ok(text) => {
                    let lower = text.to_lowercase();
                    let is_unhelpful = unhelpful.iter().any(|p| lower.contains(p));
                    if is_unhelpful || text.trim().is_empty() {
                        // Retry with an even simpler direct prompt
                        let retry_prompt = format!(
                            "Here is {language} code to analyze:\n\n{code}\n\n\
                             Find any errors, bugs, style issues, security problems, or dead code. \
                             Be specific about what you find."
                        );
                        match call_ollama(retry_prompt).await {
                            Ok(t) if !t.trim().is_empty() => format!("[retried — model initially refused]\n{}", t),
                            Ok(_) => "No analysis produced after retry.".to_string(),
                            Err(e) => format!("ERROR on retry: {}", e),
                        }
                    } else {
                        text
                    }
                }
                Err(e) => format!("ERROR: {}", e),
            };
            format!("=== Code Analysis ({analysis_type}) ===\nFile: {path}\nLanguage: {language}\n\n{analysis}")
        }

        "autofix_code" => {
            let path = args["path"].as_str().unwrap_or("");
            let dry_run = args["dry_run"].as_bool().unwrap_or(false);
            if path.is_empty() { return "ERROR: path is required".to_string(); }

            let ollama_host = std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
            let ollama_port = std::env::var("OLLAMA_PORT").unwrap_or_else(|_| "11434".to_string());
            let ollama_base = format!("http://{}:{}", ollama_host, ollama_port);
            // Code fix: use codestral / deepseek-coder-v2 priority chain
            let model = select_model(TaskKind::CodeFix, &ollama_base).await;
            let url = format!("{}/api/generate", ollama_base);
            let client = reqwest::Client::builder().timeout(Duration::from_secs(120))
                .build().unwrap_or_else(|_| reqwest::Client::new());

            // ── Directory mode: find all code files and fix each one ──────────
            if std::path::Path::new(path).is_dir() {
                let exts = ["rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "cpp", "c", "h"];
                let skip_dirs = ["node_modules", "target", "build", "dist", ".git", "__pycache__"];

                let mut code_files: Vec<std::path::PathBuf> = Vec::new();
                let mut stack = vec![std::path::PathBuf::from(path)];
                while let Some(dir) = stack.pop() {
                    let mut rd = match tokio::fs::read_dir(&dir).await {
                        Ok(r) => r, Err(_) => continue,
                    };
                    while let Ok(Some(entry)) = rd.next_entry().await {
                        let p = entry.path();
                        if p.is_dir() {
                            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            if !skip_dirs.contains(&name) { stack.push(p); }
                        } else if let Some(e) = p.extension().and_then(|e| e.to_str()) {
                            if exts.contains(&e) {
                                // Skip files > 50 KB (too large for safe AI rewrite)
                                if p.metadata().map(|m| m.len()).unwrap_or(0) < 50_000 {
                                    code_files.push(p);
                                }
                            }
                        }
                    }
                }

                if code_files.is_empty() {
                    return format!("No code files found in {}. Checked: {}",
                        path, exts.join(", "));
                }

                // Large-codebase guard: ask user before processing many files
                if code_files.len() > 20 && !session_id.is_empty() {
                    let question = format!(
                        "Found {} files to autofix in {}. This will rewrite all of them with AI fixes and create .bak backups. Continue?",
                        code_files.len(), path
                    );
                    let options = vec!["Fix all files".to_string(), "Cancel".to_string()];
                    if let Some(app) = crate::terminal::APP_HANDLE.get() {
                        let _ = app.emit("agent-question", AgentQuestionEvent {
                            session_id: session_id.to_string(),
                            question: question.clone(),
                            options: options.clone(),
                            data: None,
                        });
                    }
                    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
                    {
                        let mut map = pending_answers().lock().unwrap_or_else(|e| e.into_inner());
                        map.insert(session_id.to_string(), tx);
                    }
                    let answer = match tokio::time::timeout(Duration::from_secs(120), rx).await {
                        Ok(Ok(a)) => a,
                        _ => {
                            let mut map = pending_answers().lock().unwrap_or_else(|e| e.into_inner());
                            map.remove(session_id);
                            "Cancel".to_string()
                        }
                    };
                    if !answer.to_lowercase().contains("fix") {
                        return format!("Autofix cancelled. ({} files would have been processed)", code_files.len());
                    }
                }

                let total = code_files.len();
                let mut fixed_count = 0usize;
                let mut skipped_count = 0usize;
                let mut results: Vec<String> = Vec::new();

                for file_path in &code_files {
                    let fstr = file_path.to_string_lossy().to_string();
                    let content = match tokio::fs::read_to_string(file_path).await {
                        Ok(c) => c, Err(e) => { results.push(format!("SKIP {fstr}: read error: {e}")); skipped_count += 1; continue; }
                    };
                    if content.trim().is_empty() { skipped_count += 1; continue; }
                    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("text");
                    let prompt = format!(
                        "Fix the issues in this {ext} code:\n\n```{ext}\n{}\n```\n\nFix: syntax errors, remove unused imports/variables, remove commented-out code, improve style, optimize performance.\nReturn ONLY the fixed code in a ```{ext} code block. No explanations.",
                        if content.len() > 30_000 { &content[..30_000] } else { &content }
                    );
                    let body = serde_json::json!({
                        "model": model, "prompt": prompt, "stream": false,
                        "options": { "temperature": 0.05, "num_predict": 4096 }
                    });
                    let response_text = match client.post(&url).json(&body).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            match resp.json::<serde_json::Value>().await {
                                Ok(data) => data["response"].as_str().unwrap_or("").to_string(),
                                Err(_) => { skipped_count += 1; continue; }
                            }
                        }
                        _ => { skipped_count += 1; continue; }
                    };
                    let fixed = extract_code_block(&response_text, ext);
                    if fixed.trim().is_empty() || fixed.trim() == content.trim() {
                        results.push(format!("OK (no changes): {fstr}"));
                        continue;
                    }
                    if dry_run {
                        results.push(format!("DRY RUN would fix: {fstr} ({} → {} lines)",
                            content.lines().count(), fixed.lines().count()));
                        fixed_count += 1;
                        continue;
                    }
                    let backup = format!("{}.bak", fstr);
                    if tokio::fs::write(&backup, &content).await.is_ok() {
                        if tokio::fs::write(file_path, &fixed).await.is_ok() {
                            results.push(format!("✅ Fixed: {fstr} ({} → {} lines)",
                                content.lines().count(), fixed.lines().count()));
                            fixed_count += 1;
                        } else {
                            results.push(format!("❌ Write failed: {fstr} (backup kept at {backup})"));
                            skipped_count += 1;
                        }
                    } else {
                        results.push(format!("❌ Backup failed, skipped: {fstr}"));
                        skipped_count += 1;
                    }
                }

                let mode = if dry_run { "Dry run" } else { "Autofix" };
                return format!("=== {} complete: {} ===\nFixed: {}/{}  Skipped: {}\n\n{}",
                    mode, path, fixed_count, total, skipped_count, results.join("\n"));
            }

            // ── Single file mode ──────────────────────────────────────────────
            let content = match tokio::fs::read_to_string(path).await {
                Ok(c) => c,
                Err(e) => return format!("ERROR reading {}: {}", path, e),
            };
            if content.len() > 50_000 {
                return format!("ERROR: file too large ({}KB > 50KB). Use edit_file for targeted fixes.",
                    content.len() / 1024);
            }
            let ext = std::path::Path::new(path).extension()
                .and_then(|e| e.to_str()).unwrap_or("text");
            let prompt = format!(
                "Fix the issues in this {ext} code:\n\n```{ext}\n{content}\n```\n\nFix: syntax errors, remove unused imports/variables, remove commented-out code, improve style, optimize performance.\nReturn ONLY the fixed code in a ```{ext} code block. No explanations."
            );
            let body = serde_json::json!({
                "model": model, "prompt": prompt, "stream": false,
                "options": { "temperature": 0.05, "num_predict": 4096 }
            });
            let response_text = match client.post(&url).json(&body).send().await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<serde_json::Value>().await {
                        Ok(data) => data["response"].as_str().unwrap_or("").to_string(),
                        Err(e) => return format!("ERROR: {}", e),
                    }
                }
                Ok(resp) => return format!("ERROR: Ollama returned {}", resp.status()),
                Err(e) => return format!("ERROR: {}", e),
            };
            let fixed = extract_code_block(&response_text, ext);
            if fixed.trim().is_empty() { return "No changes suggested.".to_string(); }
            if fixed.trim() == content.trim() { return format!("No changes needed for {path}"); }
            if dry_run {
                return format!("=== Dry run: {} ===\nProposed fixed code ({} lines):\n```{ext}\n{}\n```\nRun with dry_run=false to apply.",
                    path, fixed.lines().count(), fixed);
            }
            let backup = format!("{}.bak", path);
            if let Err(e) = tokio::fs::write(&backup, &content).await {
                return format!("ERROR: backup failed: {}", e);
            }
            match tokio::fs::write(path, &fixed).await {
                Ok(_) => format!("OK: fixed {path} ({} lines → {} lines). Backup: {backup}",
                    content.lines().count(), fixed.lines().count()),
                Err(e) => format!("ERROR writing {path}: {} (backup at {backup})", e),
            }
        }

        other => format!("ERROR: unknown tool '{}'", other),
    }
}

/// Extract a fenced code block from model output.
/// Tries language-specific fence first, then plain ```, then returns empty.
fn extract_code_block(text: &str, lang: &str) -> String {
    // Try ```lang\n...\n```
    let fence = format!("```{}", lang);
    if let Some(start) = text.find(&fence) {
        let after = &text[start + fence.len()..];
        let after = after.trim_start_matches(|c: char| c != '\n');
        let after = after.trim_start_matches('\n');
        if let Some(end) = after.find("```") {
            return after[..end].to_string();
        }
    }
    // Try plain ```\n...\n```
    if let Some(start) = text.find("```\n") {
        let after = &text[start + 4..];
        if let Some(end) = after.find("```") {
            return after[..end].to_string();
        }
    }
    String::new()
}

// ── Main agent entry point (blocking) ────────────────────────────────────────

pub async fn run_agent(
    ollama_url: &str,
    model: &str,
    user_message: &str,
    history: Vec<ChatMessage>,
    cwd: &str,
    context: Option<&str>,
) -> Result<AgentResponse> {
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    let tools = build_tools();
    let mut steps: Vec<AgentStep> = Vec::new();
    let mut messages: Vec<ChatMessage> = Vec::new();
    let mut seen_calls: HashSet<String> = HashSet::new();

    // System prompt (inject cwd and optional context)
    let sys = if let Some(ctx) = context {
        format!(
            "{}\n\nCurrent working directory: {}\nContext: {}",
            SYSTEM_PROMPT, cwd, ctx
        )
    } else {
        format!(
            "{}\n\nCurrent working directory: {}",
            SYSTEM_PROMPT, cwd
        )
    };
    messages.push(ChatMessage {
        role: "system".to_string(),
        content: sys,
        tool_calls: None,
        tool_call_id: None,
    });

    // Inject prior conversation history
    for msg in history {
        messages.push(msg);
    }

    // User message
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_message.to_string(),
        tool_calls: None,
        tool_call_id: None,
    });

    // Agent loop
    for step_num in 0..MAX_STEPS {
        debug!("Agent step {}", step_num);

        let req = ChatRequest {
            model,
            messages: &messages,
            stream: false,
            tools: &tools,
            options: ChatOptions {
                temperature: 0.2,
                num_predict: 4096,
            },
        };

        let url = format!("{}/api/chat", ollama_url);
        let resp = client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Ollama request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama error {}: {}", status, body));
        }

        let chat_resp: ChatResponseBody = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse Ollama response: {}", e))?;

        let assistant_msg = chat_resp.message;
        info!(
            "Agent step {}: content={} chars, tool_calls={}",
            step_num,
            assistant_msg.content.len(),
            assistant_msg
                .tool_calls
                .as_ref()
                .map(|tc| tc.len())
                .unwrap_or(0)
        );

        // Push assistant message to history (preserving tool_calls)
        messages.push(assistant_msg.clone());

        // Check for native tool calls
        let has_tool_calls = assistant_msg
            .tool_calls
            .as_ref()
            .map(|tc| !tc.is_empty())
            .unwrap_or(false);

        if has_tool_calls {
            let tool_calls = assistant_msg.tool_calls.as_ref().unwrap();

            // Loop detection: check for duplicate calls
            let mut has_duplicate = false;
            for tc in tool_calls {
                let args = normalize_args(&tc.function.arguments);
                let key = format!(
                    "{}:{}",
                    tc.function.name,
                    serde_json::to_string(&args).unwrap_or_default()
                );
                if seen_calls.contains(&key) {
                    has_duplicate = true;
                    break;
                }
            }

            if has_duplicate {
                warn!("Duplicate tool call detected at step {}", step_num);
                messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: "You already ran this. Give your final answer now.".to_string(),
                    tool_calls: None,
                    tool_call_id: None,
                });
                continue;
            }

            // Execute each tool call
            for tc in tool_calls {
                let args = normalize_args(&tc.function.arguments);
                let key = format!(
                    "{}:{}",
                    tc.function.name,
                    serde_json::to_string(&args).unwrap_or_default()
                );
                seen_calls.insert(key);

                info!("Tool call: {} {:?}", tc.function.name, args);
                steps.push(AgentStep {
                    kind: "tool_call".to_string(),
                    content: format!(
                        "{}  {}",
                        tc.function.name,
                        serde_json::to_string(&args).unwrap_or_default()
                    ),
                });

                let result = exec_tool(&tc.function.name, &args, cwd, "").await;
                debug!("Tool result: {} chars", result.len());

                steps.push(AgentStep {
                    kind: "tool_result".to_string(),
                    content: result.clone(),
                });

                // Feed result back as role="tool" message
                messages.push(ChatMessage {
                    role: "tool".to_string(),
                    content: result,
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
        } else {
            // No tool_calls = final answer
            let answer = assistant_msg.content.clone();
            steps.push(AgentStep {
                kind: "answer".to_string(),
                content: answer.clone(),
            });

            return Ok(AgentResponse { answer, steps });
        }
    }

    // Hit step limit
    warn!("Agent hit step limit of {}", MAX_STEPS);
    Ok(AgentResponse {
        answer: "Agent reached maximum steps.".to_string(),
        steps,
    })
}

// ── Streaming agent ──────────────────────────────────────────────────────────
// Streams tokens to the frontend as they arrive, then executes tools.
// Emits Tauri events:
//   agent-token       { session_id, token }          – text chunk
//   agent-tool-call   { session_id, tool, args }     – before tool runs
//   agent-tool-result { session_id, tool, result }   – after tool runs
//   agent-done        { session_id, answer }          – final answer
//   agent-error       { session_id, error }           – on failure

pub async fn run_agent_streaming<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    session_id: String,
    ollama_url: &str,
    model: &str,
    user_message: &str,
    history: Vec<ChatMessage>,
    cwd: &str,
    context: Option<&str>,
) {
    let result = run_agent_streaming_inner(
        &app, &session_id, ollama_url, model, user_message, history, cwd, context,
    )
    .await;

    if let Err(e) = result {
        let _ = app.emit(
            "agent-error",
            AgentErrorEvent {
                session_id,
                error: e.to_string(),
            },
        );
    }
}

async fn run_agent_streaming_inner<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    session_id: &str,
    ollama_url: &str,
    model: &str,
    user_message: &str,
    history: Vec<ChatMessage>,
    cwd: &str,
    context: Option<&str>,
) -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;

    let tools = build_tools();
    let mut messages: Vec<ChatMessage> = Vec::new();
    let mut seen_calls: HashSet<String> = HashSet::new();

    let sys = if let Some(ctx) = context {
        format!(
            "{}\n\nCurrent working directory: {}\nContext: {}",
            SYSTEM_PROMPT, cwd, ctx
        )
    } else {
        format!(
            "{}\n\nCurrent working directory: {}",
            SYSTEM_PROMPT, cwd
        )
    };
    messages.push(ChatMessage {
        role: "system".to_string(),
        content: sys,
        tool_calls: None,
        tool_call_id: None,
    });
    for msg in history {
        messages.push(msg);
    }
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_message.to_string(),
        tool_calls: None,
        tool_call_id: None,
    });

    for step in 0..MAX_STEPS {
        debug!("Streaming agent step {}", step);

        // Stream request with tools — text tokens stream in real-time,
        // tool calls arrive in final chunk(s).
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
            "tools": tools,
            "options": { "temperature": 0.1, "num_predict": 8192 }
        });

        let url = format!("{}/api/chat", ollama_url);
        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Ollama request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_txt = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama error {}: {}", status, body_txt));
        }

        // Collect streamed chunks
        let mut stream = resp.bytes_stream();
        let mut full_content = String::new();
        let mut collected_tool_calls: Vec<ToolCallResponse> = Vec::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| anyhow::anyhow!("Stream read error: {}", e))?;
            let text = String::from_utf8_lossy(&chunk);

            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if let Ok(parsed) = serde_json::from_str::<StreamChunk>(line) {
                    if let Some(msg) = parsed.message {
                        // Emit content tokens in real-time
                        if !msg.content.is_empty() {
                            full_content.push_str(&msg.content);
                            let _ = app.emit(
                                "agent-token",
                                AgentTokenEvent {
                                    session_id: session_id.to_string(),
                                    token: msg.content,
                                },
                            );
                        }

                        // Collect tool calls from final chunk
                        if let Some(tcs) = msg.tool_calls {
                            collected_tool_calls.extend(tcs);
                        }
                    }

                    if parsed.done.unwrap_or(false) {
                        break;
                    }
                }
            }
        }

        info!(
            "Streaming step {}: {} chars, {} tool_calls",
            step,
            full_content.len(),
            collected_tool_calls.len()
        );

        // Build assistant message for history
        let assistant_msg = ChatMessage {
            role: "assistant".to_string(),
            content: full_content.clone(),
            tool_calls: if collected_tool_calls.is_empty() {
                None
            } else {
                Some(collected_tool_calls.clone())
            },
            tool_call_id: None,
        };
        messages.push(assistant_msg);

        // Self-correction: if the model gave text but no tool calls on the first step,
        // it's describing instead of doing. Inject a correction to force tool execution.
        // This is what makes Oz (using frontier models) reliable — the model always follows up.
        if collected_tool_calls.is_empty() && step == 0 && !full_content.trim().is_empty() {
            let correction = format!(
                "You described what to do but didn't do it. Execute the task now. Call the appropriate tools immediately. Do not explain further.\n\nYour previous response was: {}",
                if full_content.len() > 500 { &full_content[..500] } else { &full_content }
            );
            messages.push(crate::agent::ChatMessage {
                role: "user".to_string(),
                content: correction,
                tool_calls: None,
                tool_call_id: None,
            });
            // Don't emit agent-done yet — continue the loop to force tool execution
            warn!("Step 0 had text but no tool calls — injecting self-correction");
        }

        if !collected_tool_calls.is_empty() {
            // Loop detection
            let mut has_duplicate = false;
            for tc in &collected_tool_calls {
                let args = normalize_args(&tc.function.arguments);
                let key = format!(
                    "{}:{}",
                    tc.function.name,
                    serde_json::to_string(&args).unwrap_or_default()
                );
                if seen_calls.contains(&key) {
                    has_duplicate = true;
                    break;
                }
            }

            if has_duplicate {
                warn!("Duplicate tool call detected at streaming step {}", step);
                messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: "You already ran this. Give your final answer now.".to_string(),
                    tool_calls: None,
                    tool_call_id: None,
                });
                continue;
            }

            // Execute each tool call
            for tc in &collected_tool_calls {
                let args = normalize_args(&tc.function.arguments);
                let key = format!(
                    "{}:{}",
                    tc.function.name,
                    serde_json::to_string(&args).unwrap_or_default()
                );
                seen_calls.insert(key);

                let _ = app.emit(
                    "agent-tool-call",
                    AgentToolEvent {
                        session_id: session_id.to_string(),
                        tool: tc.function.name.clone(),
                        args: serde_json::to_string(&args).unwrap_or_default(),
                    },
                );

                let result = exec_tool(&tc.function.name, &args, cwd, session_id).await;

                let _ = app.emit(
                    "agent-tool-result",
                    AgentToolResultEvent {
                        session_id: session_id.to_string(),
                        tool: tc.function.name.clone(),
                        result: result.clone(),
                    },
                );

                messages.push(ChatMessage {
                    role: "tool".to_string(),
                    content: result,
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
        } else {
            // No tool calls — final answer (tokens already streamed)
            let _ = app.emit(
                "agent-done",
                AgentDoneEvent {
                    session_id: session_id.to_string(),
                    answer: full_content,
                },
            );
            return Ok(());
        }
    }

    let _ = app.emit(
        "agent-done",
        AgentDoneEvent {
            session_id: session_id.to_string(),
            answer: "Agent reached maximum steps.".to_string(),
        },
    );
    Ok(())
}
