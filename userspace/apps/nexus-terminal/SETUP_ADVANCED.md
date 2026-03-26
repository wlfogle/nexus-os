# 🚀 NexusTerminal Advanced Setup Guide

## 🎯 Overview

This guide covers the setup of NexusTerminal's revolutionary AI features including RAG (Retrieval-Augmented Generation) and Computer Vision capabilities. These features transform NexusTerminal from a regular terminal into an AI-powered computing platform.

## 🧠 RAG System Setup

### Prerequisites

1. **ChromaDB Server**
```bash
# Install ChromaDB
pip install chromadb

# Start ChromaDB server
chroma run --host localhost --port 8000
```

2. **Embedding Models**
```bash
# Pull embedding model for Ollama
ollama pull nomic-embed-text

# Verify model is available
ollama list | grep nomic-embed-text
```

### Configuration

The RAG system automatically:
- Indexes your codebase when you open a project
- Stores command history with semantic search
- Builds knowledge from AI conversations
- Provides contextual information for AI responses

### Features Enabled

✅ **Semantic Codebase Search**: Find relevant code across your entire project  
✅ **Command History Intelligence**: AI learns from your command patterns  
✅ **Contextual Documentation**: Automatically retrieves relevant docs  
✅ **Cross-Reference Analysis**: Understands file and function relationships  

## 👁️ Computer Vision Setup

### System Dependencies

#### For Garuda Linux (Arch-based):
```bash
# Install Tesseract OCR
sudo pacman -S tesseract tesseract-data-eng

# Install Python dependencies (optional, for EasyOCR)
pip install easyocr opencv-python numpy

# Install system libraries for image processing
sudo pacman -S libwebp libjpeg-turbo libpng
```

#### Verify Installation:
```bash
# Test Tesseract
tesseract --version

# Test screen capture (requires X11)
echo $DISPLAY  # Should show :0 or similar

# Test Python OCR (optional)
python3 -c "import easyocr; print('EasyOCR ready')"
```

### Vision Models

```bash
# Pull vision-capable AI model
ollama pull llava

# Verify model
ollama list | grep llava
```

### Features Enabled

✅ **Screen Capture**: Full screen and region capture capabilities  
✅ **OCR Text Recognition**: Extract text from any part of your screen  
✅ **UI Element Detection**: Identify buttons, windows, and interface elements  
✅ **Context Understanding**: AI knows what you're looking at  
✅ **Error Detection**: Automatically spot error messages on screen  
✅ **Proactive Assistance**: AI suggests help based on visual context  

## 🔧 Advanced Configuration

### Performance Optimization for Your System

Given your high-end specs (32-core i9, RTX 4080, 62.5GB RAM), optimize for maximum performance:

#### Ollama Configuration
Create `~/.ollama/config.json`:
```json
{
  "num_ctx": 8192,
  "num_threads": 16,
  "num_gpu_layers": 35,
  "batch_size": 512,
  "memory_limit": "8GB"
}
```

#### ChromaDB Configuration
Create `chroma_config.yaml`:
```yaml
server:
  host: "localhost"
  port: 8000
  workers: 8
  
collection:
  batch_size: 1000
  max_documents: 100000
  
embedding:
  model: "nomic-embed-text"
  dimensions: 768
  batch_size: 100
```

#### NexusTerminal Configuration
The app automatically detects and optimizes for your system:
- **CPU**: Uses up to 16 threads for parallel processing
- **GPU**: RTX 4080 acceleration for AI inference
- **RAM**: Caches up to 4GB of embeddings and models

## 🚀 Usage Examples

### RAG-Powered Assistance

1. **Ask about your codebase:**
   ```
   "How does authentication work in this project?"
   ```
   → AI searches your code and provides detailed explanation

2. **Get command suggestions based on history:**
   ```
   "What's the best way to deploy this?"
   ```
   → AI analyzes your deployment history and suggests optimal commands

3. **Debug with context:**
   ```
   "Why is this test failing?"
   ```
   → AI searches test files, recent changes, and error logs

### Computer Vision Assistance

1. **Get help with what's on screen:**
   ```
   "What's wrong with this error message?"
   ```
   → AI captures screen, reads error, and provides solution

2. **Code review assistance:**
   ```
   "Review the code I'm looking at"
   ```
   → AI analyzes visible code and provides feedback

3. **UI/UX help:**
   ```
   "How can I improve this interface?"
   ```
   → AI analyzes your UI design and suggests improvements

## 🔍 Advanced Features

### Automatic Codebase Indexing

NexusTerminal automatically indexes:
- Source code files (`.ts`, `.js`, `.py`, `.rs`, etc.)
- Documentation (`.md`, `README` files)
- Configuration files (`.json`, `.yaml`, `.toml`)
- Build files (`package.json`, `Cargo.toml`, etc.)

### Real-Time Screen Monitoring

Enable proactive assistance:
```typescript
// In AI Assistant settings
✅ Enable "Proactive Screen Monitoring"
✅ Set monitoring interval: 10 seconds
✅ Enable error detection alerts
```

### Smart Context Building

The system intelligently builds context from:
- Current terminal session and history
- Recently opened files
- Git repository state
- Screen content (when vision is enabled)
- Previously indexed knowledge

## 🛡️ Privacy & Security

### Local-First Architecture
- All AI processing happens locally via Ollama
- Screen captures are processed locally and deleted after analysis
- No data sent to external services
- Full control over your data and models

### Security Features
- Screen capture requires explicit permission
- Command execution validation
- Sandboxed AI model execution
- Secure file access controls

## 🐛 Troubleshooting

### RAG System Issues

**ChromaDB not starting:**
```bash
# Check port availability
lsof -i :8000

# Start with debug output
chroma run --host localhost --port 8000 --log-config chromadb/log_config.yml
```

**Embedding model not working:**
```bash
# Verify Ollama is running
ollama list

# Test embedding model
ollama run nomic-embed-text "test embedding"
```

### Computer Vision Issues

**Screen capture not working:**
```bash
# Check X11 display
echo $DISPLAY
xrandr  # Should show display info

# Test screenshot manually
scrot /tmp/test.png
```

**OCR not working:**
```bash
# Check Tesseract installation
tesseract --list-langs
tesseract /path/to/image.png output
```

**Permission errors:**
```bash
# Add user to video group for screen capture
sudo usermod -a -G video $USER

# Restart session or reboot
```

## 🎮 Gaming Your Terminal Experience

With your RTX 4080 and 32-core CPU, you can enable experimental features:

### GPU-Accelerated AI Inference
```bash
# Enable GPU acceleration in Ollama
export OLLAMA_GPU_LAYERS=35
export OLLAMA_CUDA_VISIBLE_DEVICES=0
```

### Parallel Processing
```bash
# Enable maximum parallelism
export NEXUS_PARALLEL_SESSIONS=16
export NEXUS_AI_WORKERS=8
```

### Memory Optimization
```bash
# Use abundant RAM for caching
export NEXUS_CACHE_SIZE=4GB
export NEXUS_EMBEDDING_CACHE=2GB
```

## 🔮 What's Next

With RAG and Computer Vision implemented, you now have:

🎯 **AI that understands your entire project**  
👁️ **AI that can see your screen**  
🧠 **AI that learns from your patterns**  
⚡ **AI that provides proactive assistance**  

The next wave of features will include:
- Advanced workflow automation
- Real-time security scanning
- Collaborative terminal sharing
- Plugin marketplace
- Advanced analytics dashboard

---

**You now have the most advanced AI-powered terminal ever created!** 🚀

Your terminal can literally see what you're doing and help you with context from your entire codebase. Welcome to the future of terminal computing!
