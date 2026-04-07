# 🚀 Awesome Stack - Complete Self-Hosting Infrastructure

A comprehensive, production-ready self-hosting ecosystem featuring media management, home automation, development tools, and infrastructure automation. This represents a complete modern self-hosting solution with 100+ services, scripts, and applications.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker](https://img.shields.io/badge/Docker-Compose-blue.svg)](https://docs.docker.com/compose/)
[![Self-Hosted](https://img.shields.io/badge/Self--Hosted-Complete-green.svg)](https://awesome-selfhosted.net/)

## 🌟 **What is Awesome Stack?**

This repository contains a **complete self-hosting infrastructure** that includes:

- **🎬 Media Stack**: Plex, Jellyfin, Sonarr, Radarr, qBittorrent, and more
- **🏠 Home Automation**: Home Assistant integration with Alexa
- **🔧 Development Tools**: AI coding assistants, documentation processors
- **📊 Monitoring**: Comprehensive system and service monitoring  
- **🛡️ Security**: VPN, SSL certificates, secure access
- **⚙️ Infrastructure**: Proxmox VMs, Docker orchestration, automation scripts

## 📁 **Repository Structure**

### 🎬 **Media & Entertainment**
```
proxmox-infrastructure/   # Proxmox VM/LXC configurations
├── vm-500/              # Home Assistant OS VM
├── lxc-100-279/         # Media stack LXC containers
├── ct - 200/              # Alexa/HA Bridge (Alexa media-bridge)
└── ct-900/ (Ziggy)             # AI services container
```

### 🏠 **Home Automation**
```
homeassistant-configs/    # Home Assistant configurations
├── configuration.yaml   # Main HA config
├── scripts.yaml         # Automation scripts
├── sensors.yaml         # Custom sensors
└── alexa integration    # Voice control setup
```

### 💻 **Development & Coding**
```
Coding/                  # Complete development environment
├── python-projects/     # Python applications
├── cpp-projects/       # C++ applications with Qt
├── rust-projects/      # Rust applications
├── web-projects/       # Tauri/React applications
└── scripts/            # Automation scripts
```

### 🔧 **Infrastructure & Tools**
```
scripts/                # Production automation scripts
├── media-services/     # Media stack management
├── homeassistant/     # HA troubleshooting
├── proxmox/           # VM management
└── monitoring/        # System health checks
```

### 🥷 **Ghost Mode - Ultimate Online Invisibility**
```
ghost-mode/             # Complete digital anonymity suite
├── README.md           # User guide and setup
├── scripts/           # All anonymity tools
│   ├── ghost-mode      # Main controller
│   ├── ghost-toggle    # Simple on/off toggle
│   ├── ghost-tray-widget # System tray interface
│   ├── ghost-browser   # Anonymous browser launcher
│   └── protection modules # DNS, hardware, time spoofing
├── ARCHITECTURE.md    # Technical documentation
├── API.md            # Developer reference
└── install-ghost-mode.sh # One-click installer
```

**🎯 One-Click Complete Invisibility:**
- 🌐 **Browser fingerprinting blocked** (WebRTC, Canvas, WebGL, Audio)
- 🔧 **Hardware fingerprinting spoofed** (CPU, GPU, RAM, System info)
- 🕐 **Time fingerprinting masked** (Timezone, timing attacks)
- 📡 **Network anonymization** (IPv6 disabled, DNS leak prevention)
- 👁️ **Continuous monitoring** with auto-repair and visual feedback
- 🎛️ **System tray widget** with status indicators (🟢🟡🔴)

### 📚 **Documentation & Guides**
```
docs/                   # Comprehensive documentation
├── _organized/         # Structured guides
├── fixes/             # Solution database
└── Visual-Setup-Guide.md # Step-by-step setup
```

## 🚀 **Quick Start**

### Traditional Installation (Arch/Garuda Linux)

#### Prerequisites
```bash
# Docker and Docker Compose
sudo pacman -S docker docker-compose

# For Proxmox integration
sudo pacman -S qemu-utils libvirt

# Enable services
sudo systemctl enable --now docker
sudo usermod -aG docker $USER
```

#### Deploy Media Stack
```bash
# Clone the repository
git clone https://github.com/wlfogle/awesome-stack.git
cd awesome-stack

# Deploy media stack in Proxmox LXC containers
# Access Proxmox host and deploy to containers 100-279
ssh proxmox
pct enter 104  # Example: Vaultwarden container
docker-compose up -d

# Deploy Home Assistant to VM-500
# Copy configs to HAOS VM-500
```

#### Run Infrastructure Scripts
```bash
# Fix all container issues
sudo ./scripts/fix-all-containers.sh

# Optimize system performance
sudo ./scripts/hardware_optimization.sh

# Setup Alexa integration
./scripts/setup_alexa_bridge.sh
```

### 🪨 Fedora Kinoite Installation

Awesome Stack now supports immutable Fedora Kinoite! This native containerized approach provides better security, reliability, and atomic updates.

#### Prerequisites
```bash
# Install required packages (one-time operation)
sudo rpm-ostree install podman podman-compose qemu-img libvirt git

# Reboot to apply changes
sudo systemctl reboot
```

#### One-Click Setup
```bash
# Clone the repository
git clone https://github.com/wlfogle/awesome-stack.git
cd awesome-stack

# Run the Kinoite setup script
./setup-kinoite-awesome-stack.sh

# Extract services from existing Proxmox VM (if migrating)
./extract-proxmox-services.sh
```

#### Managing Your Kinoite Stack
```bash
# Start all services
awesome-stack start

# Check service status
awesome-stack status

# View available web interfaces
awesome-stack urls

# Update container images
awesome-stack update
```

#### Key Features of Kinoite Implementation
- **Immutable OS**: System files protected from changes
- **Atomic Updates**: Reliable system updates with rollback
- **Podman Containers**: Native containers without Docker daemon
- **Flatpak Integration**: GUI applications as sandboxed Flatpaks
- **User Services**: No system service modifications required
- **Persistent Storage**: All data survives OS updates

## 🎯 **Key Features**

### 🎬 **Complete Media Ecosystem**
- **Plex & Jellyfin**: Dual media servers with 4K support
- **Acquisition**: Sonarr, Radarr, Lidarr, Readarr automation
- **Download**: qBittorrent with VPN integration (Gluetun)
- **Management**: Custom dashboards and monitoring
- **AI Enhancement**: Automated artwork and recommendations

### 🏠 **Smart Home Integration**
- **Home Assistant**: Complete automation platform
- **Alexa Integration**: Voice control for all services
- **Custom Sensors**: System monitoring and alerts
- **Mobile Apps**: Remote access and control
- **Secure Access**: SSL certificates and VPN

### 💻 **Development Suite**
- **AI Coding Assistant**: Tauri-based development tool
- **Multiple Languages**: Python, C++, Rust, TypeScript projects
- **Documentation Tools**: AI-powered doc processing
- **Project Templates**: Ready-to-use project structures
- **Development VMs**: Proxmox-based dev environments

### 🔧 **Infrastructure Automation**
- **Container Management**: Health monitoring and auto-repair
- **System Optimization**: Performance tuning scripts
- **Backup Solutions**: Automated backups and snapshots
- **Network Management**: VPN, DNS, and routing
- **Security Hardening**: SSL, firewalls, and monitoring

## 📊 **System Architecture**

```
┌─────────────────────────────────────────────────────────────┐
│                    Awesome Stack Architecture               │
├─────────────────────────────────────────────────────────────┤
│                      Garuda Linux Host                     │
├─────────────────────────────────────────────────────────────┤
│                    Proxmox Virtualization                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │   VM-500    │ │   VM-612    │ │    LXC 100-279     │   │
│  │  HAOS/HA    │ │ BlissOS/    │ │  Media Stack       │   │
│  │             │ │ Alexa       │ │  + CT-900 (AI)     │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    Container Services                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │ Plex/Jellyfin│ │Sonarr/Radarr│ │ Monitoring/Utils   │   │
│  │ CT-108/109  │ │ CT-110-130  │ │   CT-140-279       │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                     Hardware Layer                         │
│        Intel i9-13900HX │ 64GB RAM │ RTX 4080              │
└─────────────────────────────────────────────────────────────┘
```

## 🛠️ **Service Inventory**

### Media Services (15+)
- Plex Media Server
- Jellyfin
- Sonarr (TV shows)
- Radarr (Movies) 
- Lidarr (Music)
- Readarr (Books)
- qBittorrent + Gluetun VPN
- Overseerr (Requests)
- Tautulli (Analytics)
- Bazarr (Subtitles)

### Home Automation (5+)
- Home Assistant
- Alexa Bridge
- Custom Sensors
- Automation Scripts
- Mobile Integration

### Development Tools (10+)
- AI Coding Assistant
- Documentation Processor
- Code Analysis Tools
- Project Templates
- Development VMs

### Infrastructure (10+)
- Traefik Reverse Proxy
- Prometheus Monitoring
- Grafana Dashboards
- Backup Solutions
- Network Tools

## 📚 **Documentation**

### Setup Guides
- **[Visual Setup Guide](docs/Visual-Setup-Guide.md)**: Complete step-by-step setup
- **[Network Setup](docs/README-Network-Setup.md)**: Network configuration
- **[Home Assistant Setup](docs/HOME_ASSISTANT_FIXES_COMPLETE.md)**: HA integration
- **[Wireguard VPN](WIREGUARD-GLUETUN-SOLUTION.md)**: VPN setup

### Technical Documentation
- **[Project Analysis](Coding/PROJECT_ANALYSIS_REPORT.md)**: Code analysis and completion status
- **[Setup Status](SETUP_STATUS.md)**: Current deployment status
- **[Service Configuration](mediastack-reference.json)**: Service reference

### Troubleshooting
- **[Common Fixes](docs/fixes/)**: Solution database
- **[Service Issues](docs/_organized/)**: Organized troubleshooting
- **[Container Problems](scripts/fix-all-containers.sh)**: Automated fixes

## 🚧 **Installation & Deployment**

### Method 1: Full Stack Deployment
```bash
# Clone and deploy everything
git clone https://github.com/wlfogle/awesome-stack.git
cd awesome-stack
./rebuild-ultimate-stack.sh
```

### Method 2: Selective Deployment
```bash
# Deploy media stack in Proxmox LXC containers
ssh proxmox
# Enter specific containers (100-279)
pct enter 104  # Vaultwarden
pct enter 108  # Plex
# etc.

# Deploy Home Assistant configs to VM-500
# Access HAOS VM directly

# Run automation scripts from Garuda host
cd scripts
./fix-all-containers.sh
```

### Method 3: Development Setup
```bash
# Setup development environment
cd Coding
# Choose your project type:
cd python-projects/  # Python development
cd cpp-projects/     # C++ with Qt
cd rust-projects/    # Rust applications
cd web-projects/     # Tauri/React apps
```

## 🔧 **Configuration**

### Environment Variables
Create `.env` files in appropriate Proxmox containers:
```bash
# Media stack configuration (inside LXC containers)
ssh proxmox
pct enter 104  # Enter specific container
cp .env.example .env

# Edit with your settings:
# - Domain names and SSL
# - VPN credentials
# - API keys
# - Storage paths
```

### Service Customization
- **Traefik**: Edit routing configurations
- **Home Assistant**: Customize automation scripts
- **Media Services**: Configure acquisition profiles
- **Monitoring**: Set alert thresholds

## 📚 **External Dependencies**

This project references and integrates with several excellent external tools:

- **[tteck/Proxmox](https://github.com/tteck/Proxmox)**: Proxmox helper scripts (not included, referenced)
- **LinuxServer.io**: Docker images for many services
- **Official Docker Images**: For core services like Plex, Home Assistant, etc.

See [EXTERNAL_DEPENDENCIES.md](EXTERNAL_DEPENDENCIES.md) for complete attribution and usage instructions.

## 👍 **Contributing**

This project welcomes contributions from the self-hosting community!

### Areas for Contribution
- **New Services**: Additional Docker services
- **Automation Scripts**: Infrastructure automation
- **Documentation**: Setup guides and tutorials
- **Bug Fixes**: Service configuration improvements
- **Security**: Security hardening improvements

### Development Workflow
1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Test thoroughly in isolated environment
4. Document changes and configurations
5. Submit pull request with detailed description

## 📈 **Monitoring & Analytics**

### Built-in Monitoring
- **Grafana Dashboards**: System and service metrics
- **Prometheus**: Metrics collection and alerting
- **Health Checks**: Automated service monitoring
- **Log Aggregation**: Centralized logging
- **Performance Metrics**: Resource usage tracking

### Custom Monitoring
- **Home Assistant Sensors**: Custom system sensors
- **Alexa Notifications**: Voice alerts for issues
- **Mobile Alerts**: Push notifications for problems
- **Email Reports**: Regular status reports

## 🛡️ **Security Features**

### Network Security
- **VPN Integration**: All downloads through VPN
- **SSL Certificates**: Automatic certificate management
- **Firewall Configuration**: Secure access controls
- **Domain Security**: Secure domain configurations

### Access Control
- **Authentication**: Multi-level authentication
- **Authorization**: Role-based access control
- **API Security**: Secure API endpoints
- **Audit Logging**: Complete access logging

## 🎖️ **Recognition & Stats**

### Project Statistics
- **100+ Services**: Complete ecosystem
- **50+ Scripts**: Automation tools
- **6,000+ Lines**: Custom code
- **72 Directories**: Organized structure
- **Production Tested**: Real-world deployment

### Technical Achievement
- **Complete Self-Hosting Solution**: Everything you need
- **Professional Infrastructure**: Enterprise-grade setup  
- **Automated Management**: Minimal maintenance required
- **Scalable Architecture**: Grows with your needs
- **Community Impact**: Helps others self-host

## 📝 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 **Support**

- **Issues**: [GitHub Issues](https://github.com/wlfogle/awesome-stack/issues)
- **Discussions**: [GitHub Discussions](https://github.com/wlfogle/awesome-stack/discussions)
- **Wiki**: [Project Wiki](https://github.com/wlfogle/awesome-stack/wiki)
- **Documentation**: Complete guides in `/docs` directory

## 🌟 **Acknowledgments**

Built for the self-hosting community with inspiration from:
- **r/selfhosted** community
- **Home Assistant** community  
- **Docker** and containerization community
- **Proxmox** virtualization community
- **Open source** projects and contributors

---

**🏆 This represents the ultimate self-hosting infrastructure - everything you need to run your own digital life independently and securely!**

[![GitHub stars](https://img.shields.io/github/stars/wlfogle/awesome-stack.svg?style=social&label=Star)](https://github.com/wlfogle/awesome-stack)
