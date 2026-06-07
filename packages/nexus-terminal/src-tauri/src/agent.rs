/// NexusTerminal Agent — autonomous AI with tool use.
/// Works like Oz: reads files, runs commands, writes code, loops until done.
use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::Emitter;
use tracing::{debug, info, warn};

const MAX_STEPS: usize = 20;

const SYSTEM_PROMPT: &str = r#"You are NexusAI, an autonomous coding agent inside NexusTerminal.
You can read files, run shell commands, write code, and fix problems without asking permission.

TOOLS — when you need a tool, output ONLY a JSON object on its own line, nothing else before or after:
{"tool":"read_file","args":{"path":"/absolute/path/to/file"}}
{"tool":"write_file","args":{"path":"/absolute/path","content":"full file content here"}}
{"tool":"run_cmd","args":{"cmd":"ls -la","cwd":"/path"}}
{"tool":"list_dir","args":{"path":"/absolute/path"}}
{"tool":"grep","args":{"pattern":"search term","path":"/path","recursive":true}}
{"tool":"git_status","args":{"path":"/repo/path"}}

RULES:
- Be concise. No filler. No "Sure!", "Of course!", "I'll help you with that."
- Take action immediately. Don't ask permission for obvious steps.
- When you want to use a tool, output ONLY the JSON — nothing else on that line.
- After seeing a tool result, keep reasoning and use more tools or give your final answer.
- One tool call per response.
- When done with tools, give a brief plain-text answer.
- If something fails, try a different approach.
- Write complete working code — no stubs, no TODOs."#;

// ── Ollama /api/chat types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
    options: ChatOptions,
}

#[derive(Debug, Serialize)]
struct ChatOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}

// ── Tool call / result types (also sent to frontend) ─────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub kind: String,   // "think" | "tool_call" | "tool_result" | "answer"
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub answer: String,
    pub steps: Vec<AgentStep>,
}

// ── Tool execution ────────────────────────────────────────────────────────────

async fn exec_tool(call: &ToolCall, default_cwd: &str) -> String {
    match call.tool.as_str() {
        "read_file" => {
            let path = call.args["path"].as_str().unwrap_or("");
            match tokio::fs::read_to_string(path).await {
                Ok(content) => {
                    // Truncate very large files
                    if content.len() > 32_000 {
                        format!("{}\n\n[... truncated at 32KB ...]", &content[..32_000])
                    } else {
                        content
                    }
                }
                Err(e) => format!("ERROR: {}", e),
            }
        }
        "write_file" => {
            let path = call.args["path"].as_str().unwrap_or("");
            let content = call.args["content"].as_str().unwrap_or("");
            if let Some(parent) = std::path::Path::new(path).parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
            match tokio::fs::write(path, content).await {
                Ok(_) => format!("OK: wrote {} bytes to {}", content.len(), path),
                Err(e) => format!("ERROR: {}", e),
            }
        }
        "run_cmd" => {
            let cmd = call.args["cmd"].as_str().unwrap_or("");
            let cwd = call.args["cwd"].as_str().unwrap_or(default_cwd);
            if cmd.is_empty() { return "ERROR: empty command".to_string(); }
            match tokio::time::timeout(
                Duration::from_secs(30),
                tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .current_dir(cwd)
                    .output()
            ).await {
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
                    } else { result };
                    if exit != 0 {
                        format!("EXIT {}\n{}", exit, result)
                    } else {
                        result.to_string()
                    }
                }
                Ok(Err(e)) => format!("ERROR: {}", e),
                Err(_) => "ERROR: command timed out after 30s".to_string(),
            }
        }
        "list_dir" => {
            let path = call.args["path"].as_str().unwrap_or(default_cwd);
            match tokio::fs::read_dir(path).await {
                Ok(mut dir) => {
                    let mut entries = Vec::new();
                    while let Ok(Some(entry)) = dir.next_entry().await {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let is_dir = entry.file_type().await
                            .map(|t| t.is_dir()).unwrap_or(false);
                        entries.push(if is_dir { format!("{}/", name) } else { name });
                    }
                    entries.sort();
                    entries.join("\n")
                }
                Err(e) => format!("ERROR: {}", e),
            }
        }
        "grep" => {
            let pattern = call.args["pattern"].as_str().unwrap_or("");
            let path = call.args["path"].as_str().unwrap_or(default_cwd);
            let recursive = call.args["recursive"].as_bool().unwrap_or(true);
            let flag = if recursive { "-r" } else { "" };
            let cmd = if flag.is_empty() {
                format!("grep -n '{}' '{}'", pattern, path)
            } else {
                format!("grep -rn '{}' '{}'", pattern, path)
            };
            match tokio::time::timeout(
                Duration::from_secs(15),
                tokio::process::Command::new("sh").arg("-c").arg(&cmd).output()
            ).await {
                Ok(Ok(out)) => {
                    let result = String::from_utf8_lossy(&out.stdout).to_string();
                    if result.is_empty() { "no matches".to_string() }
                    else if result.len() > 8000 {
                        format!("{}\n[... truncated ...]", &result[..8000])
                    } else { result }
                }
                Ok(Err(e)) => format!("ERROR: {}", e),
                Err(_) => "ERROR: grep timed out".to_string(),
            }
        }
        "git_status" => {
            let path = call.args["path"].as_str().unwrap_or(default_cwd);
            match tokio::process::Command::new("git")
                .args(["-C", path, "status", "--short", "--branch"])
                .output().await {
                Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
                Err(e) => format!("ERROR: {}", e),
            }
        }
        other => format!("ERROR: unknown tool '{}'", other),
    }
}

// ── Parse a tool call JSON from a line of model output ───────────────────────

fn parse_tool_call(text: &str) -> Option<ToolCall> {
    // Look for a line that parses as {"tool":..., "args":...}
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') && trimmed.contains("\"tool\"") {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let (Some(tool), Some(args)) = (
                    v["tool"].as_str(),
                    v.get("args")
                ) {
                    return Some(ToolCall {
                        tool: tool.to_string(),
                        args: args.clone(),
                    });
                }
            }
        }
    }
    None
}

// ── Main agent entry point ────────────────────────────────────────────────────

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

    let mut steps: Vec<AgentStep> = Vec::new();
    let mut messages: Vec<ChatMessage> = Vec::new();

    // System prompt (inject cwd and optional context)
    let sys = if let Some(ctx) = context {
        format!("{}\n\nCurrent working directory: {}\nContext: {}", SYSTEM_PROMPT, cwd, ctx)
    } else {
        format!("{}\n\nCurrent working directory: {}", SYSTEM_PROMPT, cwd)
    };
    messages.push(ChatMessage { role: "system".to_string(), content: sys });

    // Inject prior conversation history
    for msg in history {
        messages.push(msg);
    }

    // User message
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_message.to_string(),
    });

    // Agent loop
    for step_num in 0..MAX_STEPS {
        debug!("Agent step {}", step_num);

        let req = ChatRequest {
            model,
            messages: &messages,
            stream: false,
            options: ChatOptions {
                temperature: 0.2,
                num_predict: 4096,
            },
        };

        let url = format!("{}/api/chat", ollama_url);
        let resp = client.post(&url).json(&req).send().await
            .map_err(|e| anyhow::anyhow!("Ollama request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama error {}: {}", status, body));
        }

        let chat_resp: ChatResponse = resp.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse Ollama response: {}", e))?;

        let assistant_text = chat_resp.message.content.clone();
        info!("Agent step {}: {} chars", step_num, assistant_text.len());

        // Push assistant message to history
        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: assistant_text.clone(),
        });

        // Check for tool call
        if let Some(tool_call) = parse_tool_call(&assistant_text) {
            info!("Tool call: {} {:?}", tool_call.tool, tool_call.args);

            steps.push(AgentStep {
                kind: "tool_call".to_string(),
                content: format!("{}  {}", tool_call.tool,
                    serde_json::to_string(&tool_call.args).unwrap_or_default()),
            });

            let result = exec_tool(&tool_call, cwd).await;
            debug!("Tool result: {} chars", result.len());

            steps.push(AgentStep {
                kind: "tool_result".to_string(),
                content: result.clone(),
            });

            // Feed result back as a user message (tool result)
            messages.push(ChatMessage {
                role: "user".to_string(),
                content: format!("Tool result:\n{}", result),
            });

        } else {
            // No tool call — this is the final answer
            steps.push(AgentStep {
                kind: "answer".to_string(),
                content: assistant_text.clone(),
            });

            return Ok(AgentResponse {
                answer: assistant_text,
                steps,
            });
        }
    }

    // Hit step limit
    warn!("Agent hit step limit of {}", MAX_STEPS);
    Ok(AgentResponse {
        answer: "Agent reached maximum steps. The last tool result is shown in the steps above.".to_string(),
        steps,
    })
}

// ── Streaming agent ────────────────────────────────────────────────────────────────
// Streams tokens to the frontend as they arrive, then executes tools.
// Emits Tauri events:
//   agent-token      { session_id, token: String }          – text chunk
//   agent-tool-call  { session_id, tool, args }             – before tool runs
//   agent-tool-result{ session_id, tool, result: String }   – after tool runs
//   agent-done       { session_id, answer: String }         – final answer
//   agent-error      { session_id, error: String }          – on failure

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

#[derive(Debug, Deserialize)]
struct StreamChunk {
    message: Option<StreamMessage>,
    done: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct StreamMessage {
    content: String,
}

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
        &app, &session_id, ollama_url, model, user_message, history, cwd, context
    ).await;

    if let Err(e) = result {
        let _ = app.emit("agent-error", AgentErrorEvent {
            session_id,
            error: e.to_string(),
        });
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

    let mut messages: Vec<ChatMessage> = Vec::new();

    let sys = if let Some(ctx) = context {
        format!("{}\n\nCurrent working directory: {}\nContext: {}", SYSTEM_PROMPT, cwd, ctx)
    } else {
        format!("{}\n\nCurrent working directory: {}", SYSTEM_PROMPT, cwd)
    };
    messages.push(ChatMessage { role: "system".to_string(), content: sys });
    for msg in history { messages.push(msg); }
    messages.push(ChatMessage { role: "user".to_string(), content: user_message.to_string() });

    for step in 0..MAX_STEPS {
        debug!("Streaming agent step {}", step);

        // Build request body with stream: true
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
            "options": { "temperature": 0.2, "num_predict": 4096 }
        });

        let url = format!("{}/api/chat", ollama_url);
        let resp = client.post(&url).json(&body).send().await
            .map_err(|e| anyhow::anyhow!("Ollama request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_txt = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama error {}: {}", status, body_txt));
        }

        // Stream token chunks
        let mut stream = resp.bytes_stream();
        let mut full_response = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| anyhow::anyhow!("Stream read error: {}", e))?;
            let text = String::from_utf8_lossy(&chunk);

            // Each chunk may contain one or more newline-separated JSON objects
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() { continue; }

                if let Ok(parsed) = serde_json::from_str::<StreamChunk>(line) {
                    if let Some(msg) = parsed.message {
                        let token = msg.content;
                        if !token.is_empty() {
                            full_response.push_str(&token);
                            let _ = app.emit("agent-token", AgentTokenEvent {
                                session_id: session_id.to_string(),
                                token,
                            });
                        }
                    }
                    if parsed.done.unwrap_or(false) {
                        break;
                    }
                }
            }
        }

        info!("Streaming step {}: {} chars", step, full_response.len());

        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: full_response.clone(),
        });

        // Check for tool call in the completed response
        if let Some(tool_call) = parse_tool_call(&full_response) {
            info!("Streaming tool call: {}", tool_call.tool);

            let _ = app.emit("agent-tool-call", AgentToolEvent {
                session_id: session_id.to_string(),
                tool: tool_call.tool.clone(),
                args: serde_json::to_string(&tool_call.args).unwrap_or_default(),
            });

            let result = exec_tool(&tool_call, cwd).await;

            let _ = app.emit("agent-tool-result", AgentToolResultEvent {
                session_id: session_id.to_string(),
                tool: tool_call.tool.clone(),
                result: result.clone(),
            });

            messages.push(ChatMessage {
                role: "user".to_string(),
                content: format!("Tool result:\n{}", result),
            });

        } else {
            // No tool call — done
            let _ = app.emit("agent-done", AgentDoneEvent {
                session_id: session_id.to_string(),
                answer: full_response,
            });
            return Ok(());
        }
    }

    let _ = app.emit("agent-done", AgentDoneEvent {
        session_id: session_id.to_string(),
        answer: "Agent reached maximum steps.".to_string(),
    });
    Ok(())
}
