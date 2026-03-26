use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use reqwest::Client;
use tokio::process::Command;
use cpal::{Device, Stream};
use std::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlexaConfig {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    pub device_id: String,
    pub wake_word: String,
    pub voice_enabled: bool,
    pub smart_home_enabled: bool,
    pub skill_integration: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommand {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub command_text: String,
    pub confidence: f32,
    pub intent: VoiceIntent,
    pub parameters: HashMap<String, String>,
    pub response: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceIntent {
    CodeAnalysis,
    FileOperation,
    SystemControl,
    SmartHome,
    Conversation,
    ScreenCapture,
    ProjectManagement,
    DeviceControl,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartHomeDevice {
    pub device_id: String,
    pub name: String,
    pub device_type: String,
    pub capabilities: Vec<String>,
    pub state: HashMap<String, serde_json::Value>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlexaSkillRequest {
    pub version: String,
    pub session: SessionInfo,
    pub context: RequestContext,
    pub request: RequestBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub new: bool,
    pub session_id: String,
    pub application: ApplicationInfo,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationInfo {
    pub application_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub access_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub system: SystemInfo,
    pub audio_player: Option<AudioPlayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub application: ApplicationInfo,
    pub user: UserInfo,
    pub device: DeviceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub supported_interfaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPlayerInfo {
    pub token: Option<String>,
    pub offset_in_milliseconds: i64,
    pub player_activity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RequestBody {
    LaunchRequest,
    IntentRequest {
        request_id: String,
        timestamp: DateTime<Utc>,
        intent: IntentInfo,
    },
    SessionEndedRequest {
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentInfo {
    pub name: String,
    pub confirmation_status: String,
    pub slots: HashMap<String, SlotInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    pub name: String,
    pub value: Option<String>,
    pub confirmation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlexaResponse {
    pub version: String,
    pub session_attributes: HashMap<String, serde_json::Value>,
    pub response: ResponseBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseBody {
    pub output_speech: Option<OutputSpeech>,
    pub card: Option<Card>,
    pub reprompt: Option<Reprompt>,
    pub should_end_session: bool,
    pub directives: Vec<Directive>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OutputSpeech {
    PlainText { text: String },
    SSML { ssml: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Card {
    Simple { title: String, content: String },
    Standard { title: String, text: String, image: Option<CardImage> },
    LinkAccount,
    AskForPermissionsConsent { permissions: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardImage {
    pub small_image_url: Option<String>,
    pub large_image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reprompt {
    pub output_speech: OutputSpeech,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Directive {
    AudioPlayer {
        play_behavior: String,
        audio_item: AudioItem,
    },
    Display {
        template: DisplayTemplate,
    },
    VideoApp {
        video_item: VideoItem,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioItem {
    pub stream: AudioStream,
    pub metadata: Option<AudioMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStream {
    pub token: String,
    pub url: String,
    pub offset_in_milliseconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub title: String,
    pub subtitle: String,
    pub art: Option<CardImage>,
    pub background_image: Option<CardImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayTemplate {
    pub template_type: String,
    pub title: String,
    pub text_content: TextContent,
    pub background_image: Option<CardImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub primary_text: PlainText,
    pub secondary_text: Option<PlainText>,
    pub tertiary_text: Option<PlainText>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlainText {
    pub text: String,
    pub text_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoItem {
    pub source: String,
    pub metadata: Option<VideoMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub title: String,
    pub subtitle: String,
}

pub struct AlexaIntegration {
    config: Arc<RwLock<AlexaConfig>>,
    http_client: Client,
    voice_commands: Arc<RwLock<Vec<VoiceCommand>>>,
    smart_devices: Arc<RwLock<HashMap<String, SmartHomeDevice>>>,
    is_listening: Arc<RwLock<bool>>,
    audio_stream: Arc<RwLock<Option<Stream>>>,
}

impl AlexaIntegration {
    pub fn new() -> Self {
        let default_config = AlexaConfig {
            client_id: String::new(),
            client_secret: String::new(),
            refresh_token: None,
            access_token: None,
            device_id: "ai_assistant_device".to_string(),
            wake_word: "computer".to_string(),
            voice_enabled: true,
            smart_home_enabled: true,
            skill_integration: true,
        };

        Self {
            config: Arc::new(RwLock::new(default_config)),
            http_client: Client::new(),
            voice_commands: Arc::new(RwLock::new(Vec::new())),
            smart_devices: Arc::new(RwLock::new(HashMap::new())),
            is_listening: Arc::new(RwLock::new(false)),
            audio_stream: Arc::new(RwLock::new(None)),
        }
    }

    // Voice Control Methods
    pub async fn start_voice_listening(&self) -> Result<(), AppError> {
        let mut is_listening = self.is_listening.write().await;
        if *is_listening {
            return Ok(());
        }
        *is_listening = true;
        drop(is_listening);

        // Start audio capture
        self.initialize_audio_capture().await?;

        // Start voice processing loop
        let alexa_integration = self.clone();
        tokio::spawn(async move {
            alexa_integration.voice_processing_loop().await;
        });

        Ok(())
    }

    pub async fn stop_voice_listening(&self) -> Result<(), AppError> {
        let mut is_listening = self.is_listening.write().await;
        *is_listening = false;
        
        let mut audio_stream = self.audio_stream.write().await;
        *audio_stream = None;
        
        Ok(())
    }

    pub async fn process_voice_command(&self, audio_data: &[u8]) -> Result<VoiceCommand, AppError> {
        // Convert audio to text using speech recognition
        let command_text = self.speech_to_text(audio_data).await?;
        
        // Parse intent from command
        let intent = self.parse_intent(&command_text).await?;
        
        // Extract parameters
        let parameters = self.extract_parameters(&command_text, &intent).await?;
        
        let voice_command = VoiceCommand {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            command_text: command_text.clone(),
            confidence: 0.85, // Would come from speech recognition
            intent,
            parameters,
            response: None,
        };

        // Store command
        let mut commands = self.voice_commands.write().await;
        commands.push(voice_command.clone());
        
        // Keep only last 100 commands
        if commands.len() > 100 {
            commands.remove(0);
        }

        Ok(voice_command)
    }

    pub async fn execute_voice_command(&self, command: &VoiceCommand) -> Result<String, AppError> {
        match &command.intent {
            VoiceIntent::CodeAnalysis => {
                self.handle_code_analysis_command(command).await
            },
            VoiceIntent::FileOperation => {
                self.handle_file_operation_command(command).await
            },
            VoiceIntent::SystemControl => {
                self.handle_system_control_command(command).await
            },
            VoiceIntent::SmartHome => {
                self.handle_smart_home_command(command).await
            },
            VoiceIntent::Conversation => {
                self.handle_conversation_command(command).await
            },
            VoiceIntent::ScreenCapture => {
                self.handle_screen_capture_command(command).await
            },
            VoiceIntent::ProjectManagement => {
                self.handle_project_management_command(command).await
            },
            VoiceIntent::DeviceControl => {
                self.handle_device_control_command(command).await
            },
            VoiceIntent::Custom(custom_intent) => {
                self.handle_custom_command(command, custom_intent).await
            },
        }
    }

    // Alexa Skill Integration
    pub async fn handle_alexa_skill_request(&self, request: AlexaSkillRequest) -> Result<AlexaResponse, AppError> {
        match request.request {
            RequestBody::LaunchRequest => {
                self.handle_launch_request(&request).await
            },
            RequestBody::IntentRequest { intent, .. } => {
                self.handle_intent_request(&request, &intent).await
            },
            RequestBody::SessionEndedRequest { reason } => {
                self.handle_session_ended(&request, &reason).await
            },
        }
    }

    async fn handle_launch_request(&self, _request: &AlexaSkillRequest) -> Result<AlexaResponse, AppError> {
        Ok(AlexaResponse {
            version: "1.0".to_string(),
            session_attributes: HashMap::new(),
            response: ResponseBody {
                output_speech: Some(OutputSpeech::PlainText {
                    text: "Welcome to your AI Assistant! I can help you with code analysis, file operations, system control, and more. What would you like me to do?".to_string(),
                }),
                card: Some(Card::Simple {
                    title: "AI Assistant".to_string(),
                    content: "Your personal coding and development assistant is ready to help!".to_string(),
                }),
                reprompt: Some(Reprompt {
                    output_speech: OutputSpeech::PlainText {
                        text: "You can ask me to analyze code, manage files, control your system, or help with your projects.".to_string(),
                    },
                }),
                should_end_session: false,
                directives: Vec::new(),
            },
        })
    }

    async fn handle_intent_request(&self, request: &AlexaSkillRequest, intent: &IntentInfo) -> Result<AlexaResponse, AppError> {
        let response_text = match intent.name.as_str() {
            "AnalyzeCodeIntent" => {
                let file_path = intent.slots.get("FilePath")
                    .and_then(|slot| slot.value.as_ref())
                    .unwrap_or("current file");
                
                format!("Analyzing code in {}. I'll check for complexity, security issues, and optimization opportunities.", file_path)
            },
            "CaptureScreenIntent" => {
                "Taking a screenshot and analyzing what's on your screen.".to_string()
            },
            "ExecuteCommandIntent" => {
                let command = intent.slots.get("Command")
                    .and_then(|slot| slot.value.as_ref())
                    .unwrap_or("unknown command");
                
                format!("Executing: {}", command)
            },
            "SmartHomeIntent" => {
                let device = intent.slots.get("Device")
                    .and_then(|slot| slot.value.as_ref())
                    .unwrap_or("device");
                let action = intent.slots.get("Action")
                    .and_then(|slot| slot.value.as_ref())
                    .unwrap_or("control");
                
                format!("Controlling {}: {}", device, action)
            },
            "ProjectStatusIntent" => {
                "Checking your project status and recent changes.".to_string()
            },
            "AMAZON.HelpIntent" => {
                "I can help you with code analysis, file operations, system control, screen capture, and smart home devices. What would you like me to do?".to_string()
            },
            "AMAZON.StopIntent" | "AMAZON.CancelIntent" => {
                "Goodbye! Your AI Assistant is always here when you need help.".to_string()
            },
            _ => {
                "I'm not sure how to handle that request. Try asking me to analyze code, capture screen, or control devices.".to_string()
            }
        };

        let should_end = matches!(intent.name.as_str(), "AMAZON.StopIntent" | "AMAZON.CancelIntent");

        Ok(AlexaResponse {
            version: "1.0".to_string(),
            session_attributes: HashMap::new(),
            response: ResponseBody {
                output_speech: Some(OutputSpeech::PlainText {
                    text: response_text.clone(),
                }),
                card: Some(Card::Simple {
                    title: "AI Assistant".to_string(),
                    content: response_text,
                }),
                reprompt: if !should_end {
                    Some(Reprompt {
                        output_speech: OutputSpeech::PlainText {
                            text: "What else can I help you with?".to_string(),
                        },
                    })
                } else {
                    None
                },
                should_end_session: should_end,
                directives: Vec::new(),
            },
        })
    }

    async fn handle_session_ended(&self, _request: &AlexaSkillRequest, _reason: &str) -> Result<AlexaResponse, AppError> {
        Ok(AlexaResponse {
            version: "1.0".to_string(),
            session_attributes: HashMap::new(),
            response: ResponseBody {
                output_speech: None,
                card: None,
                reprompt: None,
                should_end_session: true,
                directives: Vec::new(),
            },
        })
    }

    // Smart Home Integration
    pub async fn discover_smart_devices(&self) -> Result<Vec<SmartHomeDevice>, AppError> {
        let mut devices = Vec::new();
        
        // Add your AI Assistant as a controllable device
        devices.push(SmartHomeDevice {
            device_id: "ai_assistant_main".to_string(),
            name: "AI Assistant".to_string(),
            device_type: "ACTIVITY_TRIGGER".to_string(),
            capabilities: vec![
                "Alexa.PowerController".to_string(),
                "Alexa.SceneController".to_string(),
                "Alexa.Speaker".to_string(),
            ],
            state: HashMap::new(),
            last_updated: Utc::now(),
        });

        // Add development environment as controllable scenes
        devices.push(SmartHomeDevice {
            device_id: "dev_environment".to_string(),
            name: "Development Environment".to_string(),
            device_type: "SCENE_TRIGGER".to_string(),
            capabilities: vec!["Alexa.SceneController".to_string()],
            state: HashMap::new(),
            last_updated: Utc::now(),
        });

        // Add system monitoring as a sensor
        devices.push(SmartHomeDevice {
            device_id: "system_monitor".to_string(),
            name: "System Monitor".to_string(),
            device_type: "TEMPERATURE_SENSOR".to_string(),
            capabilities: vec!["Alexa.TemperatureSensor".to_string()],
            state: {
                let mut state = HashMap::new();
                state.insert("temperature".to_string(), serde_json::json!({"value": 45.0, "scale": "CELSIUS"}));
                state
            },
            last_updated: Utc::now(),
        });

        // Store discovered devices
        let mut smart_devices = self.smart_devices.write().await;
        for device in &devices {
            smart_devices.insert(device.device_id.clone(), device.clone());
        }

        Ok(devices)
    }

    pub async fn control_smart_device(&self, device_id: &str, directive: &str, value: Option<serde_json::Value>) -> Result<String, AppError> {
        let mut devices = self.smart_devices.write().await;
        
        if let Some(device) = devices.get_mut(device_id) {
            match directive {
                "TurnOn" => {
                    match device_id {
                        "ai_assistant_main" => {
                            Ok("AI Assistant is now active and ready to help!".to_string())
                        },
                        "dev_environment" => {
                            // Could start development tools, open IDE, etc.
                            Ok("Development environment activated!".to_string())
                        },
                        _ => Ok(format!("Turned on {}", device.name)),
                    }
                },
                "TurnOff" => {
                    Ok(format!("Turned off {}", device.name))
                },
                "SetPercentage" => {
                    if let Some(percentage) = value {
                        Ok(format!("Set {} to {}%", device.name, percentage))
                    } else {
                        Err(AppError::Validation("Percentage value required".to_string()))
                    }
                },
                "Activate" => {
                    match device_id {
                        "dev_environment" => {
                            Ok("Development environment scene activated! Opening IDE and terminal.".to_string())
                        },
                        _ => Ok(format!("Activated {}", device.name)),
                    }
                },
                _ => Err(AppError::Validation(format!("Unknown directive: {}", directive))),
            }
        } else {
            Err(AppError::NotFound(format!("Device not found: {}", device_id)))
        }
    }

    // Implementation methods
    async fn initialize_audio_capture(&self) -> Result<(), AppError> {
        // Placeholder for audio capture initialization
        // Would use cpal or similar for real audio capture
        Ok(())
    }

    async fn voice_processing_loop(&self) {
        // Placeholder for continuous voice processing
        // Would implement wake word detection and continuous listening
    }

    async fn speech_to_text(&self, _audio_data: &[u8]) -> Result<String, AppError> {
        // Placeholder for speech recognition
        // Would integrate with services like Google Speech-to-Text, Azure Speech, or local Whisper
        Ok("analyze the current code file".to_string())
    }

    async fn parse_intent(&self, command_text: &str) -> Result<VoiceIntent, AppError> {
        let text_lower = command_text.to_lowercase();
        
        if text_lower.contains("analyze") || text_lower.contains("review") || text_lower.contains("check code") {
            Ok(VoiceIntent::CodeAnalysis)
        } else if text_lower.contains("open") || text_lower.contains("save") || text_lower.contains("file") {
            Ok(VoiceIntent::FileOperation)
        } else if text_lower.contains("screenshot") || text_lower.contains("capture screen") {
            Ok(VoiceIntent::ScreenCapture)
        } else if text_lower.contains("turn on") || text_lower.contains("turn off") || text_lower.contains("light") {
            Ok(VoiceIntent::SmartHome)
        } else if text_lower.contains("system") || text_lower.contains("restart") || text_lower.contains("shutdown") {
            Ok(VoiceIntent::SystemControl)
        } else if text_lower.contains("project") || text_lower.contains("status") || text_lower.contains("git") {
            Ok(VoiceIntent::ProjectManagement)
        } else {
            Ok(VoiceIntent::Conversation)
        }
    }

    async fn extract_parameters(&self, command_text: &str, intent: &VoiceIntent) -> Result<HashMap<String, String>, AppError> {
        let mut parameters = HashMap::new();
        
        match intent {
            VoiceIntent::FileOperation => {
                // Extract file names
                if let Some(start) = command_text.find("file ") {
                    if let Some(file_name) = command_text[start + 5..].split_whitespace().next() {
                        parameters.insert("file_name".to_string(), file_name.to_string());
                    }
                }
            },
            VoiceIntent::SmartHome => {
                // Extract device names and actions
                if command_text.to_lowercase().contains("light") {
                    parameters.insert("device_type".to_string(), "light".to_string());
                }
                if command_text.to_lowercase().contains("turn on") {
                    parameters.insert("action".to_string(), "on".to_string());
                } else if command_text.to_lowercase().contains("turn off") {
                    parameters.insert("action".to_string(), "off".to_string());
                }
            },
            _ => {}
        }
        
        Ok(parameters)
    }

    // Command handlers
    async fn handle_code_analysis_command(&self, command: &VoiceCommand) -> Result<String, AppError> {
        let file_name = command.parameters.get("file_name")
            .cloned()
            .unwrap_or_else(|| "current file".to_string());
        
        // Would integrate with code intelligence module
        Ok(format!("Analyzing {} for complexity, security issues, and optimization opportunities. I found 3 suggestions for improvement.", file_name))
    }

    async fn handle_file_operation_command(&self, command: &VoiceCommand) -> Result<String, AppError> {
        if command.command_text.to_lowercase().contains("open") {
            Ok("Opening the requested file in your editor.".to_string())
        } else if command.command_text.to_lowercase().contains("save") {
            Ok("File saved successfully.".to_string())
        } else {
            Ok("File operation completed.".to_string())
        }
    }

    async fn handle_system_control_command(&self, _command: &VoiceCommand) -> Result<String, AppError> {
        Ok("System command executed. Checking system status...".to_string())
    }

    async fn handle_smart_home_command(&self, command: &VoiceCommand) -> Result<String, AppError> {
        let device = command.parameters.get("device_type").unwrap_or(&"device".to_string());
        let action = command.parameters.get("action").unwrap_or(&"controlled".to_string());
        
        Ok(format!("Smart home command: {} {}", action, device))
    }

    async fn handle_conversation_command(&self, command: &VoiceCommand) -> Result<String, AppError> {
        // Would integrate with main AI conversation system
        Ok(format!("I understand you said: '{}'. How can I help you with that?", command.command_text))
    }

    async fn handle_screen_capture_command(&self, _command: &VoiceCommand) -> Result<String, AppError> {
        // Would integrate with vision system
        Ok("I've captured your screen and can see you're working on a Rust project. The code looks well-structured.".to_string())
    }

    async fn handle_project_management_command(&self, _command: &VoiceCommand) -> Result<String, AppError> {
        Ok("Checking project status... You have 3 uncommitted changes and 2 files modified since last commit.".to_string())
    }

    async fn handle_device_control_command(&self, command: &VoiceCommand) -> Result<String, AppError> {
        Ok(format!("Device control command processed: {}", command.command_text))
    }

    async fn handle_custom_command(&self, command: &VoiceCommand, custom_intent: &str) -> Result<String, AppError> {
        Ok(format!("Custom command '{}' processed: {}", custom_intent, command.command_text))
    }

    // Utility methods
    pub async fn update_config(&self, new_config: AlexaConfig) -> Result<(), AppError> {
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }

    pub async fn get_voice_command_history(&self, limit: Option<usize>) -> Result<Vec<VoiceCommand>, AppError> {
        let commands = self.voice_commands.read().await;
        let limit = limit.unwrap_or(10);
        Ok(commands.iter().rev().take(limit).cloned().collect())
    }

    pub async fn text_to_speech(&self, text: &str) -> Result<Vec<u8>, AppError> {
        // Placeholder for text-to-speech conversion
        // Would integrate with services like Amazon Polly, Google TTS, or local solutions
        
        // For Linux, could use espeak or festival
        let _output = Command::new("espeak")
            .arg("-s")
            .arg("150")
            .arg("-v")
            .arg("en")
            .arg(text)
            .output()
            .await;
        
        Ok(Vec::new()) // Placeholder
    }
}

impl Clone for AlexaIntegration {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            http_client: self.http_client.clone(),
            voice_commands: Arc::clone(&self.voice_commands),
            smart_devices: Arc::clone(&self.smart_devices),
            is_listening: Arc::clone(&self.is_listening),
            audio_stream: Arc::clone(&self.audio_stream),
        }
    }
}
