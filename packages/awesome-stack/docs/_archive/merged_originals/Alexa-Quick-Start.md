# 🚀 Alexa Integration - Quick Start Implementation

## 🎯 **Immediate Implementation Steps**

Based on your current setup with Home Assistant (VM 500) and your comprehensive media stack, here's your quickest path to voice control:

### **Step 1: Access Home Assistant**

First, let's find and access your Home Assistant instance:

```bash
# Check Home Assistant access
ssh root@192.168.122.9 "qm guest cmd 500 'netstat -tlnp | grep LISTEN'"

# Or check common ports manually
for port in 8123 8000 80 443; do
    curl -s -o /dev/null -w "Port $port: HTTP %{http_code}\n" "http://192.168.122.52:$port"
done
```

### **Step 2: Basic Alexa Integration Configuration**

Once you access Home Assistant (typically at http://192.168.122.52:8123), add this to your `configuration.yaml`:

```yaml
# Basic Alexa Integration
alexa:
  smart_home:
    locale: en-US
    filter:
      include_domains:
        - media_player
        - script
        - switch
      include_entities:
        - script.movie_night
        - script.system_check
        - script.ai_status

# Enable discovery
discovery:

# Enable frontend
frontend:

# Enable API
api:

# Text-to-speech (for Alexa responses)
tts:
  - platform: amazon_polly
    aws_access_key_id: !secret aws_access_key_id
    aws_secret_access_key: !secret aws_secret_access_key
    region_name: us-east-1
```

### **Step 3: Create Essential Scripts**

Add these scripts to `scripts.yaml` in Home Assistant:

```yaml
movie_night:
  alias: "Movie Night Mode"
  sequence:
    - service: rest_command.check_plex
    - service: rest_command.check_jellyfin
    - delay: "00:00:02"
    - service: persistent_notification.create
      data:
        title: "Movie Night Ready"
        message: "Plex and Jellyfin are ready for streaming!"

system_check:
  alias: "System Health Check"
  sequence:
    - service: rest_command.health_check_proxmox
    - delay: "00:00:03"
    - service: persistent_notification.create
      data:
        title: "System Status"
        message: "All services are running normally"

ai_status:
  alias: "AI Services Status"
  sequence:
    - service: rest_command.check_ollama
    - service: persistent_notification.create
      data:
        title: "AI Status"
        message: "Ollama and AI services are online"

restart_service:
  alias: "Restart Media Service"
  sequence:
    - service: rest_command.restart_plex
    - delay: "00:00:05"
    - service: persistent_notification.create
      data:
        title: "Service Restarted"
        message: "Media service has been restarted successfully"
```

### **Step 4: REST Commands for Your Stack**

Add these REST commands to `configuration.yaml`:

```yaml
rest_command:
  # Health check your Proxmox system
  health_check_proxmox:
    url: "http://192.168.122.9:8006/api2/json/nodes/proxmox/status"
    method: GET
    headers:
      Authorization: "Bearer YOUR_PROXMOX_TOKEN"

  # Check Plex status
  check_plex:
    url: "http://192.168.122.230:32400/status/sessions"
    method: GET
    timeout: 10

  # Check Jellyfin status  
  check_jellyfin:
    url: "http://192.168.122.231:8096/System/Info"
    method: GET
    timeout: 10

  # Check Ollama AI service
  check_ollama:
    url: "http://192.168.122.172:11434/api/version"
    method: GET
    timeout: 10

  # Restart Plex via Proxmox
  restart_plex:
    url: "http://192.168.122.9:8080/restart-plex"
    method: POST

  # Check Traefik status
  check_traefik:
    url: "http://192.168.122.103:9080/api/overview"
    method: GET
    timeout: 10
```

### **Step 5: Sensors for Monitoring**

Add these sensors to track your services:

```yaml
sensor:
  # Plex status sensor
  - platform: rest
    name: "Plex Status"
    resource: "http://192.168.122.230:32400/status/sessions"
    json_attributes:
      - MediaContainer
    value_template: >
      {% if value_json.MediaContainer is defined %}
        {{ value_json.MediaContainer.size | default(0) }} active streams
      {% else %}
        Offline
      {% endif %}

  # AI Service status
  - platform: rest
    name: "AI Service Status"
    resource: "http://192.168.122.172:11434/api/version"
    json_attributes:
      - version
    value_template: >
      {% if value_json.version is defined %}
        Online ({{ value_json.version }})
      {% else %}
        Offline
      {% endif %}

  # System load sensor
  - platform: rest
    name: "Proxmox Load"
    resource: "http://192.168.122.103:9080/api/overview"
    value_template: "{{ value_json.http.routers | length }} services"
```

## 🎤 **Voice Commands You Can Use**

Once configured with Amazon Alexa:

### **Basic Commands:**
- "Alexa, turn on movie night"
- "Alexa, turn on system check" 
- "Alexa, turn on AI status"

### **Advanced Commands (with custom skill):**
- "Alexa, ask my server to check system status"
- "Alexa, ask my server what's playing on Plex"
- "Alexa, ask my server to restart the media service"

## 🛠️ **Amazon Alexa Setup**

### **1. Enable Home Assistant Skill**
1. Open Alexa app on your phone
2. Go to Skills & Games
3. Search for "Home Assistant"
4. Enable the skill
5. Link your Home Assistant account

### **2. Discover Devices**
1. Say "Alexa, discover devices"
2. Or use the Alexa app: Devices → Discover

### **3. Create Routines (Advanced)**
In the Alexa app, create routines like:

**"Movie Night" Routine:**
- Trigger: "Alexa, movie night"
- Actions: 
  - Turn on "movie night" (Home Assistant script)
  - Say: "Preparing your media center"
  - Wait 5 seconds
  - Say: "Your media center is ready"

## 🔧 **Optional: Dedicated Container Approach**

If you prefer a dedicated container for Alexa integration:

```bash
# Create Alexa bridge container
ssh root@192.168.122.9 "
pct create 280 local:vztmpl/ubuntu-22.04-standard_22.04-1_amd64.tar.zst \\
  --hostname alexa-bridge \\
  --cores 1 \\
  --memory 1024 \\
  --swap 512 \\
  --net0 name=eth0,bridge=vmbr0,ip=192.168.122.280/24,gw=192.168.122.1 \\
  --rootfs local-lvm:8

pct start 280

# Install Node-RED for visual automation
pct exec 280 -- bash -c '
apt update
apt install -y curl
curl -fsSL https://deb.nodesource.com/setup_18.x | bash -
apt install -y nodejs
npm install -g --unsafe-perm node-red
npm install -g node-red-contrib-alexa-home-skill-v3
'
"
```

## 🚨 **Troubleshooting**

### **Common Issues:**

1. **Home Assistant not accessible:**
   ```bash
   # Check VM status
   ssh root@192.168.122.9 "qm status 500"
   
   # Check network connectivity
   ping 192.168.122.52
   ```

2. **Alexa can't discover devices:**
   - Ensure Home Assistant is accessible from internet
   - Check firewall settings
   - Verify Amazon account linking

3. **Commands not working:**
   - Check Home Assistant logs
   - Verify REST command URLs
   - Test commands manually in HA interface

### **Testing Commands:**

```bash
# Test your REST commands manually
curl http://192.168.122.230:32400/status/sessions
curl http://192.168.122.172:11434/api/version
curl http://192.168.122.103:9080/api/overview
```

## 📋 **Quick Implementation Checklist**

- [ ] **Find Home Assistant access** (usually port 8123)
- [ ] **Add Alexa integration to configuration.yaml**
- [ ] **Create basic scripts** (movie_night, system_check, ai_status)
- [ ] **Add REST commands** for your services
- [ ] **Restart Home Assistant**
- [ ] **Enable Home Assistant skill in Alexa app**
- [ ] **Discover devices** with Alexa
- [ ] **Test voice commands**
- [ ] **Create custom routines** in Alexa app

## 🎉 **Expected Results**

After implementation, you'll be able to say:
- **"Alexa, movie night"** → Checks Plex/Jellyfin, reports status
- **"Alexa, system check"** → Runs health checks, reports system status  
- **"Alexa, AI status"** → Checks your Ollama AI services
- **"Alexa, discover devices"** → Finds all your Home Assistant entities

Your voice will now control your entire media stack! 🎊

---

**Next Steps:** Once basic integration works, we can add more advanced features like:
- Direct Plex control ("play movie X")  
- AI query integration ("ask AI about Python")
- System administration ("restart service Y")
- Custom responses with your AI models

Ready to get started? Let me know which step you'd like help with first!
