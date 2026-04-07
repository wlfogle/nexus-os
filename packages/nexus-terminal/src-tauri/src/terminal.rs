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

// Global app handle for event emission
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

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

#[derive(Debug)]
pub struct TerminalManager {
    terminals: Arc<Mutex<HashMap<String, Terminal>>>,
    pty_system: Arc<SyncPtySystemWrapper>,
}

impl TerminalManager {
    pub fn new() -> Self {
        let pty_system = Arc::new(SyncPtySystemWrapper {
            inner: portable_pty::native_pty_system(),
        });
        
        Self {
            terminals: Arc::new(Mutex::new(HashMap::new())),
            pty_system,
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

        info!("Created terminal with ID: {} using shell: {} in directory: {}", 
              terminal_id, shell_cmd, working_dir);
        Ok(terminal_id)
    }

    async fn start_output_reader(&self, terminal_id: &str) -> Result<()> {
        let terminals = Arc::clone(&self.terminals);
        let terminal_id = terminal_id.to_string();

        tokio::spawn(async move {
            let mut reader = {
                let terminals_guard = match terminals.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        error!("Failed to acquire terminals lock in output reader: {}", e);
                        return;
                    }
                };
                if let Some(terminal) = terminals_guard.get(&terminal_id) {
                    match terminal.master.try_clone_reader() {
                        Ok(reader) => reader,
                        Err(e) => {
                            error!("Failed to clone reader for terminal {}: {}", terminal_id, e);
                            return;
                        }
                    }
                } else {
                    error!("Terminal {} not found when starting output reader", terminal_id);
                    return;
                }
            };

            let mut buffer = [0u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        let output = String::from_utf8_lossy(&buffer[..n]);
                        debug!("Terminal {} output: {}", terminal_id, output);
                        
                        // Emit output to frontend via Tauri events
                        if let Some(app_handle) = APP_HANDLE.get() {
                            let event = TerminalOutputEvent {
                                terminal_id: terminal_id.clone(),
                                data: output.to_string(),
                            };
                            if let Err(e) = app_handle.emit("terminal-output", &event) {
                                error!("Failed to emit terminal output: {}", e);
                            }
                        }
                    }
                    Ok(_) => {
                        debug!("No data read from terminal {}", terminal_id);
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        error!("Error reading from terminal {}: {}", terminal_id, e);
                        break;
                    }
                }
            }

            info!("Output reader for terminal {} terminated", terminal_id);
        });

        Ok(())
    }

    pub async fn write_to_terminal(&self, terminal_id: &str, data: &str) -> Result<()> {
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
            // Terminal will be dropped and cleaned up automatically
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

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}
