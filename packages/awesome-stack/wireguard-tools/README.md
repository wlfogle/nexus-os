# 🛡️ WireGuard Advanced VPN & Anti-Tracking Suite

**Complete WireGuard VPN solution with automated rotation, API masking, and GUI management for Garuda Linux**

## 🎯 Overview

This comprehensive WireGuard suite provides:
- **🔄 Automated IP/Key Rotation** - Defeat long-term tracking with 6-hour rotation
- **🛡️ API Request Masking** - Hide AI service usage patterns (OpenAI, Claude, etc.)
- **🖥️ System Tray Widget** - GUI control with real-time status monitoring
- **🌐 Host-Based VPN Server** - Run VPN server directly on Garuda Linux
- **🥷 Stealth Mode** - Rapid rotation for maximum privacy (30-minute intervals)
- **🎛️ Complete GUI Management** - Full control panel with status monitoring

## 📁 Directory Structure

```
wireguard-tools/
├── server/          # Server-side components (Proxmox)
├── client/          # Client tools (Garuda host)  
├── api-masking/     # API obfuscation tools
├── tray-widget/     # GUI system tray widget
└── README.md        # This file
```

## 🚀 Quick Start

### Host-Based Setup (Garuda Linux)

1. **Install dependencies:**
   ```bash
   sudo pacman -S --needed wireguard-tools python-pyqt5 python-requests python-aiohttp cronie
   ```

2. **Initialize VPN server:**
   ```bash
   sudo cp server/wireguard-rotate.sh /root/
   sudo chmod +x /root/wireguard-rotate.sh
   sudo /root/wireguard-rotate.sh init
   ```

3. **Setup API masking proxy:**
   ```bash
   sudo cp api-masking/api-mask-proxy.py /root/
   sudo cp api-masking/api-mask-proxy.service /etc/systemd/system/
   sudo systemctl enable --now api-mask-proxy
   ```

4. **Install tray widget:**
   ```bash
   mkdir -p ~/.local/bin ~/.config/autostart
   cp tray-widget/wireguard-tray-widget.py ~/.local/bin/
   cp tray-widget/wireguard-manager.desktop ~/.config/autostart/
   python3 ~/.local/bin/wireguard-tray-widget.py &
   ```

5. **Enable automated rotation:**
   ```bash
   sudo crontab server/wg-rotation-cron
   ```

## 🔧 Components

### Server Components (`server/`)

#### `wireguard-rotate.sh`
- **Purpose**: Automated WireGuard key and IP rotation
- **Features**: 
  - Generates fresh client keys and server config
  - Random IP assignment from pool
  - Automatic service restart
  - Multiple client support
- **Usage**: `/root/wireguard-rotate.sh garuda-host`

#### `wg-rotation-cron`
- **Purpose**: Automated rotation schedule
- **Default**: Every 6 hours
- **Customizable**: Supports 2-4 hour intervals
- **Installation**: `crontab wg-rotation-cron`

### Client Components (`client/`)

#### `garuda-wg-manager.sh`
- **Purpose**: Command-line WireGuard management
- **Features**:
  - Status monitoring
  - Manual rotation triggers
  - API proxy control
  - AI service testing
  - Stealth mode activation
- **Usage**: `wg-manager status|rotate|test|stealth`

#### `install-garuda-widget.sh`
- **Purpose**: Automated installation for Garuda Linux
- **Features**:
  - Dependency installation
  - SSH key setup
  - Configuration deployment
  - Service activation
- **Usage**: `./install-garuda-widget.sh [PROXMOX_IP] [USER]`

#### `garuda-bypass-script.sh`
- **Purpose**: Legacy CLI bypass tools
- **Features**: Manual VPN rotation and testing

### API Masking (`api-masking/`)

#### `api-mask-proxy.py`
- **Purpose**: HTTP proxy for masking AI API requests
- **Features**:
  - Request header obfuscation
  - Payload pattern breaking
  - Timing randomization
  - Multi-service support (OpenAI, Claude, etc.)
- **Port**: 8080
- **Usage**: `python3 api-mask-proxy.py`

#### `browser-api-mask.js`
- **Purpose**: Client-side browser API masking
- **Features**:
  - Fetch/XHR interception
  - User agent randomization
  - Request timing jitter
  - Payload obfuscation
- **Usage**: Inject into browser console or use as userscript

#### `api-mask-proxy.service`
- **Purpose**: Systemd service for API proxy
- **Features**: Auto-start, restart on failure
- **Installation**: `systemctl enable --now api-mask-proxy`

### Tray Widget (`tray-widget/`)

#### `wireguard-tray-widget.py` 
- **Purpose**: PyQt5 system tray application for Garuda Linux
- **Features**:
  - 🟢/🔴 Visual VPN status indicator
  - Right-click context menu
  - Full control panel dialog
  - Real-time service monitoring
  - Auto-rotation controls
  - API proxy management
  - AI service testing
  - Stealth mode toggle
- **Dependencies**: `python-pyqt5`, `python-requests`

#### `wireguard-manager.desktop`
- **Purpose**: Desktop entry for auto-start
- **Location**: `~/.config/autostart/`
- **Features**: Launches widget on login

## 🎛️ Usage Scenarios

### 1. **Standard Protection**
```bash
# Server: Enable 6-hour rotation
crontab server/wg-rotation-cron

# Client: Use tray widget for management
# Icon shows green when protected
```

### 2. **Maximum Privacy (Stealth Mode)**  
```bash
# Client: Enable aggressive rotation
wg-manager stealth
# OR use tray widget stealth mode button
# Rotates every 30 minutes
```

### 3. **AI Service Access**
```bash
# Start API masking proxy
wg-manager start-proxy

# Configure apps to use proxy: http://127.0.0.1:8080
# Test access with: wg-manager test
```

### 4. **Developer/Power User**
```bash
# Manual rotation
wg-manager rotate

# Check status
wg-manager status

# Open web dashboard  
wg-manager dashboard
```

## 🔒 Security Features

### Network Level Protection
- ✅ **IP Address Rotation**: Changes every 6 hours (default)
- ✅ **Cryptographic Key Rotation**: Fresh keys prevent long-term tracking
- ✅ **DNS Randomization**: Multiple DNS servers used
- ✅ **Traffic Pattern Disruption**: Randomized connection timing

### Application Level Protection  
- ✅ **API Request Masking**: Headers, payloads, timing obfuscation
- ✅ **User Agent Randomization**: Prevents browser fingerprinting
- ✅ **Request Pattern Breaking**: Zero-width characters, temperature variation
- ✅ **Proxy Routing**: All AI traffic routed through masking proxy

### System Integration
- ✅ **Automated Management**: Set-and-forget operation
- ✅ **Visual Monitoring**: System tray status indicator
- ✅ **Emergency Controls**: Quick disconnect/rotation
- ✅ **Multi-Device Support**: Dashboard accessible from mobile

## 🛠️ Configuration

### Server Configuration
```bash
# Edit rotation script for custom IP ranges
vim server/wireguard-rotate.sh
# Modify AVAILABLE_IPS array

# Adjust rotation frequency  
vim server/wg-rotation-cron
# Change schedule (minimum 5 minutes recommended)
```

### Client Configuration
```bash
# Customize Proxmox host/user in widget
vim tray-widget/wireguard-tray-widget.py
# Update proxmox_host and proxmox_user variables

# Modify CLI manager settings
vim client/garuda-wg-manager.sh
# Update PROXMOX_HOST and PROXMOX_USER
```

## 📊 Monitoring & Logs

### Server Logs
```bash
# Rotation activity
tail -f /var/log/wg-rotation.log

# WireGuard status
wg show

# Service status
systemctl status wg-quick@wg0
```

### Client Logs  
```bash
# Tray widget activity (when run in terminal)
python3 ~/.local/bin/wireguard-tray-widget.py

# API proxy logs
tail -f /var/log/api-proxy.log

# WireGuard client status
sudo wg show
```

## 🧪 Testing & Verification

### Network Tests
```bash
# Check external IP (should show VPN IP)
curl ifconfig.me

# Verify DNS resolution  
nslookup google.com

# Test VPN connection
ping 8.8.8.8
```

### AI Service Tests
```bash
# Automated testing
wg-manager test

# Manual testing
curl -s https://api.openai.com
curl -s https://claude.ai
curl -s https://api.anthropic.com
```

### Privacy Verification
```bash
# Check for DNS leaks
wg-manager test

# Verify IP rotation history
grep "New IP:" /var/log/wg-rotation.log

# Test API masking
# Use browser dev tools to inspect modified requests
```

## 🔧 Troubleshooting

### Common Issues

**Tray widget not starting:**
```bash
# Check dependencies
python3 -c "import PyQt5, requests"

# Check permissions
ls -la ~/.local/bin/wireguard-tray-widget.py

# Manual start with debug
python3 ~/.local/bin/wireguard-tray-widget.py
```

**Rotation failing:**
```bash
# Check SSH connectivity  
ssh root@192.168.122.9

# Verify script permissions
ls -la /root/wireguard-rotate.sh

# Check WireGuard service
systemctl status wg-quick@wg0
```

**API proxy issues:**
```bash
# Test proxy health
curl http://127.0.0.1:8080/health

# Check process
ps aux | grep api-mask-proxy

# Review logs
tail -f /var/log/api-proxy.log
```

### Reset Instructions

**Reset client configuration:**
```bash
# Remove old configs
sudo rm -rf /etc/wireguard/*
sudo rm -rf ~/.local/bin/wireguard-*
sudo rm -rf ~/.config/autostart/wireguard-*

# Reinstall
./client/install-garuda-widget.sh
```

**Reset server rotation:**
```bash  
# Stop rotation
crontab -r

# Clear old configs
rm -rf /etc/wireguard/clients/*

# Restart fresh
./server/wireguard-rotate.sh garuda-host
crontab server/wg-rotation-cron
```

## 📋 Requirements

### Server (Proxmox)
- ✅ WireGuard installed and configured
- ✅ Python 3 with aiohttp
- ✅ Root/sudo access
- ✅ Port 51820 (UDP) open
- ✅ Port 10086 (TCP) for dashboard

### Client (Garuda Linux)  
- ✅ Python 3 with PyQt5, requests
- ✅ WireGuard tools
- ✅ SSH client
- ✅ GUI environment with system tray

## 🔗 Integration Notes

### With Existing VPN Providers
- Can work alongside commercial VPN services
- Provides additional layer of IP/key rotation
- API masking works independently of VPN provider

### With Development Tools
- Proxy can integrate with development environments
- Browser scripts work with any web application
- CLI tools integrate with shell workflows

### With Security Tools
- Compatible with firewalls and security software
- Works with DNS over HTTPS (DoH)
- Integrates with system logging

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Garuda Host   │    │  Proxmox Host   │    │    Internet     │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │Tray Widget  │◄┼────┼►│WG Dashboard │ │    │ │   AI APIs   │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
│                 │    │                 │    │       ▲         │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │       │         │
│ │API Proxy    │◄┼────┼►│WG Server    │◄┼────┼───────┘         │
│ └─────────────┘ │    │ └─────────────┘ │    │                 │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │                 │
│ │WG Client    │◄┼────┼►│Rotation     │ │    │                 │
│ └─────────────┘ │    │ │System       │ │    │                 │
└─────────────────┘    │ └─────────────┘ │    └─────────────────┘
                       └─────────────────┘
```

## 📈 Roadmap

### Planned Features
- [ ] **Mobile app** for Android/iOS management
- [ ] **Multi-server support** for load balancing  
- [ ] **Traffic analysis dashboard** with charts
- [ ] **Custom rotation policies** with schedules
- [ ] **Integration with cloud providers** (AWS, GCP)
- [ ] **Blockchain-based key exchange** for enhanced security

### Performance Optimizations
- [ ] **Connection pooling** for faster rotations
- [ ] **Predictive rotation** based on usage patterns
- [ ] **Bandwidth optimization** for mobile clients
- [ ] **Battery usage optimization** for laptops

## 🤝 Contributing

To contribute to this project:

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)  
5. Open a Pull Request

### Development Setup
```bash
git clone https://github.com/wlfogle/awesome-stack.git
cd awesome-stack/wireguard-tools
# Install development dependencies
pip install -r requirements-dev.txt
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## 🙏 Acknowledgments

- **WireGuard Team** - For the excellent VPN protocol
- **Proxmox Team** - For the virtualization platform
- **PyQt5 Contributors** - For the GUI framework
- **Garuda Linux** - For the excellent Arch-based distribution

---

**Created**: August 2025  
**Last Updated**: August 12, 2025  
**Status**: Production Ready ✅
