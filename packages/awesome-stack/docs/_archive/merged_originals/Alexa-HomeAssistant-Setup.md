# 🎤 Alexa Integration - Home Assistant Ready!

## ✅ **Your Home Assistant Status**
- **URL**: `http://homeassistant.local:8123/lovelace/0`
- **Status**: ✅ Online and accessible
- **Ready for**: Alexa integration

## 🚀 **Step-by-Step Alexa Setup**

### **Step 1: Access Home Assistant Configuration**

1. **Open your browser** and go to: `http://homeassistant.local:8123`
2. **Log in** to your Home Assistant dashboard
3. **Navigate to**: Settings → Add-ons → Add-on Store
4. **Search for**: "Alexa Media Player" (if not already installed)

### **Step 2: Enable Alexa Integration**

Add this configuration to your Home Assistant `configuration.yaml`:

```yaml
# Alexa Integration
alexa:
  smart_home:
    locale: en-US
    filter:
      include_domains:
        - script
        - switch
        - media_player
        - sensor
      include_entities:
        - script.movie_night
        - script.system_status
        - script.ai_assistant_status
        - script.media_server_control
        - sensor.plex_status
        - sensor.ai_service_status

# Enable external access (required for Alexa)
http:
  use_x_forwarded_for: true
  trusted_proxies:
    - 192.168.122.0/24
    - 127.0.0.1

# Enable API access
api:

# Alexa Media Player (for TTS responses)
alexa_media:
  accounts:
    - email: !secret alexa_email
      password: !secret alexa_password
      url: amazon.com
```

### **Step 3: Create Essential Scripts**

Create a `scripts.yaml` file (or add to existing) in your Home Assistant config:

```yaml
movie_night:
  alias: "Movie Night Mode"
  icon: mdi:movie
  sequence:
    - service: notify.persistent_notification
      data:
        title: "🎬 Movie Night"
        message: "Checking media servers..."
    - delay: "00:00:02"
    - service: rest_command.check_plex_status
    - service: rest_command.check_jellyfin_status
    - delay: "00:00:01"
    - service: notify.persistent_notification
      data:
        title: "🎬 Movie Night Ready"
        message: "Plex and Jellyfin are ready for streaming!"

system_status:
  alias: "System Status Check"
  icon: mdi:server
  sequence:
    - service: notify.persistent_notification
      data:
        title: "🖥️ System Check"
        message: "Checking all services..."
    - service: rest_command.check_traefik
    - service: rest_command.check_ollama
    - delay: "00:00:02"
    - service: notify.persistent_notification
      data:
        title: "✅ System Status"
        message: "All services are running normally"

ai_assistant_status:
  alias: "AI Assistant Status"
  icon: mdi:robot
  sequence:
    - service: notify.persistent_notification
      data:
        title: "🤖 AI Status"
        message: "Checking AI services..."
    - service: rest_command.check_ollama
    - delay: "00:00:01"
    - service: notify.persistent_notification
      data:
        title: "🤖 AI Ready"
        message: "Ollama and coding assistant are online"

media_server_control:
  alias: "Media Server Control"
  icon: mdi:play-circle
  sequence:
    - service: notify.persistent_notification
      data:
        title: "📺 Media Control"
        message: "Media servers are ready for your commands"
```

### **Step 4: Add REST Commands**

Add these REST commands to your `configuration.yaml`:

```yaml
rest_command:
  check_plex_status:
    url: "http://192.168.122.230:32400/status/sessions"
    method: GET
    timeout: 10
    
  check_jellyfin_status:
    url: "http://192.168.122.231:8096/System/Info"
    method: GET
    timeout: 10
    
  check_ollama:
    url: "http://192.168.122.172:11434/api/version"
    method: GET
    timeout: 10
    
  check_traefik:
    url: "http://192.168.122.103:9080/api/overview"
    method: GET
    timeout: 10
    
  restart_plex:
    url: "http://192.168.122.9:8080/restart/plex"
    method: POST
    timeout: 30
```

### **Step 5: Create Monitoring Sensors**

Add these sensors to track your services:

```yaml
sensor:
  - platform: rest
    name: "Plex Status"
    resource: "http://192.168.122.230:32400/status/sessions"
    value_template: >
      {% if value_json.MediaContainer is defined %}
        {{ value_json.MediaContainer.size | default(0) }} streams
      {% else %}
        Offline
      {% endif %}
    scan_interval: 30
    
  - platform: rest
    name: "AI Service Status"
    resource: "http://192.168.122.172:11434/api/version"
    value_template: >
      {% if value_json.version is defined %}
        Online v{{ value_json.version }}
      {% else %}
        Offline
      {% endif %}
    scan_interval: 60
    
  - platform: rest
    name: "Traefik Routes"
    resource: "http://192.168.122.103:9080/api/overview"
    value_template: >
      {% if value_json.http is defined %}
        {{ value_json.http.routers | length }} routes
      {% else %}
        Offline
      {% endif %}
    scan_interval: 300
```

### **Step 6: Configure Secrets**

Create/edit `secrets.yaml` in your Home Assistant config:

```yaml
# Amazon Alexa credentials
alexa_email: your-amazon-email@example.com
alexa_password: your-amazon-password

# Optional: Add other service API keys
plex_token: your-plex-token-if-needed
```

## 🎤 **Amazon Alexa App Setup**

### **1. Install and Configure Home Assistant Skill**

1. **Open Alexa App** on your phone
2. **Go to**: More → Skills & Games
3. **Search**: "Home Assistant"
4. **Enable Skill** and link your account
5. **Configure**: Use your Home Assistant URL: `http://homeassistant.local:8123`

### **2. Discover Devices**

After configuration:
1. **Say**: "Alexa, discover my devices"
2. **Or in app**: Devices → Discover Devices
3. **Wait**: 20-45 seconds for discovery

### **3. Test Basic Commands**

Try these voice commands:
- "Alexa, turn on movie night"
- "Alexa, turn on system status" 
- "Alexa, turn on AI assistant status"

## 🎯 **Advanced Voice Commands**

### **Create Alexa Routines**

In the Alexa app, create these routines:

#### **"Server Status" Routine:**
- **Trigger**: "Alexa, check my server"
- **Actions**: 
  - Turn on "system status" (Home Assistant)
  - Say: "Checking your media stack"
  - Wait 3 seconds
  - Say: "Check your Home Assistant for details"

#### **"Movie Time" Routine:**
- **Trigger**: "Alexa, it's movie time"  
- **Actions**:
  - Turn on "movie night" (Home Assistant)
  - Say: "Preparing your entertainment system"
  - Wait 2 seconds
  - Say: "Your media servers are ready"

#### **"AI Status" Routine:**
- **Trigger**: "Alexa, check AI"
- **Actions**:
  - Turn on "AI assistant status" (Home Assistant)
  - Say: "Checking your AI services"

## 🔧 **Advanced Integration Examples**

### **Voice-Controlled Media Playback**

Add this to your scripts for more advanced control:

```yaml
play_recent_movie:
  alias: "Play Recent Movie"
  sequence:
    - service: media_player.media_play
      target:
        entity_id: media_player.plex
    - service: notify.persistent_notification
      data:
        message: "Starting your most recent movie on Plex"

pause_all_media:
  alias: "Pause All Media"
  sequence:
    - service: media_player.media_pause
      target:
        entity_id: 
          - media_player.plex
          - media_player.jellyfin
    - service: notify.persistent_notification
      data:
        message: "All media playback paused"
```

### **AI Integration Commands**

Connect your AI services to Alexa:

```yaml
ask_ai_coding_help:
  alias: "AI Coding Help"
  sequence:
    - service: notify.persistent_notification
      data:
        title: "🤖 AI Coding Assistant"
        message: "Your Tauri AI coding assistant is ready at your desktop"
    - service: rest_command.wake_ai_desktop
      
check_ollama_models:
  alias: "Check AI Models"
  sequence:
    - service: rest_command.list_ollama_models
    - service: notify.persistent_notification
      data:
        title: "🧠 AI Models"
        message: "CodeLlama and Magicoder are ready for analysis"
```

## 🚨 **Troubleshooting**

### **Common Issues:**

1. **Alexa can't find devices:**
   ```bash
   # Check Home Assistant external access
   curl -I http://homeassistant.local:8123
   ```

2. **Commands not working:**
   - Check Home Assistant logs: Settings → System → Logs
   - Verify scripts are valid: Developer Tools → Check Configuration

3. **Network issues:**
   ```bash
   # Test your service endpoints
   curl http://192.168.122.230:32400/status/sessions
   curl http://192.168.122.172:11434/api/version
   ```

### **Testing Your Setup:**

```bash
# Test REST commands manually
curl -X GET http://homeassistant.local:8123/api/states
curl -X POST http://homeassistant.local:8123/api/services/script/movie_night
```

## 📋 **Implementation Checklist**

- [ ] **Access Home Assistant** at `http://homeassistant.local:8123`
- [ ] **Add Alexa configuration** to `configuration.yaml`
- [ ] **Create scripts** in `scripts.yaml`
- [ ] **Add REST commands** and sensors
- [ ] **Configure secrets** in `secrets.yaml`
- [ ] **Restart Home Assistant**
- [ ] **Install Home Assistant skill** in Alexa app
- [ ] **Discover devices** with Alexa
- [ ] **Test voice commands**
- [ ] **Create custom routines**

## 🎉 **Expected Voice Commands**

After setup, you'll be able to say:

### **System Control:**
- "Alexa, turn on system status" → Checks all services
- "Alexa, turn on movie night" → Prepares media servers
- "Alexa, turn on AI assistant status" → Checks AI services

### **Custom Routines:**
- "Alexa, check my server" → Full system status
- "Alexa, it's movie time" → Entertainment mode
- "Alexa, check AI" → AI services status

### **Direct Control:**
- "Alexa, turn on media server control" → Ready media playback
- "Alexa, discover devices" → Find new Home Assistant entities

## 🚀 **Next Steps**

Once basic integration works:
1. **Add more scripts** for specific services (Sonarr, Radarr, etc.)
2. **Create custom Alexa skills** for advanced voice interaction
3. **Integrate with AI services** for voice-to-AI queries
4. **Set up automation** based on voice commands
5. **Add TTS responses** for status updates

Your entire media stack will be voice-controlled! 🎊

---
**Ready?** Start by accessing `http://homeassistant.local:8123` and adding the Alexa configuration to your `configuration.yaml` file!
