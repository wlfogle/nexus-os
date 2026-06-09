use tauri::{command, State};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use image::{DynamicImage, GenericImageView};
use scrap::{Capturer, Display};
use reqwest::Client;
use std::collections::HashMap;
use base64::Engine as _;
use crate::{AppState, vision};

/// Resize image bytes to at most 1280px on the longest side and
/// re-encode as JPEG quality 85.  Returns a raw base64 string (no data: prefix).
/// This reduces a 1920×1080 PNG from ~4 MB to ~120 KB, avoiding VRAM pressure
/// during vision-model inference and keeping Ollama request sizes manageable.
pub fn resize_for_vision(data: &[u8]) -> Result<String, String> {
    let img = image::load_from_memory(data)
        .map_err(|e| format!("Failed to decode image: {}", e))?;
    let (w, h) = img.dimensions();
    let max_px: u32 = 1280;
    let resized = if w > max_px || h > max_px {
        let scale = max_px as f32 / w.max(h) as f32;
        let nw = (w as f32 * scale) as u32;
        let nh = (h as f32 * scale) as u32;
        img.resize(nw, nh, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let mut jpeg_buf: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut jpeg_buf);
    resized.write_to(&mut cursor, image::ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&jpeg_buf))
}

/// Tell Ollama to unload a model from VRAM immediately (keep_alive=0).
/// Call this before loading the vision model to ensure there is enough VRAM.
/// Fire-and-forget — errors are silently ignored since this is best-effort.
async fn unload_model(ollama_url: &str, model: &str) {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build() { Ok(c) => c, Err(_) => return };
    let _ = client
        .post(&format!("{}/api/generate", ollama_url))
        .json(&serde_json::json!({
            "model": model,
            "prompt": "",
            "keep_alive": 0
        }))
        .send().await;
}

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
        
        // Wait for the compositor to flush — without this, GNOME X11 returns a black frame
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Capture frame — retry up to 50 times (500ms total) until we get a non-black frame
        let frame = 'outer: {
            for attempt in 0..50 {
                match capturer.frame() {
                    Ok(frame) => {
                        // Check if frame is not all-black (compositor may return black on first frames)
                        let non_black = frame.chunks_exact(4).any(|p| p[0] > 10 || p[1] > 10 || p[2] > 10);
                        if non_black || attempt >= 20 {
                            break 'outer frame.to_vec();
                        }
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(e) => return Err(format!("Failed to capture frame: {}", e)),
                }
            }
            return Err("Screen capture timed out (all frames were black)".to_string());
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

/// OCR via Ollama vision model (replaces Tesseract — no external deps needed)
async fn perform_tesseract_ocr(image_path: PathBuf) -> Result<Vec<OCRResult>, String> {
    let data = tokio::fs::read(&image_path).await
        .map_err(|e| format!("Failed to read image: {}", e))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&data);
    let text = query_vision_ai(
        "Extract all text visible in this image. Return only the raw text, line by line.".to_string(),
        b64, None, None, None,
    ).await?;
    let results = text.lines().filter(|l| !l.trim().is_empty()).enumerate()
        .map(|(i, line)| OCRResult {
            text: line.trim().to_string(),
            confidence: 0.9,
            x: 0, y: i as i32 * 20,
            width: line.len() as i32 * 8, height: 18,
        }).collect();
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

/// Query AI with vision capabilities using Ollama chat API + llama3.2-vision:11b
#[command]
pub async fn query_vision_ai(
    prompt: String,
    image: String, // base64 encoded image (with or without data: prefix)
    _focus_region: Option<serde_json::Value>,
    ollama_host: Option<String>,
    ollama_port: Option<String>,
) -> Result<String, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;
    
    let host = ollama_host.unwrap_or_else(|| std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()));
    let port = ollama_port.unwrap_or_else(|| std::env::var("OLLAMA_PORT").unwrap_or_else(|_| "11434".to_string()));
    
    // Strip data URI prefix if present — Ollama expects raw base64
    let raw_b64 = if image.starts_with("data:") {
        image.splitn(2, ',').nth(1).unwrap_or(&image).to_string()
    } else {
        image
    };
    
    let model = "llama3.2-vision:11b";

    // Unload the text model from VRAM before loading the vision model.
    // RTX 4080 has 16 GB VRAM; codestral:22b uses ~12 GB and
    // llama3.2-vision:11b uses ~8 GB — they cannot coexist.
    let agent_model = std::env::var("AGENT_MODEL")
        .unwrap_or_else(|_| "codestral:22b".to_string());
    let ollama_base = format!("http://{}:{}", host, port);
    unload_model(&ollama_base, &agent_model).await;

    // keep_alive:0 — unload vision model immediately after response so VRAM
    // is freed for the text model on the very next agent step.
    let request_body = serde_json::json!({
        "model": model,
        "messages": [{
            "role": "user",
            "content": prompt,
            "images": [raw_b64]
        }],
        "stream": false,
        "keep_alive": 0,
        "options": { "temperature": 0.3, "num_predict": 1024 }
    });
    
    let ollama_url = format!("http://{}:{}/api/chat", host, port);
    
    let response = client
        .post(&ollama_url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Ollama error {}: {}", response.status(), response.text().await.unwrap_or_default()));
    }
    
    let response_data: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;
    
    let ai_response = response_data["message"]["content"]
        .as_str()
        .unwrap_or("No response from vision model")
        .to_string();
    
    Ok(ai_response)
}

/// One-shot: capture screen + query vision AI. Used by camera button and agent screenshot tool.
#[command]
pub async fn capture_and_ask(
    prompt: String,
    ollama_host: Option<String>,
    ollama_port: Option<String>,
) -> Result<String, String> {
    let temp_path = "/tmp/nexusai-screenshot.png";

    // On Linux/Wayland, scrap often returns a black frame under XWayland.
    // Always try shell tools first on Linux; fall back to scrap only if all fail.
    #[cfg(target_os = "linux")]
    let screen_data = {
        match try_shell_screenshot(temp_path).await {
            Ok(dims) => {
                let data = tokio::fs::read(temp_path).await
                    .map_err(|e| format!("Failed to read screenshot: {}", e))?;
                let _ = tokio::fs::remove_file(temp_path).await;
                ScreenCaptureData { data, width: dims.0, height: dims.1 }
            }
            Err(_) => {
                // Last resort: scrap (works on X11)
                capture_screen().await?
            }
        }
    };

    #[cfg(not(target_os = "linux"))]
    let screen_data = match capture_screen().await {
        Ok(data) => data,
        Err(_) => {
            let _ = try_shell_screenshot(temp_path).await?;
            let data = tokio::fs::read(temp_path).await
                .map_err(|e| format!("Failed to read screenshot: {}", e))?;
            let _ = tokio::fs::remove_file(temp_path).await;
            ScreenCaptureData { data, width: 1920, height: 1080 }
        }
    };

    // Resize to ≤1280 px and convert to JPEG before encoding.
    // A raw 1920×1080 PNG is ~4 MB base64; after resize+JPEG it is ~120 KB.
    // This avoids the VRAM-pressure crash when loading the vision model.
    let b64 = match resize_for_vision(&screen_data.data) {
        Ok(b) => b,
        Err(_) => base64::engine::general_purpose::STANDARD.encode(&screen_data.data),
    };
    query_vision_ai(prompt, b64, None, ollama_host, ollama_port).await
}

/// Try shell-based screenshot tools (Wayland fallback)
async fn try_shell_screenshot(output_path: &str) -> Result<(u32, u32), String> {
    // Try grim (Wayland/GNOME)
    let grim = tokio::process::Command::new("sh")
        .arg("-c").arg(format!("grim {} 2>/dev/null", output_path))
        .output().await;
    if let Ok(out) = grim { if out.status.success() { return Ok((1920, 1080)); } }
    
    // Try scrot (X11)
    let scrot = tokio::process::Command::new("sh")
        .arg("-c").arg(format!("scrot {} 2>/dev/null", output_path))
        .output().await;
    if let Ok(out) = scrot { if out.status.success() { return Ok((1920, 1080)); } }
    
    // Try gnome-screenshot
    let gnome = tokio::process::Command::new("sh")
        .arg("-c").arg(format!("gnome-screenshot -f {} 2>/dev/null", output_path))
        .output().await;
    if let Ok(out) = gnome { if out.status.success() { return Ok((1920, 1080)); } }
    
    // Try import (ImageMagick)
    let import = tokio::process::Command::new("sh")
        .arg("-c").arg(format!("import -window root {} 2>/dev/null", output_path))
        .output().await;
    if let Ok(out) = import { if out.status.success() { return Ok((1920, 1080)); } }
    
    Err("No screen capture tool available (tried grim, scrot, gnome-screenshot, import)".to_string())
}

/// Check if vision dependencies are available.
/// No longer requires Tesseract — we use Ollama vision models instead.
#[command]
pub async fn check_vision_dependencies() -> Result<(), String> {
    // Check if scrap works (X11) OR a shell screenshotter is available (Wayland)
    let scrap_ok = Display::primary().is_ok();
    let shell_ok = std::process::Command::new("sh")
        .arg("-c").arg("which grim || which scrot || which gnome-screenshot || which import")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    
    if !scrap_ok && !shell_ok {
        return Err("No screen capture available. Install grim (Wayland) or scrot (X11).".to_string());
    }
    
    // Check Ollama has a vision model
    let ollama_check = reqwest::Client::new()
        .get("http://127.0.0.1:11434/api/tags")
        .timeout(std::time::Duration::from_secs(3))
        .send().await;
    match ollama_check {
        Ok(r) if r.status().is_success() => {
            if let Ok(body) = r.json::<serde_json::Value>().await {
                let has_vision = body["models"].as_array().map(|models|
                    models.iter().any(|m| {
                        let name = m["name"].as_str().unwrap_or("");
                        name.contains("vision") || name.contains("llava") || name.contains("moondream")
                    })
                ).unwrap_or(false);
                if !has_vision {
                    return Err("No vision model found in Ollama. Pull llava:7b or llama3.2-vision:11b.".to_string());
                }
            }
        }
        _ => return Err("Ollama not running or not accessible at 127.0.0.1:11434".to_string()),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::resize_for_vision;
    use base64::Engine as _;

    /// Build a minimal synthetic PNG of the given dimensions.
    fn make_png(width: u32, height: u32) -> Vec<u8> {
        let img = image::DynamicImage::new_rgb8(width, height);
        let mut buf = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut buf),
            image::ImageFormat::Png,
        )
        .expect("failed to encode test PNG");
        buf
    }

    #[test]
    fn resize_large_image_is_scaled_down() {
        // 1920×1080 PNG — same size as the screenshots that were crashing the app
        let data = make_png(1920, 1080);
        let b64 = resize_for_vision(&data).expect("resize should succeed");
        let jpeg = base64::engine::general_purpose::STANDARD.decode(&b64).unwrap();
        let img = image::load_from_memory(&jpeg).unwrap();
        let (w, h) = (img.width(), img.height());
        assert!(w <= 1280, "width {} should be ≤ 1280", w);
        assert!(h <= 1280, "height {} should be ≤ 1280", h);
        // Aspect ratio preserved: 1920/1080 ≈ 1.778, output should be 1280×720
        assert_eq!(w, 1280);
        assert_eq!(h, 720);
    }

    #[test]
    fn resize_portrait_large_image() {
        // Portrait 1080×1920
        let data = make_png(1080, 1920);
        let b64 = resize_for_vision(&data).expect("resize should succeed");
        let jpeg = base64::engine::general_purpose::STANDARD.decode(&b64).unwrap();
        let img = image::load_from_memory(&jpeg).unwrap();
        assert!(img.height() <= 1280, "height {} should be ≤ 1280", img.height());
        assert_eq!(img.height(), 1280);
        assert_eq!(img.width(), 720);
    }

    #[test]
    fn small_image_dimensions_preserved() {
        // 640×480 — already under 1280, no resize should occur
        let data = make_png(640, 480);
        let b64 = resize_for_vision(&data).expect("resize should succeed");
        let jpeg = base64::engine::general_purpose::STANDARD.decode(&b64).unwrap();
        let img = image::load_from_memory(&jpeg).unwrap();
        assert_eq!(img.width(), 640);
        assert_eq!(img.height(), 480);
    }

    #[test]
    fn exact_boundary_image_not_resized() {
        // Exactly 1280×720 — must not be resized
        let data = make_png(1280, 720);
        let b64 = resize_for_vision(&data).expect("resize should succeed");
        let jpeg = base64::engine::general_purpose::STANDARD.decode(&b64).unwrap();
        let img = image::load_from_memory(&jpeg).unwrap();
        assert_eq!(img.width(), 1280);
        assert_eq!(img.height(), 720);
    }

    #[test]
    fn invalid_data_returns_err() {
        let result = resize_for_vision(b"not an image at all");
        assert!(result.is_err(), "invalid data should return Err");
    }

    #[test]
    fn output_is_jpeg_not_png() {
        // JPEG magic bytes: FF D8 FF
        let data = make_png(400, 300);
        let b64 = resize_for_vision(&data).expect("resize should succeed");
        let bytes = base64::engine::general_purpose::STANDARD.decode(&b64).unwrap();
        assert_eq!(&bytes[..3], &[0xFF, 0xD8, 0xFF], "output should be JPEG");
    }

    #[test]
    fn output_smaller_than_input() {
        // A large PNG should produce a significantly smaller JPEG
        let data = make_png(1920, 1080);
        let b64 = resize_for_vision(&data).expect("resize should succeed");
        let jpeg = base64::engine::general_purpose::STANDARD.decode(&b64).unwrap();
        // The test image is blank (all black) so JPEG is tiny, but the point is it
        // must be smaller than the original PNG which is at least a few KB.
        assert!(jpeg.len() < data.len(), "JPEG output should be smaller than PNG input");
    }
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
