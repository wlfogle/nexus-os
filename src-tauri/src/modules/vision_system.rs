use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::path::Path;
use std::process::Command;
use image::{ImageBuffer, RgbaImage, DynamicImage};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCapture {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub image_data: Vec<u8>,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub display_info: DisplayInfo,
    pub analysis: Option<ScreenAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub display_name: String,
    pub display_id: u32,
    pub resolution: String,
    pub is_primary: bool,
    pub position: (i32, i32),
    pub scale_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenAnalysis {
    pub detected_windows: Vec<WindowInfo>,
    pub ui_elements: Vec<UIElement>,
    pub text_content: Vec<TextRegion>,
    pub code_blocks: Vec<CodeBlock>,
    pub dominant_colors: Vec<String>,
    pub activity_type: ActivityType,
    pub context_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub title: String,
    pub application: String,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub is_active: bool,
    pub window_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    pub element_type: String, // button, input, menu, etc.
    pub text: Option<String>,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRegion {
    pub text: String,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub font_size: Option<f32>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub code: String,
    pub line_numbers: Option<Vec<u32>>,
    pub position: (f32, f32),
    pub size: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    Coding,
    Browsing,
    Terminal,
    Design,
    Documentation,
    Media,
    Gaming,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionSettings {
    pub capture_frequency: u64, // milliseconds
    pub auto_analyze: bool,
    pub save_captures: bool,
    pub privacy_mode: bool,
    pub excluded_windows: Vec<String>,
    pub ocr_enabled: bool,
    pub ui_detection_enabled: bool,
}

pub struct VisionSystem {
    settings: Arc<RwLock<VisionSettings>>,
    capture_history: Arc<RwLock<Vec<ScreenCapture>>>,
    active_monitors: Arc<RwLock<HashMap<u32, DisplayInfo>>>,
    is_capturing: Arc<RwLock<bool>>,
}

impl VisionSystem {
    pub fn new() -> Self {
        let default_settings = VisionSettings {
            capture_frequency: 1000, // 1 second
            auto_analyze: true,
            save_captures: false,
            privacy_mode: false,
            excluded_windows: vec![
                "password".to_string(),
                "login".to_string(),
                "private browsing".to_string(),
            ],
            ocr_enabled: true,
            ui_detection_enabled: true,
        };

        Self {
            settings: Arc::new(RwLock::new(default_settings)),
            capture_history: Arc::new(RwLock::new(Vec::new())),
            active_monitors: Arc::new(RwLock::new(HashMap::new())),
            is_capturing: Arc::new(RwLock::new(false)),
        }
    }

    // Start continuous screen monitoring
    pub async fn start_monitoring(&self) -> Result<(), AppError> {
        let mut is_capturing = self.is_capturing.write().await;
        if *is_capturing {
            return Ok(()); // Already capturing
        }
        *is_capturing = true;
        drop(is_capturing);

        // Detect available displays
        self.detect_displays().await?;

        // Start capture loop in background
        let vision_system = self.clone();
        tokio::spawn(async move {
            vision_system.capture_loop().await;
        });

        Ok(())
    }

    pub async fn stop_monitoring(&self) -> Result<(), AppError> {
        let mut is_capturing = self.is_capturing.write().await;
        *is_capturing = false;
        Ok(())
    }

    // Capture current screen
    pub async fn capture_screen(&self, display_id: Option<u32>) -> Result<ScreenCapture, AppError> {
        let displays = self.active_monitors.read().await;
        let target_display = if let Some(id) = display_id {
            displays.get(&id).cloned()
        } else {
            displays.values()
                .find(|d| d.is_primary)
                .or_else(|| displays.values().next())
                .cloned()
        };

        let display_info = target_display
            .ok_or_else(|| AppError::Internal("No display found".to_string()))?;

        // Use different capture methods based on OS
        let image_data = self.platform_capture_screen(&display_info).await?;
        
        let capture = ScreenCapture {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            image_data: image_data.clone(),
            format: "png".to_string(),
            width: display_info.resolution.split('x').next()
                .and_then(|s| s.parse().ok()).unwrap_or(1920),
            height: display_info.resolution.split('x').nth(1)
                .and_then(|s| s.parse().ok()).unwrap_or(1080),
            display_info: display_info.clone(),
            analysis: None,
        };

        // Auto-analyze if enabled
        let settings = self.settings.read().await;
        let mut final_capture = capture;
        if settings.auto_analyze {
            final_capture.analysis = Some(self.analyze_screen(&final_capture).await?);
        }

        // Store in history
        let mut history = self.capture_history.write().await;
        history.push(final_capture.clone());
        
        // Keep only last 100 captures
        if history.len() > 100 {
            history.remove(0);
        }

        Ok(final_capture)
    }

    // Analyze screen content
    pub async fn analyze_screen(&self, capture: &ScreenCapture) -> Result<ScreenAnalysis, AppError> {
        let mut analysis = ScreenAnalysis {
            detected_windows: Vec::new(),
            ui_elements: Vec::new(),
            text_content: Vec::new(),
            code_blocks: Vec::new(),
            dominant_colors: Vec::new(),
            activity_type: ActivityType::Unknown,
            context_summary: String::new(),
        };

        // Get active windows
        analysis.detected_windows = self.get_active_windows().await?;

        // OCR text extraction
        let settings = self.settings.read().await;
        if settings.ocr_enabled {
            analysis.text_content = self.extract_text_from_image(&capture.image_data).await?;
        }

        // Detect UI elements
        if settings.ui_detection_enabled {
            analysis.ui_elements = self.detect_ui_elements(&capture.image_data).await?;
        }

        // Detect code blocks
        analysis.code_blocks = self.detect_code_blocks(&analysis.text_content).await?;

        // Determine activity type
        analysis.activity_type = self.determine_activity_type(&analysis).await?;

        // Generate context summary
        analysis.context_summary = self.generate_context_summary(&analysis).await?;

        // Extract dominant colors
        analysis.dominant_colors = self.extract_dominant_colors(&capture.image_data).await?;

        Ok(analysis)
    }

    // Get what the AI should know about current screen
    pub async fn get_current_context(&self) -> Result<String, AppError> {
        let capture = self.capture_screen(None).await?;
        
        if let Some(analysis) = &capture.analysis {
            let mut context = format!(
                "Current Screen Context ({}x{}):\\n\\n",
                capture.width, capture.height
            );

            // Active windows
            if !analysis.detected_windows.is_empty() {
                context.push_str("Active Windows:\\n");
                for window in &analysis.detected_windows {
                    context.push_str(&format!(
                        "- {} ({}){}\\n",
                        window.title,
                        window.application,
                        if window.is_active { " [ACTIVE]" } else { "" }
                    ));
                }
                context.push('\\n');
            }

            // Activity type
            context.push_str(&format!(
                "Current Activity: {:?}\\n\\n",
                analysis.activity_type
            ));

            // Context summary
            if !analysis.context_summary.is_empty() {
                context.push_str(&format!("Summary: {}\\n\\n", analysis.context_summary));
            }

            // Visible text (sample)
            if !analysis.text_content.is_empty() {
                context.push_str("Visible Text (sample):\\n");
                let sample_text: String = analysis.text_content
                    .iter()
                    .take(5)
                    .map(|t| t.text.clone())
                    .collect::<Vec<_>>()
                    .join(" ");
                
                if sample_text.len() > 200 {
                    context.push_str(&format!("{}...\\n\\n", &sample_text[..200]));
                } else {
                    context.push_str(&format!("{}\\n\\n", sample_text));
                }
            }

            // Code blocks
            if !analysis.code_blocks.is_empty() {
                context.push_str("Code Visible:\\n");
                for (i, code_block) in analysis.code_blocks.iter().take(2).enumerate() {
                    context.push_str(&format!(
                        "Block {} ({}): {}\\n",
                        i + 1,
                        code_block.language.as_deref().unwrap_or("unknown"),
                        if code_block.code.len() > 100 {
                            format!("{}...", &code_block.code[..100])
                        } else {
                            code_block.code.clone()
                        }
                    ));
                }
            }

            Ok(context)
        } else {
            Ok("Screen captured but not analyzed".to_string())
        }
    }

    // Watch for specific changes
    pub async fn watch_for_changes(&self, callback: Box<dyn Fn(ScreenCapture) + Send + Sync>) -> Result<(), AppError> {
        let mut last_capture: Option<ScreenCapture> = None;
        
        loop {
            let current = self.capture_screen(None).await?;
            
            if let Some(ref last) = last_capture {
                if self.has_significant_change(last, &current).await? {
                    callback(current.clone());
                }
            }
            
            last_capture = Some(current);
            
            let settings = self.settings.read().await;
            tokio::time::sleep(std::time::Duration::from_millis(settings.capture_frequency)).await;
            
            let is_capturing = self.is_capturing.read().await;
            if !*is_capturing {
                break;
            }
        }
        
        Ok(())
    }

    // Implementation methods
    async fn capture_loop(&self) {
        loop {
            let is_capturing = self.is_capturing.read().await;
            if !*is_capturing {
                break;
            }
            drop(is_capturing);

            if let Ok(_) = self.capture_screen(None).await {
                // Capture successful
            }

            let settings = self.settings.read().await;
            tokio::time::sleep(std::time::Duration::from_millis(settings.capture_frequency)).await;
        }
    }

    async fn detect_displays(&self) -> Result<(), AppError> {
        let mut monitors = self.active_monitors.write().await;
        monitors.clear();

        // Linux - use xrandr
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("xrandr")
                .arg("--query")
                .output()
                .map_err(|e| AppError::Internal(format!("Failed to run xrandr: {}", e)))?;

            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut display_id = 0;
                
                for line in output_str.lines() {
                    if line.contains(" connected") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(name) = parts.first() {
                            let resolution = parts.iter()
                                .find(|part| part.contains("x") && part.chars().next().unwrap_or('a').is_numeric())
                                .unwrap_or(&"1920x1080")
                                .split('+').next()
                                .unwrap_or("1920x1080");

                            let is_primary = line.contains("primary");

                            monitors.insert(display_id, DisplayInfo {
                                display_name: name.to_string(),
                                display_id,
                                resolution: resolution.to_string(),
                                is_primary,
                                position: (0, 0), // Simplified
                                scale_factor: 1.0,
                            });
                            
                            display_id += 1;
                        }
                    }
                }
            }
        }

        // If no displays detected, add a default one
        if monitors.is_empty() {
            monitors.insert(0, DisplayInfo {
                display_name: "Default".to_string(),
                display_id: 0,
                resolution: "1920x1080".to_string(),
                is_primary: true,
                position: (0, 0),
                scale_factor: 1.0,
            });
        }

        Ok(())
    }

    async fn platform_capture_screen(&self, display_info: &DisplayInfo) -> Result<Vec<u8>, AppError> {
        #[cfg(target_os = "linux")]
        {
            // Try different capture methods
            
            // Method 1: scrot (most reliable)
            if let Ok(output) = Command::new("scrot")
                .arg("-z")
                .arg("-")
                .output()
            {
                if output.status.success() {
                    return Ok(output.stdout);
                }
            }

            // Method 2: import (ImageMagick)
            if let Ok(output) = Command::new("import")
                .arg("-window")
                .arg("root")
                .arg("png:-")
                .output()
            {
                if output.status.success() {
                    return Ok(output.stdout);
                }
            }

            // Method 3: gnome-screenshot
            if let Ok(output) = Command::new("gnome-screenshot")
                .arg("-f")
                .arg("/tmp/screenshot.png")
                .output()
            {
                if output.status.success() {
                    if let Ok(data) = std::fs::read("/tmp/screenshot.png") {
                        let _ = std::fs::remove_file("/tmp/screenshot.png");
                        return Ok(data);
                    }
                }
            }

            // Method 4: maim
            if let Ok(output) = Command::new("maim")
                .arg("--format=png")
                .output()
            {
                if output.status.success() {
                    return Ok(output.stdout);
                }
            }

            Err(AppError::Internal("No screen capture method available".to_string()))
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(AppError::Internal("Screen capture not implemented for this platform".to_string()))
        }
    }

    async fn get_active_windows(&self) -> Result<Vec<WindowInfo>, AppError> {
        let mut windows = Vec::new();

        #[cfg(target_os = "linux")]
        {
            // Use wmctrl to get window information
            if let Ok(output) = Command::new("wmctrl")
                .arg("-l")
                .arg("-x")
                .output()
            {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    for line in output_str.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 4 {
                            let window_id = u64::from_str_radix(parts[0].trim_start_matches("0x"), 16)
                                .unwrap_or(0);
                            let title = parts[3..].join(" ");
                            let application = parts.get(2).unwrap_or(&"unknown").to_string();

                            windows.push(WindowInfo {
                                title,
                                application,
                                position: (0, 0), // Would need additional parsing
                                size: (0, 0),     // Would need additional parsing
                                is_active: false, // Would need to check active window
                                window_id,
                            });
                        }
                    }
                }
            }

            // Get active window
            if let Ok(output) = Command::new("xprop")
                .arg("-root")
                .arg("_NET_ACTIVE_WINDOW")
                .output()
            {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if let Some(hex_id) = output_str.split_whitespace().last() {
                        if let Ok(active_id) = u64::from_str_radix(hex_id.trim_start_matches("0x"), 16) {
                            for window in &mut windows {
                                window.is_active = window.window_id == active_id;
                            }
                        }
                    }
                }
            }
        }

        Ok(windows)
    }

    async fn extract_text_from_image(&self, image_data: &[u8]) -> Result<Vec<TextRegion>, AppError> {
        // Use tesseract for OCR
        let temp_file = "/tmp/screenshot_ocr.png";
        std::fs::write(temp_file, image_data)
            .map_err(|e| AppError::FileSystem(format!("Failed to write temp file: {}", e)))?;

        let output = Command::new("tesseract")
            .arg(temp_file)
            .arg("stdout")
            .arg("-l")
            .arg("eng")
            .output();

        let _ = std::fs::remove_file(temp_file);

        match output {
            Ok(result) if result.status.success() => {
                let text = String::from_utf8_lossy(&result.stdout);
                let regions = vec![TextRegion {
                    text: text.trim().to_string(),
                    position: (0.0, 0.0),
                    size: (1.0, 1.0),
                    font_size: None,
                    confidence: 0.8,
                }];
                Ok(regions)
            },
            _ => Ok(Vec::new()),
        }
    }

    async fn detect_ui_elements(&self, _image_data: &[u8]) -> Result<Vec<UIElement>, AppError> {
        // Placeholder for UI element detection
        // Would use computer vision libraries like OpenCV
        Ok(Vec::new())
    }

    async fn detect_code_blocks(&self, text_regions: &[TextRegion]) -> Result<Vec<CodeBlock>, AppError> {
        let mut code_blocks = Vec::new();
        
        for region in text_regions {
            // Simple heuristics to detect code
            let text = &region.text;
            let lines: Vec<&str> = text.lines().collect();
            
            // Look for code patterns
            let has_braces = text.contains('{') && text.contains('}');
            let has_semicolons = text.lines().filter(|line| line.trim().ends_with(';')).count() > 2;
            let has_keywords = ["function", "class", "def", "fn", "var", "let", "const", "if", "for", "while"]
                .iter().any(|keyword| text.to_lowercase().contains(keyword));
            let has_indentation = lines.iter().any(|line| line.starts_with("    ") || line.starts_with("\\t"));

            if (has_braces && has_semicolons) || (has_keywords && has_indentation) {
                // Try to detect language
                let language = if text.contains("fn ") && text.contains("->") {
                    Some("rust".to_string())
                } else if text.contains("def ") && text.contains(":") {
                    Some("python".to_string())
                } else if text.contains("function ") || text.contains("const ") {
                    Some("javascript".to_string())
                } else if text.contains("class ") && text.contains("{") {
                    Some("java".to_string())
                } else {
                    None
                };

                code_blocks.push(CodeBlock {
                    language,
                    code: text.clone(),
                    line_numbers: None,
                    position: region.position,
                    size: region.size,
                });
            }
        }
        
        Ok(code_blocks)
    }

    async fn determine_activity_type(&self, analysis: &ScreenAnalysis) -> Result<ActivityType, AppError> {
        // Analyze windows and content to determine activity
        for window in &analysis.detected_windows {
            let app_lower = window.application.to_lowercase();
            let title_lower = window.title.to_lowercase();
            
            if window.is_active {
                if app_lower.contains("code") || app_lower.contains("vim") || 
                   app_lower.contains("emacs") || app_lower.contains("atom") ||
                   title_lower.contains(".rs") || title_lower.contains(".py") ||
                   title_lower.contains(".js") {
                    return Ok(ActivityType::Coding);
                }
                
                if app_lower.contains("terminal") || app_lower.contains("konsole") ||
                   app_lower.contains("gnome-terminal") {
                    return Ok(ActivityType::Terminal);
                }
                
                if app_lower.contains("firefox") || app_lower.contains("chrome") ||
                   app_lower.contains("browser") {
                    return Ok(ActivityType::Browsing);
                }
                
                if app_lower.contains("gimp") || app_lower.contains("inkscape") ||
                   app_lower.contains("figma") {
                    return Ok(ActivityType::Design);
                }
            }
        }

        // Check for code blocks
        if !analysis.code_blocks.is_empty() {
            return Ok(ActivityType::Coding);
        }

        Ok(ActivityType::Unknown)
    }

    async fn generate_context_summary(&self, analysis: &ScreenAnalysis) -> Result<String, AppError> {
        let mut summary = String::new();
        
        match analysis.activity_type {
            ActivityType::Coding => {
                summary.push_str("User is coding. ");
                if !analysis.code_blocks.is_empty() {
                    let languages: Vec<String> = analysis.code_blocks
                        .iter()
                        .filter_map(|block| block.language.clone())
                        .collect();
                    if !languages.is_empty() {
                        summary.push_str(&format!("Languages visible: {}. ", languages.join(", ")));
                    }
                }
            },
            ActivityType::Terminal => {
                summary.push_str("User is using terminal. ");
            },
            ActivityType::Browsing => {
                summary.push_str("User is browsing the web. ");
            },
            ActivityType::Design => {
                summary.push_str("User is doing design work. ");
            },
            _ => {
                summary.push_str("User activity unclear. ");
            }
        }

        if let Some(active_window) = analysis.detected_windows.iter().find(|w| w.is_active) {
            summary.push_str(&format!("Active: {} ({})", active_window.title, active_window.application));
        }

        Ok(summary)
    }

    async fn extract_dominant_colors(&self, _image_data: &[u8]) -> Result<Vec<String>, AppError> {
        // Placeholder for color extraction
        Ok(vec!["#2D3748".to_string(), "#4A5568".to_string(), "#718096".to_string()])
    }

    async fn has_significant_change(&self, _last: &ScreenCapture, _current: &ScreenCapture) -> Result<bool, AppError> {
        // Placeholder for change detection
        // Could compare image hashes, window titles, etc.
        Ok(true)
    }

    // Utility methods
    pub async fn update_settings(&self, new_settings: VisionSettings) -> Result<(), AppError> {
        let mut settings = self.settings.write().await;
        *settings = new_settings;
        Ok(())
    }

    pub async fn get_capture_history(&self, limit: Option<usize>) -> Result<Vec<ScreenCapture>, AppError> {
        let history = self.capture_history.read().await;
        let limit = limit.unwrap_or(10);
        Ok(history.iter().rev().take(limit).cloned().collect())
    }

    pub async fn clear_capture_history(&self) -> Result<(), AppError> {
        let mut history = self.capture_history.write().await;
        history.clear();
        Ok(())
    }
}

impl Clone for VisionSystem {
    fn clone(&self) -> Self {
        Self {
            settings: Arc::clone(&self.settings),
            capture_history: Arc::clone(&self.capture_history),
            active_monitors: Arc::clone(&self.active_monitors),
            is_capturing: Arc::clone(&self.is_capturing),
        }
    }
}
