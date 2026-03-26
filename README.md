# 🏠 Homelab Media Stack
Self-hosted media and automation stack on Proxmox (`192.168.12.242`) with per-service LXCs, WireGuard/TinyProxy kill-switch routing, and dedicated HDD storage.

## 🏗️ Current Architecture
```
Tiamat (Proxmox) - 192.168.12.242
├── Infrastructure
│   ├── CT-100 wireguard      192.168.12.100  WireGuard server
│   ├── CT-101 wg-proxy       192.168.12.101  WireGuard client + TinyProxy :8888
│   ├── CT-102 flaresolverr   192.168.12.102  FlareSolverr :8191
│   ├── CT-103 traefik        192.168.12.103  Traefik reverse proxy
│   ├── CT-104 vaultwarden    192.168.12.104  Vaultwarden :80
│   ├── CT-105 valkey         192.168.12.105  Valkey (Redis) :6379
│   ├── CT-106 postgresql     192.168.12.106  PostgreSQL :5432
│   └── CT-107 authentik      192.168.12.107  Authentik SSO :9000
├── Download Stack
│   ├── CT-210 prowlarr       192.168.12.210  :9696
│   ├── CT-212 qbittorrent    192.168.12.212  :8080 (VPN proxied)
│   ├── CT-214 sonarr         192.168.12.214  :8989
│   └── CT-215 radarr         192.168.12.215  :7878
├── Media Servers
│   ├── CT-230 plex           192.168.12.230  :32400
│   └── CT-231 jellyfin       192.168.12.231  :8096
├── Media Management
│   ├── CT-240 bazarr         DHCP            :6767
│   └── CT-242 jellyseerr     DHCP            :5055 (Jellyfin requests)
└── AI
    └── CT-900 ziggy          DHCP            Open WebUI :3000 + SearXNG :8081

Tiamat Desktop: Openbox + tint2 + Opera (via x11vnc :5900)
Traefik local DNS: *.tiamat.local → per-service routing
Backups: daily 3 AM via scripts/backup.sh → /mnt/hdd/backups

Laptop - 192.168.12.172
├── Ollama :11434 (RTX 4080 GPU, 41 models)
└── NFS shares → Tiamat

Ziggy Pi - 192.168.12.20
├── AdGuard Home (primary DNS)
├── wg-easy
└── Vaultwarden + Caddy
```

## 🔐 Download VPN Path
`qBittorrent/Prowlarr -> CT-101 TinyProxy :8888 -> WG tunnel -> CT-100 -> internet`

CT-101 runs `wireguard-tools` + `tinyproxy` (container name may still mention gluetun, but software is WG+TinyProxy).

## 💾 Storage
- 2TB HDD mounted at `/mnt/hdd`
- Downloads: `/mnt/hdd/torrents/*`
- Libraries: `/mnt/hdd/media/*`
- Backups: `/mnt/hdd/backups`

### Ollama (Laptop → CT-900)
- Laptop runs Ollama on RTX 4080 (12GB VRAM), bound to `0.0.0.0:11434`
- CT-900 runs Open WebUI (:3000) + SearXNG (:8081)
- Models stored on external drive, 41 models available

## 📱 Client Apps
- `android-app/` — TiamatsStack WebView app (mobile + Fire TV flavors)
- `clients/firetv.md` — Fire TV setup guide
- `clients/tablet.md` — Android phone/tablet setup guide

## 📚 Docs
- `docs/PLAN.md` — Full deployment plan & container reference
- `docs/NETWORKING.md` — LAN layout, VPN architecture, service URLs
- `docs/AI.md` — Ollama + Open WebUI setup
- `docs/NFS.md` — Laptop NFS shares
- `docs/HARDWARE.md` — Server, Pi, laptop specs
- `docs/PROXMOX-INSTALL.md`
- `docs/INDEXERS.md`
- `docs/BACKUPS.md`
- `docs/REAL-DEBRID.md`
