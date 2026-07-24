// terminal.rs — PTY management + block lifecycle integration for NexusTerminal.
//
// Block-tracking logic (Warp-style) is ported from warpdotdev/warp (AGPL-3.0):
// https://github.com/warpdotdev/warp
//
// `blocks` is declared here with #[path] so that `cargo check` compiles it as
// part of this module.  The integrator must also add `mod blocks;` to main.rs
// (and change this declaration to `use crate::blocks;`) once all four parallel
// branches are merged.

// Declare blocks.rs as a pub sub-module so the file is compiled even before
// main.rs grows its own `mod blocks;` declaration.
#[path = "blocks.rs"]
pub mod blocks;

use anyhow::{Context, Result};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, PtySystem};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info};
use uuid::Uuid;
use tauri::{AppHandle, Emitter};

// Global app handle for event emission — pub so agent.rs can emit ask_user questions
pub static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// Initialize the global app handle for event emission
pub fn init_app_handle(app_handle: AppHandle) {
    let _ = APP_HANDLE.set(app_handle);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInfo {
    pub id: String,
    pub shell: String,
    pub cwd: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

struct Terminal {
    _child: Box<dyn Child + Send + Sync>,
    master: Box<dyn MasterPty + Send>,
    info: TerminalInfo,
}

// Manual Debug implementation since Child and MasterPty don't implement Debug
impl std::fmt::Debug for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Terminal")
            .field("info", &self.info)
            .finish_non_exhaustive()
    }
}

// Wrapper to make PtySystem + Send + Sync
struct SyncPtySystemWrapper {
    inner: Box<dyn portable_pty::PtySystem + Send>,
}

impl std::fmt::Debug for SyncPtySystemWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncPtySystemWrapper")
            .finish_non_exhaustive()
    }
}

unsafe impl Sync for SyncPtySystemWrapper {}

impl PtySystem for SyncPtySystemWrapper {
    fn openpty(&self, size: PtySize) -> Result<portable_pty::PtyPair, anyhow::Error> {
        self.inner.openpty(size)
    }
}

// ─── OSC sequence parser ───────────────────────────────────────────────────

/// Recognised OSC (Operating System Command) events from the PTY byte stream.
#[derive(Debug)]
enum OscEvent {
    /// \x1b]133;A\x07 — shell prompt is about to be displayed (command done)
    PromptStart,
    /// \x1b]133;B\x07 or \x1b]133;B;cmdline=CMD\x07 — command about to execute
    CommandStart(String),
    /// \x1b]133;D;N\x07 — command finished with exit code N (or None)
    CommandEnd(Option<i32>),
    /// \x1b]7;file://host/path\x07 — shell reported the current working directory
    Cwd(String),
}

/// Scan `raw` for OSC escape sequences, strip them out, and return both the
/// cleaned text and a list of parsed events in encounter order.
///
/// OSC sequences have the form  `\x1b]CONTENT\x07`  (BEL-terminated) or
/// `\x1b]CONTENT\x1b\\`  (ST-terminated).  ANSI colour / CSI sequences
/// (`\x1b[...`) are left intact so the frontend terminal emulator can render
/// them normally.
fn extract_osc_events(raw: &str) -> (String, Vec<OscEvent>) {
    let mut stripped = String::with_capacity(raw.len());
    let mut events = Vec::new();
    let mut remaining = raw;

    while !remaining.is_empty() {
        match remaining.find("\x1b]") {
            None => {
                stripped.push_str(remaining);
                break;
            }
            Some(pos) => {
                // Append text before this OSC sequence
                stripped.push_str(&remaining[..pos]);
                remaining = &remaining[pos + 2..]; // consume \x1b]

                // Locate the BEL (\x07) or ST (\x1b\\) terminator
                let term = remaining
                    .find('\x07')
                    .map(|i| (i, i + 1))
                    .or_else(|| remaining.find("\x1b\\\\").map(|i| (i, i + 2)));

                match term {
                    None => {
                        // Unterminated / partial sequence at end of buffer — drop it
                        break;
                    }
                    Some((content_end, skip_to)) => {
                        let content = &remaining[..content_end];
                        parse_osc_content(content, &mut events);
                        remaining = &remaining[skip_to..];
                    }
                }
            }
        }
    }

    (stripped, events)
}

/// Classify a single OSC payload string (the bytes between `\x1b]` and `\x07`).
fn parse_osc_content(content: &str, events: &mut Vec<OscEvent>) {
    if let Some(rest) = content.strip_prefix("133;") {
        if rest == "A" {
            events.push(OscEvent::PromptStart);
        } else if rest == "B" {
            events.push(OscEvent::CommandStart(String::new()));
        } else if let Some(cmd) = rest.strip_prefix("B;cmdline=") {
            events.push(OscEvent::CommandStart(cmd.to_string()));
        } else if rest == "D" {
            events.push(OscEvent::CommandEnd(None));
        } else if let Some(code_str) = rest.strip_prefix("D;") {
            events.push(OscEvent::CommandEnd(code_str.parse::<i32>().ok()));
        }
    } else if let Some(uri) = content.strip_prefix("7;") {
        if let Some(cwd) = parse_osc7_uri(uri) {
            events.push(OscEvent::Cwd(cwd));
        }
    }
}

/// Decode the path from an OSC 7 `file://hostname/absolute/path` URI.
fn parse_osc7_uri(uri: &str) -> Option<String> {
    let after_scheme = uri.strip_prefix("file://")?;
    let path = if let Some(slash) = after_scheme.find('/') {
        &after_scheme[slash..]
    } else {
        return None;
    };
    if path.is_empty() || !path.starts_with('/') {
        return None;
    }
    Some(
        path.replace("%20", " ")
            .replace("%21", "!")
            .replace("%23", "#")
            .replace("%24", "$")
            .replace("%25", "%")
            .replace("%26", "&")
            .replace("%27", "'")
            .replace("%28", "(")
            .replace("%29", ")")
            .replace("%2B", "+")
            .replace("%2C", ",")
            .replace("%3D", "=")
            .replace("%40", "@")
            .replace("%5B", "[")
            .replace("%5D", "]"),
    )
}

// ─── Per-terminal mutable state ────────────────────────────────────────────

/// Runtime state tracked for each live terminal (separate from the PTY handle).
#[derive(Debug, Default)]
struct TerminalState {
    /// Characters the user has typed since the last Enter keypress.
    input_buffer: String,
    /// The last complete command line (captured when a newline is written).
    /// Consumed by the OSC 133;B handler to populate `block:start.command`.
    pending_command: Option<String>,
    /// Most recent working directory, updated via OSC 7 notifications.
    cwd: String,
}

// ─── TerminalManager ───────────────────────────────────────────────────────

#[derive(Debug)]
pub struct TerminalManager {
    terminals: Arc<Mutex<HashMap<String, Terminal>>>,
    pty_system: Arc<SyncPtySystemWrapper>,
    /// Per-terminal runtime state (input tracking, cwd, pending command).
    terminal_states: Arc<Mutex<HashMap<String, TerminalState>>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        let pty_system = Arc::new(SyncPtySystemWrapper {
            inner: portable_pty::native_pty_system(),
        });

        Self {
            terminals: Arc::new(Mutex::new(HashMap::new())),
            pty_system,
            terminal_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_terminal(&mut self, shell: Option<String>) -> Result<String> {
        self.create_terminal_with_config(shell, None, None, None).await
    }

    pub async fn create_terminal_with_config(
        &mut self,
        shell: Option<String>,
        args: Option<Vec<String>>,
        cwd: Option<String>,
        env: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let terminal_id = Uuid::new_v4().to_string();
        
        // Determine shell
        let shell_cmd = shell.unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| {
                if cfg!(windows) {
                    "powershell.exe".to_string()
                } else {
                    "/bin/bash".to_string()
                }
            })
        });

        // Determine working directory
        let working_dir = cwd.unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("/"))
                .to_string_lossy()
                .to_string()
        });

        // Expand ~ to home directory if needed
        let working_dir = if working_dir.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                working_dir.replacen('~', home.to_string_lossy().as_ref(), 1)
            } else {
                working_dir
            }
        } else {
            working_dir
        };

        // Create PTY
        let pty_pair = self.pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to create PTY")?;

        // Build command
        let mut cmd = CommandBuilder::new(&shell_cmd);
        
        // Add shell arguments if provided
        if let Some(shell_args) = args {
            for arg in shell_args {
                cmd.arg(&arg);
            }
        } else {
            // Default arguments based on shell type
            if shell_cmd.contains("bash") {
                cmd.arg("--login");
            } else if shell_cmd.contains("zsh") {
                cmd.arg("-l");
            } else if shell_cmd.contains("fish") {
                cmd.arg("--login");
            }
        }

        // Set environment variables
        if !cfg!(windows) {
            cmd.env("TERM", "xterm-256color");
        }
        
        if let Some(environment) = env {
            for (key, value) in environment {
                cmd.env(&key, &value);
            }
        }

        // Set working directory
        cmd.cwd(&working_dir);

        // Spawn process
        let child = pty_pair.slave
            .spawn_command(cmd)
            .context("Failed to spawn shell process")?;

        let terminal_info = TerminalInfo {
            id: terminal_id.clone(),
            shell: shell_cmd.clone(),
            cwd: working_dir.clone(),
            created_at: chrono::Utc::now(),
        };

        let terminal = Terminal {
            _child: child,
            master: pty_pair.master,
            info: terminal_info,
        };

        // Initialise per-terminal state BEFORE starting the output reader so it
        // is always present when the first OSC sequences arrive.
        {
            let mut states = self.terminal_states.lock()
                .unwrap_or_else(|e| e.into_inner());
            states.insert(terminal_id.clone(), TerminalState {
                input_buffer: String::new(),
                pending_command: None,
                cwd: working_dir.clone(),
            });
        }

        // Store terminal
        {
            let mut terminals = match self.terminals.lock() {
                Ok(terminals) => terminals,
                Err(e) => {
                    error!("Failed to acquire terminal lock: {}", e);
                    return Err(anyhow::anyhow!("Terminal lock poisoned"));
                }
            };
            terminals.insert(terminal_id.clone(), terminal);
        }

        // Start reading output in a separate thread
        self.start_output_reader(&terminal_id).await?;

        // Inject OSC 133 bootstrap so the shell emits command-boundary sequences.
        // A brief sleep lets the shell finish its startup (load rc files) before
        // we write to its stdin; 50 ms is negligible for UX.
        {
            let bootstrap = osc133_bootstrap(&shell_cmd);
            let tid = terminal_id.clone();
            let terminals_arc = Arc::clone(&self.terminals);
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let guard = terminals_arc.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(t) = guard.get(&tid) {
                    match t.master.take_writer() {
                        Ok(mut w) => {
                            let _ = w.write_all(bootstrap.as_bytes());
                            let _ = w.flush();
                            debug!("Injected OSC 133 bootstrap for terminal {}", tid);
                        }
                        Err(e) => error!("Bootstrap writer error on terminal {}: {}", tid, e),
                    }
                }
            });
        }

        info!("Created terminal {} shell={} cwd={}", terminal_id, shell_cmd, working_dir);
        Ok(terminal_id)
    }

    async fn start_output_reader(&self, terminal_id: &str) -> Result<()> {
        let terminals = Arc::clone(&self.terminals);
        let terminal_states = Arc::clone(&self.terminal_states);
        let terminal_id = terminal_id.to_string();

        tokio::spawn(async move {
            // Grab a cloned reader handle from the PTY master.
            let mut reader = {
                let guard = match terminals.lock() {
                    Ok(g) => g,
                    Err(e) => {
                        error!("terminals lock poisoned in output reader: {}", e);
                        return;
                    }
                };
                match guard.get(&terminal_id) {
                    Some(t) => match t.master.try_clone_reader() {
                        Ok(r) => r,
                        Err(e) => {
                            error!("clone reader failed for terminal {}: {}", terminal_id, e);
                            return;
                        }
                    },
                    None => {
                        error!("Terminal {} not found when starting output reader", terminal_id);
                        return;
                    }
                }
            };

            let block_tracker = blocks::get_block_tracker();
            let mut buffer = [0u8; 8192];

            loop {
                match reader.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        let raw = String::from_utf8_lossy(&buffer[..n]).to_string();
                        debug!("Terminal {} raw output ({} bytes)", terminal_id, n);

                        // Parse all OSC sequences from this chunk.
                        let (stripped, events) = extract_osc_events(&raw);

                        // Process semantic events FIRST (so blocks are started before
                        // we emit block:output for content in the same chunk).
                        for event in &events {
                            match event {
                                OscEvent::Cwd(cwd) => {
                                    // Update per-terminal CWD.
                                    {
                                        let mut states = terminal_states
                                            .lock()
                                            .unwrap_or_else(|e| e.into_inner());
                                        if let Some(state) = states.get_mut(&terminal_id) {
                                            state.cwd = cwd.clone();
                                        }
                                    }
                                    // Notify the frontend.
                                    if let Some(app_handle) = APP_HANDLE.get() {
                                        let _ = app_handle.emit(
                                            "terminal-cwd",
                                            serde_json::json!({
                                                "terminal_id": terminal_id,
                                                "cwd": cwd,
                                            }),
                                        );
                                    }
                                }

                                OscEvent::CommandStart(cmd_text) => {
                                    if let Some(app_handle) = APP_HANDLE.get() {
                                        // Resolve command: prefer text from the OSC sequence
                                        // (fish provides it via fish_preexec $argv[1]);  fall
                                        // back to the pending_command captured at write time.
                                        let (command, cwd) = {
                                            let mut states = terminal_states
                                                .lock()
                                                .unwrap_or_else(|e| e.into_inner());
                                            if let Some(state) = states.get_mut(&terminal_id) {
                                                let cmd = if !cmd_text.is_empty() {
                                                    cmd_text.clone()
                                                } else {
                                                    state.pending_command
                                                        .take()
                                                        .unwrap_or_default()
                                                };
                                                (cmd, state.cwd.clone())
                                            } else {
                                                (cmd_text.clone(), String::new())
                                            }
                                        };
                                        block_tracker.start_block(
                                            &terminal_id,
                                            &command,
                                            &cwd,
                                            app_handle,
                                        );
                                    }
                                }

                                OscEvent::CommandEnd(exit_code) => {
                                    if let Some(app_handle) = APP_HANDLE.get() {
                                        block_tracker.end_block(
                                            &terminal_id,
                                            *exit_code,
                                            app_handle,
                                        );
                                    }
                                }

                                OscEvent::PromptStart => {
                                    // 133;A fires after 133;D so end_block is usually already
                                    // a no-op here.  We call it anyway as a safety net for
                                    // shells that don't emit 133;D (e.g. if the user aborts
                                    // the bootstrap injection).
                                    if let Some(app_handle) = APP_HANDLE.get() {
                                        if block_tracker.has_active_block(&terminal_id) {
                                            block_tracker.end_block(
                                                &terminal_id,
                                                None,
                                                app_handle,
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(app_handle) = APP_HANDLE.get() {
                            // Always forward the raw bytes to the frontend terminal
                            // emulator (xterm.js / similar) — it handles ANSI & OSC.
                            let event = TerminalOutputEvent {
                                terminal_id: terminal_id.clone(),
                                data: raw.clone(),
                            };
                            if let Err(e) = app_handle.emit("terminal-output", &event) {
                                error!("Failed to emit terminal-output: {}", e);
                            }

                            // Forward OSC-stripped content as block:output (ANSI colour
                            // codes are preserved; only the OSC sequences are stripped).
                            if !stripped.is_empty() {
                                block_tracker.emit_output(
                                    &terminal_id,
                                    &stripped,
                                    "stdout",
                                    app_handle,
                                );
                            }
                        }
                    }

                    Ok(_) => {
                        // Zero-byte read — PTY returned nothing yet; yield briefly.
                        thread::sleep(Duration::from_millis(10));
                    }

                    Err(e) => {
                        error!("Read error on terminal {}: {}", terminal_id, e);
                        break;
                    }
                }
            }

            // Clean up any in-flight block when the PTY dies.
            if let Some(app_handle) = APP_HANDLE.get() {
                block_tracker.end_block(&terminal_id, None, app_handle);
            }

            info!("Output reader for terminal {} terminated", terminal_id);
        });

        Ok(())
    }

    pub async fn write_to_terminal(&self, terminal_id: &str, data: &str) -> Result<()> {
        // ── Track user input for block:start command resolution ──────────────
        // We snapshot what the user types so that when OSC 133;B fires
        // (preexec hook) we know which command was submitted.
        {
            let mut states = self.terminal_states
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Some(state) = states.get_mut(terminal_id) {
                let has_newline = data.contains('\n') || data.contains('\r');
                if has_newline {
                    // User pressed Enter — capture accumulated input as pending command.
                    let cmd = state.input_buffer.trim().to_string();
                    if !cmd.is_empty() {
                        state.pending_command = Some(cmd);
                    }
                    state.input_buffer.clear();
                } else {
                    // Accumulate printable input (backspace / Ctrl chars are left in
                    // the buffer; the OSC 133;B cmdline from fish takes priority anyway).
                    state.input_buffer.push_str(data);
                    // Bound the buffer so it never grows unbounded.
                    if state.input_buffer.len() > 16_384 {
                        let drain_to = state.input_buffer.len() - 8_192;
                        state.input_buffer.drain(..drain_to);
                    }
                }
            }
        } // Release terminal_states lock before acquiring terminals lock

        // ── Write to PTY ─────────────────────────────────────────────────────
        let terminals = self.terminals.lock()
            .map_err(|_| anyhow::anyhow!("Terminal lock poisoned"))?;

        if let Some(terminal) = terminals.get(terminal_id) {
            let mut writer = terminal.master.take_writer()
                .context("Failed to get terminal writer")?;

            writer.write_all(data.as_bytes())
                .context("Failed to write to terminal")?;

            writer.flush()
                .context("Failed to flush terminal writer")?;

            debug!("Wrote {} bytes to terminal {}", data.len(), terminal_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Terminal {} not found", terminal_id))
        }
    }

    pub async fn resize_terminal(&self, terminal_id: &str, cols: u16, rows: u16) -> Result<()> {
        let terminals = self.terminals.lock()
            .map_err(|_| anyhow::anyhow!("Terminal lock poisoned"))?;
        
        if let Some(terminal) = terminals.get(terminal_id) {
            let new_size = PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            };
            
            terminal.master.resize(new_size)
                .context("Failed to resize terminal")?;
            
            debug!("Resized terminal {} to {}x{}", terminal_id, cols, rows);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Terminal {} not found", terminal_id))
        }
    }

    pub async fn kill_terminal(&mut self, terminal_id: &str) -> Result<()> {
        let mut terminals = self.terminals.lock()
            .map_err(|_| anyhow::anyhow!("Terminal lock poisoned"))?;

        if let Some(_terminal) = terminals.remove(terminal_id) {
            // Terminal will be dropped and cleaned up automatically.
            // Also remove per-terminal state.
            {
                let mut states = self.terminal_states
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                states.remove(terminal_id);
            }
            info!("Killed terminal {}", terminal_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Terminal {} not found", terminal_id))
        }
    }

    pub fn get_terminal_info(&self, terminal_id: &str) -> Option<TerminalInfo> {
        let terminals = self.terminals.lock().ok()?;
        terminals.get(terminal_id).map(|t| t.info.clone())
    }

    pub fn list_terminals(&self) -> Vec<TerminalInfo> {
        match self.terminals.lock() {
            Ok(terminals) => terminals.values().map(|t| t.info.clone()).collect(),
            Err(_) => {
                error!("Failed to acquire terminal lock in list_terminals");
                Vec::new()
            }
        }
    }

    pub fn get_terminal_count(&self) -> usize {
        match self.terminals.lock() {
            Ok(terminals) => terminals.len(),
            Err(_) => {
                error!("Failed to acquire terminal lock in get_terminal_count");
                0
            }
        }
    }
}

// Events for frontend communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutputEvent {
    pub terminal_id: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalExitEvent {
    pub terminal_id: String,
    pub exit_code: Option<i32>,
}

/// Generate shell-specific OSC 133 (FTCS) hook commands.
///
/// These hooks make the shell emit command-boundary escape sequences so
/// terminal.rs can drive the block lifecycle.  The technique is the same
/// as Warp (warpdotdev/warp, AGPL-3.0):
///
/// * `133;B;cmdline=CMD` — preexec (command about to run, with text)
/// * `133;D;N`           — postexec (command finished, N = exit code)
/// * `133;A`             — prompt start
///
/// The injection is wrapped in `stty -echo / stty echo` so the function
/// definitions never appear visibly in the terminal window.
fn osc133_bootstrap(shell: &str) -> String {
    if shell.contains("fish") {
        // fish exposes $status (exit code) in fish_prompt and $argv[1]
        // (command text) in fish_preexec.
        concat!(
            "stty -echo; ",
            "function __osc133_prompt --on-event fish_prompt; ",
            "printf '\x1b]133;D;%d\x07' $status; ",
            "printf '\x1b]133;A\x07'; ",
            "end; ",
            "function __osc133_preexec --on-event fish_preexec; ",
            "printf '\x1b]133;B;cmdline=%s\x07' \"$argv[1]\"; ",
            "end; ",
            "stty echo\n",
        ).to_string()
    } else if shell.contains("zsh") {
        // zsh exposes $? in precmd and $1 in preexec.
        concat!(
            "stty -echo; ",
            "precmd() { ",
            "printf \"\x1b]133;D;$?\x07\"; ",
            "printf \"\x1b]133;A\x07\"; ",
            "}; ",
            "preexec() { printf \"\x1b]133;B;cmdline=%s\x07\" \"$1\"; }; ",
            "stty echo\n",
        ).to_string()
    } else {
        // bash / sh: PS0 fires before each command; PROMPT_COMMAND fires after.
        // $BASH_COMMAND in PS0 gives the command text.
        concat!(
            "stty -echo; ",
            "PROMPT_COMMAND='printf \"\x1b]133;D;$?\x07\x1b]133;A\x07\"'; ",
            "PS0='\x1b]133;B\x07'; ",
            "stty echo\n",
        ).to_string()
    }
}

/// Extract the working directory from an OSC 7 escape sequence.
/// Fish, zsh, and bash all emit \x1b]7;file://hostname/absolute/path\x07
/// on every prompt display, giving us a reliable cwd signal without polling.
fn extract_osc7_cwd(output: &str) -> Option<String> {
    let marker = "\x1b]7;";
    let start = output.find(marker)?;
    let rest = &output[start + marker.len()..];
    // Sequence ends with BEL (\x07) or ST (\x1b\\)
    let end = rest.find('\x07')
        .or_else(|| rest.find("\x1b\\\\"))
        .unwrap_or_else(|| rest.len().min(512));
    let uri = &rest[..end];
    // URI format: file://hostname/absolute/path
    // Strip "file://" then skip the hostname up to the first "/"
    let after_scheme = uri.strip_prefix("file://")?;
    let path = if let Some(slash) = after_scheme.find('/') {
        &after_scheme[slash..]
    } else {
        return None;
    };
    if path.is_empty() || !path.starts_with('/') {
        return None;
    }
    // Decode common percent-encoded chars (spaces are the main one on Linux)
    let decoded = path.replace("%20", " ")
        .replace("%21", "!")
        .replace("%23", "#")
        .replace("%24", "$")
        .replace("%25", "%")
        .replace("%26", "&")
        .replace("%27", "'")
        .replace("%28", "(")
        .replace("%29", ")")
        .replace("%2B", "+")
        .replace("%2C", ",")
        .replace("%3D", "=")
        .replace("%40", "@")
        .replace("%5B", "[")
        .replace("%5D", "]");
    Some(decoded)
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::extract_osc7_cwd;

    #[test]
    fn osc7_fish_standard_format() {
        // Fish emits \x1b]7;file://hostname/path\x07 on every prompt
        let out = "\x1b]7;file://pop-os/home/loufogle/nexus-os\x07";
        assert_eq!(extract_osc7_cwd(out), Some("/home/loufogle/nexus-os".to_string()));
    }

    #[test]
    fn osc7_embedded_in_other_output() {
        let out = "\x1b[32muser@host\x1b[0m \x1b]7;file://host/tmp/work\x07$ ";
        assert_eq!(extract_osc7_cwd(out), Some("/tmp/work".to_string()));
    }

    #[test]
    fn osc7_root_path() {
        let out = "\x1b]7;file://host/\x07";
        assert_eq!(extract_osc7_cwd(out), Some("/".to_string()));
    }

    #[test]
    fn osc7_percent_encoded_space() {
        let out = "\x1b]7;file://host/home/user/my%20documents\x07";
        assert_eq!(extract_osc7_cwd(out), Some("/home/user/my documents".to_string()));
    }

    #[test]
    fn osc7_absent_returns_none() {
        assert_eq!(extract_osc7_cwd("normal output with no escape"), None);
        assert_eq!(extract_osc7_cwd(""), None);
    }

    #[test]
    fn osc7_rejects_relative_path() {
        // Malformed: path doesn't start with /
        let out = "\x1b]7;file://hostsomepath\x07";
        assert_eq!(extract_osc7_cwd(out), None);
    }

    #[test]
    fn osc7_nested_deep_path() {
        let out = "\x1b]7;file://pop-os/home/loufogle/nexus-os/packages/nexus-terminal/src-tauri/src\x07";
        assert_eq!(
            extract_osc7_cwd(out),
            Some("/home/loufogle/nexus-os/packages/nexus-terminal/src-tauri/src".to_string())
        );
    }
}
