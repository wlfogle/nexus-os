# 🗣️ Alexa Integration Setup Guide

## Overview

Your AI Assistant now includes full Alexa integration, allowing you to:

- **Voice Control**: Use voice commands to control your AI assistant
- **Smart Home Integration**: Control your development environment as smart devices
- **Custom Alexa Skill**: Create a custom skill for your AI assistant
- **Wake Word Detection**: "Computer, analyze this code"
- **Text-to-Speech**: Get spoken responses from your AI

## 🎯 Voice Commands You Can Use

### Code Analysis
- *"Computer, analyze the current code file"*
- *"Alexa, check this function for security issues"*
- *"Review the code quality in main.rs"*

### File Operations
- *"Open the config file"*
- *"Save the current file"*
- *"Create a new Rust file"*

### Screen Capture & Analysis  
- *"Take a screenshot and tell me what you see"*
- *"Analyze what's on my screen"*
- *"Capture the current window"*

### System Control
- *"Check system status"*
- *"Show running processes"*
- *"Get network information"*

### Project Management
- *"What's the project status?"*
- *"Show recent git changes"*
- *"Check for uncommitted files"*

### Smart Home Integration
- *"Turn on development environment"*
- *"Activate coding mode"*
- *"Set system monitoring to high"*

---

## 🚀 Quick Setup

### 1. Install Required Dependencies

```bash
# Install speech recognition tools
sudo pacman -S espeak-ng festival tesseract alsa-utils pulseaudio

# Install additional audio libraries
sudo pacman -S portaudio jack2 pipewire

# For advanced speech recognition (optional)
pip install openai-whisper speechrecognition pyaudio
```

### 2. Configure Audio Permissions

```bash
# Add user to audio group
sudo usermod -a -G audio $USER

# Test microphone
arecord -l
pactl list sources

# Test speakers
speaker-test -t wav -c 2
```

### 3. Enable Voice Control in Your AI Assistant

```rust
// In your main application
use crate::modules::alexa_integration::AlexaIntegration;

let alexa = AlexaIntegration::new();

// Start voice listening
alexa.start_voice_listening().await?;

// Process voice commands
let command = alexa.process_voice_command(&audio_data).await?;
let response = alexa.execute_voice_command(&command).await?;
```

---

## 🏠 Smart Home Setup

### 1. Amazon Developer Console Setup

1. **Create Developer Account**: Go to [developer.amazon.com](https://developer.amazon.com)
2. **Create New Skill**: Choose "Custom" skill type
3. **Configure Skill**: Use the provided interaction model

### 2. Skill Interaction Model

```json
{
  "interactionModel": {
    "languageModel": {
      "invocationName": "ai assistant",
      "intents": [
        {
          "name": "AnalyzeCodeIntent",
          "slots": [
            {
              "name": "FilePath",
              "type": "AMAZON.SearchQuery"
            }
          ],
          "samples": [
            "analyze code in {FilePath}",
            "review the {FilePath} file",
            "check code quality"
          ]
        },
        {
          "name": "CaptureScreenIntent",
          "samples": [
            "take a screenshot",
            "capture my screen",
            "analyze what I'm working on"
          ]
        },
        {
          "name": "ExecuteCommandIntent",
          "slots": [
            {
              "name": "Command",
              "type": "AMAZON.SearchQuery"
            }
          ],
          "samples": [
            "execute {Command}",
            "run {Command}",
            "perform {Command}"
          ]
        },
        {
          "name": "SmartHomeIntent",
          "slots": [
            {
              "name": "Device",
              "type": "AMAZON.Device"
            },
            {
              "name": "Action",
              "type": "AMAZON.Action"
            }
          ],
          "samples": [
            "turn {Action} {Device}",
            "activate {Device}",
            "control {Device}"
          ]
        },
        {
          "name": "ProjectStatusIntent",
          "samples": [
            "what's my project status",
            "show git changes",
            "check project health"
          ]
        }
      ]
    }
  }
}
```

### 3. Smart Home Device Discovery

Your AI Assistant will appear as these controllable devices:

```json
{
  "devices": [
    {
      "device_id": "ai_assistant_main",
      "name": "AI Assistant",
      "type": "ACTIVITY_TRIGGER",
      "capabilities": [
        "Alexa.PowerController",
        "Alexa.SceneController",
        "Alexa.Speaker"
      ]
    },
    {
      "device_id": "dev_environment", 
      "name": "Development Environment",
      "type": "SCENE_TRIGGER",
      "capabilities": ["Alexa.SceneController"]
    },
    {
      "device_id": "system_monitor",
      "name": "System Monitor", 
      "type": "TEMPERATURE_SENSOR",
      "capabilities": ["Alexa.TemperatureSensor"]
    }
  ]
}
```

---

## 🔧 Advanced Configuration

### Wake Word Detection

```toml
# config.toml
[alexa]
wake_word = "computer"  # or "alexa", "hey assistant"
voice_enabled = true
sensitivity = 0.7
timeout_ms = 5000
```

### Voice Recognition Settings

```toml
[voice_recognition]
engine = "whisper"  # or "google", "azure", "local"
language = "en-US"
model_size = "base"  # tiny, base, small, medium, large
confidence_threshold = 0.8
```

### Smart Home Configuration

```toml
[smart_home]
enabled = true
auto_discovery = true
device_timeout = 30
scene_activation = true

[smart_home.devices]
dev_environment = { scenes = ["coding", "debugging", "testing"] }
system_monitor = { sensors = ["cpu", "memory", "disk"] }
```

---

## 🛠️ Installation Script

Create and run this installation script:

```bash
#!/bin/bash
# alexa_setup.sh

echo "🗣️ Setting up Alexa Integration for AI Assistant"

# Install system dependencies
echo "Installing system dependencies..."
if command -v pacman &> /dev/null; then
    sudo pacman -S --noconfirm espeak-ng festival tesseract alsa-utils pulseaudio portaudio jack2
elif command -v apt &> /dev/null; then
    sudo apt update
    sudo apt install -y espeak-ng festival tesseract-ocr alsa-utils pulseaudio-utils portaudio19-dev jackd2
elif command -v yum &> /dev/null; then
    sudo yum install -y espeak-ng festival tesseract alsa-utils pulseaudio portaudio-devel jack-audio-connection-kit
fi

# Setup audio permissions
echo "Configuring audio permissions..."
sudo usermod -a -G audio $USER

# Install Python dependencies for advanced speech recognition
echo "Installing Python speech recognition..."
pip install --user openai-whisper speechrecognition pyaudio numpy

# Create Alexa skill configuration
echo "Creating Alexa skill configuration..."
mkdir -p ~/.config/ai-assistant/alexa

cat > ~/.config/ai-assistant/alexa/skill.json << 'EOF'
{
  "skill_id": "amzn1.ask.skill.YOUR_SKILL_ID",
  "client_id": "YOUR_CLIENT_ID",
  "client_secret": "YOUR_CLIENT_SECRET",
  "redirect_uri": "http://localhost:3000/auth/callback",
  "scopes": ["alexa::ask:skills:readwrite", "alexa::ask:models:readwrite"]
}
EOF

# Create wake word configuration
cat > ~/.config/ai-assistant/alexa/wake_word.json << 'EOF'
{
  "wake_words": ["computer", "ai assistant", "hey assistant"],
  "sensitivity": 0.7,
  "timeout_ms": 5000,
  "audio_device": "default",
  "sample_rate": 16000
}
EOF

# Test audio setup
echo "Testing audio setup..."
echo "Testing microphone..." | espeak
arecord -d 1 -f cd /tmp/test.wav 2>/dev/null && echo "✅ Microphone working"
speaker-test -t wav -c 2 -l 1 &>/dev/null && echo "✅ Speakers working"

# Create systemd service for voice control
cat > ~/.config/systemd/user/ai-assistant-voice.service << 'EOF'
[Unit]
Description=AI Assistant Voice Control
After=pulseaudio.service

[Service]
Type=simple
ExecStart=/usr/local/bin/ai-assistant --voice-mode
Restart=always
RestartSec=5
Environment=PULSE_RUNTIME_PATH=%i/pulse

[Install]
WantedBy=default.target
EOF

# Enable and start the service
systemctl --user daemon-reload
systemctl --user enable ai-assistant-voice.service

echo "🎉 Alexa integration setup complete!"
echo ""
echo "Next steps:"
echo "1. Configure your Amazon Developer Console skill"
echo "2. Set up OAuth credentials in ~/.config/ai-assistant/alexa/skill.json"
echo "3. Test voice commands: 'Computer, analyze this code'"
echo "4. Enable smart home discovery in Alexa app"
echo ""
echo "Voice commands you can try:"
echo "- 'Computer, what's on my screen?'"
echo "- 'Alexa, analyze the current code'"
echo "- 'Turn on development environment'"
```

Make it executable and run:

```bash
chmod +x alexa_setup.sh
./alexa_setup.sh
```

---

## 🧪 Testing Your Setup

### 1. Test Voice Recognition

```bash
# Test basic speech-to-text
echo "Testing voice recognition..." | espeak

# Test microphone
arecord -d 3 -f cd test.wav
aplay test.wav
```

### 2. Test AI Assistant Voice Commands

```bash
# In your application, enable debug mode
export RUST_LOG=debug
cargo run

# Try voice commands:
# "Computer, analyze this code"
# "Take a screenshot"
# "Check system status"
```

### 3. Test Smart Home Integration

1. **Alexa App**: Go to Smart Home → Discover Devices
2. **Find Devices**: Look for "AI Assistant", "Development Environment", "System Monitor"
3. **Test Commands**: 
   - *"Alexa, turn on AI Assistant"*
   - *"Alexa, activate Development Environment"*
   - *"Alexa, what's the temperature of System Monitor?"*

---

## 🔍 Troubleshooting

### Audio Issues

```bash
# Check audio devices
pactl list sources short
pactl list sinks short

# Fix permissions
sudo usermod -a -G audio,pulse-access $USER

# Restart audio services
systemctl --user restart pulseaudio
```

### Voice Recognition Issues

```bash
# Test speech recognition
python3 -c "
import speech_recognition as sr
r = sr.Recognizer()
with sr.Microphone() as source:
    print('Say something!')
    audio = r.listen(source, timeout=5)
    print(r.recognize_google(audio))
"
```

### Alexa Skill Issues

1. **Check Skill Status**: Amazon Developer Console → Your Skills
2. **Verify Endpoints**: Ensure your server is accessible 
3. **Test in Simulator**: Use Alexa Skills Kit simulator
4. **Check Logs**: Monitor application logs for errors

### Smart Home Discovery Issues

1. **Device Discovery**: Ensure devices are properly registered
2. **Account Linking**: Check OAuth flow in Alexa app
3. **Capabilities**: Verify device capabilities are correctly defined

---

## 📋 Configuration Reference

### Complete Configuration File

```toml
# ~/.config/ai-assistant/config.toml

[alexa]
enabled = true
wake_word = "computer"
voice_enabled = true
smart_home_enabled = true
skill_integration = true

[alexa.oauth]
client_id = "YOUR_CLIENT_ID"
client_secret = "YOUR_CLIENT_SECRET"
redirect_uri = "http://localhost:3000/auth/callback"

[alexa.voice]
engine = "whisper"
language = "en-US"
model_size = "base"
confidence_threshold = 0.8
timeout_ms = 5000

[alexa.audio]
input_device = "default"
output_device = "default"
sample_rate = 16000
channels = 1
format = "s16le"

[alexa.smart_home]
auto_discovery = true
device_timeout = 30
scene_activation = true
temperature_monitoring = true

[alexa.responses]
voice_feedback = true
screen_feedback = false
notification_sounds = true
```

---

## 🎯 Usage Examples

### Development Workflow

```bash
# Start your development session with voice
"Computer, activate development environment"
# → Opens IDE, starts servers, checks git status

"Analyze the current code file"
# → Runs code analysis, reports issues

"Take a screenshot and explain what I'm working on"
# → Captures screen, analyzes code, provides context

"Check project status"
# → Shows git status, recent changes, system metrics

"Turn off development environment"
# → Saves files, stops servers, commits changes
```

### Smart Home Automation

```bash
# Morning routine
"Alexa, good morning"
# → Activates development environment
# → Shows system status
# → Reads recent notifications

# During work
"Computer, I'm debugging"
# → Sets verbose logging
# → Opens debugging tools
# → Monitors performance

# End of day
"Alexa, wrap up work"
# → Commits changes
# → Backs up important files
# → Shows daily summary
```

---

Your AI Assistant now has powerful voice control and smart home integration! You can control your entire development environment with voice commands and integrate it seamlessly with your existing Alexa ecosystem.

The system is designed to be:
- **Privacy-focused**: Can run entirely locally
- **Customizable**: Easily add new voice commands and devices
- **Integrated**: Works with your existing tools and workflows
- **Intelligent**: Learns from your usage patterns

Start with simple commands like *"Computer, what's on my screen?"* and gradually explore more advanced features! 🚀

# 🎤 Amazon Alexa Integration with Your Media Stack

## 🏗️ **Integration Architecture Overview**

Your current stack provides multiple excellent integration points for Alexa:

```
┌─────────────────┐    Voice Commands    ┌─────────────────┐
│   Amazon Alexa  │──────────────────────▶│  Home Assistant │
│   (Echo Device)  │                       │    (VM 500)     │
└─────────────────┘                       └─────────────────┘
                                                   │
                                                   │ Controls
                                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Your Media Stack                             │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │    Plex     │  │  Jellyfin   │  │ AI Services │             │
│  │  (CT 230)   │  │  (CT 231)   │  │  (CT 900)   │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │  Traefik    │  │ Monitoring  │  │   Other     │             │
│  │  (CT 103)   │  │ CT 260-261  │  │  Services   │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
└─────────────────────────────────────────────────────────────────┘
```

## 🎯 **Integration Approaches**

### **Option 1: Home Assistant Native Integration (Recommended)**

This is the most powerful and flexible approach using your existing Home Assistant VM.

#### **Benefits:**
- ✅ Full control over all services
- ✅ Custom skills and routines
- ✅ Privacy-focused (minimal data to Amazon)
- ✅ Works with your existing AI services
- ✅ Integrates with your entire media stack

#### **Setup Steps:**

1. **Install Alexa Media Player in Home Assistant**
   ```yaml
   # Add to configuration.yaml in Home Assistant (VM 500)
   alexa_media:
     accounts:
       - email: your-amazon-email@example.com
         password: your-amazon-password
         url: amazon.com
   ```

2. **Configure Alexa Integration**
   ```yaml
   # Enable discovery for your devices
   alexa:
     smart_home:
       endpoint: https://api.amazonalexa.com/v1/events
       client_id: your_client_id
       client_secret: your_client_secret
       filter:
         include_entities:
           - media_player.plex
           - media_player.jellyfin
           - switch.ai_services
   ```

### **Option 2: Alexa Skills Kit (ASK) Custom Skill**

Create a custom Alexa skill that directly interfaces with your services.

#### **Benefits:**
- ✅ Highly customized voice commands
- ✅ Direct integration with your APIs
- ✅ Professional user experience
- ✅ Can integrate with your AI services

#### **Architecture:**
```
Alexa → AWS Lambda → Your Proxmox Server → Services
```

### **Option 3: Bridge Integration via Node-RED**

Use Node-RED as a bridge between Alexa and your services.

#### **Benefits:**
- ✅ Visual workflow design
- ✅ Easy to modify and extend
- ✅ Can run in its own container
- ✅ Excellent for complex automations

## 🚀 **Recommended Implementation: Home Assistant + Alexa**

Let me provide a step-by-step implementation for the most powerful approach:

### **Phase 1: Home Assistant Alexa Integration**

#### **1. Configure Home Assistant for Alexa**
```yaml
# Add to /config/configuration.yaml in Home Assistant (VM 500)

# Alexa integration
alexa:
  smart_home:
    locale: en-US
    endpoint: https://api.amazonalexa.com/v1/events
    filter:
      include_domains:
        - media_player
        - switch
        - light
        - script
      include_entities:
        - media_player.plex
        - media_player.jellyfin
        - script.start_movie_night
        - script.system_status

# Alexa Media Player for TTS and notifications
alexa_media:
  accounts:
    - email: !secret alexa_email
      password: !secret alexa_password
      url: amazon.com

# Intent Script for custom voice commands
intent_script:
  PlayMovieOnPlex:
    speech:
      text: "Starting your movie on Plex"
    action:
      service: media_player.media_play
      target:
        entity_id: media_player.plex
        
  CheckSystemStatus:
    speech:
      text: "Checking system status"
    action:
      service: script.system_health_check
```

#### **2. Create Useful Scripts for Voice Control**
```yaml
# Add to /config/scripts.yaml

start_movie_night:
  alias: "Start Movie Night"
  sequence:
    - service: media_player.turn_on
      target:
        entity_id: media_player.plex
    - service: light.turn_off
      target:
        entity_id: light.living_room
    - service: tts.alexa_media_say
      data:
        entity_id: media_player.echo_dot
        message: "Movie night mode activated. Plex is ready and lights are dimmed."

system_health_check:
  alias: "System Health Check"
  sequence:
    - service: rest_command.health_check
    - delay: "00:00:05"
    - service: tts.alexa_media_say
      data:
        entity_id: media_player.echo_dot
        message: "System health check complete. All services are running normally."

ai_code_analysis:
  alias: "AI Code Analysis"
  sequence:
    - service: tts.alexa_media_say
      data:
        entity_id: media_player.echo_dot
        message: "AI coding assistant is ready. You can access it on your desktop."
```

#### **3. REST Commands for System Control**
```yaml
# Add to configuration.yaml
rest_command:
  health_check:
    url: "http://192.168.122.9:8080/health-check"
    method: GET
    
  restart_plex:
    url: "http://192.168.122.9:8080/restart/plex"
    method: POST
    
  system_stats:
    url: "http://192.168.122.103:9080/api/overview"
    method: GET
```

### **Phase 2: Voice Commands for Your Media Stack**

#### **Example Voice Commands You Can Implement:**

1. **Media Control:**
   - "Alexa, start movie night"
   - "Alexa, play music on Plex"
   - "Alexa, check what's recording on Plex"

2. **System Management:**
   - "Alexa, check system status"
   - "Alexa, restart Plex server"
   - "Alexa, what's my server load?"

3. **AI Integration:**
   - "Alexa, start AI coding session"
   - "Alexa, is Ollama running?"
   - "Alexa, check AI models"

4. **Smart Home Integration:**
   - "Alexa, turn on media room"
   - "Alexa, set viewing mode"
   - "Alexa, good night" (shutdown non-essential services)

### **Phase 3: Advanced AI Integration**

#### **Connect Alexa to Your AI Services**
```python
# Create a bridge script in Home Assistant
# /config/python_scripts/ai_bridge.py

import requests
import json

def call_ollama_ai(prompt):
    url = "http://192.168.122.172:11434/api/generate"
    data = {
        "model": "codellama:7b",
        "prompt": prompt,
        "stream": False
    }
    
    response = requests.post(url, json=data)
    if response.status_code == 200:
        return response.json().get("response", "AI service unavailable")
    return "Error contacting AI service"

# Voice command: "Alexa, ask AI to explain Python functions"
prompt = data.get("prompt", "")
ai_response = call_ollama_ai(prompt)

# Respond via Alexa TTS
service_data = {
    "entity_id": "media_player.echo_dot", 
    "message": f"AI says: {ai_response[:100]}..."  # Truncate for voice
}
hass.services.call("tts", "alexa_media_say", service_data)
```

## 🛠️ **Implementation Container (Optional)**

### **Create Dedicated Alexa Bridge Container**

If you want a dedicated container for Alexa integration:

```bash
# Create new LXC container for Alexa services
ssh root@192.168.122.9 "
pct create 280 local:vztmpl/ubuntu-22.04-standard_22.04-1_amd64.tar.zst \\
  --hostname alexa-bridge \\
  --cores 1 \\
  --memory 1024 \\
  --swap 512 \\
  --net0 name=eth0,bridge=vmbr0,ip=192.168.122.280/24,gw=192.168.122.1 \\
  --rootfs local-lvm:8 \\
  --features nesting=1

pct start 280
"
```

### **Install Node-RED for Visual Alexa Workflows**
```bash
# Inside the Alexa bridge container
ssh root@192.168.122.9 "pct exec 280 -- bash -c '
# Install Node.js and Node-RED
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
apt-get install -y nodejs
npm install -g --unsafe-perm node-red
npm install -g node-red-contrib-alexa-remote2

# Start Node-RED
node-red &
'"
```

## 🎤 **Voice Command Examples**

### **Media Stack Control**
- "Alexa, start my media server" → Powers on Plex/Jellyfin
- "Alexa, check my downloads" → Status of Sonarr/Radarr
- "Alexa, movie night mode" → Dims lights, starts Plex
- "Alexa, pause all media" → Pauses active streams

### **System Monitoring**
- "Alexa, system health report" → Runs your health check script
- "Alexa, server temperature" → Reports system stats
- "Alexa, storage space" → Disk usage report
- "Alexa, network status" → Connectivity check

### **AI Integration**
- "Alexa, start coding session" → Opens AI assistant
- "Alexa, explain this code" → Bridges to AI analysis
- "Alexa, system optimization tips" → AI-powered suggestions

## 🔒 **Security Considerations**

### **Privacy Protection**
- Use Home Assistant cloud or Nabu Casa for secure external access
- Implement authentication for sensitive commands
- Limit Alexa access to non-critical systems only
- Use secure secrets management

### **Network Security**
```yaml
# Restrict access in Home Assistant
alexa:
  smart_home:
    filter:
      include_entities:
        - media_player.plex
        - script.safe_commands_only
      exclude_entities:
        - switch.critical_system_controls
```

## 📱 **Mobile Integration Bonus**

### **Alexa App Integration**
- Create custom routines in the Alexa app
- Set up location-based triggers
- Configure family member access levels
- Enable drop-in for system notifications

## 🚀 **Quick Start Implementation**

### **Step 1: Enable Alexa in Home Assistant**
```bash
# SSH into Home Assistant VM
ssh homeassistant@192.168.122.52

# Edit configuration
nano /config/configuration.yaml
# Add the Alexa integration code above
```

### **Step 2: Install Required Add-ons**
- Install "Alexa Media Player" from HACS
- Enable "Amazon Alexa" integration
- Configure your Amazon account

### **Step 3: Test Basic Commands**
- "Alexa, discover devices"
- "Alexa, turn on [your first script]"

### **Step 4: Expand with Custom Skills**
- Create more complex automations
- Add AI service integration
- Implement system monitoring commands

## 📋 **Implementation Checklist**

- [ ] **Home Assistant Alexa integration configured**
- [ ] **Amazon account linked to Home Assistant**
- [ ] **Basic voice commands working**
- [ ] **Media player controls functional**
- [ ] **System status scripts created**
- [ ] **AI service integration tested**
- [ ] **Security restrictions implemented**
- [ ] **Family member access configured**

---

## 🎉 **Result: Complete Voice Control**

Once implemented, you'll have voice control over your entire media stack:
- **Media servers** (Plex, Jellyfin)
- **Download managers** (Sonarr, Radarr)
- **System monitoring** (Grafana, Prometheus)
- **AI services** (Ollama, Open-Interpreter)
- **Infrastructure** (containers, services)

**Example workflow:** "Alexa, movie night" → Starts Plex, dims lights, checks system status, announces readiness!

Would you like me to help you implement any specific part of this Alexa integration?
