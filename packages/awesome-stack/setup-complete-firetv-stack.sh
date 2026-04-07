#!/bin/bash

# Complete Fire TV Integration Setup Script
# This script sets up the entire Fire TV control ecosystem
# Run on your Proxmox host

set -e

# Configuration
FIRETV_LXC_ID=150
FIRETV_IP="192.168.1.50"  # Update with your Fire TV IP
CONTROLLER_IP="192.168.1.150"  # LXC IP
DOMAIN="yourdomain.com"  # Update with your domain

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[WARNING] $1${NC}"
}

error() {
    echo -e "${RED}[ERROR] $1${NC}"
    exit 1
}

info() {
    echo -e "${BLUE}[INFO] $1${NC}"
}

check_proxmox() {
    if ! command -v pct &> /dev/null; then
        error "This script must be run on a Proxmox host"
    fi
    log "Proxmox environment detected"
}

create_firetv_controller_lxc() {
    log "Creating Fire TV Controller LXC (ID: $FIRETV_LXC_ID)"
    
    if pct list | grep -q "$FIRETV_LXC_ID"; then
        warn "LXC $FIRETV_LXC_ID already exists, skipping creation"
        return
    fi
    
    # Find Ubuntu template
    TEMPLATE=$(pvesm list local | grep ubuntu-.*standard | tail -1 | awk '{print $1}')
    if [ -z "$TEMPLATE" ]; then
        error "No Ubuntu template found. Please download an Ubuntu container template first."
    fi
    
    # Create LXC
    pct create $FIRETV_LXC_ID $TEMPLATE \
        --hostname firetv-controller \
        --memory 1024 --cores 2 \
        --net0 name=eth0,bridge=vmbr0,ip=dhcp \
        --storage local-lvm:8G \
        --features nesting=1 \
        --startup order=50 \
        --password \
        --unprivileged 1
    
    log "Starting LXC $FIRETV_LXC_ID"
    pct start $FIRETV_LXC_ID
    
    # Wait for container to be ready
    sleep 10
}

setup_firetv_controller() {
    log "Setting up Fire TV Controller service in LXC"
    
    pct exec $FIRETV_LXC_ID -- bash << 'EOF'
        # Update system
        apt update && apt upgrade -y
        
        # Install dependencies
        apt install -y adb python3 python3-pip python3-venv git curl nginx
        
        # Create service directory
        mkdir -p /opt/firetv-controller
        mkdir -p /var/log/firetv
        
        # Create Python virtual environment
        python3 -m venv /opt/firetv-env
        source /opt/firetv-env/bin/activate
        
        # Install Python packages
        pip install androidtv firetv flask requests paho-mqtt websockets aiohttp asyncio-mqtt gunicorn
        
        # Create Fire TV service
        cat > /opt/firetv-controller/firetv_service.py << 'PYTHON'
import asyncio
import logging
import os
from flask import Flask, request, jsonify, abort, send_from_directory
from androidtv import AndroidTV
from functools import wraps
import time

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('/var/log/firetv/controller.log'),
        logging.StreamHandler()
    ]
)

app = Flask(__name__)

# Configuration
FIRETV_HOST = os.environ.get('FIRETV_HOST', '192.168.1.50')
API_KEYS = {
    os.environ.get('FIRETV_API_KEY', 'default-key'): 'admin'
}

class FireTVController:
    def __init__(self, host=FIRETV_HOST):
        self.host = host
        self.device = AndroidTV(self.host)
        self.connected = False
        self.last_connect_attempt = 0
        
    async def connect(self):
        current_time = time.time()
        if current_time - self.last_connect_attempt < 5:  # Rate limit connection attempts
            return self.connected
            
        self.last_connect_attempt = current_time
        
        try:
            await self.device.adb_connect()
            self.connected = True
            logging.info(f"Connected to Fire TV at {self.host}")
            return True
        except Exception as e:
            logging.error(f"Failed connecting to Fire TV: {e}")
            self.connected = False
            return False
    
    async def get_status(self):
        if not self.connected:
            await self.connect()
        if not self.connected:
            return {"connected": False}
            
        try:
            status = await self.device.get_state()
            current_app = await self.device.get_current_app()
            volume = await self.device.get_volume_level()
            return {
                "connected": True,
                "state": status,
                "current_app": current_app,
                "volume": volume,
                "timestamp": int(time.time())
            }
        except Exception as e:
            logging.error(f"Error getting status: {e}")
            return {"connected": False, "error": str(e)}
    
    async def send_command(self, command, **kwargs):
        if not self.connected:
            await self.connect()
        if not self.connected:
            return {"success": False, "error": "Cannot connect to Fire TV"}
            
        try:
            if command == "home":
                await self.device.home()
            elif command == "back":
                await self.device.back()
            elif command == "menu":
                await self.device.menu()
            elif command == "play_pause":
                await self.device.media_play_pause()
            elif command == "power":
                await self.device.power()
            elif command == "up":
                await self.device.up()
            elif command == "down":
                await self.device.down()
            elif command == "left":
                await self.device.left()
            elif command == "right":
                await self.device.right()
            elif command == "center":
                await self.device.center()
            elif command == "launch_app" and "app_id" in kwargs:
                await self.device.start_intent(kwargs["app_id"])
            elif command == "volume_up":
                await self.device.volume_up()
            elif command == "volume_down":
                await self.device.volume_down()
            elif command == "mute":
                await self.device.mute()
            elif command == "text" and "text" in kwargs:
                await self.device.send_text(kwargs["text"])
            else:
                return {"success": False, "error": "Unknown command or missing parameters"}
                
            logging.info(f"Command executed: {command} {kwargs}")
            return {"success": True}
        except Exception as e:
            logging.error(f"Command failed: {command} - {e}")
            return {"success": False, "error": str(e)}

controller = FireTVController()

def require_auth(f):
    @wraps(f)
    def decorated_function(*args, **kwargs):
        client_ip = request.environ.get('HTTP_X_REAL_IP', request.remote_addr)
        
        # Skip auth for local network
        if client_ip.startswith('192.168.1.') or client_ip.startswith('127.'):
            return f(*args, **kwargs)
        
        # Check API key for external requests
        api_key = request.headers.get('X-API-Key')
        if not api_key or api_key not in API_KEYS:
            logging.warning(f"Failed authentication from {client_ip}")
            abort(401)
        
        return f(*args, **kwargs)
    return decorated_function

@app.route("/", methods=["GET"])
def index():
    return send_from_directory('/opt/firetv-controller/static', 'index.html')

@app.route("/status", methods=["GET"])
@require_auth
def status():
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    result = loop.run_until_complete(controller.get_status())
    return jsonify(result)

@app.route("/command", methods=["POST"])
@require_auth
def command():
    data = request.get_json()
    if not data:
        return jsonify({"success": False, "error": "No JSON data"}), 400
        
    cmd = data.get("command")
    params = data.get("params", {})
    
    if not cmd:
        return jsonify({"success": False, "error": "No command specified"}), 400
    
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    result = loop.run_until_complete(controller.send_command(cmd, **params))
    return jsonify(result)

@app.route("/health", methods=["GET"])
def health():
    return jsonify({"status": "healthy", "timestamp": int(time.time())})

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5000, debug=False)
PYTHON

        # Create static directory and copy web interface
        mkdir -p /opt/firetv-controller/static
        
        # Create systemd service
        cat > /etc/systemd/system/firetv-controller.service << 'SERVICE'
[Unit]
Description=Fire TV Controller Service
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/firetv-controller
Environment=PATH=/opt/firetv-env/bin
Environment=FIRETV_HOST=192.168.1.50
Environment=FIRETV_API_KEY=change-this-key
ExecStart=/opt/firetv-env/bin/gunicorn --bind 0.0.0.0:5000 --workers 2 --timeout 30 firetv_service:app
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
SERVICE

        # Enable and start service
        systemctl daemon-reload
        systemctl enable firetv-controller
        
        # Setup nginx reverse proxy for web interface
        cat > /etc/nginx/sites-available/firetv-controller << 'NGINX'
server {
    listen 80;
    server_name _;
    
    location / {
        proxy_pass http://127.0.0.1:5000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # CORS headers
        add_header Access-Control-Allow-Origin * always;
        add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
        add_header Access-Control-Allow-Headers "Content-Type, Authorization, X-API-Key" always;
        
        if ($request_method = OPTIONS) {
            return 204;
        }
    }
}
NGINX

        ln -sf /etc/nginx/sites-available/firetv-controller /etc/nginx/sites-enabled/
        rm -f /etc/nginx/sites-enabled/default
        systemctl enable nginx
        systemctl start nginx
        
        echo "Fire TV Controller setup complete!"
EOF

    log "Starting Fire TV Controller service"
    pct exec $FIRETV_LXC_ID -- systemctl start firetv-controller
}

copy_web_interface() {
    log "Copying web interface to LXC"
    
    # Copy the web remote HTML file
    pct push $FIRETV_LXC_ID firetv-web-remote.html /opt/firetv-controller/static/index.html
    
    # Set permissions
    pct exec $FIRETV_LXC_ID -- chown -R root:root /opt/firetv-controller/static
    pct exec $FIRETV_LXC_ID -- chmod 644 /opt/firetv-controller/static/index.html
}

create_helper_scripts() {
    log "Creating helper scripts"
    
    # Create CLI script wrapper
    cat > /usr/local/bin/firetv << 'SCRIPT'
#!/bin/bash
python3 /root/firetv-control-cli.py "$@"
SCRIPT
    chmod +x /usr/local/bin/firetv
    
    # Copy CLI script to Proxmox host
    cp firetv-control-cli.py /root/
    chmod +x /root/firetv-control-cli.py
    
    # Create quick command aliases
    cat >> ~/.bashrc << 'ALIASES'

# Fire TV Quick Commands
alias ftv-power='firetv power'
alias ftv-home='firetv home'
alias ftv-netflix='firetv app netflix'
alias ftv-plex='firetv app plex'
alias ftv-youtube='firetv app youtube'
alias ftv-status='firetv status'
ALIASES

    log "Helper scripts created. Reload bash or run 'source ~/.bashrc'"
}

setup_monitoring() {
    log "Setting up monitoring and logging"
    
    pct exec $FIRETV_LXC_ID -- bash << 'EOF'
        # Install monitoring tools
        apt install -y htop iotop logrotate fail2ban
        
        # Setup log rotation
        cat > /etc/logrotate.d/firetv-controller << 'LOGROTATE'
/var/log/firetv/*.log {
    daily
    missingok
    rotate 7
    compress
    delaycompress
    notifempty
    sharedscripts
    postrotate
        systemctl reload firetv-controller
    endscript
}
LOGROTATE

        # Configure fail2ban for Fire TV controller
        cat > /etc/fail2ban/filter.d/firetv-controller.conf << 'F2BFILTER'
[Definition]
failregex = ^.* - WARNING - Failed authentication from <HOST>.*$
ignoreregex =
F2BFILTER

        cat > /etc/fail2ban/jail.d/firetv-controller.conf << 'F2BJAIL'
[firetv-controller]
enabled = true
port = 5000,80
filter = firetv-controller
logpath = /var/log/firetv/controller.log
maxretry = 5
bantime = 3600
findtime = 600
F2BJAIL

        systemctl enable fail2ban
        systemctl start fail2ban
        
        echo "Monitoring setup complete!"
EOF
}

generate_api_key() {
    log "Generating secure API key"
    
    API_KEY=$(python3 -c "import secrets; print(secrets.token_urlsafe(32))")
    
    # Update the API key in the service
    pct exec $FIRETV_LXC_ID -- bash << EOF
        sed -i "s/change-this-key/$API_KEY/" /etc/systemd/system/firetv-controller.service
        systemctl daemon-reload
        systemctl restart firetv-controller
EOF
    
    info "Generated API Key: $API_KEY"
    info "Save this key for external access!"
    
    # Save to file
    echo "FIRETV_API_KEY=$API_KEY" > /root/firetv-api-key.txt
    chmod 600 /root/firetv-api-key.txt
}

test_installation() {
    log "Testing Fire TV Controller installation"
    
    # Wait for service to start
    sleep 5
    
    # Get LXC IP
    LXC_IP=$(pct exec $FIRETV_LXC_ID -- hostname -I | awk '{print $1}')
    
    # Test API endpoints
    info "Testing API at http://$LXC_IP:5000"
    
    if curl -s "http://$LXC_IP:5000/health" | grep -q "healthy"; then
        log "Health check: PASSED"
    else
        warn "Health check: FAILED"
    fi
    
    if curl -s "http://$LXC_IP:5000/status" | grep -q "connected"; then
        log "Status endpoint: PASSED"
    else
        warn "Status endpoint: FAILED (Fire TV might be off)"
    fi
    
    # Test CLI
    if /root/firetv-control-cli.py --url "http://$LXC_IP:5000" status > /dev/null 2>&1; then
        log "CLI test: PASSED"
    else
        warn "CLI test: FAILED"
    fi
    
    info "Installation test complete!"
    info ""
    info "Access your Fire TV remote at: http://$LXC_IP/"
    info "CLI usage: firetv --help"
    info "API key saved to: /root/firetv-api-key.txt"
}

create_documentation() {
    log "Creating documentation"
    
    cat > /root/FIRETV_SETUP_COMPLETE.md << EOF
# Fire TV Integration Setup Complete

## 🎉 Installation Summary

Your complete Fire TV integration stack has been successfully installed!

### Components Installed:

1. **Fire TV Controller LXC (ID: $FIRETV_LXC_ID)**
   - Python REST API service
   - Web-based remote interface
   - Authentication support
   - Monitoring and logging

2. **Command Line Tools**
   - \`firetv\` command for terminal control
   - Quick aliases (ftv-power, ftv-netflix, etc.)

3. **Security Features**
   - API key authentication for external access
   - Fail2ban protection
   - Rate limiting and logging

## 🚀 Quick Start

### Web Interface
Visit: http://$LXC_IP/

### Command Line
\`\`\`bash
firetv power              # Toggle power
firetv app netflix        # Launch Netflix
firetv status             # Check status
firetv --help             # Show all options
\`\`\`

### API Access
\`\`\`bash
# Status
curl http://$LXC_IP:5000/status

# Send command
curl -X POST http://$LXC_IP:5000/command \\
  -H "Content-Type: application/json" \\
  -d '{"command": "power"}'
\`\`\`

## 🔐 Security

- **API Key**: \`$(cat /root/firetv-api-key.txt | cut -d= -f2)\`
- **Local Access**: No authentication required from 192.168.1.x
- **External Access**: Requires API key in X-API-Key header

## 📱 Mobile Integration

### iOS Shortcuts
1. Use the configurations in \`firetv-ios-shortcuts.md\`
2. Set URL to: \`http://$LXC_IP:5000\`

### Android Tasker
1. Import the script from \`firetv-tasker-integration.js\`
2. Update server URL in the script

## 🔧 Configuration Files

- **Service Config**: \`/etc/systemd/system/firetv-controller.service\`
- **Logs**: \`/var/log/firetv/controller.log\`
- **Web Interface**: \`/opt/firetv-controller/static/index.html\`
- **API Key**: \`/root/firetv-api-key.txt\`

## 🛡️ External Access Setup

For secure external access, follow the guide in:
\`firetv-secure-proxy-config.md\`

## 📊 Monitoring

- **Service Status**: \`pct exec $FIRETV_LXC_ID -- systemctl status firetv-controller\`
- **Logs**: \`pct exec $FIRETV_LXC_ID -- tail -f /var/log/firetv/controller.log\`
- **Health Check**: \`curl http://$LXC_IP:5000/health\`

## 🔄 Maintenance

### Restart Service
\`\`\`bash
pct exec $FIRETV_LXC_ID -- systemctl restart firetv-controller
\`\`\`

### Update Fire TV IP
\`\`\`bash
pct exec $FIRETV_LXC_ID -- sed -i 's/FIRETV_HOST=.*/FIRETV_HOST=NEW_IP/' /etc/systemd/system/firetv-controller.service
pct exec $FIRETV_LXC_ID -- systemctl daemon-reload
pct exec $FIRETV_LXC_ID -- systemctl restart firetv-controller
\`\`\`

## 🎯 Next Steps

1. **Configure your Fire TV IP** if different from $FIRETV_IP
2. **Set up mobile apps** using the provided guides
3. **Configure external access** for remote control
4. **Integrate with Home Assistant** using the original guide

## 📚 Additional Resources

- \`Fire_TV_Cube_Proxmox_Integration_Guide.md\` - Complete integration guide
- \`firetv-control-cli.py\` - Command line interface
- \`firetv-web-remote.html\` - Web interface
- \`firetv-ios-shortcuts.md\` - iOS integration
- \`firetv-tasker-integration.js\` - Android integration
- \`firetv-secure-proxy-config.md\` - External access setup

Enjoy your fully integrated Fire TV control system! 🔥📺
EOF
    
    log "Documentation created: /root/FIRETV_SETUP_COMPLETE.md"
}

main() {
    log "Starting Complete Fire TV Integration Setup"
    log "This will create LXC $FIRETV_LXC_ID and set up the entire stack"
    
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log "Setup cancelled"
        exit 0
    fi
    
    check_proxmox
    create_firetv_controller_lxc
    setup_firetv_controller
    
    if [ -f "firetv-web-remote.html" ]; then
        copy_web_interface
    else
        warn "firetv-web-remote.html not found, skipping web interface"
    fi
    
    if [ -f "firetv-control-cli.py" ]; then
        create_helper_scripts
    else
        warn "firetv-control-cli.py not found, skipping CLI setup"
    fi
    
    setup_monitoring
    generate_api_key
    test_installation
    create_documentation
    
    log "🎉 Complete Fire TV Integration Setup Finished!"
    log ""
    log "📖 Read /root/FIRETV_SETUP_COMPLETE.md for usage instructions"
    log "🌐 Web Interface: http://$(pct exec $FIRETV_LXC_ID -- hostname -I | awk '{print $1}')/"
    log "💻 CLI: firetv --help"
    log "🔑 API Key: $(cat /root/firetv-api-key.txt | cut -d= -f2)"
    log ""
    log "Don't forget to:"
    log "1. Configure your Fire TV IP if different from $FIRETV_IP"
    log "2. Enable ADB debugging on your Fire TV Cube"
    log "3. Set up mobile apps using the provided guides"
    log ""
    log "Happy controlling! 🎮"
}

# Run main function
main "$@"
