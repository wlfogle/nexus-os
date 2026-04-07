# 🥷 Ghost Mode - Ultimate Online Invisibility Suite

**Complete digital anonymity system that makes you completely invisible online with one click.**

## 🚀 Quick Start

```bash
# Activate complete invisibility
ghost-mode

# Simple toggle
ghost-toggle

# Launch ultra-anonymous browser
ghost-browser

# Check status
ghost-mode status
```

## 🎯 System Tray Widget

**Look for the colored circle in your system tray:**
- 🟢 **Green** = Fully invisible online
- 🟡 **Yellow** = Active but needs attention  
- 🔴 **Red** = Visible online (not protected)

**Controls:**
- **Left Click** = Toggle on/off
- **Right Click** = Context menu
- **Double Click** = Control panel

## 🛡️ Complete Protection Coverage

### 📡 Network Level Anonymization
- ✅ **WireGuard VPN** with 6-hour key rotation
- ✅ **IPv6 completely disabled** (prevents major leaks)
- ✅ **DNS forced through VPN** (firewall-enforced)
- ✅ **DNS-over-TLS encryption**
- ✅ **MAC address randomization**
- ✅ **Traffic pattern obfuscation**

### 🌐 Browser Fingerprinting Protection
- ✅ **WebRTC completely disabled** (no IP leaks)
- ✅ **Canvas fingerprinting blocked**
- ✅ **WebGL fingerprinting disabled**
- ✅ **Audio fingerprinting blocked**
- ✅ **User agent randomization**
- ✅ **Screen resolution spoofing**
- ✅ **Font fingerprinting prevention**
- ✅ **JavaScript capabilities limited**
- ✅ **All tracking disabled**

### 🔧 Hardware Fingerprinting Spoofing
- ✅ **CPU spoofed**: Intel i5-8250U (4 cores)
- ✅ **RAM spoofed**: 8GB DDR4
- ✅ **GPU spoofed**: Intel UHD Graphics 620
- ✅ **System spoofed**: ASUS VivoBook X570ZD
- ✅ **Display resolution randomized**
- ✅ **Minimal font set** (5 fonts only)

### 🕐 Time & Clock Masking
- ✅ **Timezone randomized** per session
- ✅ **Browser time spoofing**
- ✅ **Performance timing jitter**
- ✅ **System clock jitter**
- ✅ **Timing attack prevention**

### 👁️ Continuous Monitoring
- ✅ **Auto-detects VPN disconnections**
- ✅ **Auto-detects IPv6 leaks**
- ✅ **Auto-detects DNS leaks**
- ✅ **Auto-restarts failed protections**
- ✅ **Real-time status monitoring**

## 📦 Installation

### Prerequisites
```bash
# Install dependencies
sudo pacman -S firefox wireguard-tools python-pyqt5 python-requests cronie

# Or on other distros:
sudo apt install firefox wireguard-tools python3-pyqt5 python3-requests cron
```

### Setup
```bash
# Clone repository
git clone https://github.com/your-username/awesome-stack.git
cd awesome-stack/ghost-mode

# Run setup
./install-ghost-mode.sh

# Start system tray widget
ghost-tray-widget &
```

## 🎛️ Usage

### Command Line Interface

```bash
# Main ghost mode control
ghost-mode [start|stop|status|test]

# Simple toggle
ghost-toggle [on|off|toggle|status]

# Launch applications with protections
ghost-browser                    # Ultra-anonymous Firefox
ghost-exec [program]            # Run with spoofed hardware
ghost-time [command]            # Run with spoofed time

# Testing and monitoring
dns-leak-test                   # Test for DNS leaks
ghost-monitor [start|stop|status|logs]
traffic-obfuscation [start|stop|status]
clock-jitter [start|stop|status]

# Information
ghost-help                      # Complete guide
ghost-widget-info              # Widget usage guide
```

### System Tray Widget

The tray widget provides visual status and one-click control:

| Icon Color | Status | Description |
|------------|---------|-------------|
| 🟢 Green | Fully Protected | Complete invisibility active |
| 🟡 Yellow | Partially Protected | Some protections need attention |
| 🔴 Red | Not Protected | Visible online |

**Widget Actions:**
- **Left Click**: Toggle Ghost Mode on/off
- **Right Click**: Context menu with quick actions
- **Double Click**: Open detailed Control Panel

**Context Menu Options:**
- 🔄 Toggle Ghost Mode
- 🌐 Launch Ghost Browser
- 🧪 Test Anonymity
- ⚙️ Control Panel
- ❌ Exit

## 🧪 Testing Your Anonymity

### Automated Testing
```bash
# Run built-in tests
ghost-mode test

# Test specific components
dns-leak-test
```

### Manual Verification Sites
Visit these sites to verify your invisibility:

- **[browserleaks.com](https://browserleaks.com)** - Complete browser testing
- **[dnsleaktest.com](https://dnsleaktest.com)** - DNS leak detection
- **[ipleak.net](https://ipleak.net)** - IP and WebRTC leak testing
- **[whatismyipaddress.com](https://whatismyipaddress.com)** - Basic IP check
- **[panopticlick.eff.org](https://panopticlick.eff.org)** - Browser fingerprint analysis

### Expected Results When Protected
- **IP Address**: Shows VPN server location only
- **DNS Servers**: Shows VPN DNS servers only
- **WebRTC**: No local IP addresses detected
- **Canvas Fingerprint**: Blocked or randomized
- **WebGL Info**: Disabled/blocked
- **System Info**: Shows spoofed hardware (Intel i5-8250U, 8GB RAM, etc.)
- **Screen Resolution**: Shows spoofed resolution
- **Timezone**: Shows spoofed timezone
- **Fonts**: Shows minimal font set only

## 🔧 Configuration

### WireGuard Integration
Ghost Mode integrates with your existing WireGuard setup:
```bash
# Your WireGuard rotation script location
~/github/awesome-stack/wireguard-tools/server/wireguard-rotate.sh

# Client management script
~/github/awesome-stack/wireguard-tools/client/garuda-wg-manager.sh
```

### Custom Configuration
Configuration files are stored in `~/.config/ghost-mode/`:
- `status` - Current activation status
- `ghost-mode.log` - Activity logs
- `monitor.log` - Monitoring logs
- `time-spoof.env` - Time spoofing environment
- `fonts.conf` - Minimal font configuration

## 🚨 Important Notes

### Website Compatibility
- **Many websites will break** due to extreme privacy settings
- **Banking/payment sites** may not work properly
- **Video streaming** may be limited
- **Social media** features may be restricted

### Performance Impact
- **Slight latency increase** due to VPN routing
- **Background processes** for monitoring and obfuscation
- **Memory usage** from multiple protection layers

### Security Considerations
- **VPN connection is critical** - monitor status regularly
- **IPv6 must stay disabled** to prevent leaks
- **MAC randomization** requires network reconnection
- **Some features require sudo** privileges

## 🛠️ Troubleshooting

### Common Issues

**No tray icon visible:**
```bash
# Check if PyQt5 is installed
python3 -c "import PyQt5"

# Restart widget
pkill ghost-tray-widget
ghost-tray-widget &
```

**Ghost mode won't activate:**
```bash
# Check prerequisites
ghost-mode status

# Check VPN connection
ip link show wg0
systemctl status wg-quick@wg0

# Check logs
tail -f ~/.config/ghost-mode/ghost-mode.log
```

**DNS leaks detected:**
```bash
# Reapply DNS protection
secure-dns

# Check IPv6 status
ip -6 addr show

# Test DNS resolution
dns-leak-test
```

**Websites not loading:**
```bash
# Check VPN connection
ping 8.8.8.8

# Temporarily disable strict protections
firefox --safe-mode

# Use ghost browser with different settings
ghost-browser --allow-insecure
```

### Log Files
- `~/.config/ghost-mode/ghost-mode.log` - Main activity log
- `~/.config/ghost-mode/monitor.log` - Monitoring events
- `/var/log/wireguard/` - WireGuard connection logs

## 🔄 Updates and Maintenance

### Regular Maintenance
```bash
# Update ghost mode components
cd ~/github/awesome-stack
git pull origin main

# Refresh VPN keys
wireguard-rotate.sh garuda-host

# Clean old logs
find ~/.config/ghost-mode/ -name "*.log" -mtime +7 -delete
```

### Auto-Start Configuration
The system tray widget auto-starts via:
- `~/.config/autostart/ghost-mode-tray.desktop`

Manual start:
```bash
# Start tray widget
ghost-tray-widget &

# Start monitoring
ghost-monitor start
```

## 📋 Component Architecture

### Core Scripts
- `ghost-mode` - Main control script
- `ghost-toggle` - Simple on/off toggle
- `ghost-tray-widget` - PyQt5 system tray widget
- `ghost-browser` - Ultra-anonymous browser launcher
- `ghost-exec` - Hardware spoofing wrapper
- `ghost-time` - Time spoofing wrapper

### Protection Modules
- `setup-ghost-firefox` - Firefox anonymity configuration
- `spoof-hardware` - Hardware fingerprint spoofing
- `secure-dns` - DNS leak prevention
- `mask-time` - Time and clock masking
- `traffic-obfuscation` - Network traffic obfuscation

### Monitoring Tools
- `ghost-monitor` - Continuous leak detection
- `dns-leak-test` - DNS leak testing
- `clock-jitter` - Timing attack prevention

### Information Scripts
- `ghost-help` - Complete usage guide
- `ghost-widget-info` - Widget usage information

## 🤝 Contributing

### Reporting Issues
- Use GitHub Issues for bug reports
- Include log files and system information
- Test with minimal configuration first

### Feature Requests
- Check existing WireGuard tools integration
- Consider privacy/security impact
- Provide use case examples

## 📄 License

This project is part of the awesome-stack and follows the same licensing terms.

## ⚠️ Disclaimer

This tool is for legitimate privacy protection purposes. Users are responsible for compliance with local laws and website terms of service. The developers are not responsible for any misuse or legal issues arising from the use of this software.

---

**🥷 You are now a digital ghost - completely invisible online! 🔥**
