# Laptop Setup Guide

**Machine**: Intel Core i9-13900HX, 62.5 GB RAM, RTX 4080, Pop!_OS 22.04
The laptop is the **primary admin device** for the full stack.

## Admin Access

| Service | URL | Notes |
|---------|-----|-------|
| Proxmox Web UI | https://192.168.12.242:8006 | Accept self-signed cert warning |
| Prowlarr | http://192.168.12.210:9696 | |
| qBittorrent | http://192.168.12.212:8080 | |
| Sonarr | http://192.168.12.214:8989 | |
| Radarr | http://192.168.12.215:7878 | |
| Plex | http://192.168.12.230:32400/web | |
| Jellyfin | http://192.168.12.231:8096 | |
| FlareSolverr | http://192.168.12.102:8191 | |
| wg-easy | http://192.168.12.20:51821 | |
| Vaultwarden | https://192.168.12.20 | |

## SSH Access

```bash
# Proxmox host
ssh root@192.168.12.242

# Enter LXC containers
ssh root@192.168.12.242 "pct exec 100 -- sh"   # WireGuard server
ssh root@192.168.12.242 "pct exec 101 -- sh"   # WG client + TinyProxy
ssh root@192.168.12.242 "pct exec 212 -- bash" # qBittorrent
ssh root@192.168.12.242 "pct exec 214 -- bash" # Sonarr
ssh root@192.168.12.242 "pct exec 215 -- bash" # Radarr
ssh root@192.168.12.242 "pct exec 900 -- bash" # Ollama

# Ziggy (Raspberry Pi 3B+)
ssh ziggy
```

## Passwordless SSH Setup

Run once after Proxmox is installed and Pi is up:
```bash
chmod +x /opt/homelab-media-stack/scripts/setup-ssh-keys.sh
/opt/homelab-media-stack/scripts/setup-ssh-keys.sh
```

This pushes your key to every system and writes `~/.ssh/config` with aliases.
The config is already pre-applied on this laptop.

## SSH Aliases (after setup)

```bash
ssh tiamat        # Proxmox host (root@192.168.12.242)
ssh ziggy            # Ziggy (Raspberry Pi 3B+) (pi@192.168.12.20)
ssh ct-wg         # CT-100 WireGuard server
ssh ct-proxy      # CT-101 WG client + TinyProxy
ssh ct-qbit       # CT-212
ssh ct-sonarr     # CT-214
ssh ct-radarr     # CT-215
ssh ct-ollama     # CT-900
```

## Useful Commands

```bash
# Check all container status on Proxmox
ssh root@192.168.12.242 "pct list"

# Check media stack Docker containers
ssh root@192.168.12.242 "pct exec 212 -- docker ps"

# Check WireGuard tunnel status
ssh root@192.168.12.242 "pct exec 100 -- wg show"

# Verify qBittorrent egress through proxy
ssh root@192.168.12.242 "pct exec 212 -- curl -x http://192.168.12.101:8888 https://icanhazip.com"

# Check Ollama model catalog
ssh root@192.168.12.242 "pct exec 900 -- ollama list | head -20"

# Build + sideload TiamatsStack APK to all Fire TVs
cd /opt/homelab-media-stack/android-app && ./build-app.sh install-firetv
```

## Bitwarden (Vaultwarden)
1. Install **Bitwarden** browser extension
2. Settings → Self-hosted → Server URL: `https://192.168.12.20`
3. Create your account

## Remote Access (WireGuard)
1. Install WireGuard: `sudo nala install wireguard`
2. Open wg-easy at `http://192.168.12.20:51821`
3. Create client → download config
4. `sudo wg-quick up /path/to/config.conf`

## Game Streaming (Sunshine + Moonlight)
Sunshine is running on this laptop and exposes RTX 4080 GPU streaming.

| Item | Value |
|------|-------|
| Web UI | https://localhost:47990 |
| Service | `systemctl --user status sunshine-appimage` |
| Apps | Desktop, Switch (Ryubing), Steam Big Picture |

**Client setup** (phone / Fire TV / another PC):
1. Install **Moonlight** app
2. Add host: laptop LAN IP or Tailscale IP
3. Enter PIN from Sunshine web UI to pair
4. Launch desired app

**Service control**:
```bash
systemctl --user start|stop|restart sunshine-appimage
journalctl --user -u sunshine-appimage -f   # live logs
```
