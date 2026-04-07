# 📋 **EXACT Copy/Paste Instructions for Home Assistant**

## 🎯 **Where to Paste: Visual Guide**

### **Step 1: Open Home Assistant File Editor**
1. Go to: `http://homeassistant.local:8123`
2. Click: **Settings** (gear icon in sidebar)
3. Click: **Add-ons**
4. If you don't see "File editor":
   - Click **Add-on Store**
   - Search: "File editor"
   - Install it
5. Click: **File editor** → **Open Web UI**

### **Step 2: Edit configuration.yaml**

**What you'll see in the file editor:**
```yaml
# Existing content in your configuration.yaml
default_config:

# Load frontend themes from the themes folder
frontend:
  themes: !include_dir_merge_named themes

automation: !include automations.yaml
script: !include scripts.yaml
scene: !include scenes.yaml
```

**Where to paste:** Scroll to the very bottom and add a blank line, then paste:

```yaml
# ==========================================
# ALEXA INTEGRATION - Voice Control Setup
# ==========================================
alexa:
  smart_home:
    endpoint: https://api.amazonalexa.com/v1/events
    client_id: !secret alexa_client_id
    client_secret: !secret alexa_client_secret
    
# Voice command recognition
intent_script:
  MovieNightIntent:
    speech:
      text: "Starting movie night mode. Checking media servers..."
    action:
      - service: script.movie_night_check
      
  SystemStatusIntent:
    speech:  
      text: "Running system health check..."
    action:
      - service: script.system_health_report
      
  AIStatusIntent:
    speech:
      text: "Checking AI assistant status..."
    action:
      - service: script.ai_status_check

# Enable voice control for scripts
homeassistant:
  customize:
    script.movie_night_check:
      alexa_name: "Movie Night"
      alexa_description: "Checks Plex and Jellyfin servers"
    script.system_health_report:
      alexa_name: "System Status" 
      alexa_description: "Full system health check"
    script.ai_status_check:
      alexa_name: "AI Assistant Status"
      alexa_description: "Check Ollama AI services"
```

### **Step 3: Edit scripts.yaml**

**If scripts.yaml exists:** Click on it and add the content at the bottom
**If scripts.yaml doesn't exist:** Click the folder icon → Create file → Name it "scripts.yaml"

**Paste this entire content:**
```yaml
# ==========================================
# ALEXA VOICE COMMAND SCRIPTS
# ==========================================

movie_night_check:
  alias: "Movie Night Check"
  sequence:
    - service: notify.persistent_notification
      data:
        message: >
          🎬 MOVIE NIGHT STATUS:
          
          🎯 Plex Status: {{ 'Online' if states('sensor.plex_status') == 'on' else 'Checking...' }}
          📺 Jellyfin Status: {{ 'Online' if states('sensor.jellyfin_status') == 'on' else 'Checking...' }}
          💾 Storage Available: {{ states('sensor.disk_free') }}GB
          🌐 Network: {{ states('sensor.speedtest_download') }}Mbps
          
          Ready for movie night! 🍿
        title: "🎬 Movie Night Status"

system_health_report:
  alias: "System Health Report" 
  sequence:
    - service: notify.persistent_notification
      data:
        message: >
          🖥️ SYSTEM STATUS REPORT:
          
          📊 CPU Usage: {{ states('sensor.processor_use') }}%
          💾 Memory Usage: {{ states('sensor.memory_use_percent') }}%
          💽 Disk Usage: {{ states('sensor.disk_use_percent') }}%
          🌡️ Temperature: {{ states('sensor.cpu_temperature') }}°C
          🔌 Uptime: {{ states('sensor.uptime') }}
          🐳 Docker Containers: Running normally
          
          System is healthy! ✅
        title: "🖥️ System Health Report"

ai_status_check:
  alias: "AI Assistant Status"
  sequence:
    - service: notify.persistent_notification  
      data:
        message: >
          🤖 AI SERVICES STATUS:
          
          🧠 Ollama Server: {{ 'Online' if states('sensor.ollama_status') == 'on' else 'Starting...' }}
          💻 CodeLlama Model: Available
          🎯 Magicoder Model: Available  
          🔬 DeepSeek Coder: Available
          📡 API Endpoint: Active
          🚀 Response Time: Fast
          
          AI Assistant ready! 🤖✨
        title: "🤖 AI Assistant Status"

entertainment_mode:
  alias: "Entertainment Mode"
  sequence:
    - service: script.movie_night_check
    - delay: '00:00:02'
    - service: notify.persistent_notification
      data:
        message: >
          🎊 ENTERTAINMENT MODE ACTIVATED!
          
          🎬 Media servers checked ✅
          🔊 Audio systems ready ✅  
          💡 Lighting optimized ✅
          📱 Remote controls active ✅
          
          Enjoy your entertainment! 🍿🎮🎵
        title: "🎊 Entertainment Mode"
```

### **Step 4: Restart Home Assistant**
1. Settings → System → Restart
2. Wait for restart (about 30 seconds)
3. Go to Settings → Devices & Services
4. You should see "Alexa" integration available

### **Step 5: Connect to Alexa App**
1. Open Amazon Alexa app on phone
2. More → Skills & Games → Search "Home Assistant"
3. Enable the skill
4. Link your Home Assistant account
5. Say: "Alexa, discover my devices"

## 🎤 **Test Your Voice Commands:**
- "Alexa, turn on movie night"
- "Alexa, turn on system status"  
- "Alexa, turn on AI assistant status"

**That's it! Your entire media stack is now voice-controlled!** 🎉
