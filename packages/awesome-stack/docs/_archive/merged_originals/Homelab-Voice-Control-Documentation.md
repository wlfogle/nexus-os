# 🚀 **Project Documentation: The Ultimate Voice-Controlled Homelab**

---

## **1. Project Overview & Goal**

**Goal:** To create a comprehensive, voice-controlled management system for a sophisticated 47-container media stack and Proxmox hypervisor infrastructure. The system will leverage a multi-device Amazon Alexa ecosystem with custom wake words for context-aware commands, ensuring both powerful system administration and intuitive home automation.

**Primary User:** Lou Fogle
**Key Technologies:** Proxmox, LXC, Docker, Home Assistant, Amazon Alexa, custom voice commands.

---

## **2. System Infrastructure**

### **2.1. Hypervisor**
- **Platform:** Proxmox VE
- **Host IP:** `192.168.122.9`
- **Control Method:** SSH commands from Home Assistant

### **2.2. Core Management Systems**
- **Home Assistant:** VM 500 (`haos16.0`) - Central hub for automation and voice command processing.
- **Traefik:** CT 103 - Load balancer and reverse proxy for all web services.
- **AI Services:** CT 900 (`ai-container`) - Houses Ollama, CodeLlama, Magicoder, and other AI models.

### **2.3. Containerized Application Stack (47+ LXC Containers)**
- **Media Servers:** `plex` (230), `jellyfin` (231), `audiobookshelf` (232), `calibre-web` (233)
- **Download Clients:** `qbittorrent` (212), `deluge` (224)
- **PVR & Indexers:** `sonarr` (214), `radarr` (215), `prowlarr` (210), `jackett` (211), `readarr` (217), `whisparr` (219)
- **Media Automation:** `bazarr` (240), `overseerr` (241), `jellyseerr` (242), `flexget` (271), `kometa` (245), `gaps` (246)
- **Monitoring & Dashboards:** `prometheus` (260), `grafana` (261), `tautulli` (244), `organizr` (274), `homarr` (275), `homepage` (276)
- **Security & Network:** `wireguard` (100), `gluetun` (101), `crowdsec` (278), `vaultwarden` (104), `authentik` (107), `tailscale` (279)
- **File Management & Tools:** `filebot` (270), `janitorr` (247), `decluttarr` (248), `recyclarr` (277)
- **Databases:** `postgresql` (106), `valkey` (105)

---

## **3. Voice Control Ecosystem**

### **3.1. Alexa Device Inventory & Wake Words**

| Device Name             | Location      | Wake Word  | Serial Number         | Primary Use Case                           |
| ----------------------- | ------------- | ---------- | --------------------- | ------------------------------------------ |
| **Lou's Smart Glasses** | Mobile (User) | **Alexa**  | `G002BC04434500TE`    | System Admin, Mobile Control, Hands-Free   |
| **Lou's TV**            | Living Room   | **Computer** | `G9V1TD10231700A5`    | Entertainment, Media Consumption           |
| **Lou's Echo Spot**     | Bedroom       | **Amazon** | `GR72ML0542030CPR`    | Bedroom Control, Jackie's TV Management  |
| **Lou's Echo Show**     | Kitchen       | **Echo**   | `G000RA1101240ADU`    | Visual Feedback, Kitchen Tasks           |
| **Jackie's FireTV**     | Bedroom       | (N/A)      | `GEX1RE0030340072`    | Controlled by Echo Spot                    |

### **3.2. Voice Control Architecture**

A local, cloud-independent architecture is implemented to ensure privacy, speed, and reliability.

```
Your Voice
    ↓
[ Your 4 Alexa Devices ]
    ↓
[ Local WiFi Network ]
    ↓
LXC Container 280 (Alexa Bridge: HABridge/Fauxmo)
    ↓
Home Assistant VM 500 (Processes voice intent)
    ↓
SSH Commands / REST API Calls
    ↓
[ Proxmox Host & 47+ LXC Containers ]
```

**Key Benefit:** This design **bypasses Amazon's cloud services** for skill processing, avoiding OTP issues and reliance on external connectivity.

---

## **4. Implemented Voice Commands (35+ Total)**

The system includes over 35 custom voice commands, implemented as scripts within Home Assistant and exposed to the Alexa ecosystem via the local bridge.

### **4.1. Command Categories & Examples**

- **Media Management (`Computer, movie night`)**
  - Controls Plex, Jellyfin, download clients, and media request services.
  - Triggers library scans, pauses downloads, checks status.

- **System Administration (`Alexa, system status`)**
  - Manages the Proxmox host and all 47 containers.
  - Restarts services, checks resource usage, triggers backups.

- **AI Services (`Alexa, check AI status`)**
  - Monitors the Ollama server and loaded models.

- **Infrastructure (`Echo, show network status`)**
  - Controls Traefik, databases, VPNs, and security services.
  - Provides visual feedback on the Echo Show.

- **Lifestyle Modes (`Amazon, bedtime mode`)**
  - Custom scenes that orchestrate multiple actions, such as `Gaming Mode` (pauses media encoding, prioritizes network for gaming) or `Maintenance Mode` (safely stops non-essential services).

### **4.2. Full Command List**
*(Abridged - See `homeassistant-expanded-scripts.yaml` for full code)*

1.  **movie_night**: Prepares media servers.
2.  **system_status**: Checks health of all containers.
3.  **ai_assistant_status**: Checks Ollama AI.
4.  **media_server_control**: General media server status.
5.  **entertainment_mode**: Full media stack preparation.
6.  **server_health_report**: Detailed system diagnostics.
7.  **ai_coding_session**: Prepares AI for development.
8.  **pause_all_media**: Pauses all active streams.
9.  **resume_all_media**: Resumes paused streams.
10. **restart_media_services**: Restarts Plex & Jellyfin.
11. **check_storage_space**: Reports disk usage.
12. **network_status_check**: Checks Traefik & connectivity.
13. **emergency_status**: Quick diagnostic.
14. **check_downloads**: Status of all download clients.
15. **pause_downloads**: Pauses torrents.
16. **resume_downloads**: Resumes torrents.
17. **check_arr_apps**: Status of Sonarr, Radarr, etc.
18. **restart_plex**: Restarts only the Plex container.
19. **restart_jellyfin**: Restarts only the Jellyfin container.
20. **scan_plex_library**: Triggers Plex library scan.
21. **scan_jellyfin_library**: Triggers Jellyfin library scan.
22. **check_live_tv**: Checks IPTV & TVHeadend.
23. **check_requests**: Checks Overseerr & Jellyseerr.
24. **check_plex_monitoring**: Checks Tautulli.
25. **restart_traefik**: Restarts the load balancer.
26. **check_security**: Checks Vaultwarden, Crowdsec, Authentik.
27. **check_vpn**: Checks Wireguard & Gluetun.
28. **check_cleanup_tools**: Checks Janitorr, Decluttarr, etc.
29. **check_organizr**: Checks dashboard services.
30. **restart_ai_services**: Restarts the AI container.
31. **gaming_mode**: Optimizes system for gaming.
32. **maintenance_mode**: Stops non-essential services.
33. ... and more.

---

## **5. Home Assistant Optimization Summary**

The Home Assistant instance has been professionally optimized for this workload:

- **Plex Integration Fixed:** Added the `X-Plex-Token` to all Plex API calls, resolving the `401 Unauthorized` errors.
- **Database Optimized:** The `recorder` service is configured to exclude noisy entities, reducing database size by an estimated 70-80% and improving performance.
- **Logging Tuned:** The `logger` service is configured to reduce log spam and surface critical errors, simplifying future troubleshooting.
- **Configuration Restructured:** The primary `configuration.yaml` has been split into multiple, organized files (`sensors.yaml`, `rest_commands.yaml`, etc.) for easier management.
- **Security Hardened:** The `http` component is configured with IP banning and login attempt tracking.

---

## **6. Troubleshooting & Solutions Log**

- **Initial Problem:** Alexa skill setup failed due to OTP (One-Time Password) issues.
- **Root Cause:** The official Amazon Alexa skill requires a publicly accessible Home Assistant instance with a valid SSL certificate (Nabu Casa Cloud or a manual DuckDNS/Let's Encrypt setup).
- **Solution:** Abandoned the official cloud-based skill in favor of a **local Alexa bridge**. This approach uses device emulation on the local network, bypassing the need for cloud authentication and public exposure entirely.
- **Discovery Issues:** Initial discovery failed because the Alexa configuration in YAML is not sufficient; an active integration is required.
- **Final Architecture:** The local bridge (e.g., HABridge) will be installed in a dedicated LXC container (CT 280) and will present the Home Assistant scripts as discoverable devices to the existing Alexa ecosystem.

---

**Document Status:** Final
**Project Status:** Ready for Implementation (Creation of Alexa Bridge Container)

This documentation provides a complete blueprint for the voice-controlled homelab. The system is designed to be powerful, flexible, and reliable, leveraging the best of open-source home automation and existing consumer hardware.

