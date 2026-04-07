# Optimizations — Tiamat Media Stack

## Hardware (Current)
- **CPU**: AMD Ryzen 5 3600, 6c/12t
- **RAM**: 32GB DDR4-3200 (upgraded from 8GB)
- **Storage**: 240GB SSD (OS) + 2TB HDD (`/mnt/hdd` — media + downloads)
- **GPU**: RX 580 4GB (VFIO passthrough to VM-901)
- **Network**: Gigabit Ethernet

## Container RAM Allocation
With 32GB, recommended allocation:

| Container | Current | Recommended | Notes |
|-----------|---------|-------------|-------|
| CT-230 Plex | 8GB | 8GB | Transcoding benefits from RAM |
| CT-231 Jellyfin | 4GB | 4GB | Hardware transcode not available (GPU passed through) |
| CT-212 qBittorrent | 2GB | 2GB | Sufficient for heavy downloading |
| CT-214 Sonarr | 1GB | 2GB | Larger libraries benefit |
| CT-215 Radarr | 1GB | 2GB | Larger libraries benefit |
| CT-210 Prowlarr | 1GB | 1GB | Lightweight |
| CT-102 FlareSolverr | 1GB | 2GB | Chrome headless needs RAM for Cloudflare solving |
| VM-901 Windows | 8GB | 12GB | Gaming VM — can now run alongside stack |
| Infrastructure (100-107) | ~4GB total | ~4GB | WG, Traefik, Authentik, etc. |

## Storage Layout
Hard-link friendly — downloads and media on same filesystem (`/mnt/hdd`):
```
/mnt/hdd/torrents/movies  →  /mnt/hdd/media/movies   (hard-link on import)
/mnt/hdd/torrents/tv      →  /mnt/hdd/media/tv       (hard-link on import)
/mnt/hdd/torrents/music   →  /mnt/hdd/media/music
/mnt/hdd/torrents/books   →  /mnt/hdd/media/books
```
All containers bind-mount `/mnt/hdd` → `/data` so paths are consistent.

## Transcoding
- **Plex**: CPU-only transcode (RX 580 passed to VM-901). Ryzen 5 3600 handles 2-3 1080p streams.
- **Jellyfin**: Same — CPU transcode. Prefer direct play on LAN clients.
- **Tdarr** (planned CT-236/237): Batch transcode to H.265 to save HDD space. Laptop RTX 4080 can NVENC transcode via Tdarr node.

## Network
- All CTs on bridged `vmbr0` → LAN `192.168.12.x`
- Static IPs for infrastructure (100-107) and download stack (210-224)
- DHCP for management CTs (240+) — routed via Traefik (`*.tiamat.local`)
- VPN kill-switch: qBit/Prowlarr → TinyProxy (CT-101:8888) → WG tunnel (CT-100)

## Now Possible with 32GB
- Run VM-901 (Windows gaming) alongside full media stack
- Deploy FlareSolverr Cloudflare indexers (1337x, EZTV, TorrentGalaxy) — Chrome needs ~2GB
- Deploy Readarr (CT-217), Audiobookshelf (CT-232), Calibre-Web (CT-233)
- Run Prometheus/Grafana monitoring (CT-260/261)
- Tdarr transcoding server (CT-236)
