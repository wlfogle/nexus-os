# TigerVNC Installation in CT-261 (grafana)

## Overview
This document provides step-by-step instructions for installing and configuring TigerVNC on a Proxmox container, specifically targeting container CT-261 running Grafana.

## Installation Details
- **Container Name:** CT-261
- **Service:** Grafana
- **TigerVNC Version:** Latest available via Debian repositories
- **Desktop Environment:** OpenBox + LXPanel

## Installation Instructions
1. **Access the Proxmox Server:**
   ```bash
   ssh proxmox
   ```

2. **Ensure Container CT-261 is Running:**
   ```bash
   pct list
   ```

3. **Install TigerVNC Server:**
   ```bash
   pct exec 261 -- apt update && apt install -y tigervnc-standalone-server
   ```

4. **Install Desktop Environment (OpenBox & LXPanel):**
   ```bash
   pct exec 261 -- apt install -y openbox lxpanel
   ```

5. **Configure VNC Server:**
   - Create necessary directories and files:
     ```bash
     pct exec 261 -- mkdir -p /root/.vnc
     pct exec 261 -- touch /root/.Xresources
     ```
   - Setup xstartup script:
     ```bash
     pct exec 261 -- bash -c 'cat \<< EOF > /root/.vnc/xstartup
     #!/bin/bash
     xrdb $HOME/.Xresources
     openbox-session &
     lxpanel &
     EOF'
     ```
   - Make the script executable:
     ```bash
     pct exec 261 -- chmod +x /root/.vnc/xstartup
     ```

6. **Set VNC Password:**
   ```bash
   pct exec 261 -- bash -c 'echo password | vncpasswd -f > /root/.vnc/passwd'
   pct exec 261 -- chmod 600 /root/.vnc/passwd
   ```

7. **Start VNC Server:**
   ```bash
   pct exec 261 -- vncserver :1 -geometry 1024x768 -depth 24
   ```

## Connection Details
- **IP Address:** 192.168.122.214
- **VNC Port:** 5901 (display :1)
- **Password:** `password`

#### Connect using any VNC client at `192.168.122.214:5901` with `password`.

## Notes
- Ensure the container has network access.
- Adjust display geometry as necessary.
- Security configurations (such as encrypted connections) are not covered in this document.
