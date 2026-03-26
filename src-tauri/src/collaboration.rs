use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tokio::sync::{RwLock, broadcast};
use std::sync::Arc;

// Missing types expected by main.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinResult {
    pub success: bool,
    pub session_id: String,
    pub participant_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPermissions {
    pub can_read: bool,
    pub can_write: bool,
    pub can_execute: bool,
    pub can_resize: bool,
    pub can_control: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Option<String>,
    pub author_id: String,
    pub content: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub message_type: MessageType,
}

// Alias for main.rs compatibility
pub type CollaborationSession = CollaborativeSession;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborativeSession {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub participants: Vec<Participant>,
    pub permissions: SessionPermissions,
    pub status: SessionStatus,
    pub terminal_id: Option<String>,
    pub shared_state: SharedState,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub settings: SessionSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub user_id: String,
    pub username: String,
    pub role: ParticipantRole,
    pub permissions: UserPermissions,
    pub status: ParticipantStatus,
    pub joined_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub cursor_position: Option<CursorPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantRole {
    Owner,
    Admin,
    Editor,
    Viewer,
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    pub can_execute_commands: bool,
    pub can_edit_files: bool,
    pub can_view_screen: bool,
    pub can_share_screen: bool,
    pub can_manage_participants: bool,
    pub can_change_settings: bool,
    pub allowed_commands: Option<Vec<String>>,
    pub restricted_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantStatus {
    Online,
    Away,
    Busy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Paused,
    Ended,
    Waiting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPermissions {
    pub is_public: bool,
    pub allow_anonymous: bool,
    pub max_participants: u32,
    pub require_approval: bool,
    pub allow_recording: bool,
    pub password_protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedState {
    pub current_directory: String,
    pub environment_variables: HashMap<String, String>,
    pub active_processes: Vec<ProcessInfo>,
    pub shared_files: Vec<SharedFile>,
    pub terminal_history: Vec<TerminalEvent>,
    pub screen_sharing: Option<ScreenShare>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub status: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFile {
    pub path: String,
    pub content: String,
    pub last_modified: DateTime<Utc>,
    pub modified_by: String,
    pub version: u32,
    pub lock_holder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalEvent {
    pub id: String,
    pub session_id: String,
    pub user_id: String,
    pub event_type: TerminalEventType,
    pub timestamp: DateTime<Utc>,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminalEventType {
    Command,
    Output,
    Input,
    KeyPress,
    Resize,
    CursorMove,
    Selection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenShare {
    pub is_active: bool,
    pub sharer_id: String,
    pub viewers: Vec<String>,
    pub quality: ScreenQuality,
    pub frame_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreenQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    pub auto_save_interval: u32,
    pub sync_cursor: bool,
    pub sync_selection: bool,
    pub show_participants: bool,
    pub enable_voice_chat: bool,
    pub enable_video_chat: bool,
    pub record_session: bool,
    pub notifications_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: u32,
    pub column: u32,
    pub selection_start: Option<(u32, u32)>,
    pub selection_end: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationEvent {
    pub id: String,
    pub session_id: String,
    pub user_id: String,
    pub event_type: CollaborationEventType,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationEventType {
    UserJoined,
    UserLeft,
    MessageSent,
    FileShared,
    ScreenShareStarted,
    ScreenShareStopped,
    CursorMoved,
    SelectionChanged,
    CommandExecuted,
    PermissionChanged,
    SettingUpdated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamWorkspace {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner_id: String,
    pub members: Vec<TeamMember>,
    pub projects: Vec<Project>,
    pub channels: Vec<Channel>,
    pub integrations: Vec<Integration>,
    pub settings: WorkspaceSettings,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub role: TeamRole,
    pub status: MemberStatus,
    pub joined_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TeamRole {
    Owner,
    Admin,
    Developer,
    Tester,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberStatus {
    Active,
    Inactive,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub repository_url: Option<String>,
    pub local_path: String,
    pub assigned_members: Vec<String>,
    pub status: ProjectStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    OnHold,
    Completed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub channel_type: ChannelType,
    pub members: Vec<String>,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelType {
    General,
    Project(String),
    Private,
    Announcement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub author_id: String,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub reactions: Vec<Reaction>,
    pub thread_replies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Code,
    File,
    Image,
    Command,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub users: Vec<String>,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    pub id: String,
    pub name: String,
    pub integration_type: IntegrationType,
    pub config: HashMap<String, String>,
    pub enabled: bool,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationType {
    Git,
    Slack,
    Discord,
    Jira,
    Trello,
    GitHub,
    GitLab,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    pub default_permissions: UserPermissions,
    pub session_timeout: u32,
    pub auto_cleanup: bool,
    pub notification_settings: NotificationSettings,
    pub security_settings: SecuritySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_notifications: bool,
    pub desktop_notifications: bool,
    pub webhook_notifications: bool,
    pub notify_on_join: bool,
    pub notify_on_leave: bool,
    pub notify_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub require_2fa: bool,
    pub session_encryption: bool,
    pub audit_logging: bool,
    pub ip_whitelist: Vec<String>,
    pub max_session_duration: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CollaborationManager {
    sessions: Arc<RwLock<HashMap<String, CollaborativeSession>>>,
    workspaces: Arc<RwLock<HashMap<String, TeamWorkspace>>>,
    event_sender: broadcast::Sender<CollaborationEvent>,
    active_connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConnectionInfo {
    pub user_id: String,
    pub session_id: String,
    pub ip_address: String,
    pub connected_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

#[allow(dead_code)]
impl CollaborationManager {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            active_connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }



    pub async fn leave_session(&self, session_id: &str, user_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.participants.retain(|p| p.user_id != user_id);
            session.last_activity = Utc::now();

            self.broadcast_event(CollaborationEvent {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: session_id.to_string(),
                user_id: user_id.to_string(),
                event_type: CollaborationEventType::UserLeft,
                timestamp: Utc::now(),
                data: serde_json::json!({}),
            }).await?;

            // Clean up empty sessions
            if session.participants.is_empty() {
                sessions.remove(session_id);
            }

            Ok(())
        } else {
            Err(anyhow!("Session not found"))
        }
    }

    pub async fn execute_command(
        &self,
        session_id: &str,
        user_id: &str,
        command: &str,
    ) -> Result<String> {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(session_id) {
            // Check if user has permission to execute commands
            if let Some(participant) = session.participants.iter().find(|p| p.user_id == user_id) {
                if !participant.permissions.can_execute_commands {
                    return Err(anyhow!("User does not have permission to execute commands"));
                }

                // Check if command is allowed
                if let Some(allowed_commands) = &participant.permissions.allowed_commands {
                    let command_name = command.split_whitespace().next().unwrap_or("");
                    if !allowed_commands.contains(&command_name.to_string()) {
                        return Err(anyhow!("Command not allowed"));
                    }
                }

                // Execute command (simplified implementation)
                let output = format!("Executed: {}", command);

                self.broadcast_event(CollaborationEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    user_id: user_id.to_string(),
                    event_type: CollaborationEventType::CommandExecuted,
                    timestamp: Utc::now(),
                    data: serde_json::json!({ "command": command, "output": &output }),
                }).await?;

                Ok(output)
            } else {
                Err(anyhow!("User not found in session"))
            }
        } else {
            Err(anyhow!("Session not found"))
        }
    }

    pub async fn share_file(
        &self,
        session_id: &str,
        user_id: &str,
        file_path: &str,
        content: &str,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(participant) = session.participants.iter().find(|p| p.user_id == user_id) {
                if !participant.permissions.can_edit_files {
                    return Err(anyhow!("User does not have permission to share files"));
                }

                let shared_file = SharedFile {
                    path: file_path.to_string(),
                    content: content.to_string(),
                    last_modified: Utc::now(),
                    modified_by: user_id.to_string(),
                    version: 1,
                    lock_holder: None,
                };

                session.shared_state.shared_files.push(shared_file);
                session.last_activity = Utc::now();

                self.broadcast_event(CollaborationEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    user_id: user_id.to_string(),
                    event_type: CollaborationEventType::FileShared,
                    timestamp: Utc::now(),
                    data: serde_json::json!({ "file_path": file_path }),
                }).await?;

                Ok(())
            } else {
                Err(anyhow!("User not found in session"))
            }
        } else {
            Err(anyhow!("Session not found"))
        }
    }

    pub async fn start_screen_share(
        &self,
        session_id: &str,
        user_id: &str,
        quality: ScreenQuality,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(participant) = session.participants.iter().find(|p| p.user_id == user_id) {
                if !participant.permissions.can_share_screen {
                    return Err(anyhow!("User does not have permission to share screen"));
                }

                session.shared_state.screen_sharing = Some(ScreenShare {
                    is_active: true,
                    sharer_id: user_id.to_string(),
                    viewers: vec![],
                    quality: quality.clone(),
                    frame_rate: 30,
                });

                session.last_activity = Utc::now();

                self.broadcast_event(CollaborationEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    user_id: user_id.to_string(),
                    event_type: CollaborationEventType::ScreenShareStarted,
                    timestamp: Utc::now(),
                    data: serde_json::json!({ "quality": format!("{:?}", quality) }),
                }).await?;

                Ok(())
            } else {
                Err(anyhow!("User not found in session"))
            }
        } else {
            Err(anyhow!("Session not found"))
        }
    }

    pub async fn stop_screen_share(&self, session_id: &str, user_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(screen_share) = &session.shared_state.screen_sharing {
                if screen_share.sharer_id != user_id {
                    return Err(anyhow!("User is not the screen sharer"));
                }
            }

            session.shared_state.screen_sharing = None;
            session.last_activity = Utc::now();

            self.broadcast_event(CollaborationEvent {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: session_id.to_string(),
                user_id: user_id.to_string(),
                event_type: CollaborationEventType::ScreenShareStopped,
                timestamp: Utc::now(),
                data: serde_json::json!({}),
            }).await?;

            Ok(())
        } else {
            Err(anyhow!("Session not found"))
        }
    }

    pub async fn update_cursor_position(
        &self,
        session_id: &str,
        user_id: &str,
        position: CursorPosition,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(participant) = session.participants.iter_mut().find(|p| p.user_id == user_id) {
                participant.cursor_position = Some(position.clone());
                participant.last_seen = Utc::now();

                if session.settings.sync_cursor {
                    self.broadcast_event(CollaborationEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        session_id: session_id.to_string(),
                        user_id: user_id.to_string(),
                        event_type: CollaborationEventType::CursorMoved,
                        timestamp: Utc::now(),
                        data: serde_json::to_value(&position).unwrap_or_default(),
                    }).await?;
                }

                Ok(())
            } else {
                Err(anyhow!("User not found in session"))
            }
        } else {
            Err(anyhow!("Session not found"))
        }
    }

    pub async fn get_session(&self, session_id: &str) -> Result<CollaborativeSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned().ok_or_else(|| anyhow!("Session not found"))
    }

    pub async fn list_sessions(&self) -> Vec<CollaborativeSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn create_workspace(
        &self,
        name: String,
        description: String,
        owner_id: String,
    ) -> Result<String> {
        let workspace_id = uuid::Uuid::new_v4().to_string();
        
        let workspace = TeamWorkspace {
            id: workspace_id.clone(),
            name,
            description,
            owner_id: owner_id.clone(),
            members: vec![TeamMember {
                user_id: owner_id,
                username: "Owner".to_string(),
                email: "owner@example.com".to_string(),
                role: TeamRole::Owner,
                status: MemberStatus::Active,
                joined_at: Utc::now(),
                last_active: Utc::now(),
            }],
            projects: vec![],
            channels: vec![],
            integrations: vec![],
            settings: WorkspaceSettings {
                default_permissions: UserPermissions {
                    can_execute_commands: true,
                    can_edit_files: true,
                    can_view_screen: true,
                    can_share_screen: false,
                    can_manage_participants: false,
                    can_change_settings: false,
                    allowed_commands: None,
                    restricted_paths: vec![],
                },
                session_timeout: 3600,
                auto_cleanup: true,
                notification_settings: NotificationSettings {
                    email_notifications: true,
                    desktop_notifications: true,
                    webhook_notifications: false,
                    notify_on_join: true,
                    notify_on_leave: true,
                    notify_on_error: true,
                },
                security_settings: SecuritySettings {
                    require_2fa: false,
                    session_encryption: true,
                    audit_logging: true,
                    ip_whitelist: vec![],
                    max_session_duration: 14400,
                },
            },
            created_at: Utc::now(),
        };

        let mut workspaces = self.workspaces.write().await;
        workspaces.insert(workspace_id.clone(), workspace);

        Ok(workspace_id)
    }

    pub async fn get_workspace(&self, workspace_id: &str) -> Result<TeamWorkspace> {
        let workspaces = self.workspaces.read().await;
        workspaces.get(workspace_id).cloned().ok_or_else(|| anyhow!("Workspace not found"))
    }

    async fn broadcast_event(&self, event: CollaborationEvent) -> Result<()> {
        self.event_sender.send(event).map_err(|e| anyhow!("Failed to broadcast event: {}", e))?;
        Ok(())
    }

    pub fn subscribe_to_events(&self) -> broadcast::Receiver<CollaborationEvent> {
        self.event_sender.subscribe()
    }

    pub async fn cleanup_inactive_sessions(&self, timeout_minutes: u64) -> Result<u32> {
        let mut sessions = self.sessions.write().await;
        let cutoff_time = Utc::now() - chrono::Duration::minutes(timeout_minutes as i64);
        
        let inactive_sessions: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| session.last_activity < cutoff_time)
            .map(|(id, _)| id.clone())
            .collect();

        let count = inactive_sessions.len() as u32;
        
        for session_id in inactive_sessions {
            sessions.remove(&session_id);
        }

        Ok(count)
    }

    pub async fn get_session_stats(&self) -> HashMap<String, u64> {
        let sessions = self.sessions.read().await;
        let mut stats = HashMap::new();

        stats.insert("total_sessions".to_string(), sessions.len() as u64);
        
        let active_count = sessions.values()
            .filter(|s| matches!(s.status, SessionStatus::Active))
            .count() as u64;
        stats.insert("active_sessions".to_string(), active_count);

        let total_participants = sessions.values()
            .map(|s| s.participants.len() as u64)
            .sum();
        stats.insert("total_participants".to_string(), total_participants);

        stats
    }

    // Methods expected by main.rs
    pub async fn create_session(&self, name: &str, permissions: SessionPermissions) -> Result<CollaborationSession> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let owner_id = "system".to_string(); // Default owner
        
        let session = CollaborativeSession {
            id: session_id.clone(),
            name: name.to_string(),
            description: String::new(),
            owner_id: owner_id.clone(),
            participants: vec![Participant {
                user_id: owner_id,
                username: "System".to_string(),
                role: ParticipantRole::Owner,
                permissions: UserPermissions {
                    can_execute_commands: true,
                    can_edit_files: true,
                    can_view_screen: true,
                    can_share_screen: true,
                    can_manage_participants: true,
                    can_change_settings: true,
                    allowed_commands: None,
                    restricted_paths: vec![],
                },
                status: ParticipantStatus::Online,
                joined_at: Utc::now(),
                last_seen: Utc::now(),
                cursor_position: None,
            }],
            permissions,
            status: SessionStatus::Active,
            terminal_id: None,
            shared_state: SharedState {
                current_directory: "/".to_string(),
                environment_variables: HashMap::new(),
                active_processes: vec![],
                shared_files: vec![],
                terminal_history: vec![],
                screen_sharing: None,
            },
            created_at: Utc::now(),
            last_activity: Utc::now(),
            settings: SessionSettings {
                auto_save_interval: 30,
                sync_cursor: true,
                sync_selection: true,
                show_participants: true,
                enable_voice_chat: false,
                enable_video_chat: false,
                record_session: false,
                notifications_enabled: true,
            },
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session.clone());

        self.broadcast_event(CollaborationEvent {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.clone(),
            user_id: "system".to_string(),
            event_type: CollaborationEventType::UserJoined,
            timestamp: Utc::now(),
            data: serde_json::json!({ "session_created": true }),
        }).await?;

        Ok(session)
    }

    pub async fn join_session(&self, session_id: &str, user_id: &str) -> Result<JoinResult> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            // Check if user is already in session
            if session.participants.iter().any(|p| p.user_id == user_id) {
                return Ok(JoinResult {
                    success: false,
                    session_id: session_id.to_string(),
                    participant_id: user_id.to_string(),
                    message: "User already in session".to_string(),
                });
            }

            // Check session capacity
            if session.participants.len() >= session.permissions.max_participants as usize {
                return Ok(JoinResult {
                    success: false,
                    session_id: session_id.to_string(),
                    participant_id: user_id.to_string(),
                    message: "Session is full".to_string(),
                });
            }

            let participant_id = uuid::Uuid::new_v4().to_string();
            let participant = Participant {
                user_id: user_id.to_string(),
                username: format!("User_{}", user_id),
                role: ParticipantRole::Editor,
                permissions: UserPermissions {
                    can_execute_commands: true,
                    can_edit_files: true,
                    can_view_screen: true,
                    can_share_screen: false,
                    can_manage_participants: false,
                    can_change_settings: false,
                    allowed_commands: None,
                    restricted_paths: vec![],
                },
                status: ParticipantStatus::Online,
                joined_at: Utc::now(),
                last_seen: Utc::now(),
                cursor_position: None,
            };

            session.participants.push(participant);
            session.last_activity = Utc::now();

            self.broadcast_event(CollaborationEvent {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: session_id.to_string(),
                user_id: user_id.to_string(),
                event_type: CollaborationEventType::UserJoined,
                timestamp: Utc::now(),
                data: serde_json::json!({ "participant_id": participant_id }),
            }).await?;

            Ok(JoinResult {
                success: true,
                session_id: session_id.to_string(),
                participant_id,
                message: "Successfully joined session".to_string(),
            })
        } else {
            Ok(JoinResult {
                success: false,
                session_id: session_id.to_string(),
                participant_id: user_id.to_string(),
                message: "Session not found".to_string(),
            })
        }
    }

    pub async fn share_terminal(&self, terminal_id: &str, session_id: &str, permissions: TerminalPermissions) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.terminal_id = Some(terminal_id.to_string());
            session.last_activity = Utc::now();

            // Add terminal-specific permissions to shared state
            let terminal_info = serde_json::json!({
                "terminal_id": terminal_id,
                "permissions": permissions,
                "shared_at": Utc::now()
            });

            self.broadcast_event(CollaborationEvent {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: session_id.to_string(),
                user_id: "system".to_string(),
                event_type: CollaborationEventType::SettingUpdated,
                timestamp: Utc::now(),
                data: terminal_info,
            }).await?;

            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }

    pub async fn get_sessions(&self) -> Result<Vec<CollaborationSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }

    pub async fn send_message(&self, session_id: &str, message: ChatMessage) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            // Find appropriate channel or create a default one
            if session.shared_state.terminal_history.is_empty() {
                // Initialize terminal history if empty
                session.shared_state.terminal_history = Vec::new();
            }

            // Create a terminal event for the chat message
            let terminal_event = TerminalEvent {
                id: message.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                session_id: session_id.to_string(),
                user_id: message.author_id.clone(),
                event_type: TerminalEventType::Input,
                timestamp: message.timestamp.unwrap_or_else(|| Utc::now()),
                data: message.content.clone(),
            };

            session.shared_state.terminal_history.push(terminal_event);
            session.last_activity = Utc::now();

            self.broadcast_event(CollaborationEvent {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: session_id.to_string(),
                user_id: message.author_id,
                event_type: CollaborationEventType::MessageSent,
                timestamp: Utc::now(),
                data: serde_json::json!({
                    "content": message.content,
                    "message_type": message.message_type
                }),
            }).await?;

            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collaboration_manager_creation() {
        let manager = CollaborationManager::new();
        let sessions = manager.list_sessions().await;
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_create_session() {
        let manager = CollaborationManager::new();
        let permissions = SessionPermissions {
            is_public: false,
            allow_anonymous: false,
            max_participants: 10,
            require_approval: false,
            allow_recording: false,
            password_protected: false,
        };
        
        let session = manager.create_session("Test Session", permissions).await.unwrap();
        let session_id = session.id.clone();
        
        assert_eq!(session.name, "Test Session");
        assert_eq!(session.participants.len(), 1);
        assert_eq!(session.participants[0].role, ParticipantRole::Owner);
    }

    #[tokio::test]
    async fn test_join_session() {
        let manager = CollaborationManager::new();
        let permissions = SessionPermissions {
            is_public: false,
            allow_anonymous: false,
            max_participants: 10,
            require_approval: false,
            allow_recording: false,
            password_protected: false,
        };
        
        let session = manager.create_session("Test Session", permissions).await.unwrap();
        let session_id = session.id.clone();

        let result = manager.join_session(&session_id, "user2").await.unwrap();
        assert!(result.success);
        
        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.participants.len(), 2);
        assert_eq!(session.participants[1].user_id, "user2");
    }
}
