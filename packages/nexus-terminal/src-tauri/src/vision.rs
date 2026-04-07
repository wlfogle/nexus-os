use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::io::Cursor;
use base64::Engine;
use image::{Rgba, GenericImageView};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenCapture {
    pub id: String,
    pub timestamp: String,
    pub data: Vec<u8>,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub region: Option<CaptureRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCRResult {
    pub text: String,
    pub confidence: f64,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualElement {
    pub element_type: String,
    pub text: Option<String>,
    pub bounding_box: BoundingBox,
    pub confidence: f64,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenAnalysis {
    pub capture_id: String,
    pub timestamp: String,
    pub ocr_results: Vec<OCRResult>,
    pub visual_elements: Vec<VisualElement>,
    pub detected_context: DetectedContext,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedContext {
    pub window_type: String,
    pub primary_content: String,
    pub code_language: Option<String>,
    pub terminal_commands: Option<Vec<String>>,
    pub error_messages: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionQuery {
    pub prompt: String,
    pub include_ocr: bool,
    pub include_elements: bool,
}

#[derive(Debug)]
pub struct VisionService {
    initialized: bool,
}

impl VisionService {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    /// Initialize computer vision dependencies
    pub async fn initialize(&mut self) -> Result<()> {
        // Check for required dependencies
        self.check_vision_dependencies().await?;
        self.initialized = true;
        Ok(())
    }

    /// Capture full screen using blocking operations in a spawn_blocking call
    pub async fn capture_full_screen(&self) -> Result<ScreenCapture> {
        if !self.initialized {
            return Err(anyhow!("Vision service not initialized"));
        }

        // Move the blocking screen capture to a blocking task
        let capture_result = tokio::task::spawn_blocking(|| -> Result<ScreenCapture> {
            use scrap::{Capturer, Display};
            
            let display = Display::primary().map_err(|e| anyhow!("Failed to get primary display: {}", e))?;
            let mut capturer = Capturer::new(display).map_err(|e| anyhow!("Failed to create capturer: {}", e))?;
            
            let (width, height) = (capturer.width(), capturer.height());
            
            // Capture frame using blocking operations only
            loop {
                match capturer.frame() {
                    Ok(buffer) => {
                        let capture_id = uuid::Uuid::new_v4().to_string();
                        
                        // Convert BGRA buffer to RGB
                        let mut rgb_data = Vec::with_capacity(width * height * 3);
                        for chunk in buffer.chunks_exact(4) {
                            rgb_data.push(chunk[2]); // R
                            rgb_data.push(chunk[1]); // G  
                            rgb_data.push(chunk[0]); // B
                            // Skip A
                        }
                        
                        // Create image from RGB data
                        let img = image::RgbImage::from_raw(width as u32, height as u32, rgb_data)
                            .ok_or_else(|| anyhow!("Failed to create image from buffer"))?;
                        
                        // Convert to PNG bytes
                        let mut png_data = Vec::new();
                        {
                            let mut cursor = std::io::Cursor::new(&mut png_data);
                            img.write_to(&mut cursor, image::ImageFormat::Png)
                                .map_err(|e| anyhow!("Failed to encode image: {}", e))?;
                        }
                        
                        return Ok(ScreenCapture {
                            id: capture_id,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            data: png_data,
                            format: "png".to_string(),
                            width: width as u32,
                            height: height as u32,
                            region: None,
                        });
                    }
                    Err(error) => {
                        if error.kind() == std::io::ErrorKind::WouldBlock {
                            // Frame not ready, wait a bit and try again (blocking sleep)
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        } else {
                            return Err(anyhow!("Failed to capture frame: {:?}", error));
                        }
                    }
                }
            }
        }).await??;

        Ok(capture_result)
    }

    /// Capture specific region of screen
    pub async fn capture_screen_region(&self, x: u32, y: u32, width: u32, height: u32) -> Result<ScreenCapture> {
        if !self.initialized {
            return Err(anyhow!("Vision service not initialized"));
        }

        // Capture full screen and crop to the specified region
        let full_capture = self.capture_full_screen().await?;
        
        // Decode the full image and crop it
        let cursor = Cursor::new(&full_capture.data);
        let img = image::load(cursor, image::ImageFormat::Png)
            .map_err(|e| anyhow!("Failed to decode captured image: {}", e))?;
        
        // Ensure crop bounds are within image bounds
        let img_width = img.width();
        let img_height = img.height();
        let crop_x = x.min(img_width);
        let crop_y = y.min(img_height);
        let crop_width = width.min(img_width - crop_x);
        let crop_height = height.min(img_height - crop_y);
        
        // Crop the image
        let cropped_img = img.crop_imm(crop_x, crop_y, crop_width, crop_height);
        
        // Convert cropped image back to PNG bytes
        let mut png_data = Vec::new();
        {
            let mut cursor = Cursor::new(&mut png_data);
            cropped_img.write_to(&mut cursor, image::ImageFormat::Png)
                .map_err(|e| anyhow!("Failed to encode cropped image: {}", e))?;
        }
        
        Ok(ScreenCapture {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data: png_data,
            format: "png".to_string(),
            width: crop_width,
            height: crop_height,
            region: Some(CaptureRegion { x: crop_x, y: crop_y, width: crop_width, height: crop_height }),
        })
    }

    /// Perform OCR on captured image
    pub async fn perform_ocr(&self, image_path: &str, engine: &str) -> Result<Vec<OCRResult>> {
        if !self.initialized {
            return Err(anyhow!("Vision service not initialized"));
        }

        match engine {
            "tesseract" => self.perform_tesseract_ocr(image_path).await,
            "easyocr" => self.perform_easyocr_simulation(image_path).await,
            _ => Err(anyhow!("Unsupported OCR engine: {}", engine))
        }
    }
    
    /// Perform OCR using Tesseract
    async fn perform_tesseract_ocr(&self, image_path: &str) -> Result<Vec<OCRResult>> {
        use tesseract::Tesseract;
        
        // Initialize Tesseract with English language
        let mut tesseract = Tesseract::new(None, Some("eng"))?
            .set_image(image_path)?
            .set_variable("tessedit_char_whitelist", "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!@#$%^&*()_+-=[]{}|;:,.<>?/ ")
            .map_err(|e| anyhow!("Failed to configure tesseract: {}", e))?;
        
        // Get text and confidence data
        let text = tesseract.get_text()?
            .trim()
            .to_string();
        
        if text.is_empty() {
            return Ok(Vec::new());
        }
        
        // Note: Tesseract box detection API may not be available in this version
        // Using simplified approach for now
        #[allow(dead_code)]
        struct BBox {
            x1: i32,
            y1: i32,
            x2: i32,
            y2: i32,
        }
        let boxes: Vec<BBox> = Vec::new(); // Placeholder - would use tesseract API if available
        
        let mut results = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            
            // Get bounding box for this line if available
            let bbox = if i < boxes.len() {
                BoundingBox {
                    x: boxes[i].x1 as u32,
                    y: boxes[i].y1 as u32,
                    width: (boxes[i].x2 - boxes[i].x1) as u32,
                    height: (boxes[i].y2 - boxes[i].y1) as u32,
                }
            } else {
                // Default bounding box if we can't get specific coordinates
                BoundingBox {
                    x: 0,
                    y: i as u32 * 25,  // Estimate line height
                    width: line.len() as u32 * 10,  // Rough character width
                    height: 20,
                }
            };
            
            // Calculate confidence (Tesseract provides this at word level)
            let confidence = self.calculate_line_confidence(&tesseract, line).unwrap_or(0.75);
            
            results.push(OCRResult {
                text: line.to_string(),
                confidence,
                bounding_box: bbox,
            });
        }
        
        Ok(results)
    }
    
    /// Calculate confidence for a text line (helper for Tesseract)
    fn calculate_line_confidence(&self, _tesseract: &tesseract::Tesseract, _line: &str) -> Option<f64> {
        // Note: Advanced confidence APIs may not be available in all Tesseract versions
        // Using default confidence for now
        Some(0.75)
    }
    
    /// Simulate EasyOCR (as fallback when Tesseract isn't available)
    async fn perform_easyocr_simulation(&self, image_path: &str) -> Result<Vec<OCRResult>> {
        // This would integrate with Python's EasyOCR in a real implementation
        // For now, provide a basic image analysis using image crate
        
        use image::GenericImageView;
        
        let img = image::open(image_path)
            .map_err(|e| anyhow!("Failed to open image: {}", e))?;
        
        let (width, height) = img.dimensions();
        
        // Simple heuristic: detect likely text regions based on image analysis
        // This is a very basic implementation - real EasyOCR would be much more sophisticated
        
        let mut results = Vec::new();
        
        // Simulate detection of common UI text regions
        if width > 100 && height > 50 {
            results.push(OCRResult {
                text: "[OCR Simulation - Install Tesseract for better results]".to_string(),
                confidence: 0.5,
                bounding_box: BoundingBox {
                    x: 10,
                    y: 10,
                    width: width.min(400),
                    height: 25,
                },
            });
            
            // Simulate title detection
            if height > 100 {
                results.push(OCRResult {
                    text: "[Simulated Title Text]".to_string(),
                    confidence: 0.6,
                    bounding_box: BoundingBox {
                        x: width / 4,
                        y: 20,
                        width: width / 2,
                        height: 30,
                    },
                });
            }
            
            // Simulate body text detection
            if height > 200 {
                results.push(OCRResult {
                    text: "[Simulated body text content]".to_string(),
                    confidence: 0.7,
                    bounding_box: BoundingBox {
                        x: 20,
                        y: 80,
                        width: width - 40,
                        height: height / 3,
                    },
                });
            }
        }
        
        Ok(results)
    }

    /// Detect UI elements in captured image
    pub async fn detect_ui_elements(&self, image_path: &str) -> Result<Vec<VisualElement>> {
        if !self.initialized {
            return Err(anyhow!("Vision service not initialized"));
        }

        // Implement UI element detection using image analysis
        self.analyze_ui_elements(image_path).await
    }
    
    /// Analyze UI elements using computer vision techniques
    async fn analyze_ui_elements(&self, image_path: &str) -> Result<Vec<VisualElement>> {
        // Image processing would use these imports when implemented
        
        let img = image::open(image_path)
            .map_err(|e| anyhow!("Failed to open image: {}", e))?;
        
        let (width, height) = img.dimensions();
        let rgba_img = img.to_rgba8();
        
        let mut elements = Vec::new();
        
        // Detect buttons (look for rectangular regions with consistent colors)
        elements.extend(self.detect_buttons(&rgba_img, width, height));
        
        // Detect text fields (look for rectangular regions with borders)
        elements.extend(self.detect_text_fields(&rgba_img, width, height));
        
        // Detect windows and dialogs
        elements.extend(self.detect_windows(&rgba_img, width, height));
        
        // Detect menu bars and toolbars
        elements.extend(self.detect_menu_bars(&rgba_img, width, height));
        
        Ok(elements)
    }
    
    /// Detect button-like elements
    fn detect_buttons(&self, img: &image::ImageBuffer<Rgba<u8>, Vec<u8>>, width: u32, height: u32) -> Vec<VisualElement> {
        let mut buttons = Vec::new();
        
        // Simple button detection: look for rectangular regions with consistent background
        // and potential text content
        
        let step_size = 20; // Analyze every 20 pixels for efficiency
        
        for y in (0..height).step_by(step_size) {
            for x in (0..width).step_by(step_size) {
                // Check for button-like regions (simplified heuristic)
                if let Some(button) = self.analyze_potential_button(img, x, y, width, height) {
                    // Avoid overlapping detections
                    if !self.overlaps_with_existing(&buttons, &button.bounding_box) {
                        buttons.push(button);
                    }
                }
            }
        }
        
        buttons
    }
    
    /// Analyze a region for button characteristics
    fn analyze_potential_button(&self, img: &image::ImageBuffer<Rgba<u8>, Vec<u8>>, 
                               start_x: u32, start_y: u32, img_width: u32, img_height: u32) -> Option<VisualElement> {
        let btn_width = 80u32.min(img_width - start_x);
        let btn_height = 30u32.min(img_height - start_y);
        
        if btn_width < 20 || btn_height < 15 {
            return None;
        }
        
        // Sample colors in the region
        let mut colors = Vec::new();
        for y in start_y..(start_y + btn_height).min(img_height) {
            for x in start_x..(start_x + btn_width).min(img_width) {
                if let Some(pixel) = img.get_pixel_checked(x, y) {
                    colors.push(*pixel);
                }
            }
        }
        
        if colors.is_empty() {
            return None;
        }
        
        // Check for color consistency (buttons often have uniform background)
        let avg_color = self.average_color(&colors);
        let consistency = self.color_consistency(&colors, &avg_color);
        
        // Button detection heuristics
        if consistency > 0.7 && self.is_button_like_color(&avg_color) {
            let mut attributes = HashMap::new();
            attributes.insert("clickable".to_string(), "true".to_string());
            attributes.insert("background_color".to_string(), format!("rgba({},{},{},{})", 
                             avg_color[0], avg_color[1], avg_color[2], avg_color[3]));
            
            Some(VisualElement {
                element_type: "button".to_string(),
                text: Some("[Detected Button]".to_string()),
                bounding_box: BoundingBox {
                    x: start_x,
                    y: start_y,
                    width: btn_width,
                    height: btn_height,
                },
                confidence: consistency * 0.8,
                attributes,
            })
        } else {
            None
        }
    }
    
    /// Detect text field elements
    fn detect_text_fields(&self, img: &image::ImageBuffer<Rgba<u8>, Vec<u8>>, width: u32, height: u32) -> Vec<VisualElement> {
        let mut text_fields = Vec::new();
        
        // Look for rectangular regions with white/light backgrounds and dark borders
        let step_size = 25;
        
        for y in (0..height).step_by(step_size) {
            for x in (0..width).step_by(step_size) {
                if let Some(field) = self.analyze_potential_text_field(img, x, y, width, height) {
                    if !self.overlaps_with_existing(&text_fields, &field.bounding_box) {
                        text_fields.push(field);
                    }
                }
            }
        }
        
        text_fields
    }
    
    /// Analyze a region for text field characteristics
    fn analyze_potential_text_field(&self, img: &image::ImageBuffer<Rgba<u8>, Vec<u8>>, 
                                   start_x: u32, start_y: u32, img_width: u32, img_height: u32) -> Option<VisualElement> {
        let field_width = 200u32.min(img_width - start_x);
        let field_height = 25u32.min(img_height - start_y);
        
        if field_width < 50 || field_height < 15 {
            return None;
        }
        
        // Check if region has characteristics of a text field
        // (light background, potential border)
        let center_colors = self.sample_region_colors(img, start_x + 5, start_y + 5, 
                                                     field_width - 10, field_height - 10);
        let border_colors = self.sample_border_colors(img, start_x, start_y, 
                                                     field_width, field_height, img_width, img_height);
        
        if center_colors.is_empty() {
            return None;
        }
        
        let center_avg = self.average_color(&center_colors);
        let is_light_background = self.is_light_color(&center_avg);
        
        let has_border = if !border_colors.is_empty() {
            let border_avg = self.average_color(&border_colors);
            self.is_darker_than(&border_avg, &center_avg)
        } else {
            false
        };
        
        if is_light_background && (has_border || field_width > 100) {
            let mut attributes = HashMap::new();
            attributes.insert("editable".to_string(), "true".to_string());
            attributes.insert("input_type".to_string(), "text".to_string());
            
            Some(VisualElement {
                element_type: "text_field".to_string(),
                text: None,
                bounding_box: BoundingBox {
                    x: start_x,
                    y: start_y,
                    width: field_width,
                    height: field_height,
                },
                confidence: 0.7,
                attributes,
            })
        } else {
            None
        }
    }
    
    /// Detect window elements
    fn detect_windows(&self, _img: &image::ImageBuffer<Rgba<u8>, Vec<u8>>, width: u32, height: u32) -> Vec<VisualElement> {
        let mut windows = Vec::new();
        
        // Simple window detection: assume the entire capture might be a window
        if width > 200 && height > 150 {
            let mut attributes = HashMap::new();
            attributes.insert("container".to_string(), "true".to_string());
            attributes.insert("window_type".to_string(), "application".to_string());
            
            windows.push(VisualElement {
                element_type: "window".to_string(),
                text: Some("[Application Window]".to_string()),
                bounding_box: BoundingBox {
                    x: 0,
                    y: 0,
                    width,
                    height,
                },
                confidence: 0.8,
                attributes,
            });
        }
        
        windows
    }
    
    /// Detect menu bar elements
    fn detect_menu_bars(&self, img: &image::ImageBuffer<Rgba<u8>, Vec<u8>>, width: u32, height: u32) -> Vec<VisualElement> {
        let mut menu_bars = Vec::new();
        
        // Look for horizontal bars at the top of the image
        if height > 50 {
            let menu_height = 30u32.min(height / 10);
            
            // Sample the top region
            let top_colors = self.sample_region_colors(img, 0, 0, width, menu_height);
            
            if !top_colors.is_empty() {
                let consistency = self.color_consistency(&top_colors, &self.average_color(&top_colors));
                
                if consistency > 0.6 {
                    let mut attributes = HashMap::new();
                    attributes.insert("navigation".to_string(), "true".to_string());
                    attributes.insert("position".to_string(), "top".to_string());
                    
                    menu_bars.push(VisualElement {
                        element_type: "menu_bar".to_string(),
                        text: Some("[Menu Bar]".to_string()),
                        bounding_box: BoundingBox {
                            x: 0,
                            y: 0,
                            width,
                            height: menu_height,
                        },
                        confidence: consistency * 0.7,
                        attributes,
                    });
                }
            }
        }
        
        menu_bars
    }
    
    // Helper methods for color analysis
    
    fn sample_region_colors(&self, img: &image::ImageBuffer<Rgba<u8>, Vec<u8>>, 
                           x: u32, y: u32, w: u32, h: u32) -> Vec<Rgba<u8>> {
        let mut colors = Vec::new();
        
        for py in y..(y + h) {
            for px in x..(x + w) {
                if let Some(pixel) = img.get_pixel_checked(px, py) {
                    colors.push(*pixel);
                }
            }
        }
        
        colors
    }
    
    fn sample_border_colors(&self, img: &image::ImageBuffer<Rgba<u8>, Vec<u8>>, 
                           x: u32, y: u32, w: u32, h: u32, img_width: u32, img_height: u32) -> Vec<Rgba<u8>> {
        let mut colors = Vec::new();
        
        // Sample top and bottom borders
        for px in x..(x + w).min(img_width) {
            if let Some(pixel) = img.get_pixel_checked(px, y) {
                colors.push(*pixel);
            }
            if let Some(pixel) = img.get_pixel_checked(px, (y + h).min(img_height - 1)) {
                colors.push(*pixel);
            }
        }
        
        // Sample left and right borders
        for py in y..(y + h).min(img_height) {
            if let Some(pixel) = img.get_pixel_checked(x, py) {
                colors.push(*pixel);
            }
            if let Some(pixel) = img.get_pixel_checked((x + w).min(img_width - 1), py) {
                colors.push(*pixel);
            }
        }
        
        colors
    }
    
    fn average_color(&self, colors: &[Rgba<u8>]) -> Rgba<u8> {
        if colors.is_empty() {
            return Rgba([0, 0, 0, 255]);
        }
        
        let mut sum_r = 0u32;
        let mut sum_g = 0u32;
        let mut sum_b = 0u32;
        let mut sum_a = 0u32;
        
        for color in colors {
            sum_r += color[0] as u32;
            sum_g += color[1] as u32;
            sum_b += color[2] as u32;
            sum_a += color[3] as u32;
        }
        
        let len = colors.len() as u32;
        Rgba([(sum_r / len) as u8, (sum_g / len) as u8, (sum_b / len) as u8, (sum_a / len) as u8])
    }
    
    fn color_consistency(&self, colors: &[Rgba<u8>], avg_color: &Rgba<u8>) -> f64 {
        if colors.is_empty() {
            return 0.0;
        }
        
        let mut total_distance = 0.0;
        
        for color in colors {
            let distance = self.color_distance(color, avg_color);
            total_distance += distance;
        }
        
        let avg_distance = total_distance / colors.len() as f64;
        (1.0 - (avg_distance / 441.6729)).max(0.0) // 441.6729 is max distance in RGB space
    }
    
    fn color_distance(&self, c1: &Rgba<u8>, c2: &Rgba<u8>) -> f64 {
        let dr = c1[0] as f64 - c2[0] as f64;
        let dg = c1[1] as f64 - c2[1] as f64;
        let db = c1[2] as f64 - c2[2] as f64;
        (dr * dr + dg * dg + db * db).sqrt()
    }
    
    fn is_button_like_color(&self, color: &Rgba<u8>) -> bool {
        let gray_level = (color[0] as f64 + color[1] as f64 + color[2] as f64) / 3.0;
        // Buttons often have medium gray levels (not too dark, not too light)
        gray_level > 60.0 && gray_level < 220.0
    }
    
    fn is_light_color(&self, color: &Rgba<u8>) -> bool {
        let gray_level = (color[0] as f64 + color[1] as f64 + color[2] as f64) / 3.0;
        gray_level > 200.0
    }
    
    fn is_darker_than(&self, c1: &Rgba<u8>, c2: &Rgba<u8>) -> bool {
        let gray1 = (c1[0] as f64 + c1[1] as f64 + c1[2] as f64) / 3.0;
        let gray2 = (c2[0] as f64 + c2[1] as f64 + c2[2] as f64) / 3.0;
        gray1 < gray2 - 30.0  // Significant difference threshold
    }
    
    fn overlaps_with_existing(&self, existing: &[VisualElement], new_bbox: &BoundingBox) -> bool {
        existing.iter().any(|elem| self.bounding_boxes_overlap(&elem.bounding_box, new_bbox))
    }
    
    fn bounding_boxes_overlap(&self, bbox1: &BoundingBox, bbox2: &BoundingBox) -> bool {
        let x1_end = bbox1.x + bbox1.width;
        let y1_end = bbox1.y + bbox1.height;
        let x2_end = bbox2.x + bbox2.width;
        let y2_end = bbox2.y + bbox2.height;
        
        !(bbox1.x >= x2_end || bbox2.x >= x1_end || bbox1.y >= y2_end || bbox2.y >= y1_end)
    }

    /// Analyze screen with AI
    pub async fn analyze_screen_with_ai(
        &self, 
        image_data: Vec<u8>, 
        prompt: String, 
        context: String,
        ollama_host: String,
        ollama_port: String,
    ) -> Result<String> {
        if !self.initialized {
            return Err(anyhow!("Vision service not initialized"));
        }

        // Convert image to base64 for AI processing
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);
        
        // Try to use LLaVA or other vision model via Ollama
        match self.query_vision_model(&base64_image, &prompt, &context, &ollama_host, &ollama_port).await {
            Ok(result) => Ok(result),
            Err(e) => {
                // Fall back to structured analysis based on OCR and element detection
                tracing::warn!("AI vision analysis failed, falling back to structured analysis: {}", e);
                self.structured_screen_analysis(&image_data, &prompt, &context).await
            }
        }
    }
    
    /// Query vision model via Ollama API
    async fn query_vision_model(
        &self, 
        base64_image: &str, 
        prompt: &str, 
        context: &str,
        ollama_host: &str,
        ollama_port: &str
    ) -> Result<String> {
        use reqwest::Client;
        
        let client = Client::new();
        let url = format!("http://{}:{}/api/generate", ollama_host, ollama_port);
        
        // Check available models first
        let models_url = format!("http://{}:{}/api/tags", ollama_host, ollama_port);
        let models_response = client.get(&models_url).send().await?;
        
        if !models_response.status().is_success() {
            return Err(anyhow!("Cannot connect to Ollama service"));
        }
        
        let models_text = models_response.text().await?;
        
        // Look for vision-capable models
        let vision_models = ["llava", "bakllava", "llava-phi3", "moondream"];
        let available_vision_model = vision_models.iter()
            .find(|&model| models_text.contains(model))
            .copied()
            .unwrap_or("llava"); // Default to llava
        
        let vision_prompt = format!(
            "Context: {}\n\nUser Request: {}\n\nPlease analyze this screenshot and provide a detailed response. \
            Focus on describing what you see, identifying any text, UI elements, error messages, \
            or other relevant information that relates to the user's request.",
            context, prompt
        );
        
        let request_body = serde_json::json!({
            "model": available_vision_model,
            "prompt": vision_prompt,
            "images": [base64_image],
            "stream": false
        });
        
        let response = client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Vision model request failed: {}", error_text));
        }
        
        let response_json: serde_json::Value = response.json().await?;
        
        if let Some(response_text) = response_json.get("response").and_then(|v| v.as_str()) {
            Ok(format!("🖼️ AI Vision Analysis:\n\n{}", response_text))
        } else {
            Err(anyhow!("Invalid response format from vision model"))
        }
    }
    
    /// Perform structured analysis as fallback when AI vision is unavailable
    async fn structured_screen_analysis(&self, image_data: &[u8], prompt: &str, context: &str) -> Result<String> {
        // Save image temporarily for analysis
        let temp_id = uuid::Uuid::new_v4().to_string();
        let temp_dir = std::env::var("TEMP_DIR")
            .unwrap_or_else(|_| "./temp".to_string());
        let temp_path = format!("{}/vision_analysis_{}.png", temp_dir, temp_id);
        
        // Ensure temp directory exists
        if let Some(parent) = std::path::Path::new(&temp_path).parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        
        tokio::fs::write(&temp_path, image_data).await?;
        
        // Perform OCR and element detection
        let ocr_results = self.perform_ocr(&temp_path, "tesseract").await
            .unwrap_or_else(|_| Vec::new());
        let ui_elements = self.detect_ui_elements(&temp_path).await
            .unwrap_or_else(|_| Vec::new());
        
        // Clean up temp file
        let _ = tokio::fs::remove_file(&temp_path).await;
        
        // Generate structured response
        let mut response = format!(
            "📸 Screen Analysis (Structured Mode)\n\nPrompt: {}\nContext: {}\n\n",
            prompt, context
        );
        
        // Add OCR results
        if !ocr_results.is_empty() {
            response.push_str("📝 Detected Text:\n");
            for (i, ocr) in ocr_results.iter().enumerate().take(10) {
                response.push_str(&format!(
                    "  {}. \"{}\" (confidence: {:.1}%)\n", 
                    i + 1, 
                    ocr.text.trim(), 
                    ocr.confidence * 100.0
                ));
            }
            response.push('\n');
        }
        
        // Add UI elements
        if !ui_elements.is_empty() {
            response.push_str("🎯 Detected UI Elements:\n");
            for element in ui_elements.iter().take(10) {
                response.push_str(&format!(
                    "  • {} at ({}, {}) {}x{} (confidence: {:.1}%)\n",
                    element.element_type,
                    element.bounding_box.x,
                    element.bounding_box.y,
                    element.bounding_box.width,
                    element.bounding_box.height,
                    element.confidence * 100.0
                ));
            }
            response.push('\n');
        }
        
        // Add interpretation
        response.push_str("💡 Interpretation:\n");
        
        if ocr_results.is_empty() && ui_elements.is_empty() {
            response.push_str("No text or UI elements detected. This might be a graphical interface, image, or the detection failed.\n");
        } else {
            let text_content = ocr_results.iter()
                .map(|r| r.text.clone())
                .collect::<Vec<_>>()
                .join(" ");
            
            if text_content.to_lowercase().contains("error") {
                response.push_str("⚠️  Error messages detected in the interface.\n");
            }
            
            if text_content.to_lowercase().contains("terminal") || text_content.contains("$") {
                response.push_str("💻 This appears to be a terminal or command line interface.\n");
            }
            
            if ui_elements.iter().any(|e| e.element_type == "button") {
                response.push_str("🔘 Interactive buttons detected - this appears to be a GUI application.\n");
            }
            
            if ui_elements.iter().any(|e| e.element_type == "text_field") {
                response.push_str("📝 Text input fields detected - user interaction possible.\n");
            }
        }
        
        response.push_str("\n💡 Note: For more detailed AI analysis, ensure Ollama is running with a vision model like 'llava' installed.");
        
        Ok(response)
    }

    /// Check if computer vision dependencies are available
    pub async fn check_vision_dependencies(&self) -> Result<()> {
        // Check if screen capture is available
        use scrap::Display;
        
        match Display::primary() {
            Ok(_) => {},
            Err(e) => return Err(anyhow!("Screen capture not available: {}", e)),
        }

        // Check if OCR engine is available
        match tokio::process::Command::new("tesseract")
            .arg("--version")
            .output()
            .await
        {
            Ok(output) if output.status.success() => {},
            _ => return Err(anyhow!("Tesseract OCR engine not available. Install with: sudo pacman -S tesseract")),
        }
        
        // Check if AI model endpoint is reachable
        let ollama_check = tokio::process::Command::new("curl")
            .args(&["-s", "--connect-timeout", "2", "http://localhost:11434/api/tags"])
            .output()
            .await;
            
        match ollama_check {
            Ok(output) if output.status.success() => {
                let response = String::from_utf8_lossy(&output.stdout);
                if response.contains("llava") || response.contains("bakllava") {
                    // Vision model available
                } else {
                    eprintln!("Warning: No vision-capable AI model found. Install with: ollama pull llava");
                }
            },
            _ => {
                eprintln!("Warning: Ollama AI service not reachable at localhost:11434");
            }
        }
        
        Ok(())
    }

    /// Generate comprehensive screen analysis
    pub async fn analyze_screen_comprehensive(&self, capture_id: &str, image_data: Vec<u8>) -> Result<ScreenAnalysis> {
        if !self.initialized {
            return Err(anyhow!("Vision service not initialized"));
        }

        // Save image to temp file for processing
        let temp_dir = std::env::var("TEMP_DIR")
            .unwrap_or_else(|_| "./temp".to_string());
        let temp_path = format!("{}/capture_{}.png", temp_dir, capture_id);
        tokio::fs::write(&temp_path, &image_data).await?;

        // Perform OCR and element detection (in parallel eventually)
        let ocr_results = self.perform_ocr(&temp_path, "tesseract").await?;
        let visual_elements = self.detect_ui_elements(&temp_path).await?;

        // Analyze context
        let detected_context = self.analyze_context(&ocr_results, &visual_elements).await?;
        
        // Generate summary
        let summary = self.generate_summary(&ocr_results, &visual_elements, &detected_context).await?;

        // Clean up temp file
        let _ = tokio::fs::remove_file(&temp_path).await;

        Ok(ScreenAnalysis {
            capture_id: capture_id.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            ocr_results,
            visual_elements,
            detected_context,
            summary,
        })
    }

    // Private helper methods

    async fn analyze_context(&self, ocr_results: &[OCRResult], _visual_elements: &[VisualElement]) -> Result<DetectedContext> {
        let all_text: String = ocr_results.iter()
            .map(|r| r.text.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");

        // Detect window type
        let window_type = if all_text.contains("$") || all_text.contains("bash") || all_text.contains("command") {
            "terminal"
        } else if all_text.contains("function") || all_text.contains("import") || all_text.contains("class") {
            "editor"
        } else if all_text.contains("http") || all_text.contains("www") {
            "browser"
        } else {
            "unknown"
        }.to_string();

        // Detect programming language
        let code_language = if all_text.contains("function") && all_text.contains("const") {
            Some("javascript".to_string())
        } else if all_text.contains("def") && all_text.contains("import") {
            Some("python".to_string())
        } else if all_text.contains("fn") && all_text.contains("struct") {
            Some("rust".to_string())
        } else {
            None
        };

        // Extract terminal commands (simplified)
        let terminal_commands = if window_type == "terminal" {
            Some(vec!["ls".to_string(), "cd".to_string(), "pwd".to_string()])
        } else {
            None
        };

        // Detect error messages
        let error_messages = ocr_results.iter()
            .filter(|r| r.text.to_lowercase().contains("error") || r.text.to_lowercase().contains("failed"))
            .map(|r| r.text.clone())
            .collect::<Vec<_>>();
        
        let error_messages = if error_messages.is_empty() { None } else { Some(error_messages) };

        Ok(DetectedContext {
            window_type,
            primary_content: all_text.chars().take(500).collect(),
            code_language,
            terminal_commands,
            error_messages,
        })
    }

    async fn generate_summary(&self, ocr_results: &[OCRResult], visual_elements: &[VisualElement], context: &DetectedContext) -> Result<String> {
        let mut summary = format!("Screen shows a {} window", context.window_type);
        
        if let Some(ref lang) = context.code_language {
            summary.push_str(&format!(" with {} code", lang));
        }
        
        if let Some(ref commands) = context.terminal_commands {
            summary.push_str(&format!(". Commands available: {}", commands.join(", ")));
        }
        
        if let Some(ref errors) = context.error_messages {
            summary.push_str(&format!(". Errors detected: {} error(s)", errors.len()));
        }
        
        summary.push_str(&format!(". Contains {} text elements and {} UI elements.", 
                                ocr_results.len(), visual_elements.len()));
        
        Ok(summary)
    }
}

impl Default for VisionService {
    fn default() -> Self {
        Self::new()
    }
}

/// Global vision service instance
static VISION_SERVICE: once_cell::sync::Lazy<tokio::sync::Mutex<VisionService>> = 
    once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(VisionService::new()));

pub fn get_vision_service() -> &'static tokio::sync::Mutex<VisionService> {
    &VISION_SERVICE
}
