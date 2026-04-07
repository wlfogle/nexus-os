import { invoke } from '@tauri-apps/api/core';
import { visionLogger } from '../utils/logger';

export interface ScreenCapture {
  id: string;
  timestamp: string;
  data: Uint8Array;
  format: 'png' | 'jpeg';
  width: number;
  height: number;
  region?: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
}

export interface OCRResult {
  text: string;
  confidence: number;
  boundingBox: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
}

export interface VisualElement {
  type: 'button' | 'text' | 'input' | 'window' | 'code' | 'terminal' | 'menu' | 'icon';
  text?: string;
  boundingBox: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  confidence: number;
  attributes: Record<string, any>;
}

export interface ScreenAnalysis {
  captureId: string;
  timestamp: string;
  ocrResults: OCRResult[];
  visualElements: VisualElement[];
  detectedContext: {
    windowType: 'terminal' | 'editor' | 'browser' | 'desktop' | 'unknown';
    primaryContent: string;
    codeLanguage?: string;
    terminalCommands?: string[];
    errorMessages?: string[];
  };
  summary: string;
}

export interface VisionQuery {
  prompt: string;
  includeOCR?: boolean;
  includeElements?: boolean;
  focusRegion?: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
}

class VisionService {
  private isInitialized = false;
  private ocrEngine: 'tesseract' | 'paddleocr' | 'easyocr' = 'tesseract';
  private captures: Map<string, ScreenCapture> = new Map();

  /**
   * Initialize computer vision service
   */
  async initialize(): Promise<void> {
    try {
      // Check if required dependencies are available
      await this.checkDependencies();
      this.isInitialized = true;
      visionLogger.info('Vision service initialized successfully', 'init_complete');
    } catch (error) {
      visionLogger.error('Failed to initialize vision service', error as Error, 'init_failed');
      throw error;
    }
  }

  /**
   * Capture the entire screen
   */
  async captureScreen(): Promise<ScreenCapture> {
    visionLogger.debug('Screen capture requested', 'capture_screen', { initialized: this.isInitialized });
    if (!this.isInitialized) {
      visionLogger.info('Initializing vision service before capture', 'init_before_capture');
      await this.initialize();
    }

    try {
      visionLogger.debug('Executing backend screen capture command', 'backend_capture');
      const captureResult = await invoke('capture_screen') as {
        data: number[];
        width: number;
        height: number;
      };

      const capture: ScreenCapture = {
        id: `screen_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        timestamp: new Date().toISOString(),
        data: new Uint8Array(captureResult.data),
        format: 'png',
        width: captureResult.width,
        height: captureResult.height
      };

      this.captures.set(capture.id, capture);
      return capture;
    } catch (error) {
      visionLogger.error('Screen capture failed', error as Error, 'capture_failed');
      throw error;
    }
  }

  /**
   * Capture a specific region of the screen
   */
  async captureRegion(x: number, y: number, width: number, height: number): Promise<ScreenCapture> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    try {
      const captureResult = await invoke('capture_screen_region', {
        x, y, width, height
      }) as {
        data: number[];
        width: number;
        height: number;
      };

      const capture: ScreenCapture = {
        id: `region_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        timestamp: new Date().toISOString(),
        data: new Uint8Array(captureResult.data),
        format: 'png',
        width: captureResult.width,
        height: captureResult.height,
        region: { x, y, width, height }
      };

      this.captures.set(capture.id, capture);
      return capture;
    } catch (error) {
      visionLogger.error('Screen region capture failed', error as Error, 'region_capture_failed', { region: { x, y, width, height } });
      throw error;
    }
  }

  /**
   * Perform OCR on a screen capture
   */
  async performOCR(captureId: string): Promise<OCRResult[]> {
    const capture = this.captures.get(captureId);
    if (!capture) {
      throw new Error(`Capture ${captureId} not found`);
    }

    try {
      // Save capture to temporary file for processing
      const tempPath = await this.saveCaptureToTemp(capture);
      
      const ocrResults = await invoke('perform_ocr', {
        imagePath: tempPath,
        engine: this.ocrEngine
      }) as Array<{
        text: string;
        confidence: number;
        x: number;
        y: number;
        width: number;
        height: number;
      }>;

      return ocrResults.map(result => ({
        text: result.text,
        confidence: result.confidence,
        boundingBox: {
          x: result.x,
          y: result.y,
          width: result.width,
          height: result.height
        }
      }));
    } catch (error) {
      visionLogger.error('OCR processing failed', error as Error, 'ocr_failed', { captureId });
      throw error;
    }
  }

  /**
   * Detect UI elements in a screen capture
   */
  async detectElements(captureId: string): Promise<VisualElement[]> {
    const capture = this.captures.get(captureId);
    if (!capture) {
      throw new Error(`Capture ${captureId} not found`);
    }

    try {
      const tempPath = await this.saveCaptureToTemp(capture);
      
      const elements = await invoke('detect_ui_elements', {
        imagePath: tempPath
      }) as Array<{
        type: string;
        text?: string;
        x: number;
        y: number;
        width: number;
        height: number;
        confidence: number;
        attributes: Record<string, any>;
      }>;

      return elements.map(element => ({
        type: element.type as VisualElement['type'],
        text: element.text,
        boundingBox: {
          x: element.x,
          y: element.y,
          width: element.width,
          height: element.height
        },
        confidence: element.confidence,
        attributes: element.attributes
      }));
    } catch (error) {
      visionLogger.error('Element detection failed', error as Error, 'element_detection_failed', { captureId });
      throw error;
    }
  }

  /**
   * Analyze screen capture comprehensively
   */
  async analyzeScreen(captureId: string): Promise<ScreenAnalysis> {
    const capture = this.captures.get(captureId);
    if (!capture) {
      throw new Error(`Capture ${captureId} not found`);
    }

    try {
      // Perform OCR and element detection in parallel
      const [ocrResults, visualElements] = await Promise.all([
        this.performOCR(captureId),
        this.detectElements(captureId)
      ]);

      // Analyze the content to determine context
      const detectedContext = this.analyzeContext(ocrResults, visualElements);
      
      // Generate summary
      const summary = this.generateSummary(ocrResults, visualElements, detectedContext);

      return {
        captureId,
        timestamp: capture.timestamp,
        ocrResults,
        visualElements,
        detectedContext,
        summary
      };
    } catch (error) {
      visionLogger.error('Screen analysis failed', error as Error, 'analysis_failed', { captureId });
      throw error;
    }
  }

  /**
   * Ask AI to interpret screen capture with specific query
   */
  async queryScreenWithAI(captureId: string, query: VisionQuery): Promise<string> {
    const capture = this.captures.get(captureId);
    if (!capture) {
      throw new Error(`Capture ${captureId} not found`);
    }

    try {
      // Get screen analysis if requested
      let context = '';
      
      if (query.includeOCR || query.includeElements) {
        const analysis = await this.analyzeScreen(captureId);
        
        if (query.includeOCR) {
          const ocrText = analysis.ocrResults
            .map(result => `"${result.text}" (confidence: ${Math.round(result.confidence * 100)}%)`)
            .join(', ');
          context += `\nText detected on screen: ${ocrText}`;
        }
        
        if (query.includeElements) {
          const elements = analysis.visualElements
            .map(element => `${element.type}${element.text ? `: "${element.text}"` : ''}`)
            .join(', ');
          context += `\nUI elements detected: ${elements}`;
        }
        
        context += `\nDetected context: ${analysis.detectedContext.windowType}`;
        if (analysis.detectedContext.codeLanguage) {
          context += `, programming language: ${analysis.detectedContext.codeLanguage}`;
        }
      }

      // Convert image to base64 for AI processing
      const base64Image = await this.convertCaptureToBase64(capture);
      
      const prompt = `Looking at this screen capture, ${query.prompt}
      
      ${context}
      
      Please provide a helpful response based on what you can see in the image.`;

      // Send to AI service with vision capabilities
      const response = await invoke('query_vision_ai', {
        prompt,
        image: base64Image,
        focusRegion: query.focusRegion,
        ollamaHost: this.getEnvVar('OLLAMA_HOST', 'localhost'),
        ollamaPort: this.getEnvVar('OLLAMA_PORT', '11434')
      }) as string;

      return response;
    } catch (error) {
      visionLogger.error('AI vision query failed', error as Error, 'ai_query_failed', { captureId, prompt: query.prompt });
      throw error;
    }
  }

  /**
   * Get contextual help based on what's currently on screen
   */
  async getContextualHelp(): Promise<string> {
    try {
      const capture = await this.captureScreen();
      const analysis = await this.analyzeScreen(capture.id);
      
      let helpQuery = 'What can you see on this screen and how can I help with it?';
      
      // Customize query based on detected context
      if (analysis.detectedContext.windowType === 'terminal') {
        helpQuery = 'I can see a terminal window. What commands or operations might be helpful here?';
      } else if (analysis.detectedContext.windowType === 'editor') {
        helpQuery = `I can see a code editor${analysis.detectedContext.codeLanguage ? ` with ${analysis.detectedContext.codeLanguage} code` : ''}. What programming assistance might be needed?`;
      }

      // Check for error messages
      if (analysis.detectedContext.errorMessages && analysis.detectedContext.errorMessages.length > 0) {
        helpQuery = `I can see error messages on the screen: "${analysis.detectedContext.errorMessages.join(', ')}". How can I help resolve these errors?`;
      }

      return await this.queryScreenWithAI(capture.id, {
        prompt: helpQuery,
        includeOCR: true,
        includeElements: true
      });
    } catch (error) {
      visionLogger.error('Failed to get contextual help', error as Error, 'contextual_help_failed');
      throw error;
    }
  }

  /**
   * Monitor screen for changes and provide proactive assistance
   */
  async startScreenMonitoring(callback: (analysis: ScreenAnalysis) => void, intervalMs: number = 5000): Promise<void> {
    const monitor = async () => {
      try {
        const capture = await this.captureScreen();
        const analysis = await this.analyzeScreen(capture.id);
        
        // Only trigger callback if significant changes detected
        if (this.hasSignificantChanges(analysis)) {
          callback(analysis);
        }
      } catch (error) {
        visionLogger.error('Screen monitoring error', error as Error, 'monitoring_error');
      }
    };

    // Start monitoring
    setInterval(monitor, intervalMs);
    // Screen monitoring started
  }

  /**
   * Save capture to temporary file for processing
   */
  private async saveCaptureToTemp(capture: ScreenCapture): Promise<string> {
    try {
      const { tempDir: getTempDir } = await import('@tauri-apps/api/path');
      const { join: joinPath } = await import('@tauri-apps/api/path');
      const { writeFile: writeTauriFile } = await import('@tauri-apps/plugin-fs');
      
      const tempDirPath = await getTempDir();
      const filename = `${capture.id}.${capture.format}`;
      const tempPath = await joinPath(tempDirPath, filename);
      
      await writeTauriFile(tempPath, capture.data);
      return tempPath;
    } catch (error) {
      visionLogger.error('Failed to save capture to temp file', error as Error, 'temp_save_failed', { captureId: capture.id });
      throw new Error(`Failed to save screen capture: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Convert capture to base64 for AI processing
   */
  private async convertCaptureToBase64(capture: ScreenCapture): Promise<string> {
    const base64 = btoa(String.fromCharCode(...capture.data));
    return `data:image/${capture.format};base64,${base64}`;
  }

  /**
   * Analyze context from OCR and element detection results
   */
  private analyzeContext(ocrResults: OCRResult[], visualElements: VisualElement[]): ScreenAnalysis['detectedContext'] {
    const allText = ocrResults.map(result => result.text.toLowerCase()).join(' ');
    
    // Detect window type
    let windowType: 'terminal' | 'editor' | 'browser' | 'desktop' | 'unknown' = 'unknown';
    
    if (allText.includes('$') || allText.includes('bash') || allText.includes('command') || 
        visualElements.some(el => el.type === 'terminal')) {
      windowType = 'terminal';
    } else if (allText.includes('function') || allText.includes('import') || allText.includes('class') ||
               visualElements.some(el => el.type === 'code')) {
      windowType = 'editor';
    } else if (allText.includes('http') || allText.includes('www') || allText.includes('browser')) {
      windowType = 'browser';
    }

    // Detect programming language
    let codeLanguage: string | undefined;
    const languageKeywords = {
      'javascript': ['function', 'const', 'let', 'var', 'import', 'export'],
      'typescript': ['interface', 'type', 'extends', 'implements'],
      'python': ['def', 'import', 'from', 'class', 'self'],
      'rust': ['fn', 'struct', 'impl', 'use', 'mod'],
      'go': ['func', 'package', 'import', 'type', 'struct'],
      'java': ['public', 'class', 'extends', 'implements', 'package']
    };

    for (const [lang, keywords] of Object.entries(languageKeywords)) {
      if (keywords.some(keyword => allText.includes(keyword))) {
        codeLanguage = lang;
        break;
      }
    }

    // Extract terminal commands
    const terminalCommands: string[] = [];
    const commandRegex = /\$\s*([a-zA-Z0-9\-_\.\/\s]+)/g;
    let match;
    while ((match = commandRegex.exec(allText)) !== null) {
      terminalCommands.push(match[1].trim());
    }

    // Detect error messages
    const errorMessages: string[] = [];
    const errorKeywords = ['error', 'failed', 'exception', 'warning', 'not found', 'permission denied'];
    
    for (const result of ocrResults) {
      const text = result.text.toLowerCase();
      if (errorKeywords.some(keyword => text.includes(keyword))) {
        errorMessages.push(result.text);
      }
    }

    return {
      windowType,
      primaryContent: ocrResults.map(r => r.text).join(' ').substring(0, 500),
      codeLanguage,
      terminalCommands: terminalCommands.length > 0 ? terminalCommands : undefined,
      errorMessages: errorMessages.length > 0 ? errorMessages : undefined
    };
  }

  /**
   * Generate summary of screen analysis
   */
  private generateSummary(ocrResults: OCRResult[], visualElements: VisualElement[], context: ScreenAnalysis['detectedContext']): string {
    let summary = `Screen shows a ${context.windowType} window`;
    
    if (context.codeLanguage) {
      summary += ` with ${context.codeLanguage} code`;
    }
    
    if (context.terminalCommands && context.terminalCommands.length > 0) {
      summary += `. Recent commands: ${context.terminalCommands.slice(-3).join(', ')}`;
    }
    
    if (context.errorMessages && context.errorMessages.length > 0) {
      summary += `. Errors detected: ${context.errorMessages.length} error(s)`;
    }
    
    summary += `. Contains ${ocrResults.length} text elements and ${visualElements.length} UI elements.`;
    
    return summary;
  }

  /**
   * Check if screen analysis shows significant changes
   */
  private lastAnalysis: ScreenAnalysis | null = null;
  
  private hasSignificantChanges(analysis: ScreenAnalysis): boolean {
    if (!this.lastAnalysis) {
      this.lastAnalysis = analysis;
      return true;
    }
    
    // Check for window type changes
    if (this.lastAnalysis.detectedContext.windowType !== analysis.detectedContext.windowType) {
      this.lastAnalysis = analysis;
      return true;
    }
    
    // Check for new error messages
    const oldErrors = this.lastAnalysis.detectedContext.errorMessages || [];
    const newErrors = analysis.detectedContext.errorMessages || [];
    if (newErrors.length > oldErrors.length) {
      this.lastAnalysis = analysis;
      return true;
    }
    
    // Check for significant text changes (more than 30% different)
    const oldTextLength = this.lastAnalysis.ocrResults.reduce((sum, r) => sum + r.text.length, 0);
    const newTextLength = analysis.ocrResults.reduce((sum, r) => sum + r.text.length, 0);
    const lengthDifference = Math.abs(newTextLength - oldTextLength) / Math.max(oldTextLength, 1);
    
    if (lengthDifference > 0.3) {
      this.lastAnalysis = analysis;
      return true;
    }
    
    return false;
  }

  /**
   * Get environment variable with fallback - works in both browser and Tauri
   */
  private getEnvVar(key: string, fallback: string): string {
    // Try process.env first (Node.js/Tauri context)
    if (typeof process !== 'undefined' && process.env && process.env[key]) {
      return process.env[key]!;
    }
    
    // Try window environment (browser context) 
    if (typeof window !== 'undefined' && (window as any).__TAURI__ && (window as any).__TAURI_ENV__) {
      const envValue = (window as any).__TAURI_ENV__[key];
      if (envValue) return envValue;
    }
    
    // Try reading from .env file via Tauri
    try {
      // This would need to be implemented via Tauri command if needed
      return fallback;
    } catch {
      return fallback;
    }
  }

  /**
   * Check if required dependencies are available
   */
  private async checkDependencies(): Promise<void> {
    try {
      await invoke('check_vision_dependencies');
    } catch (error) {
      throw new Error('Computer vision dependencies not available. Please install required packages.');
    }
  }

  /**
   * Clean up old captures to free memory
   */
  cleanup(): void {
    const oneHourAgo = Date.now() - 60 * 60 * 1000;
    
    for (const [id, capture] of this.captures) {
      if (new Date(capture.timestamp).getTime() < oneHourAgo) {
        this.captures.delete(id);
      }
    }
  }
}

// Singleton instance
export const visionService = new VisionService();
