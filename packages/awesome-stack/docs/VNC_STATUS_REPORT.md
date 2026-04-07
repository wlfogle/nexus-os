# VNC Status Report

**Generated:** August 3, 2025  
**Scope:** All Proxmox Containers and VMs

## Active VNC Servers

### CT-200 (Alexa Desktop Container)
- **Status:** ✅ ACTIVE
- **IP Address:** 192.168.122.200
- **VNC Port:** 5901
- **Display:** :1
- **Resolution:** 1280x720
- **Color Depth:** 24-bit
- **Process Details:**
  - VNC Server: `/usr/bin/perl /usr/bin/vncserver :1`
  - X Server: `/usr/bin/Xtigervnc :1`
  - Session: `/bin/sh /etc/X11/Xtigervnc-session`
- **Access:** `vncviewer 192.168.122.200:5901`
- **Desktop Environment:** Alexa voice assistant interface
- **Authentication:** VNC password protected

## Inactive/No VNC Services

### CT-100 (WireGuard VPN Server)
- **Status:** ❌ NO VNC
- **Type:** Headless service container

### CT-900 (Ollama AI Container)
- **Status:** ❌ NO VNC
- **Type:** Headless service container

### CT-950 (Message Broker)
- **Status:** ❌ NO VNC
- **Type:** Headless service container

### CT-210 (Jellyfin Media Server)
- **Status:** ❌ NO VNC
- **Type:** Headless service container

### CT-230 (Radarr Movies)
- **Status:** ❌ NO VNC
- **Type:** Headless service container

### CT-260 (qBittorrent Downloads)
- **Status:** ❌ NO VNC
- **Type:** Headless service container

## Connection Instructions

### From Main System (Garuda Linux)
```bash
# Install VNC viewer if not present
sudo pacman -S tigervnc

# Connect to Alexa desktop
vncviewer 192.168.122.200:5901
```

### From External Network
```bash
# SSH tunnel through Proxmox host
ssh -L 5901:192.168.122.200:5901 proxmox

# Then connect locally
vncviewer localhost:5901
```

## Security Notes

- ⚠️ VNC traffic is not encrypted by default
- 🔒 VNC server is configured with password authentication
- 🌐 VNC server allows non-localhost connections (`-localhost no`)
- 🚫 Consider SSH tunneling for external access

## Use Cases

### CT-200 Alexa Desktop
- Voice assistant interface testing
- Desktop application management
- GUI-based configuration tasks
- Audio/video troubleshooting

## Recommendations

1. **Security Enhancement:**
   - Consider enabling TLS encryption on VNC server
   - Restrict VNC access to trusted networks only
   
2. **Additional VNC Services:**
   - VM-611 (BlissOS) could benefit from VNC for Android GUI access
   - Consider VNC for any future desktop VMs

3. **Monitoring:**
   - Add VNC port monitoring to infrastructure checks
   - Include VNC status in automated health reports

## Network Topology

```
Main System (192.168.0.100)
    ↓
Proxmox Host (192.168.122.9)
    ↓
CT-200 Alexa Desktop (192.168.122.200:5901)
```

---
*This report was generated automatically by scanning all containers for active VNC processes.*
