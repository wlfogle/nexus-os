# Proxmox Infrastructure Admin

A Tauri-based desktop application for managing Proxmox LXC containers and virtual machines with a focus on media stack administration.

## Overview

This application provides a beautiful, modern interface to manage your comprehensive Proxmox infrastructure including:

### Core Infrastructure (100-199)
- **Container 100**: WireGuard - VPN access and secure tunneling
- **Container 101**: Gluetun - VPN client container for other services
- **Container 102**: Flaresolverr - Cloudflare solver proxy
- **Container 103**: Traefik - Reverse proxy and load balancer
- **Container 104**: Vaultwarden - Password manager server
- **Container 105**: Valkey - Redis-compatible in-memory database
- **Container 106**: PostgreSQL - Primary database server
- **Container 107**: Authentik - Identity provider and SSO

### Essential Media Services (210-229)
- **Container 210**: Prowlarr - Indexer manager and proxy
- **Container 211**: Jackett - Torrent indexer proxy
- **Container 212**: QBittorrent - BitTorrent client
- **Container 214**: Sonarr - TV series management
- **Container 215**: Radarr - Movie management
- **Container 216**: Proxarr - Proxy management for *arr apps
- **Container 217**: Readarr - Book and audiobook management
- **Container 219**: Whisparr - Adult content management
- **Container 220**: Sonarr Extended - Extended TV series management
- **Container 221**: Radarr Extended - Extended movie management
- **Container 223**: Autobrr - Automated torrent management
- **Container 224**: Deluge - Alternative BitTorrent client

### Media Servers (230-239)
- **Container 230**: Plex - Media server and streaming platform
- **Container 231**: Jellyfin - Open-source media server
- **Container 232**: Audiobookshelf - Audiobook and podcast server
- **Container 233**: Calibre-web - E-book server and manager
- **Container 234**: IPTV-Proxy - IPTV streaming proxy
- **Container 235**: TVHeadend - TV streaming server
- **Container 236**: Tdarr Server - Media transcoding server
- **Container 237**: Tdarr Node - Media transcoding worker

### Enhancement Services (240-250)
- **Container 240**: Bazarr - Subtitle management
- **Container 241**: Overseerr - Media request management
- **Container 242**: Jellyseerr - Jellyfin request management
- **Container 243**: Ombi - Media request platform
- **Container 244**: Tautulli - Plex monitoring and statistics
- **Container 245**: Kometa - Plex metadata management
- **Container 246**: Gaps - Plex collection gap finder
- **Container 247**: Janitorr - Media cleanup automation
- **Container 248**: Decluttarr - Media library decluttering
- **Container 249**: Watchlistarr - Watchlist synchronization
- **Container 250**: Traktarr - Trakt.tv integration

### Monitoring & Analytics (260-269)
- **Container 260**: Prometheus - Metrics collection and monitoring
- **Container 261**: Grafana - Metrics visualization and dashboards
- **Container 262**: Checkrr - Service health checking

### Management & Utilities (270-279)
- **Container 270**: FileBot - File renaming and organization
- **Container 271**: FlexGet - Automated content downloading
- **Container 272**: Buildarr - Configuration management for *arr apps
- **Container 274**: Organizr - Service organization dashboard
- **Container 275**: Homarr - Modern dashboard for services
- **Container 276**: Homepage - Customizable homepage dashboard
- **Container 277**: Recyclarr - Configuration recycling for *arr apps
- **Container 278**: CrowdSec - Collaborative security engine
- **Container 279**: Tailscale - Secure networking mesh

### Virtual Machines
- **VM 500**: Home Assistant - Home automation platform
- **VM 611**: Alexa - Voice assistant system
- **VM 900**: AI System - Artificial intelligence services

## Features

- 🖥️ **Modern UI**: Beautiful, responsive interface with glassmorphism design
- 📊 **System Overview**: Real-time status of all containers and VMs
- 🎛️ **Container Management**: Start, stop, restart containers with one click
- 💻 **VM Management**: Full virtual machine lifecycle management
- 📱 **Categorized View**: Organized by service type for easy navigation
- 🔄 **Real-time Updates**: Automatic refresh and status monitoring
- 🎨 **Category Icons**: Visual organization with intuitive icons
- 📈 **Status Tracking**: Running/stopped status with visual indicators

## Requirements

- Proxmox VE server with LXC containers and VMs
- Rust toolchain (for building)
- Node.js (for Tauri frontend)
- Access to `pct` and `qm` commands on the target system

## Installation

### From Source

1. Clone the repository:
```bash
git clone https://github.com/wlfogle/proxmox-infrastructure-admin.git
cd proxmox-infrastructure-admin
```

2. Install dependencies:
```bash
cargo install tauri-cli
```

3. Build and run:
```bash
cargo tauri dev
```

4. Build for production:
```bash
cargo tauri build
```

## Usage

1. Launch the application
2. The app will automatically scan for configured containers and VMs
3. Use the sidebar to navigate between different service categories
4. Click action buttons to start/stop/restart services
5. Use the refresh button to update status information

## Configuration

The application automatically discovers containers and VMs based on their IDs. Make sure your Proxmox containers and VMs are configured with the expected IDs as listed above.

## Development

### Project Structure

```
├── src-tauri/          # Rust backend
│   ├── src/
│   │   └── main.rs     # Main Tauri application
│   ├── Cargo.toml      # Rust dependencies
│   └── tauri.conf.json # Tauri configuration
├── dist/               # Frontend files
│   └── index.html      # Main UI
└── README.md
```

### Adding New Services

To add new containers or VMs:

1. Update the service definitions in `src-tauri/src/main.rs`
2. Add appropriate category mapping
3. Update the frontend categories in `index.html`

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

MIT License - see LICENSE file for details

## Screenshots

The application features a modern glassmorphism design with:
- Dark gradient background
- Translucent panels with blur effects
- Categorized service organization
- Real-time status indicators
- Intuitive action buttons

## Support

For issues and feature requests, please use the GitHub issues tracker.

## Roadmap

- [ ] Resource usage monitoring (CPU, Memory)
- [ ] Log viewing capabilities
- [ ] Backup management
- [ ] Scheduled operations
- [ ] Mobile companion app
- [ ] Configuration management
- [ ] Alert system
- [ ] Performance metrics dashboard
