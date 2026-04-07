# Secure Fire TV Remote Access Setup

This guide shows how to securely expose your Fire TV controller for external access using reverse proxy with authentication.

## Architecture Overview

```
Internet → Cloudflare → Nginx Proxy Manager → Fire TV Controller LXC
         (SSL/WAF)    (Auth + Rate Limiting)   (Local API)
```

## Option 1: Nginx Proxy Manager (Recommended)

### Prerequisites
- Nginx Proxy Manager running in Proxmox (LXC/Docker)
- Domain name pointed to your external IP
- Cloudflare or similar DNS/CDN service

### 1. Create Nginx Proxy Manager Host

**Access Lists (Authentication):**
```nginx
# Create new Access List: "Fire TV Users"
# Authorization: Basic Auth
username: firetv-admin
password: [generate strong password]

# Or use LDAP/OAuth if available
```

**Proxy Host Configuration:**
```nginx
Domain Name: firetv.yourdomain.com
Scheme: http
Forward Hostname/IP: 192.168.1.150
Forward Port: 5000

# SSL Tab:
SSL Certificate: Request New (Let's Encrypt)
Force SSL: Yes
HSTS Enabled: Yes
HTTP/2 Support: Yes

# Advanced Tab:
Access List: Fire TV Users
Block Common Exploits: Yes
Websockets Support: No

# Custom Nginx Configuration:
location / {
    proxy_pass http://192.168.1.150:5000;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    
    # Rate limiting
    limit_req zone=api burst=20 nodelay;
    limit_req_status 429;
    
    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
    
    # CORS headers for web remote
    add_header Access-Control-Allow-Origin "https://firetv.yourdomain.com" always;
    add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
    add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;
    
    # Timeout settings
    proxy_connect_timeout 30s;
    proxy_send_timeout 30s;
    proxy_read_timeout 30s;
}

# Rate limiting zone (add to main nginx.conf)
http {
    limit_req_zone $binary_remote_addr zone=api:10m rate=60r/m;
}
```

### 2. Enhanced Fire TV Controller with Auth

Update the Fire TV controller service to support authentication:

```python
# Add to /opt/firetv-controller/firetv_service.py

from flask import Flask, request, jsonify, abort
from functools import wraps
import base64
import hashlib
import secrets
import time

# Simple API key authentication
API_KEYS = {
    'your-api-key-here': 'admin',
    'mobile-app-key': 'user'  
}

def require_auth(f):
    @wraps(f)
    def decorated_function(*args, **kwargs):
        # Skip auth for local network
        client_ip = request.environ.get('HTTP_X_REAL_IP', request.remote_addr)
        if client_ip.startswith('192.168.1.'):
            return f(*args, **kwargs)
        
        # Check API key
        api_key = request.headers.get('X-API-Key')
        if not api_key or api_key not in API_KEYS:
            abort(401)
        
        return f(*args, **kwargs)
    
    return decorated_function

# Apply auth to routes
@app.route("/status", methods=["GET"])
@require_auth
def status():
    # existing code...

@app.route("/command", methods=["POST"])
@require_auth  
def command():
    # existing code...
```

## Option 2: Cloudflare Zero Trust (Advanced)

### 1. Cloudflare Tunnel Setup

```yaml
# cloudflared config.yml
tunnel: firetv-tunnel
credentials-file: /etc/cloudflared/credentials.json

ingress:
  - hostname: firetv.yourdomain.com
    service: http://192.168.1.150:5000
    originRequest:
      noTLSVerify: true
      connectTimeout: 30s
  - service: http_status:404
```

### 2. Access Policy

```javascript
// Cloudflare Zero Trust Access Policy
{
  "name": "Fire TV Remote Access",
  "decision": "allow",
  "rules": [
    {
      "name": "Allow specific users",
      "require": [
        {
          "email": ["you@yourdomain.com", "family@yourdomain.com"]
        }
      ]
    },
    {
      "name": "Rate limiting", 
      "require": [
        {
          "geo": ["US"]  // Restrict by country
        }
      ]
    }
  ]
}
```

## Option 3: Tailscale/Wireguard VPN (Most Secure)

### Tailscale Setup

```bash
# On Fire TV Controller LXC
curl -fsSL https://tailscale.com/install.sh | sh
tailscale up --advertise-routes=192.168.1.0/24

# On mobile devices
# Install Tailscale app and connect
# Fire TV accessible via: http://100.x.x.x:5000
```

### Wireguard Configuration

```ini
# Server config: /etc/wireguard/wg0.conf
[Interface]
PrivateKey = [server-private-key]
Address = 10.0.0.1/24
ListenPort = 51820

[Peer]
# Mobile device
PublicKey = [client-public-key]
AllowedIPs = 10.0.0.2/32

[Peer] 
# Another device
PublicKey = [client2-public-key]
AllowedIPs = 10.0.0.3/32
```

## Updated Mobile Apps with Authentication

### Web Remote with Auth

```html
<!-- Add to firetv-web-remote.html -->
<script>
// Add API key configuration
let apiKey = localStorage.getItem('firetvApiKey') || '';

function saveApiKey() {
    apiKey = document.getElementById('apiKey').value;
    localStorage.setItem('firetvApiKey', apiKey);
    showNotification('API key saved!');
}

// Update sendCommand function
async function sendCommand(command, params = {}) {
    const headers = {
        'Content-Type': 'application/json'
    };
    
    // Add API key if using external access
    if (serverUrl.includes('yourdomain.com')) {
        headers['X-API-Key'] = apiKey;
    }
    
    try {
        const response = await fetch(`${serverUrl}/command`, {
            method: 'POST',
            headers: headers,
            body: JSON.stringify({
                command: command,
                params: params
            })
        });
        
        if (response.status === 401) {
            showNotification('Authentication failed! Check API key.', 'error');
            return;
        }
        
        const result = await response.json();
        // ... rest of function
    } catch (error) {
        showNotification(`Network error: ${error.message}`, 'error');
    }
}
</script>

<!-- Add API key input to config section -->
<div class="config-section">
    <h3>Configuration</h3>
    <input type="text" class="config-input" id="serverUrl" 
           placeholder="Fire TV Controller URL" value="http://192.168.1.150:5000">
    <input type="password" class="config-input" id="apiKey" 
           placeholder="API Key (for external access)">
    <button class="btn" onclick="saveConfig()" style="width: 100%;">Save Configuration</button>
</div>
```

### CLI with Auth Support

```python
# Update firetv-control-cli.py
import os

def get_auth_headers(url):
    """Get authentication headers if needed"""
    headers = {"Content-Type": "application/json"}
    
    # If external URL, add API key
    if not url.startswith('http://192.168.1.'):
        api_key = os.environ.get('FIRETV_API_KEY')
        if api_key:
            headers['X-API-Key'] = api_key
        else:
            print("Warning: External URL detected but no API key set.")
            print("Set FIRETV_API_KEY environment variable.")
    
    return headers

def send_command(url, command, params=None):
    """Updated send_command with auth"""
    # ... existing code ...
    
    headers = get_auth_headers(url)
    
    try:
        response = requests.post(
            endpoint,
            headers=headers,
            data=json.dumps({"command": command, "params": params}),
            timeout=5
        )
        
        if response.status_code == 401:
            print("Authentication failed! Check API key.")
            return {"success": False, "error": "Authentication failed"}
        
        return response.json()
    except requests.exceptions.RequestException as e:
        # ... existing error handling
```

## Security Best Practices

### 1. API Key Management

```bash
# Generate secure API keys
python3 -c "import secrets; print(secrets.token_urlsafe(32))"

# Store in environment variables
echo 'export FIRETV_API_KEY="your-key-here"' >> ~/.bashrc
```

### 2. Rate Limiting

```nginx
# Nginx rate limiting
http {
    limit_req_zone $binary_remote_addr zone=firetv:10m rate=30r/m;
    limit_req_zone $request_uri zone=commands:10m rate=10r/m;
}

server {
    location /command {
        limit_req zone=commands burst=5 nodelay;
        limit_req zone=firetv burst=10 nodelay;
        # ... proxy config
    }
}
```

### 3. Monitoring & Logging

```python
# Add to Fire TV controller service
import logging
from datetime import datetime

# Enhanced logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('/var/log/firetv-controller.log'),
        logging.StreamHandler()
    ]
)

@app.before_request
def log_request():
    client_ip = request.environ.get('HTTP_X_REAL_IP', request.remote_addr)
    user_agent = request.headers.get('User-Agent', '')
    
    logging.info(f"Request from {client_ip}: {request.method} {request.path} - {user_agent}")
```

### 4. Fail2Ban Integration

```ini
# /etc/fail2ban/filter.d/firetv-controller.conf
[Definition]
failregex = ^.* - ERROR - Failed authentication from <HOST>.*$
ignoreregex =

# /etc/fail2ban/jail.d/firetv-controller.conf
[firetv-controller]
enabled = true
port = 5000,80,443
filter = firetv-controller
logpath = /var/log/firetv-controller.log
maxretry = 5
bantime = 3600
findtime = 600
```

## Usage Examples

### External Access URLs

```bash
# CLI with external access
export FIRETV_API_KEY="your-secure-api-key"
python3 firetv-control-cli.py --url https://firetv.yourdomain.com power

# Web remote
# Visit: https://firetv.yourdomain.com/remote.html
# Enter API key in configuration

# iOS Shortcuts
# URL: https://firetv.yourdomain.com/command
# Headers: X-API-Key: your-api-key
```

This setup provides secure, authenticated access to your Fire TV from anywhere while maintaining strong security practices!
