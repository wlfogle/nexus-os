# 🥷 Ghost Mode API Documentation

## Command Line Interface

### Main Control Commands

#### `ghost-mode`
**Usage:** `ghost-mode [start|stop|status|test]`

**Description:** Main controller for all anonymity protections.

**Commands:**
- `start` - Activate complete invisibility suite
- `stop` - Deactivate all protections  
- `status` - Show detailed status of all components
- `test` - Run comprehensive anonymity tests

**Exit Codes:**
- `0` - Success
- `1` - General error
- `2` - Missing dependencies
- `3` - VPN connection failed

**Examples:**
```bash
# Activate complete ghost mode
ghost-mode start

# Check current status
ghost-mode status

# Test for leaks
ghost-mode test
```

#### `ghost-toggle`
**Usage:** `ghost-toggle [on|off|toggle|status]`

**Description:** Simple on/off toggle for ghost mode.

**Commands:**
- `on|start|activate` - Turn on ghost mode
- `off|stop|deactivate` - Turn off ghost mode
- `toggle` - Switch current state (default)
- `status` - Show current status

**Examples:**
```bash
# Toggle current state
ghost-toggle

# Explicitly turn on
ghost-toggle on

# Check status
ghost-toggle status
```

### Application Launchers

#### `ghost-browser`
**Usage:** `ghost-browser [URL]`

**Description:** Launch ultra-anonymous Firefox browser.

**Features:**
- Complete WebRTC disabling
- Canvas/WebGL fingerprinting blocked
- Hardware fingerprint spoofing
- Time zone spoofing
- Minimal font set

**Examples:**
```bash
# Launch anonymous browser
ghost-browser

# Open specific URL
ghost-browser https://browserleaks.com
```

#### `ghost-exec`
**Usage:** `ghost-exec [program] [args...]`

**Description:** Run programs with spoofed hardware fingerprints.

**Environment Variables Set:**
- `GHOST_MODE=1`
- `FONTCONFIG_FILE` - Minimal font config
- `TZ` - Spoofed timezone
- `FAKE_HOME` - Temporary home directory

**Examples:**
```bash
# Run Firefox with spoofed hardware
ghost-exec firefox

# Run curl with spoofed environment
ghost-exec curl ipinfo.io
```

#### `ghost-time`
**Usage:** `ghost-time [command] [args...]`

**Description:** Execute commands with spoofed time and timezone.

**Features:**
- Random timezone per session
- Command timing randomization
- Environment variable spoofing

**Examples:**
```bash
# SSH with spoofed time
ghost-time ssh user@server

# Curl with spoofed timezone
ghost-time curl worldtimeapi.org/api/ip
```

### Protection Modules

#### `secure-dns`
**Usage:** `secure-dns`

**Description:** Configure DNS leak prevention system.

**Actions:**
- Disable IPv6 completely
- Set up DNS-over-TLS
- Create firewall rules
- Configure MAC randomization

**Requires:** sudo privileges

#### `spoof-hardware`
**Usage:** `spoof-hardware`

**Description:** Set up hardware fingerprint spoofing.

**Spoofed Components:**
- CPU: Intel i5-8250U (4 cores)
- RAM: 8GB DDR4
- GPU: Intel UHD Graphics 620
- System: ASUS VivoBook X570ZD
- Screen: Randomized resolution
- Fonts: Minimal set (5 fonts)

#### `mask-time`
**Usage:** `mask-time`

**Description:** Configure time and clock masking.

**Features:**
- Random timezone selection
- Browser time spoofing scripts
- Timing attack prevention
- Command execution delays

### Monitoring Tools

#### `ghost-monitor`
**Usage:** `ghost-monitor [start|stop|status|logs]`

**Description:** Background monitoring and leak detection.

**Monitoring:**
- VPN connection status (every 30s)
- IPv6 leak detection
- DNS leak detection
- Service health checking
- Automatic recovery

**Commands:**
- `start` - Start background monitoring
- `stop` - Stop monitoring service
- `status` - Show monitoring status
- `logs` - Display recent monitoring events

#### `dns-leak-test`
**Usage:** `dns-leak-test`

**Description:** Test for DNS leaks and misconfigurations.

**Tests:**
- Current DNS configuration
- Direct DNS server access
- IPv6 global addresses
- WebRTC leak indicators

#### `traffic-obfuscation`
**Usage:** `traffic-obfuscation [start|stop|status]`

**Description:** Generate background traffic for pattern masking.

**Commands:**
- `start` - Begin traffic obfuscation
- `stop` - Stop background traffic
- `status` - Show obfuscation status

**Background Activity:**
- Random HTTP requests to decoy domains
- Variable timing (1-60 second intervals)
- Limited concurrent connections (max 3)

#### `clock-jitter`
**Usage:** `clock-jitter [start|stop|status]`

**Description:** Add random timing variations to prevent timing attacks.

**Commands:**
- `start` - Start timing jitter service
- `stop` - Stop jitter service
- `status` - Show jitter status

### Information Commands

#### `ghost-help`
**Usage:** `ghost-help`

**Description:** Display complete usage guide with all protections and commands.

#### `ghost-widget-info`
**Usage:** `ghost-widget-info`

**Description:** Show system tray widget usage information and current status.

## System Tray Widget API

### PyQt5 Widget Interface

#### Class: `GhostModeTrayIcon`

**Methods:**
- `update_status(status_dict)` - Update visual indicators
- `toggle_ghost_mode()` - Toggle protection state
- `show_control_panel()` - Display detailed control dialog
- `activated(reason)` - Handle click events

**Status Dictionary:**
```python
status = {
    'ghost_active': bool,      # Ghost mode active
    'vpn_active': bool,        # VPN connection status
    'ipv6_disabled': bool,     # IPv6 disabled status
    'traffic_obfuscation': bool, # Traffic masking active
    'monitoring': bool         # Background monitoring active
}
```

#### Class: `GhostModeDialog`

**Methods:**
- `update_status(status)` - Update status labels
- `toggle_ghost_mode()` - Toggle via GUI
- `launch_ghost_browser()` - Start anonymous browser
- `test_anonymity()` - Run leak tests
- `update_logs()` - Refresh activity logs

### Icon States

| Icon Color | Meaning | Status |
|------------|---------|---------|
| 🟢 Green | Fully Protected | All systems operational |
| 🟡 Yellow | Partially Protected | Some components need attention |
| 🔴 Red | Not Protected | Ghost mode inactive |

## Configuration Files

### Status File: `~/.config/ghost-mode/status`
```
active      # Ghost mode is active
inactive    # Ghost mode is inactive
```

### Time Environment: `~/.config/ghost-mode/time-spoof.env`
```bash
export TZ="America/Los_Angeles"
export LC_TIME="C"
export LC_ALL="C"
```

### Hardware Spoofing Files
- `~/.config/ghost-mode/fake-cpuinfo` - Spoofed CPU information
- `~/.config/ghost-mode/fake-meminfo` - Spoofed memory information
- `~/.config/ghost-mode/fake-gpu` - Spoofed GPU information

### Firefox Profile: `~/.mozilla/firefox-ghost/user.js`
```javascript
// Network anonymization
user_pref("media.peerconnection.enabled", false);
user_pref("network.dns.disableIPv6", true);

// Fingerprinting protection
user_pref("privacy.resistFingerprinting", true);
user_pref("webgl.disabled", true);
user_pref("media.webaudio.enabled", false);

// Tracking protection
user_pref("privacy.trackingprotection.enabled", true);
user_pref("privacy.trackingprotection.socialtracking.enabled", true);
```

## Integration APIs

### WireGuard Integration

**Server Rotation:**
```bash
# Trigger key rotation
~/github/awesome-stack/wireguard-tools/server/wireguard-rotate.sh garuda-host
```

**Client Management:**
```bash
# Client control interface
~/github/awesome-stack/wireguard-tools/client/garuda-wg-manager.sh [rotate|status|test]
```

### System Integration

**NetworkManager MAC Randomization:**
```ini
# /etc/NetworkManager/conf.d/99-random-mac.conf
[device]
wifi.scan-rand-mac-address=yes

[connection]  
wifi.cloned-mac-address=random
ethernet.cloned-mac-address=random
```

**Systemd DNS Configuration:**
```ini
# /etc/systemd/resolved.conf.d/ghost-dns.conf
[Resolve]
DNS=1.1.1.1 1.0.0.1
DNSOverTLS=yes
DNSSEC=yes
```

## Error Handling

### Common Error Codes

| Exit Code | Description | Resolution |
|-----------|-------------|------------|
| 0 | Success | Operation completed |
| 1 | General error | Check logs and dependencies |
| 2 | Missing dependencies | Install required packages |
| 3 | VPN connection failed | Check WireGuard configuration |
| 4 | Permission denied | Run with sudo if required |
| 5 | Service not running | Start required services |

### Log File Locations

- **Main Activity:** `~/.config/ghost-mode/ghost-mode.log`
- **Monitoring Events:** `~/.config/ghost-mode/monitor.log`
- **VPN Connection:** `/var/log/wireguard/`
- **System DNS:** `journalctl -u systemd-resolved`

### Debugging Commands

```bash
# Check component status
ghost-mode status

# Test for leaks
dns-leak-test
ghost-mode test

# View recent activity
tail -f ~/.config/ghost-mode/ghost-mode.log

# Check service health
ghost-monitor status
traffic-obfuscation status
clock-jitter status

# Test VPN connectivity
ping -c 3 8.8.8.8
ip link show wg0
```

## Development API

### Adding Protection Modules

1. **Create Script:** Place in `scripts/` directory
2. **Add to Controller:** Include in `ghost-mode` activation
3. **Add Monitoring:** Include health checks in `ghost-monitor`
4. **Update Widget:** Add status reporting to tray widget

**Template Script Structure:**
```bash
#!/bin/bash

# Protection Module Template
MODULE_NAME="my-protection"
CONFIG_DIR="$HOME/.config/ghost-mode"

activate_protection() {
    # Implementation here
    echo "active" > "$CONFIG_DIR/$MODULE_NAME.status"
}

deactivate_protection() {
    # Cleanup here
    echo "inactive" > "$CONFIG_DIR/$MODULE_NAME.status"
}

check_status() {
    if [ -f "$CONFIG_DIR/$MODULE_NAME.status" ] && \
       [ "$(cat "$CONFIG_DIR/$MODULE_NAME.status")" = "active" ]; then
        return 0  # Active
    else
        return 1  # Inactive
    fi
}

case "$1" in
    "start") activate_protection ;;
    "stop") deactivate_protection ;;
    "status") check_status && echo "active" || echo "inactive" ;;
    *) echo "Usage: $0 [start|stop|status]" ;;
esac
```

### Widget Extension

**Adding Status Indicators:**
```python
# In GhostModeMonitor.check_status()
status['my_protection'] = self.is_my_protection_active()

# In GhostModeDialog.update_status()
if status['my_protection']:
    self.status_labels['my_protection'].setText("✅ My Protection: ACTIVE")
else:
    self.status_labels['my_protection'].setText("❌ My Protection: INACTIVE")
```

---

This API documentation provides comprehensive integration points for developers wanting to extend or integrate with the Ghost Mode anonymity suite.
