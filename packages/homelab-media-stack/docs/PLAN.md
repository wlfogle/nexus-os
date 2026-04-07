# Deployment Plan
## Architecture Summary
```
Tiamat — Proxmox VE 9.x (192.168.12.242)

Infrastructure (static IPs 192.168.12.100–107)
├── CT-100 wireguard      192.168.12.100   WireGuard VPN server, subnet 10.0.0.1/24
├── CT-101 wg-proxy       192.168.12.101   WireGuard client + TinyProxy :8888 (kill-switch)
├── CT-102 flaresolverr   192.168.12.102   FlareSolverr :8191 (Cloudflare bypass)
├── CT-103 traefik        192.168.12.103   Traefik reverse proxy :80/:443/:8080
├── CT-104 vaultwarden    192.168.12.104   Vaultwarden :443 (Caddy TLS) / :8080 (backend)
├── CT-105 valkey         192.168.12.105   Valkey (Redis) :6379
├── CT-106 postgresql     192.168.12.106   PostgreSQL :5432
└── CT-107 authentik      192.168.12.107   Authentik SSO :9000

Download stack (static IPs 192.168.12.210–224)
├── CT-210 prowlarr       192.168.12.210   Prowlarr :9696  ← proxied via CT-101
├── CT-211 jackett        192.168.12.211   Jackett :9117 (legacy fallback)
├── CT-212 qbittorrent    192.168.12.212   qBittorrent :8080  ← proxied via CT-101
├── CT-213 rdt-client     192.168.12.213   rdt-client :6500   ← Real-Debrid download client
├── CT-214 sonarr         192.168.12.214   Sonarr :8989
├── CT-215 radarr         192.168.12.225   Radarr :7878  ← moved from .215 (HDHR conflict)
├── CT-216 proxarr        192.168.12.216   Proxarr (proxy routing for *arr)
├── CT-217 readarr        192.168.12.217   Readarr :8787
├── CT-219 whisparr       192.168.12.219   Whisparr :6969
├── CT-220 sonarr-ext     192.168.12.220   Sonarr Extended :8989
├── CT-223 autobrr        192.168.12.223   Autobrr :7474
└── CT-224 deluge         192.168.12.224   Deluge :8112

Media servers
├── CT-230 plex           192.168.12.230   Plex Media Server :32400
├── CT-231 jellyfin       192.168.12.231   Jellyfin :8096
├── CT-232 audiobookshelf DHCP             Audiobookshelf :13378
├── CT-233 calibre-web    DHCP             Calibre-Web :8083
├── CT-234 iptv-proxy     DHCP             IPTV Proxy
├── CT-235 tvheadend      DHCP             TVHeadend :9981
├── CT-236 tdarr-server   DHCP             Tdarr Server :8265
└── CT-237 tdarr-node     DHCP             Tdarr Node (transcode worker)

Media management (all DHCP — accessed via Traefik CT-103)
├── CT-240 bazarr | CT-241 overseerr | CT-242 jellyseerr | CT-243 ombi
├── CT-244 tautulli | CT-245 kometa | CT-246 gaps | CT-247 janitorr
└── CT-248 decluttarr | CT-249 watchlistarr | CT-250 traktarr

Monitoring (DHCP): CT-260 prometheus | CT-261 grafana | CT-262 checkrr
Tools     (DHCP): CT-270 filebot | CT-271 flexget | CT-272 buildarr | CT-274 organizr
                  CT-275 homarr | CT-276 homepage | CT-277 recyclarr
                  CT-278 crowdsec | CT-279 tailscale

VMs / Special
├── VM-200 alexa-media-bridge  192.168.12.200   Ubuntu 4GB — Alexa media bridge
├── VM-500 haos                192.168.12.250   Home Assistant OS (smart home + Alexa integration)
├── VM-901 windows-gaming      192.168.12.201   Windows 11 26H1, RX 580 GPU passthrough (VFIO pre-configured)
│                                               sdb (240GB SSD) disk passthrough, 300GB LVM OS disk
│                                               PlayOn Home → saves to /mnt/hdd/media/playon
│                                               ⚠ RAM constrained until Tiamat upgrade — stop CTs before gaming
├── CT-900 ziggy               DHCP             Open WebUI :3000, SearXNG :8081 — AI frontend
│                                               Ollama served by laptop RTX 4080 at 192.168.12.172:11434
└── CT-950 agent-comms         DHCP             Agent communication broker

Laptop — GPU inference server (192.168.12.172)
├── Ollama :11434 (RTX 4080, 12GB VRAM)   LLM inference for entire stack
└── Tdarr node (Phase 6)                  Batch NVENC transcoding worker

Bahamut (Raspberry Pi 4B 2GB) — 192.168.12.244
├── AdGuard Home :53/:3000    Network-wide DNS + ad-blocking (sole instance)
├── wg-easy :51821/:51820     Remote client VPN access
└── Vaultwarden :443          via Caddy

VPN kill-switch flow:
  qBittorrent/Prowlarr → TinyProxy :8888 (CT-101) → WireGuard tunnel → CT-100 → Internet
  Tunnel drops → proxy unreachable → downloads halt — real IP never exposed

Storage — 2TB WD HDD at /mnt/hdd (wiped, previously Windows Steam library)
├── /mnt/hdd/torrents/movies  )  qBit (CT-212) + Deluge (CT-224) download dirs
├── /mnt/hdd/torrents/tv      )  *arr services hard-link completed files into
├── /mnt/hdd/torrents/music   )  /mnt/hdd/media/ — same filesystem = instant,
├── /mnt/hdd/torrents/books   )  zero-copy import
├── /mnt/hdd/media/movies        Radarr managed library → Plex / Jellyfin
├── /mnt/hdd/media/tv            Sonarr managed library → Plex / Jellyfin
├── /mnt/hdd/media/music         Music library
├── /mnt/hdd/media/books         Readarr / Calibre-Web library
├── /mnt/hdd/media/playon        PlayOn Home recordings (VM-901) → Plex / Jellyfin
└── /mnt/hdd/backups             vzdump container snapshots + Restic app-data backups

Laptop NFS shares (192.168.12.172) → mounted on Tiamat at /mnt/laptop/
├── /mnt/laptop/calibre          Calibre Library (16GB) → CT-233 Calibre-Web
├── /mnt/laptop/cookbooks        Cookbooks (2.4GB) → CT-233 second library
├── /mnt/laptop/videos           Personal Videos (29GB) → Plex / Jellyfin Home Videos
├── /mnt/laptop/isos             ISOs (195GB) → Proxmox ISO uploads
└── /mnt/laptop/roms             ROMs (89GB, Switch NSPs) → future emulation frontend

IPTV — lou.m3u at /media/loufogle/Data/Downloads/lou.m3u on laptop
└── Served to Jellyfin Live TV or TVHeadend (CT-235) via NFS mount (see docs/NFS.md)

Backups — Restic (daily cron on Proxmox host)
├── Source:      /opt/appdata (all container configs)
├── Destination: /mnt/hdd/backups/restic
└── Retention:   7 daily, 4 weekly, 3 monthly  (see docs/BACKUPS.md)

Bind-mount /mnt/hdd → /data inside each *arr + download container

Client Devices
├── Fire TV  — Jellyfin/Plex + Silk Browser (Overseerr/Homarr)
├── Android  — Jellyfin, Plex, nzb360, Bitwarden, WireGuard
└── Laptop   — Full admin (Proxmox, Homarr, Homepage, SSH)
```
## Container Reference
### Infrastructure (100–107)
| CT | Hostname | IP | OS | RAM | Software |
|----|----------|----|----|-----|----------|
| CT-100 | wireguard | 192.168.12.100 | Alpine | 512MB | wireguard-tools — server, 10.0.0.1/24, :51820/UDP |
| CT-101 | wg-proxy | 192.168.12.101 | Alpine | 512MB | wireguard-tools (client) + TinyProxy :8888 |
| CT-102 | flaresolverr | 192.168.12.102 | Debian | 1GB | FlareSolverr :8191 |
| CT-103 | traefik | 192.168.12.103 | Alpine | 512MB | Traefik v2 :80/:443/:8080 |
| CT-104 | vaultwarden | 192.168.12.104 | Debian | 1GB | Vaultwarden :8080 + Caddy :443 (internal TLS) |
| CT-105 | valkey | 192.168.12.105 | Alpine | 1GB | Valkey (Redis fork) :6379 |
| CT-106 | postgresql | 192.168.12.106 | Alpine | 1GB | PostgreSQL :5432 |
| CT-107 | authentik | 192.168.12.107 | Ubuntu | 1GB | Authentik SSO :9000 |
### Download Stack (210–224)
| CT | Hostname | IP | RAM | Port |
|----|----------|----|-----|------|
| CT-210 | prowlarr | 192.168.12.210 | 1GB | 9696 |
| CT-211 | jackett | 192.168.12.211 | 512MB | 9117 |
| CT-212 | qbittorrent | 192.168.12.212 | 2GB | 8080 |
| CT-213 | rdt-client | 192.168.12.213 | 1GB | 6500 |
| CT-214 | sonarr | 192.168.12.214 | 1GB | 8989 |
| CT-215 | radarr | 192.168.12.225 | 1GB | 7878 |
| CT-216 | proxarr | 192.168.12.216 | 512MB | — |
| CT-217 | readarr | 192.168.12.217 | 512MB | 8787 |
| CT-219 | whisparr | 192.168.12.219 | 512MB | 6969 |
| CT-220 | sonarr-extended | 192.168.12.220 | 1GB | 8989 |
| CT-223 | autobrr | 192.168.12.223 | 1GB | 7474 |
| CT-224 | deluge | 192.168.12.224 | 2GB | 8112 |
### Media Servers (230–237)
| CT | Hostname | IP | RAM | Port |
|----|----------|----|-----|------|
| CT-230 | plex | 192.168.12.230 | 8GB | 32400 |
| CT-231 | jellyfin | 192.168.12.231 | 4GB | 8096 |
| CT-232 | audiobookshelf | 192.168.12.232 | 2GB | 13378 |
| CT-233 | calibre-web | 192.168.12.233 | 2GB | 8083 |
| CT-234 | iptv-proxy | DHCP | 512MB | — |
| CT-235 | tvheadend | DHCP | 2GB | 9981 |
| CT-236 | tdarr-server | DHCP | 4GB | 8265 |
| CT-237 | tdarr-node | DHCP | 2GB | — |
### Media Management (240–250, all DHCP)
| CT | Hostname | Software | Port |
|----|----------|----------|------|
| CT-240 | bazarr | Bazarr (subtitles) | 6767 |
| CT-241 | overseerr | Overseerr (Plex requests) | 5055 |
| CT-242 | jellyseerr | Jellyseerr (Jellyfin requests) | 5055 |
| CT-243 | ombi | Ombi (requests) | 3579 |
| CT-244 | tautulli | Tautulli (Plex analytics) | 8181 |
| CT-245 | kometa | Kometa / Plex Meta Manager | — |
| CT-246 | gaps | Gaps (collection finder) | — |
| CT-247 | janitorr | Janitorr (media cleanup) | — |
| CT-248 | decluttarr | Decluttarr (queue cleanup) | — |
| CT-249 | watchlistarr | Watchlistarr | — |
| CT-250 | traktarr | Traktarr (Trakt.tv sync) | — |
### Monitoring & Tools (260–279, all DHCP)
| CT | Hostname | Software |
|----|----------|----------|
| CT-260 | prometheus | Prometheus :9090 |
| CT-261 | grafana | Grafana :3000 |
| CT-262 | checkrr | Checkrr |
| CT-274 | organizr | Organizr :80 |
| CT-275 | homarr | Homarr :7575 |
| CT-276 | homepage | Homepage :3000 |
| CT-277 | recyclarr | Recyclarr (*arr quality sync) |
| CT-278 | crowdsec | CrowdSec (IDS/IPS) |
| CT-279 | tailscale | Tailscale mesh VPN |
## Current Deployment Status
> Last updated: 2026-03-27 — reflects live state of Tiamat

### Running containers (27 total)
All containers below are running and passing HTTP health checks.

| CT | Service | Status | Notes |
|----|---------|--------|-------|
| CT-100 | wireguard | ✅ running | |
| CT-101 | wg-proxy | ✅ running | |
| CT-102 | flaresolverr | ✅ running | Docker v3.4.6 (switched from native) |
| CT-103 | traefik | ✅ running | 13 routes configured via file provider |
| CT-104 | vaultwarden | ✅ running | |
| CT-105 | valkey | ✅ running | |
| CT-106 | postgresql | ✅ running | |
| CT-107 | authentik | ✅ running | |
| CT-210 | prowlarr | ✅ running | Sonarr + Radarr + Readarr + Lidarr synced |
| CT-212 | qbittorrent | ✅ running | categories: sonarr/radarr/readarr/lidarr |
| CT-214 | sonarr | ✅ running | HTTP 200 |
| CT-215 | radarr | ✅ running | HTTP 200 |
| CT-217 | readarr | ✅ running | :8787 — root folder + qBit + Prowlarr configured |
| CT-218 | lidarr | ✅ running | :8686 — root folder + qBit + Prowlarr configured |
| CT-230 | plex | ✅ running | HTTP 401 (auth required, normal) |
| CT-231 | jellyfin | ✅ running | jellyfin/jellyfin |
| CT-232 | audiobookshelf | ✅ running | :13378, /audiobooks bound |
| CT-233 | calibre-web | ✅ running | calibre/calibre — DB path pending laptop NFS (set /calibre at /admin/dbconfig) |
| CT-240 | bazarr | ✅ running | HTTP 200 |
| CT-242 | jellyseerr | ✅ running | configured: Jellyfin + Sonarr + Radarr, library scan triggered |
| CT-244 | tautulli | ✅ running | :8181 connected=True (Plex 192.168.12.230:32400) |
| CT-245 | kometa | ⚠ needs config | restart-loops until config.yml added — see below |
| CT-275 | homarr | ✅ running | :7575 — ready for dashboard configuration |
| CT-276 | homepage | ✅ running | :3000 — ready for dashboard configuration |
| CT-277 | recyclarr | ✅ running | needs recyclarr.yml with Sonarr/Radarr keys |
| CT-900 | ziggy | ✅ running | Open WebUI + SearXNG |

### Kometa (CT-245) — needs config.yml
Container is deployed but will restart-loop until configured.
Create `/opt/appdata/kometa/config.yml` on CT-245 with Plex URL and token:
```
pct exec 245 -- bash -c 'mkdir -p /opt/appdata/kometa'
# Copy/write your config.yml to /opt/appdata/kometa/config.yml
# Then: pct exec 245 -- docker restart kometa
```
See https://kometa.wiki/en/latest/config/configuration/ for full config reference.

### Recyclarr (CT-277) — needs recyclarr.yml
Sync won't run until `/opt/appdata/recyclarr/recyclarr.yml` is configured.
```
pct exec 277 -- docker exec recyclarr recyclarr config create
# Edit the generated config, then: pct exec 277 -- docker restart recyclarr
```

### Traefik (CT-103) — routes live
All `*.tiamat.local` routes configured and hot-loaded. See `docs/NETWORKING.md` for full table.
To add a new route: drop a YAML file in `/etc/traefik/dynamic/` on CT-103 — no restart needed.

### Jellyseerr (CT-242) — first-run pending
Docker container deployed (`fallenbagel/jellyseerr:2.7.3`), port 5055 responding.
Complete setup at `http://192.168.12.151:5055` (or `http://jellyseerr.tiamat.local`):
- Connect to Jellyfin: `http://192.168.12.231:8096`
- Connect to Sonarr: `http://192.168.12.214:8989`
- Connect to Radarr: `http://192.168.12.215:7878`

### Stale ARP issue — router
Several static-IP CTs experience stale ARP entries causing connectivity failures until the host ARP cache is flushed.
Symptom: `HTTP:000` from host but CT-to-CT works fine. Fix: `arp -d <IP>` on the Proxmox host.
New static-IP CTs should get internet via DHCP first (so the router registers the MAC), then switch to static.
See `/etc/pve/lxc/<ctid>.conf` — change `ip=dhcp` ↔ `ip=192.168.12.X/24` as needed.

### NFS laptop mounts — pending
All 5 laptop NFS shares are configured in `/etc/fstab` using `192.168.12.205` (laptop Ethernet).
Mounts fail when laptop is off (expected — `soft,_netdev` prevents boot hang).
WiFi (.172) is blocked by AP isolation — always use Ethernet (.205).
When laptop comes online: `mount -a` or `for mp in calibre cookbooks videos isos roms; do mount /mnt/laptop/$mp; done`
CT-233 calibre-web needs `mp1` added to `/etc/pve/lxc/233.conf` once calibre mounts:
```
echo 'mp1: /mnt/laptop/calibre,mp=/calibre' >> /etc/pve/lxc/233.conf
echo 'mp2: /mnt/laptop/cookbooks,mp=/cookbooks' >> /etc/pve/lxc/233.conf
pct restart 233
```

### Backup cron — active
Vzdump of all 27 CTs runs daily at 03:00 → `/mnt/hdd/backups/`.
See `docs/BACKUPS.md` for full scope and restore instructions.

### Tiamat desktop (Openbox autologin)
LightDM configured for root autologin → Openbox session on `:0`.
Opera launches on session start with 5 service tabs (Traefik dashboard, Jellyfin, Sonarr, Radarr, Prowlarr).
Autostart: `~/.config/openbox/autostart`

### IP conflict note — .231
MAC `00:11:d9:b8:80:a7` (TiVo OUI) was previously flagged squatting on `.231`.
Device is currently offline. CT-231 properly owns `.231`.
If TiVo comes back online and conflicts, add a DHCP reservation on the router to push it elsewhere.

## Deployment Order
### Phase 1 — Proxmox Host
1. Boot Tiamat from USB → Proxmox VE 9.0 ISO
2. Install: target 240GB SSD, hostname `tiamat.local`, IP `192.168.12.242`
3. SSH in: `ssh root@192.168.12.242`
4. Clone repo: `git clone https://github.com/wlfogle/homelab-media-stack.git /opt/homelab-media-stack`
5. Run: `bash /opt/homelab-media-stack/scripts/setup-proxmox.sh`
6. Prepare 2TB HDD (see PROXMOX-INSTALL.md §3.3 — wipes existing Steam library partition)
### Phase 2 — Infrastructure
7. Create CT-100, run `infrastructure/wireguard-server/setup-wg-server.sh`
8. Create CT-101, copy wg client config from CT-100, run `infrastructure/wireguard-server/setup-gluetun-client.sh`
   - CT-101 becomes the WireGuard client + TinyProxy kill-switch proxy
9. Create CT-102 (FlareSolverr), CT-103 (Traefik)
10. Create CT-104 (Vaultwarden), CT-105 (Valkey), CT-106 (PostgreSQL), CT-107 (Authentik)
### Phase 3 — Download Stack
11. Create CT-210–CT-224
12. Configure qBittorrent (CT-212) + Prowlarr (CT-210): proxy → `192.168.12.101:8888`
13. Configure Prowlarr: FlareSolverr → `http://192.168.12.102:8191`
14. Bind-mount `/mnt/hdd` → `/data` in all download + *arr containers
15. Create CT-213 rdt-client, add Real-Debrid API key, wire as download client in Sonarr/Radarr
    - rdt-client downloads from Real-Debrid cloud cache — no VPN needed for public indexers
    - qBit stays for private trackers / anything not cached on Real-Debrid
    - See docs/REAL-DEBRID.md for setup
### Phase 4 — Media Servers
15. Create CT-230 (Plex), CT-231 (Jellyfin)
16. Bind-mount `/mnt/hdd/media` → `/data/media` for Plex and Jellyfin
17. Get Plex claim token at https://plex.tv/claim — expires in 4 min, do immediately before first start
### Phase 5 — Media Management
18. Create CT-240–CT-250
19. Connect Overseerr/Jellyseerr to Plex/Jellyfin + Sonarr/Radarr
20. Configure Bazarr subtitle providers
### Phase 6 — Monitoring & Tools
21. Create CT-260–CT-279
22. Configure Homarr/Homepage with all service links
23. Set up Recyclarr quality profiles sync
### Phase 6.5 — AI Services (Laptop + CT-900)
24. On laptop: install Ollama, bind to `0.0.0.0:11434`, enable systemd service
    - Verify RTX 4080 is used: `ollama run llama3` and check `nvidia-smi`
25. In CT-900: deploy Open WebUI → `OLLAMA_BASE_URL=http://192.168.12.172:11434`
26. In CT-900: deploy SearXNG → configure as web search backend in Open WebUI
    - See docs/AI.md for full setup

### Phase 7 — Bahamut (Raspberry Pi 4B 2GB)
24. DietPi already installed at 192.168.12.244 (headless, Ethernet, Dropbear SSH)
25. Run `bash pi/setup-pi.sh`
26. Configure wg-easy: set `WG_HOST` to public IP or DuckDNS hostname
27. Forward port 51820/UDP on router to 192.168.12.244
28. Set router DNS 1 = `192.168.12.244` (AdGuard Home), DNS 2 = `1.1.1.1`
### Phase 8 — Security
29. Configure CrowdSec (CT-278) with Traefik bouncer
30. Set up Authentik SSO for externally-exposed services
31. Enable Tailscale mesh (CT-279)

### Phase 9 — Windows Gaming VM (VM-901)
32. Recreate VM-901 .conf (disks already exist, VFIO pre-configured):
    - GPU: 09:00.0 + 09:00.1 (RX 580, already vfio-pci)
    - Disk 0: local-lvm:vm-901-disk-1 (300GB, fresh Win11 install)
    - Disk 1: /dev/sdb passthrough (240GB SSD, games storage)
    - ISOs: 28000.1_MULTI_X64_EN-US.ISO + virtio-win.iso already on Tiamat
    - TPM 2.0 + OVMF (UEFI) — existing efivars + tpmstate volumes
    - See proxmox/vm-windows-playon.md for full config
33. Install PlayOn Home → set save path to mapped share → /mnt/hdd/media/playon
    ⚠ RAM: stop non-essential CTs before gaming until RAM upgrade (8GB total on Tiamat)
    Priority RAM upgrade: 2×32GB DDR4-3200 (B450M DS3H, 3 slots free)

### Phase 10 — Home Assistant
34. Create VM-500 via community-scripts HAOS installer
35. Configure HA integrations: Plex, Jellyfin, AdGuard, Alexa, WireGuard
36. Set up VM-200 Alexa media bridge for voice control
    - See awesome-stack repo homeassistant-configs/
## VPN Kill Switch Verification
```bash
# Confirm traffic exits through CT-100 VPN, not home IP
pct exec 212 -- curl -x http://192.168.12.101:8888 https://icanhazip.com

# Stop WireGuard on CT-100 — downloads should halt
pct exec 100 -- wg-quick down wg0
pct exec 212 -- curl -x http://192.168.12.101:8888 https://icanhazip.com --max-time 5
# Should timeout — kill-switch confirmed

# Restore
pct exec 100 -- wg-quick up wg0
```
## AdGuard Home Blocklists (Ziggy)
- AdGuard DNS filter: `https://adguardteam.github.io/AdGuardSDNSFilter/Filters/filter.txt`
- EasyList: `https://easylist.to/easylist/easylist.txt`
- EasyPrivacy: `https://easylist.to/easylist/easyprivacy.txt`
- Steven Black Hosts: `https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts`
