# ⚡ MediaStack Control

A lightweight, self-hosted master control panel for monitoring and managing your entire media stack of Docker containers from a single dashboard.

![Dark Theme Dashboard](https://img.shields.io/badge/theme-dark-0d1117) ![Python](https://img.shields.io/badge/python-3.11-blue) ![Docker](https://img.shields.io/badge/docker-ready-2496ED)

## Features

- **Real-time Monitoring** — Live status of all Docker containers with 5-second auto-refresh
- **Container Management** — Start, stop, restart any service from the browser
- **Log Viewer** — View container logs with configurable line count (50–5000)
- **System Resources** — CPU, memory, and disk usage at a glance
- **Smart Categorization** — Services auto-grouped by type (Arr Suite, Media Servers, Downloads, etc.)
- **Web UI Links** — One-click access to each service's web interface with smart port detection
- **Search & Filter** — Instantly find any service by name or category
- **Keyboard Shortcuts** — `/` to search, `Esc` to close modals
- **Dark Theme** — Modern GitHub-style dark interface
- **Zero External Dependencies** — Fully self-contained, no CDN or external requests

## Quick Start

### Docker Compose (Recommended)

```bash
git clone https://github.com/wlfogle/mediastack-control.git
cd mediastack-control
docker compose up -d
```

Then open **http://localhost:9900**

### Docker Run

```bash
docker build -t mediastack-control:latest .
docker run -d \
  --name mediastack-control \
  -p 9900:9900 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v /:/rootfs:ro \
  --pid host \
  --restart unless-stopped \
  mediastack-control:latest
```

### Standalone (no Docker)

```bash
pip install -r requirements.txt
python app.py
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `MSC_PORT` | `9900` | Web UI port |
| `MSC_DEBUG` | `false` | Flask debug mode |
| `DISK_PATH` | `/` | Path for disk usage stats (set to `/rootfs` when running in Docker) |

## API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/services` | All containers grouped by category with status |
| `POST` | `/api/services/:name/start` | Start a container |
| `POST` | `/api/services/:name/stop` | Stop a container |
| `POST` | `/api/services/:name/restart` | Restart a container |
| `GET` | `/api/services/:name/logs?lines=100` | Tail container logs |
| `GET` | `/api/system` | CPU, memory, disk stats |
| `GET` | `/api/health` | Health check |

## Supported Services (50+)

MediaStack Control auto-detects and categorizes popular self-hosted services:

- **Media Servers** — Plex, Jellyfin, Emby, TVHeadend
- **Arr Suite** — Sonarr, Radarr, Lidarr, Readarr, Bazarr, Prowlarr, Whisparr
- **Downloads** — qBittorrent, Deluge, Transmission, SABnzbd
- **Requests** — Overseerr, Jellyseerr, Ombi, Doplarr
- **Monitoring** — Tautulli, Grafana, Prometheus
- **Automation** — FlexGet, Autobrr, Recyclarr, Unpackerr, Kometa, Autoscan, and more
- **Media Management** — Audiobookshelf, Calibre-Web, Tdarr
- **Network** — Gluetun, WireGuard, WG-Easy, Tailscale
- **Infrastructure** — Home Assistant, Vaultwarden, Ollama, CrowdSec
- **Photos** — Immich (with DB and cache)

Any unrecognized containers appear in an **Other** category.

## HomeDockOS Integration

To install as a HomeDockOS app:

1. Build the image:
   ```bash
   docker build -t mediastack-control:latest ~/mediastack-control/
   ```
2. Copy the app-store YML:
   ```bash
   sudo cp homedockos/mediastack-control.yml /path/to/HomeDockOS/app-store/
   ```
3. Add the JSON entry to `AppStoreDefault.json`
4. Restart HomeDockOS

## License

MIT
