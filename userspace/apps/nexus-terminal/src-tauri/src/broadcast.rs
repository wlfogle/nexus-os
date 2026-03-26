use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub session_type: SessionType,
    pub status: SessionStatus,
    pub tags: Vec<String>,
    pub environment: HashMap<String, String>,
    pub working_directory: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionType {
    Local,
    Remote,
    Container,
    Wsl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Inactive,
    Error,
    Connecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResult {
    pub session_id: String,
    pub session_name: String,
    pub status: String,
    pub output: String,
    pub exit_code: i32,
    pub execution_time: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastResult {
    pub group_id: String,
    pub command: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub results: Vec<SessionResult>,
    pub overall_status: String,
    pub summary: BroadcastSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastSummary {
    pub total_sessions: usize,
    pub successful_sessions: usize,
    pub failed_sessions: usize,
    pub average_execution_time: u64,
}

#[derive(Debug)]
pub struct BroadcastManager {
    sessions: Arc<RwLock<HashMap<String, TerminalSession>>>,
    active_broadcasts: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl BroadcastManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            active_broadcasts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new terminal session for broadcasting
    pub async fn register_session(&self, session: TerminalSession) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
        Ok(())
    }

    /// Remove a terminal session
    pub async fn unregister_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }

    /// Get all registered sessions
    pub async fn get_sessions(&self) -> Result<Vec<TerminalSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }

    /// Get a specific session
    pub async fn get_session(&self, session_id: &str) -> Result<Option<TerminalSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    /// Update session status
    pub async fn update_session_status(&self, session_id: &str, status: SessionStatus) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = status;
            session.last_active = Utc::now();
        }
        Ok(())
    }

    /// Execute command on a single session
    pub async fn execute_on_session(&self, session_id: &str, command: &str) -> Result<SessionResult> {
        let session = {
            let sessions = self.sessions.read().await;
            sessions.get(session_id).cloned()
                .ok_or_else(|| anyhow!("Session not found: {}", session_id))?
        };

        let start_time = std::time::Instant::now();
        let timestamp = Utc::now();

        let result = match session.session_type {
            SessionType::Local => self.execute_local_command(command).await,
            SessionType::Remote => self.execute_remote_command(&session, command).await,
            SessionType::Container => self.execute_container_command(&session, command).await,
            SessionType::Wsl => self.execute_wsl_command(&session, command).await,
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(output) => Ok(SessionResult {
                session_id: session.id.clone(),
                session_name: session.name.clone(),
                status: "success".to_string(),
                output,
                exit_code: 0,
                execution_time,
                timestamp,
            }),
            Err(e) => Ok(SessionResult {
                session_id: session.id.clone(),
                session_name: session.name.clone(),
                status: "failed".to_string(),
                output: format!("Error: {}", e),
                exit_code: 1,
                execution_time,
                timestamp,
            }),
        }
    }

    /// Execute command on multiple sessions
    pub async fn broadcast_command(&self, session_ids: &[String], command: &str) -> Result<BroadcastResult> {
        let broadcast_id = Uuid::new_v4().to_string();
        let start_time = Utc::now();

        let mut results = Vec::new();
        
        // Start long-running broadcast tracking
        let _broadcast_handle = self.start_broadcast_tracking(broadcast_id.clone(), session_ids.to_vec()).await;
        
        // Execute on each session in parallel
        let futures: Vec<_> = session_ids.iter()
            .map(|id| self.execute_on_session(id, command))
            .collect();

        let session_results = futures::future::join_all(futures).await;

        for result in session_results {
            match result {
                Ok(session_result) => results.push(session_result),
                Err(e) => {
                    results.push(SessionResult {
                        session_id: "unknown".to_string(),
                        session_name: "unknown".to_string(),
                        status: "failed".to_string(),
                        output: format!("Error: {}", e),
                        exit_code: 1,
                        execution_time: 0,
                        timestamp: Utc::now(),
                    });
                }
            }
        }
        
        // Stop broadcast tracking
        self.stop_broadcast_tracking(&broadcast_id).await;

        let successful_sessions = results.iter().filter(|r| r.status == "success").count();
        let failed_sessions = results.len() - successful_sessions;
        let average_execution_time = if !results.is_empty() {
            results.iter().map(|r| r.execution_time).sum::<u64>() / results.len() as u64
        } else {
            0
        };

        let overall_status = if failed_sessions == 0 {
            "success"
        } else if successful_sessions == 0 {
            "failed"  
        } else {
            "partial"
        }.to_string();

        Ok(BroadcastResult {
            group_id: broadcast_id,
            command: command.to_string(),
            start_time,
            end_time: Some(Utc::now()),
            results,
            overall_status,
            summary: BroadcastSummary {
                total_sessions: session_ids.len(),
                successful_sessions,
                failed_sessions,
                average_execution_time,
            },
        })
    }
    
    /// Start tracking a broadcast operation
    async fn start_broadcast_tracking(&self, broadcast_id: String, _session_ids: Vec<String>) -> tokio::task::JoinHandle<()> {
        let active_broadcasts = self.active_broadcasts.clone();
        
        let handle = tokio::spawn(async move {
            // Simulate ongoing broadcast monitoring
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            
            for _i in 0..10 { // Monitor for 10 seconds
                interval.tick().await;
                // In a real implementation, this would track broadcast progress
            }
        });
        
        // Store the handle in active_broadcasts
        {
            let mut broadcasts = active_broadcasts.write().await;
            broadcasts.insert(broadcast_id.clone(), handle);
        }
        
        // Return a dummy handle since the real one is now owned by active_broadcasts
        tokio::spawn(async {})
    }
    
    /// Stop tracking a broadcast operation
    async fn stop_broadcast_tracking(&self, broadcast_id: &str) {
        let mut broadcasts = self.active_broadcasts.write().await;
        if let Some(handle) = broadcasts.remove(broadcast_id) {
            handle.abort();
        }
    }
    
    /// Get currently active broadcasts
    pub async fn get_active_broadcasts(&self) -> Result<Vec<String>> {
        let broadcasts = self.active_broadcasts.read().await;
        Ok(broadcasts.keys().cloned().collect())
    }

    /// Execute a local command
    async fn execute_local_command(&self, command: &str) -> Result<String> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", command])
                .output()
                .await?
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .await?
        };

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Execute a remote command via SSH
    async fn execute_remote_command(&self, session: &TerminalSession, command: &str) -> Result<String> {
        let ssh_command = if let Some(username) = &session.username {
            format!("ssh {}@{} '{}'", username, session.host, command)
        } else {
            format!("ssh {} '{}'", session.host, command)
        };

        self.execute_local_command(&ssh_command).await
    }

    /// Execute a command in a Docker container
    async fn execute_container_command(&self, session: &TerminalSession, command: &str) -> Result<String> {
        let docker_command = format!("docker exec {} {}", session.host, command);
        self.execute_local_command(&docker_command).await
    }

    /// Execute a command in WSL
    async fn execute_wsl_command(&self, session: &TerminalSession, command: &str) -> Result<String> {
        let wsl_command = format!("wsl -d {} -- {}", session.host, command);
        self.execute_local_command(&wsl_command).await
    }

    /// Import sessions from configuration
    pub async fn import_sessions(&self, sessions: Vec<TerminalSession>) -> Result<usize> {
        let mut session_map = self.sessions.write().await;
        let count = sessions.len();
        
        for session in sessions {
            session_map.insert(session.id.clone(), session);
        }
        
        Ok(count)
    }

    /// Export sessions configuration
    pub async fn export_sessions(&self) -> Result<Vec<TerminalSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }

    /// Create a default local session
    pub fn create_local_session(name: String) -> TerminalSession {
        TerminalSession {
            id: Uuid::new_v4().to_string(),
            name,
            host: "localhost".to_string(),
            port: None,
            username: None,
            session_type: SessionType::Local,
            status: SessionStatus::Active,
            tags: vec!["local".to_string()],
            environment: std::env::vars().collect(),
            working_directory: std::env::current_dir().ok()
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            created_at: Utc::now(),
            last_active: Utc::now(),
        }
    }
}

/// Global broadcast manager instance
static BROADCAST_MANAGER: once_cell::sync::Lazy<BroadcastManager> = 
    once_cell::sync::Lazy::new(|| BroadcastManager::new());

pub fn get_broadcast_manager() -> &'static BroadcastManager {
    &BROADCAST_MANAGER
}
