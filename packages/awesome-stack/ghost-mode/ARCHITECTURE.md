# 🥷 Ghost Mode - Technical Architecture

## System Overview

Ghost Mode is a comprehensive digital anonymity suite that provides multiple layers of protection against online tracking and fingerprinting. It operates through interconnected modules that each handle specific aspects of privacy protection.

## Core Components

### 1. Control Layer
```
ghost-mode (Main Controller)
├── ghost-toggle (Simple Toggle)
├── ghost-tray-widget (GUI Interface)
└── ghost-monitor (Background Monitor)
```

**Purpose:** Orchestrates all anonymity components and provides user interfaces.

**Key Features:**
- Single command activation of all protections
- Visual status feedback through system tray
- Continuous monitoring and automatic repair
- Comprehensive logging and status reporting

### 2. Network Anonymization Layer
```
Network Protection
├── WireGuard Integration (VPN with key rotation)
├── secure-dns (DNS Leak Prevention)
├── IPv6 Disabling (Leak Prevention)
├── MAC Randomization (Hardware ID Spoofing)
└── traffic-obfuscation (Traffic Pattern Masking)
```

**DNS Leak Prevention (`secure-dns`):**
- Completely disables IPv6 to prevent dual-stack leaks
- Forces all DNS traffic through VPN using iptables rules
- Implements DNS-over-TLS for encrypted resolution
- Randomizes DNS servers across multiple providers
- Creates firewall rules blocking direct DNS access

**Traffic Obfuscation (`traffic-obfuscation`):**
- Generates random background HTTP requests
- Varies request timing to mask real activity patterns
- Uses multiple domains to create diverse traffic
- Limits concurrent connections to avoid detection

### 3. Browser Fingerprinting Protection
```
Firefox Ghost Profile
├── WebRTC Disabling (IP Leak Prevention)
├── Canvas Fingerprinting Blocking
├── WebGL Fingerprinting Disabling
├── Audio Fingerprinting Prevention
├── User Agent Randomization
├── JavaScript API Restrictions
└── Tracking Protection (Comprehensive)
```

**Firefox Configuration (`setup-ghost-firefox`):**
```javascript
// Complete WebRTC Disabling
user_pref("media.peerconnection.enabled", false);
user_pref("media.navigator.enabled", false);

// Fingerprinting Resistance
user_pref("privacy.resistFingerprinting", true);
user_pref("privacy.resistFingerprinting.randomDataOnCanvasExtract", true);

// Hardware Concealment
user_pref("webgl.disabled", true);
user_pref("media.webaudio.enabled", false);
user_pref("dom.maxHardwareConcurrency", 4);
```

### 4. Hardware Fingerprinting Spoofing
```
Hardware Spoofing
├── CPU Information (Intel i5-8250U, 4 cores)
├── Memory Information (8GB DDR4)
├── GPU Information (Intel UHD Graphics 620)
├── System Information (ASUS VivoBook X570ZD)
├── Display Resolution (Randomized)
└── Font Enumeration (Minimal Set)
```

**Spoofing Implementation (`spoof-hardware`):**
- Creates fake `/proc/cpuinfo` and `/proc/meminfo` files
- Implements LD_PRELOAD hooks for system info queries
- Generates realistic but consistent fake hardware specs
- Randomizes screen resolution per session
- Restricts font availability to common subset

### 5. Temporal Fingerprinting Protection
```
Time Masking
├── Timezone Randomization (Per Session)
├── JavaScript Date Object Spoofing
├── Performance Timing Jitter
├── System Clock Jitter
└── Command Timing Randomization
```

**Time Spoofing (`mask-time`):**
```javascript
// Browser Time Spoofing
const SPOOF_OFFSET_HOURS = (Math.random() * 24) - 12;
window.Date = SpoofedDate;

// Performance Timing Jitter
window.performance.now = function() {
    const jitter = (Math.random() * 10) - 5;
    return originalPerformanceNow.call(this) + jitter;
};
```

### 6. Monitoring and Maintenance
```
Monitoring System
├── VPN Connection Monitoring
├── IPv6 Leak Detection
├── DNS Leak Detection  
├── Service Health Checking
└── Automatic Recovery
```

**Monitoring Implementation (`ghost-monitor`):**
- Checks VPN status every 30 seconds
- Detects and prevents IPv6 address assignment
- Monitors DNS resolution for leaks to local network
- Automatically restarts failed protection services
- Logs all events with timestamps

## Data Flow Architecture

```
User Action (Click Tray Icon)
         ↓
   ghost-toggle
         ↓
    ghost-mode (Main Controller)
         ↓
   ┌─────────┬─────────┬─────────┬─────────┐
   │ Network │ Browser │Hardware │  Time   │
   │ Layer   │ Layer   │ Layer   │ Layer   │
   └─────────┴─────────┴─────────┴─────────┘
         ↓
   ghost-monitor (Background Monitoring)
         ↓
   Status Updates → Tray Widget → User Feedback
```

## Security Model

### Defense in Depth
1. **Network Level:** VPN tunnel with rotating credentials
2. **Transport Level:** DNS-over-TLS encryption
3. **Application Level:** Browser fingerprinting resistance  
4. **System Level:** Hardware information spoofing
5. **Temporal Level:** Timing correlation prevention

### Threat Model Coverage

| Attack Vector | Protection Method | Implementation |
|---------------|-------------------|----------------|
| IP Address Tracking | WireGuard VPN | Rotating keys every 6 hours |
| DNS Leaks | Forced VPN DNS | iptables rules + IPv6 disable |
| WebRTC Leaks | Complete WebRTC Disable | Firefox configuration |
| Canvas Fingerprinting | Canvas Data Randomization | Browser resistance mode |
| WebGL Fingerprinting | WebGL Complete Disable | Browser configuration |
| Audio Fingerprinting | Audio API Disable | Browser configuration |
| Hardware Fingerprinting | Fake Hardware Info | LD_PRELOAD + fake /proc |
| Screen Fingerprinting | Resolution Spoofing | Randomized per session |
| Font Fingerprinting | Minimal Font Set | FontConfig restriction |
| Timezone Correlation | Random Timezone | Per-session randomization |
| Timing Attacks | Clock Jitter | Random delays + timing fuzzing |
| Traffic Analysis | Pattern Obfuscation | Background noise generation |
| MAC Address Tracking | MAC Randomization | NetworkManager configuration |

## Configuration Management

### File Structure
```
~/.config/ghost-mode/
├── status                 # Current activation state
├── ghost-mode.log        # Main activity log
├── monitor.log           # Monitoring events
├── time-spoof.env        # Time spoofing environment
├── fonts.conf            # Minimal font configuration
├── fake-cpuinfo          # Spoofed CPU information
├── fake-meminfo          # Spoofed memory information
└── quick-reference.txt   # User reference
```

### State Management
- **Active State:** Stored in `~/.config/ghost-mode/status`
- **Process Tracking:** PID files for background services
- **Configuration Persistence:** Environment files sourced by wrappers
- **Log Rotation:** Automatic cleanup of old log entries

## Integration Points

### WireGuard Integration
```bash
# Server rotation script
~/github/awesome-stack/wireguard-tools/server/wireguard-rotate.sh

# Client management
~/github/awesome-stack/wireguard-tools/client/garuda-wg-manager.sh
```

### System Integration
- **Systemd Services:** Integration with existing VPN services
- **NetworkManager:** MAC randomization configuration
- **Firewall Rules:** iptables integration for DNS leak prevention
- **Desktop Environment:** System tray and notification integration

## Performance Considerations

### Resource Usage
- **Memory Overhead:** ~50MB for all background processes
- **CPU Impact:** <1% additional CPU usage
- **Network Latency:** VPN routing adds ~20-50ms
- **Disk Usage:** <100MB for all components and logs

### Optimization Strategies
- **Lazy Loading:** Components activated only when needed
- **Resource Pooling:** Shared connections for traffic obfuscation
- **Efficient Polling:** 30-second intervals for monitoring
- **Log Rotation:** Automatic cleanup prevents disk usage growth

## Failure Modes and Recovery

### Automatic Recovery
1. **VPN Disconnection:** Auto-restart WireGuard service
2. **IPv6 Leak:** Re-disable IPv6 interfaces
3. **DNS Leak:** Reapply firewall rules
4. **Service Crash:** Monitor and restart failed services

### Manual Recovery Procedures
```bash
# Complete system reset
ghost-mode stop
ghost-mode start

# Individual component restart  
ghost-monitor restart
traffic-obfuscation restart

# Emergency leak testing
dns-leak-test
ghost-mode test
```

## Extension Points

### Adding New Protection Modules
1. Create protection script in `scripts/`
2. Add activation logic to `ghost-mode`
3. Add monitoring logic to `ghost-monitor`
4. Update status reporting in `ghost-tray-widget`

### Custom Configuration
- Override default spoofed hardware in `spoof-hardware`
- Modify DNS servers in `secure-dns`
- Adjust monitoring intervals in `ghost-monitor`
- Customize Firefox preferences in `setup-ghost-firefox`

## Security Considerations

### Limitations
- **Root Privileges:** Some features require sudo access
- **VPN Dependency:** Critical dependency on VPN connection
- **Browser Compatibility:** Extreme settings break many websites
- **Performance Impact:** Additional latency and resource usage

### Best Practices
- **Regular Testing:** Verify anonymity with external tools
- **Log Monitoring:** Check logs for warning signs
- **Update Maintenance:** Keep all components current
- **Backup Configuration:** Preserve working configurations

---

This architecture provides comprehensive protection against modern tracking techniques while maintaining usability through automated management and visual feedback systems.
