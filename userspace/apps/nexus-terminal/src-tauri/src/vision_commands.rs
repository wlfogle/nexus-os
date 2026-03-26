use tauri::{command, State};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use image::{DynamicImage};
use scrap::{Capturer, Display};
use tesseract::Tesseract;
use reqwest::Client;
use std::collections::HashMap;
use crate::{AppState, vision};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenCaptureData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OCRResult {
    pub text: String,
    pub confidence: f32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UIElement {
    pub element_type: String,
    pub text: Option<String>,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub confidence: f32,
    pub attributes: HashMap<String, serde_json::Value>,
}

/// Capture the entire screen
#[command]
pub async fn capture_screen() -> Result<ScreenCaptureData, String> {
    // Use spawn_blocking to handle the non-Send capturer properly
    let capture_result = tokio::task::spawn_blocking(|| -> Result<ScreenCaptureData, String> {
        let display = Display::primary()
            .map_err(|e| format!("Failed to get primary display: {}", e))?;
        
        let mut capturer = Capturer::new(display)
            .map_err(|e| format!("Failed to create capturer: {}", e))?;
        
        // Get dimensions before capturing
        let width = capturer.width();
        let height = capturer.height();
        
        // Capture frame
        let frame = loop {
            match capturer.frame() {
                Ok(frame) => break frame,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Try again
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(e) => return Err(format!("Failed to capture frame: {}", e)),
            }
        };
        
        // Convert frame to RGBA
        let mut rgba_data = Vec::with_capacity(width * height * 4);
        for pixel in frame.chunks_exact(4) {
            rgba_data.push(pixel[2]); // R
            rgba_data.push(pixel[1]); // G
            rgba_data.push(pixel[0]); // B
            rgba_data.push(pixel[3]); // A
        }
        
        // Create image buffer and encode as PNG
        let img_buf = image::ImageBuffer::from_raw(width as u32, height as u32, rgba_data)
            .ok_or_else(|| "Failed to create image buffer".to_string())?;
        let dynamic_img = image::DynamicImage::ImageRgba8(img_buf);
        
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        dynamic_img
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode image: {}", e))?;
        
        Ok(ScreenCaptureData {
            data: buffer,
            width: width as u32,
            height: height as u32,
        })
    }).await.map_err(|e| format!("Task join error: {}", e))??;
    
    Ok(capture_result)
}

/// Capture a specific region of the screen
#[command]
pub async fn capture_screen_region(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<ScreenCaptureData, String> {
    // First capture the full screen
    let screen_data = capture_screen().await?;
    
    // Decode the captured image
    let cursor = std::io::Cursor::new(&screen_data.data);
    let dynamic_img = image::load(cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to decode captured image: {}", e))?;
    
    // Crop the region
    let cropped = dynamic_img.crop_imm(x as u32, y as u32, width as u32, height as u32);
    
    // Encode as PNG
    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    cropped
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image: {}", e))?;
    
    Ok(ScreenCaptureData {
        data: buffer,
        width: width as u32,
        height: height as u32,
    })
}

/// Perform OCR on an image file
#[command]
pub async fn perform_ocr(image_path: String, engine: String) -> Result<Vec<OCRResult>, String> {
    let path = PathBuf::from(image_path);
    
    match engine.as_str() {
        "tesseract" => perform_tesseract_ocr(path).await,
        "easyocr" => perform_easyocr_ocr(path).await,
        _ => Err(format!("Unsupported OCR engine: {}", engine)),
    }
}

/// Perform OCR using Tesseract
async fn perform_tesseract_ocr(image_path: PathBuf) -> Result<Vec<OCRResult>, String> {
    let path_str = image_path.to_str()
        .ok_or_else(|| "Invalid image path encoding".to_string())?;
    let mut tesseract = Tesseract::new(None, Some("eng"))
        .map_err(|e| format!("Failed to initialize Tesseract: {}", e))?
        .set_image(path_str)
        .map_err(|e| format!("Failed to set image: {}", e))?;
    
    // Get text from the image
    let text = tesseract
        .get_text()
        .map_err(|e| format!("Failed to extract text: {}", e))?;
    
    let mut results = Vec::new();
    
    // Split text into lines and create OCRResult for each non-empty line
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            results.push(OCRResult {
                text: trimmed.to_string(),
                confidence: 0.8, // Default confidence since Tesseract API doesn't provide easy per-word confidence
                x: 0,     // Default position - would need more complex API usage for actual bounds
                y: i as i32 * 20, // Approximate line spacing
                width: trimmed.len() as i32 * 10, // Approximate character width
                height: 18, // Approximate line height
            });
        }
    }
    
    Ok(results)
}

/// Perform OCR using EasyOCR (via Python subprocess)
async fn perform_easyocr_ocr(image_path: PathBuf) -> Result<Vec<OCRResult>, String> {
    use std::process::Command;
    
    let output = Command::new("python3")
        .arg("-c")
        .arg(format!(
            r#"
import easyocr
import json

reader = easyocr.Reader(['en'])
results = reader.readtext('{}', detail=1)

formatted_results = []
for (bbox, text, confidence) in results:
    x1, y1 = int(bbox[0][0]), int(bbox[0][1])
    x2, y2 = int(bbox[2][0]), int(bbox[2][1])
    formatted_results.append({{
        'text': text,
        'confidence': float(confidence),
        'x': x1,
        'y': y1,
        'width': x2 - x1,
        'height': y2 - y1
    }})

print(json.dumps(formatted_results))
            "#,
            image_path.display()
        ))
        .output()
        .map_err(|e| format!("Failed to run EasyOCR: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("EasyOCR failed: {}", stderr));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: Vec<OCRResult> = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse EasyOCR output: {}", e))?;
    
    Ok(results)
}

/// Detect UI elements in an image
#[command]
pub async fn detect_ui_elements(image_path: String) -> Result<Vec<UIElement>, String> {
    let path = PathBuf::from(image_path);
    
    // Load and analyze image
    let image = image::open(&path)
        .map_err(|e| format!("Failed to load image: {}", e))?;
    
    let mut elements = Vec::new();
    
    // Simple element detection based on image analysis
    // This is a basic implementation - in production, you'd use ML models
    
    // Detect potential terminal windows (dark backgrounds)
    if is_likely_terminal(&image) {
        elements.push(UIElement {
            element_type: "terminal".to_string(),
            text: None,
            x: 0,
            y: 0,
            width: image.width() as i32,
            height: image.height() as i32,
            confidence: 0.8,
            attributes: HashMap::from([
                ("background".to_string(), serde_json::Value::String("dark".to_string())),
            ]),
        });
    }
    
    // Detect code editor patterns (syntax highlighting colors)
    if is_likely_code_editor(&image) {
        elements.push(UIElement {
            element_type: "code".to_string(),
            text: None,
            x: 0,
            y: 0,
            width: image.width() as i32,
            height: image.height() as i32,
            confidence: 0.7,
            attributes: HashMap::from([
                ("syntax_highlighting".to_string(), serde_json::Value::Bool(true)),
            ]),
        });
    }
    
    // Detect buttons and clickable elements
    let button_candidates = detect_button_candidates(&image);
    elements.extend(button_candidates);
    
    Ok(elements)
}

/// Query AI with vision capabilities
#[command]
pub async fn query_vision_ai(
    prompt: String,
    image: String, // base64 encoded image
    _focus_region: Option<serde_json::Value>,
    ollama_host: Option<String>,
    ollama_port: Option<String>,
) -> Result<String, String> {
    let client = Client::new();
    
    let host = ollama_host.unwrap_or_else(|| std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "localhost".to_string()));
    let port = ollama_port.unwrap_or_else(|| std::env::var("OLLAMA_PORT").unwrap_or_else(|_| "11434".to_string()));
    
    let request_body = serde_json::json!({
        "model": "llava", // LLaVA model for vision + language
        "prompt": prompt,
        "images": [image],
        "stream": false,
        "options": {
            "temperature": 0.7,
            "top_p": 0.9
        }
    });
    
    let ollama_url = format!("http://{}:{}/api/generate", host, port);
    
    let response = client
        .post(&ollama_url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to Ollama: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "Ollama API error: {} - {}",
            response.status(),
            response.text().await.unwrap_or_default()
        ));
    }
    
    let response_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;
    
    let ai_response = response_data["response"]
        .as_str()
        .unwrap_or("No response from AI")
        .to_string();
    
    Ok(ai_response)
}

/// Check if required vision dependencies are available
#[command]
pub async fn check_vision_dependencies() -> Result<(), String> {
    // Check if Tesseract is available
    match Tesseract::new(None, Some("eng")) {
        Ok(_) => {},
        Err(_) => return Err("Tesseract OCR not available. Please install tesseract-ocr.".to_string()),
    }
    
    // Check if scrap library works
    match Display::primary() {
        Ok(_) => {},
        Err(_) => return Err("Screen capture not available.".to_string()),
    }
    
    // Check if Python and EasyOCR are available (optional)
    let python_check = std::process::Command::new("python3")
        .arg("-c")
        .arg("import easyocr; print('EasyOCR available')")
        .output();
    
    match python_check {
        Ok(output) if output.status.success() => {
            tracing::info!("EasyOCR is available as fallback OCR engine");
        },
        _ => {
            tracing::debug!("EasyOCR not available, using Tesseract only");
        },
    }
    
    Ok(())
}

/// Simple heuristic to detect if image likely contains a terminal
fn is_likely_terminal(image: &DynamicImage) -> bool {
    let rgba_image = image.to_rgba8();
    let pixels = rgba_image.pixels();
    
    let mut dark_pixel_count = 0;
    let mut total_pixels = 0;
    
    // Sample pixels to determine if background is predominantly dark
    for pixel in pixels.step_by(100) { // Sample every 100th pixel for performance
        total_pixels += 1;
        let brightness = (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0;
        
        if brightness < 50.0 { // Dark pixel threshold
            dark_pixel_count += 1;
        }
    }
    
    if total_pixels == 0 {
        return false;
    }
    
    let dark_ratio = dark_pixel_count as f32 / total_pixels as f32;
    dark_ratio > 0.6 // If more than 60% of pixels are dark, likely a terminal
}

/// Simple heuristic to detect if image likely contains code editor
fn is_likely_code_editor(image: &DynamicImage) -> bool {
    let rgba_image = image.to_rgba8();
    let pixels = rgba_image.pixels();
    
    let mut color_variety = std::collections::HashSet::new();
    
    // Sample pixels to check for syntax highlighting variety
    for pixel in pixels.step_by(200) { // Sample pixels
        // Quantize colors to reduce noise
        let r = (pixel[0] / 32) * 32;
        let g = (pixel[1] / 32) * 32;
        let b = (pixel[2] / 32) * 32;
        
        color_variety.insert((r, g, b));
        
        if color_variety.len() > 10 {
            break; // Enough variety detected
        }
    }
    
    // Code editors typically have more color variety due to syntax highlighting
    color_variety.len() > 6
}

/// Detect potential button candidates in the image
fn detect_button_candidates(image: &DynamicImage) -> Vec<UIElement> {
    let mut buttons = Vec::new();
    
    // This is a simplified button detection
    // In practice, you'd use computer vision techniques or ML models
    
    let width = image.width() as i32;
    let height = image.height() as i32;
    
    // Look for common button regions (simplified)
    let common_button_regions = vec![
        (10, 10, 80, 30),           // Top-left button area
        (width - 90, 10, 80, 30),   // Top-right button area
        (10, height - 40, 80, 30),  // Bottom-left button area
        (width - 90, height - 40, 80, 30), // Bottom-right button area
    ];
    
    for (x, y, w, h) in common_button_regions {
        if x >= 0 && y >= 0 && x + w <= width && y + h <= height {
            buttons.push(UIElement {
                element_type: "button".to_string(),
                text: None,
                x,
                y,
                width: w,
                height: h,
                confidence: 0.5, // Low confidence for heuristic detection
                attributes: HashMap::from([
                    ("detection_method".to_string(), serde_json::Value::String("heuristic".to_string())),
                ]),
            });
        }
    }
    
    buttons
}

/// Enhanced capture screen using VisionService
#[command]
pub async fn capture_screen_enhanced(
    state: State<'_, AppState>,
) -> Result<vision::ScreenCapture, String> {
    let vision_service = state.vision_service.read().await;
    vision_service.capture_full_screen().await.map_err(|e| e.to_string())
}

/// Enhanced capture region using VisionService
#[command]
pub async fn capture_region_enhanced(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    state: State<'_, AppState>,
) -> Result<vision::ScreenCapture, String> {
    let vision_service = state.vision_service.read().await;
    vision_service
        .capture_screen_region(x as u32, y as u32, width, height)
        .await
        .map_err(|e| e.to_string())
}

/// Perform OCR using VisionService
#[command]
pub async fn perform_ocr_enhanced(
    image_data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<vision::OCRResult, String> {
    let vision_service = state.vision_service.read().await;
    // Convert image data to temp file for OCR processing
    let temp_dir = std::env::var("TEMP").unwrap_or_else(|_| "/tmp".to_string());
    let temp_path = format!("{}/temp_ocr_{}.png", temp_dir, uuid::Uuid::new_v4());
    
    // Save image data to temp file
    tokio::fs::write(&temp_path, &image_data)
        .await
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    
    let result = vision_service.perform_ocr(&temp_path, "tesseract").await;
    
    // Clean up temp file
    let _ = tokio::fs::remove_file(&temp_path).await;
    
    // Convert Vec<OCRResult> to single OCRResult
    match result {
        Ok(results) => {
            if let Some(first_result) = results.first() {
                Ok(vision::OCRResult {
                    text: first_result.text.clone(),
                    confidence: first_result.confidence,
                    bounding_box: vision::BoundingBox {
                        x: first_result.bounding_box.x,
                        y: first_result.bounding_box.y,
                        width: first_result.bounding_box.width,
                        height: first_result.bounding_box.height,
                    },
                })
            } else {
                Ok(vision::OCRResult {
                    text: String::new(),
                    confidence: 0.0,
                    bounding_box: vision::BoundingBox { x: 0, y: 0, width: 0, height: 0 },
                })
            }
        }
        Err(e) => Err(e.to_string())
    }
}

/// Analyze screenshot using VisionService
#[command]
pub async fn analyze_screenshot(
    capture_id: String,
    image_data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<vision::ScreenAnalysis, String> {
    let vision_service = state.vision_service.read().await;
    vision_service
        .analyze_screen_comprehensive(&capture_id, image_data)
        .await
        .map_err(|e| e.to_string())
}

/// Get vision service statistics
#[command]
pub async fn get_vision_stats(
    state: State<'_, AppState>,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let _vision_service = state.vision_service.read().await;
    
    // Create stats from vision service state
    let mut stats = HashMap::new();
    stats.insert("initialized".to_string(), serde_json::Value::Bool(true));
    stats.insert("capture_count".to_string(), serde_json::Value::Number(serde_json::Number::from(0)));
    
    Ok(stats)
}

/// Check vision service status
#[command]
pub async fn check_vision_service_status(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let vision_service = state.vision_service.read().await;
    
    // Check if the service can perform a basic operation
    match vision_service.capture_full_screen().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
