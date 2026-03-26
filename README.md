# AI Assistant - Tauri Edition

A comprehensive AI-powered assistant built with Tauri, Vue.js, and Rust. This application connects to your AI LXC container (CT 900) at `192.168.122.172` to provide intelligent code analysis, system optimization, and conversational AI assistance.

## 🚀 Features

### 🤖 AI Code Analysis
- **Multi-Language Support**: Python, JavaScript, TypeScript, Rust, Go, Java, C++, and more
- **Comprehensive Analysis**: Bug detection, optimization suggestions, and code improvements
- **Multiple Operations**: Code analysis, documentation generation, and test generation

### 💬 Conversational AI Assistant
- **Natural Language Queries**: Ask questions about system optimization, troubleshooting, or general topics
- **Interactive Command Execution**: AI suggests and executes safe system commands automatically
- **Conversation History**: Keep track of all your AI interactions and their results

### 🔧 Fix Code/Text Feature
- **Intelligent Code Fixing**: Paste broken code, error messages, or configuration files
- **Detailed Analysis**: Get explanations of what's wrong and how it's fixed
- **Multiple Content Types**: Shell scripts, error logs, config files, code snippets

### 🤖 AI Actions (One-Click System Management)
- **🔍 System Analysis**: Check performance and resource usage
- **💾 Disk Cleanup**: Find large files and optimize disk space
- **🧠 Memory Check**: Monitor RAM usage and identify memory consumers
- **🌐 Network Status**: Check network connections and diagnose issues
- **📋 System Logs**: Analyze system logs for errors and warnings
- **🖥️ Hardware Info**: Display hardware and driver information

### 🚀 Quick Actions
- **⚡ System Optimization**: Get performance improvement recommendations
- **🔒 Security Tips**: Learn Linux security best practices with commands
- **🔧 Troubleshooting**: Diagnose high CPU/memory usage issues
- **💻 Terminal Commands**: Discover useful system management commands

### 🎨 Modern Features
- **Responsive UI**: Clean, modern interface built with Vue 3
- **Real-time Execution**: Commands are executed and results displayed instantly
- **Safe Command Filtering**: Only safe diagnostic commands are auto-executed
- **Cross-platform**: Works on Linux, Windows, and macOS

## 📋 Prerequisites

- **Node.js** (v16 or higher)
- **Rust** (latest stable)
- **AI LXC Container** (CT 900) running Ollama at `192.168.122.172:11434`

## 🛠️ Installation & Setup

### 1. Clone and Setup
```bash
cd /home/lou/open-interpreter-tauri
npm install
```

### 2. Development Mode
```bash
npm run tauri dev
```

### 3. Build for Production
```bash
npm run tauri build
```

## 🔧 Configuration

### AI Container Setup
1. Ensure your AI LXC container (CT 900) is running at `192.168.122.172`
2. Verify Ollama is running on port `11434`
3. Install required models in your container:
   ```bash
   ollama pull codellama:7b
   ollama pull magicoder
   ollama pull qwen2.5-coder
   ```

### Application Settings
- Open the Settings tab in the application
- Configure your AI container IP and port
- Test the connection to ensure it's working
- Set your preferred default programming language

## 📱 Usage

### 💬 AI Assistant (Main Feature)
1. **Ask Questions**: Type any question about system optimization, troubleshooting, or general topics
2. **Get Smart Responses**: AI provides detailed answers with actionable advice
3. **View History**: All conversations are saved and displayed in chronological order
4. **Clear Chat**: Use the clear button to start fresh conversations

### 🤖 AI Actions (One-Click System Management)
1. **System Analysis**: Click to automatically check system performance and resource usage
2. **Disk Cleanup**: Get disk usage analysis and large file recommendations
3. **Memory Check**: Monitor RAM usage and identify memory-consuming processes  
4. **Network Status**: Check network connections and diagnose connectivity issues
5. **System Logs**: Analyze system logs for errors, warnings, and issues
6. **Hardware Info**: Display detailed hardware and driver information

### 🔧 Fix Code/Text Feature
1. **Paste Content**: Add broken code, error messages, shell scripts, or config files
2. **Click Fix**: AI analyzes the content and provides:
   - What the issue is
   - The corrected version
   - Detailed explanation of the fix
3. **Examples**: Shell scripts, Python code, error logs, configuration files, system logs
4. **Clear Section**: Clear the input area when done

### 🚀 Quick Actions
1. **System Optimization**: Get Garuda Linux performance improvement tips with commands
2. **Security Tips**: Learn Linux security best practices with executable commands
3. **Troubleshooting**: Get help diagnosing high CPU/memory usage with diagnostic commands
4. **Terminal Commands**: Discover useful Linux system management commands

### 🤖 Code Analysis (Classic Feature)
1. **Select Operation**: Choose from Analyze, Fix Bugs, Optimize, Document, or Test
2. **Input Code**: Paste your code in the input area
3. **Set Language**: Auto-detect or manually select the programming language
4. **Analyze**: Click "Analyze Code" to get AI-powered insights

### 📁 File Management
1. **Browse Files**: Use the File Manager tab to browse project files
2. **Select Files**: Click on files to load them for analysis
3. **Quick Analysis**: Selected files can be quickly analyzed

### 🔗 Connection Status
- Green dot: Connected to AI container at 192.168.122.172:11434
- Red dot: Disconnected or connection failed
- Use the refresh button to check connection status
- Connection required for all AI features to work

## 🏧 Architecture

### Frontend (Vue.js)
- **App.vue**: Main application layout with tabbed navigation
- **AIAssistant.vue**: 🎆 **New!** Conversational AI interface with:
  - Natural language query input
  - AI Actions for one-click system management
  - Fix Code/Text feature for intelligent debugging
  - Quick Actions for common tasks
  - Conversation history with command execution results
- **CodeAnalyzer.vue**: Core code analysis interface (classic feature)
- **ConnectionStatus.vue**: AI container connection monitoring
- **FileManager.vue**: File browsing and selection
- **ModelSelector.vue**: AI model selection
- **Settings.vue**: Application configuration with AI Assistant options

### Backend (Rust)
- **lib.rs**: Main Tauri application logic with enhanced command handlers
- **AI Integration**: RESTful API calls to Ollama at 192.168.122.172:11434
- **Command Handlers**: 
  - `analyze_code`: Send code to AI for analysis
  - `general_ai_query`: 🎆 **New!** Handle conversational AI queries
  - `execute_command`: 🎆 **New!** Execute safe system commands with filtering
  - `check_ai_connection`: Verify container connectivity
  - `get_available_models`: Fetch available AI models

### Command Safety Features
- **Safe Command Filtering**: Only diagnostic commands are auto-executed
- **Allowed Commands**: `free`, `df`, `ps`, `top`, `lscpu`, `lsblk`, `uname`, `uptime`, `who`, `w`, `systemctl status`, `journalctl`
- **Command Extraction**: AI responses are parsed for executable commands
- **Result Integration**: Command outputs are displayed in conversation history

### AI Container Communication
```
Tauri App (Rust) → HTTP Request → AI LXC Container (192.168.122.172:11434)
                                      ↓
                               Ollama API (codellama, etc.)
                                      ↓
                               AI Analysis Response
```

## 🔐 Security

- All AI requests are sent over HTTP to your local LXC container
- No external AI services are used
- Code never leaves your local network
- Container IP is configurable for different network setups

## 🚨 Troubleshooting

### Connection Issues
1. **Verify container is running**: `ping 192.168.122.172`
2. **Check Ollama service**: SSH into container and verify Ollama is running
3. **Firewall**: Ensure port 11434 is accessible
4. **Network**: Verify the container IP is correct for your setup

### Build Issues
1. **Missing dependencies**: Run `npm install` and ensure Rust is installed
2. **Tauri CLI**: Install with `cargo install tauri-cli`
3. **Node version**: Ensure Node.js v16+ is installed

### Performance
1. **Large files**: Break down large code files for better analysis
2. **Model selection**: Use appropriate models for your code language
3. **Container resources**: Ensure your LXC container has sufficient RAM

## 🛣️ Roadmap

- [ ] **File tree integration** with Tauri's filesystem APIs
- [ ] **Real-time code analysis** as you type
- [ ] **Multiple AI models** selection per operation
- [ ] **Export analysis reports** to markdown/PDF
- [ ] **Project-wide analysis** for entire codebases
- [ ] **Custom prompt templates** for specialized analysis
- [ ] **Integration** with popular code editors

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📄 License

This project is open source. See LICENSE file for details.

## 🙏 Acknowledgments

- Built with [Tauri](https://tauri.app/) for the desktop application framework
- Uses [Vue.js 3](https://vuejs.org/) for the frontend
- Powered by [Ollama](https://ollama.ai/) for AI model hosting
- Inspired by the original Open Interpreter GUI project

---

**Happy Coding!** 🚀

*Connect your AI container and start analyzing code with the power of local AI models.*
