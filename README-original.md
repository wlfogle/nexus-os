# 🚀 NexusOS - Universal AI-Native Operating System

**The world's first truly universal Linux distribution that can natively install packages from ANY Linux distribution, integrated with complete self-hosting infrastructure and ultimate security.**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Architecture](https://img.shields.io/badge/Architecture-x86_64-green.svg)](https://github.com/nexusos/nexusos)
[![ZFS](https://img.shields.io/badge/Filesystem-ZFS-orange.svg)](https://openzfs.org/)
[![AI-Native](https://img.shields.io/badge/AI-Native-purple.svg)](https://github.com/nexusos/nexusos)

## 🌟 Revolutionary Features

### 📦 **Universal Package Compatibility**
- **Native installation from ANY Linux distribution**
- Support for DEB, RPM, ZST, Flatpak, Snap, AppImage, Python, NPM, Cargo packages
- Automatic format detection and conversion
- One package manager to rule them all: **NexusPkg**

### 🧠 **AI-Native Architecture** 
- **Nexus-Rani AI Assistant** - Your intelligent system companion
- AI-powered package recommendations and system optimization
- Intelligent dependency resolution across all package formats
- Semantic package search and automatic categorization

### 🛡️ **Digital Fortress Security Suite**
- **Ultimate online invisibility** with Ghost Mode
- Browser fingerprinting protection (WebRTC, Canvas, WebGL, Audio)
- Hardware fingerprinting spoofing (CPU, GPU, RAM, System info)
- Network anonymization (IPv6 disabled, DNS leak prevention)
- Continuous monitoring with auto-repair capabilities

### 🏠 **Complete Self-Hosting Infrastructure**
- **Awesome-Stack integration** - 100+ services ready to deploy
- Media stack: Plex, Jellyfin, Sonarr, Radarr, qBittorrent
- Home automation: Home Assistant with Alexa integration
- Development tools: GitLab, Jenkins, Code Server, AI coding assistants
- Monitoring: Grafana, Prometheus, comprehensive dashboards

### 🔐 **Vaultwarden Base System**
- **Self-hosted Bitwarden** integrated at the OS level
- Premium password management features for free
- Browser extensions and mobile app synchronization
- End-to-end encryption with security breach monitoring

### ⚡ **Advanced Technologies**
- **ZFS as default filesystem** with compression and snapshots
- **UEFI boot** with modern bootloader
- **Systemd integration** with all package formats
- **Calamares installer** with ZFS optimization
- **Qt-based setup assistant** like Garuda Linux

## 🎯 **What Makes NexusOS Unique?**

Unlike traditional Linux distributions that lock you into their package ecosystem, NexusOS breaks down barriers:

- **Install Firefox from Ubuntu** while running **VS Code from Arch**
- **Use Fedora's RPMs** alongside **Alpine's APKs** seamlessly
- **Flatpaks, Snaps, AppImages** all work natively
- **Python, Node.js, Rust** packages installed universally
- **Docker containers** managed as first-class packages

All powered by a **single AI assistant** that understands your needs and optimizes your system automatically.

## 🏗️ **Architecture Overview**

```
┌─────────────────────────────────────────────────────────────────┐
│                    NexusOS Universal Layer                     │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              Nexus Setup Assistant (Qt)                    │ │
│  │    🎬 Media Stack  🛡️ Security  📦 Packages  ⚙️ Config     │ │
│  └────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                     Package Compatibility                      │
│  ┌──────────┐┌──────────┐┌──────────┐┌──────────┐┌──────────┐  │
│  │   DEB    ││   RPM    ││   ZST    ││ Flatpak  ││   Snap   │  │
│  │ (dpkg)   ││ (rpm)    ││(pacman)  ││(flatpak) ││ (snapd)  │  │
│  └──────────┘└──────────┘└──────────┘└──────────┘└──────────┘  │
│  ┌──────────┐┌──────────┐┌──────────┐┌──────────┐┌──────────┐  │
│  │AppImage  ││ Python   ││  Node.js ││   Rust   ││  Docker  │  │
│  │ (FUSE)   ││  (pip)   ││  (npm)   ││ (cargo)  ││(containerd)│ │
│  └──────────┘└──────────┘└──────────┘└──────────┘└──────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                      NexusPkg Manager                          │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  🔍 Auto-detect  📊 Dependency  🔄 Convert  ✅ Install    │ │
│  └────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    Base System Services                        │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐   │
│  │Digital Fortress │ │   Vaultwarden   │ │ Awesome Stack   │   │
│  │  (Ghost Mode)   │ │  (Passwords)    │ │ (Self-hosting)  │   │
│  └─────────────────┘ ┌─────────────────┐ └─────────────────┘   │
│                      │   Nexus-Rani    │                       │
│                      │ (AI Assistant)  │                       │
│                      └─────────────────┘                       │
├─────────────────────────────────────────────────────────────────┤
│                     NexusOS Kernel                             │
│      🗃️ ZFS Root  💾 Memory Manager  🔧 Package Compat        │
└─────────────────────────────────────────────────────────────────┘
```

## 🚀 **Quick Start**

### Option 1: Build from Source (Recommended for Development)

```bash
# Clone the repository
git clone https://github.com/nexusos/nexusos.git
cd nexusos

# Build the complete system
chmod +x build-nexusos.sh
./build-nexusos.sh

# Build NexusPkg universal package manager
make -C userspace/system/nexuspkg deps
make -C userspace/system/nexuspkg
sudo make -C userspace/system/nexuspkg install

# Build Setup Assistant
make -C userspace/system/nexus-setup-assistant deps
make -C userspace/system/nexus-setup-assistant
sudo make -C userspace/system/nexus-setup-assistant install
```

### Option 2: Install Pre-built Components

```bash
# Install NexusPkg (universal package manager)
wget https://releases.nexusos.org/nexuspkg-latest.tar.gz
tar -xzf nexuspkg-latest.tar.gz
sudo ./install-nexuspkg.sh

# Test universal package installation
nexuspkg flatpak org.mozilla.firefox        # Install Firefox via Flatpak
nexuspkg deb install ./package.deb          # Install local DEB package
nexuspkg snap discord                       # Install Discord via Snap
nexuspkg pip numpy                          # Install Python package
nexuspkg npm typescript                     # Install Node.js package
nexuspkg cargo ripgrep                      # Install Rust package
```

### Option 3: Full NexusOS Installation

```bash
# Build complete bootable ISO
./build-nexusos.sh --iso

# Or download pre-built ISO
wget https://releases.nexusos.org/nexusos-latest.iso

# Create bootable USB
sudo dd if=nexusos-latest.iso of=/dev/sdX bs=4M status=progress

# Boot from USB and follow Calamares installer
```

## 🎮 **Usage Examples**

### Universal Package Management

```bash
# NexusPkg automatically detects and installs from any format
nexuspkg install firefox                    # Auto-detects best source
nexuspkg search "media player"              # Search across all formats
nexuspkg list                               # List all installed packages
nexuspkg status                             # Show system status

# Format-specific installation
nexuspkg deb install google-chrome.deb     # Install DEB directly
nexuspkg rpm install package.rpm           # Install RPM package
nexuspkg flatpak com.spotify.Client        # Install from Flathub
nexuspkg appimage ./Firefox.AppImage       # Install AppImage
nexuspkg docker nginx                      # Install Docker container
```

### AI Assistant (Nexus-Rani)

```bash
# Interactive AI assistant
nexus-rani                                 # Launch interactive mode

# Command-line usage
nexus-rani --status                        # Show system status
nexus-rani --install firefox --format flatpak
nexus-rani --media-stack                   # Setup complete media server
nexus-rani --digital-fortress              # Enable privacy protection
nexus-rani --vaultwarden                   # Setup password manager
nexus-rani --optimize                      # Optimize system performance
nexus-rani --recommendations               # Get AI recommendations
```

### Setup Assistant (GUI)

```bash
# Launch graphical setup assistant
nexus-setup-assistant

# Available tabs:
# 🎬 Media Stack    - Plex, Jellyfin, Sonarr, Radarr, etc.
# 🏠 Self-Hosting   - Nextcloud, Home Assistant, Grafana
# 💻 Development    - GitLab, Jenkins, VS Code, databases
# 🎮 Gaming         - Steam, Lutris, RetroArch
# 🛡️ Security Base  - Digital Fortress, Vaultwarden
# 📦 Install Packages - Universal package installation GUI
```

### Digital Fortress (Ultimate Privacy)

```bash
# Enable ghost mode (ultimate online invisibility)
sudo systemctl enable --now digital-fortress.service

# Toggle ghost mode on/off
nexus-rani --digital-fortress

# Features automatically enabled:
# ✅ Browser fingerprinting blocked
# ✅ Hardware fingerprinting spoofed  
# ✅ Network anonymization active
# ✅ DNS leak prevention
# ✅ IPv6 disabled for privacy
# ✅ Continuous monitoring active
```

## 🛠️ **Development & Customization**

### Project Structure

```
nexus-os/
├── kernel/                          # NexusOS kernel with universal package support
│   ├── init/main.c                 # Kernel initialization
│   ├── pkg/pkg_compat.c           # Package compatibility layer
│   └── include/pkg_compat.h       # Package format definitions
├── bootloader/                     # UEFI bootloader
├── userspace/
│   ├── system/
│   │   ├── nexuspkg/               # Universal package manager
│   │   │   ├── main.c             # NexusPkg implementation
│   │   │   └── Makefile          # Build configuration
│   │   ├── nexus-setup-assistant/ # GUI setup tool
│   │   │   ├── main.cpp          # Qt-based setup assistant
│   │   │   └── Makefile          # Build configuration
│   │   ├── nexus-ai-assistant/    # AI command-line assistant
│   │   │   └── nexus-rani.py     # Nexus-Rani AI assistant
│   │   └── services/              # System services
│   │       ├── digital-fortress.service
│   │       └── vaultwarden.service
│   └── desktop/                    # Desktop environment components
├── installer/                      # Calamares-based installer with ZFS
└── build-nexusos.sh               # Main build script
```

### Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

### Building Components

```bash
# Build individual components
make -C userspace/system/nexuspkg                    # Universal package manager
make -C userspace/system/nexus-setup-assistant      # GUI setup assistant
make -C kernel                                      # Kernel with package support
make -C bootloader                                  # UEFI bootloader
```

## 📋 **System Requirements**

### Minimum Requirements
- **CPU**: x86_64 processor (Intel/AMD 64-bit)
- **RAM**: 4GB (8GB recommended for AI features)
- **Storage**: 32GB free space (SSD recommended)
- **UEFI**: Modern UEFI firmware support
- **Network**: Internet connection for package installation

### Recommended Configuration
- **CPU**: Modern multi-core processor (Intel Core i5/AMD Ryzen 5+)
- **RAM**: 16GB+ for media stack and AI features
- **Storage**: 256GB+ NVMe SSD for optimal ZFS performance
- **GPU**: Dedicated graphics card for AI workloads (optional)

### Supported Package Sources
- **Ubuntu/Debian**: Official repositories + PPAs
- **Fedora/RedHat**: DNF/YUM repositories  
- **Arch Linux**: Official repos + AUR (Arch User Repository)
- **Alpine Linux**: APK repositories
- **Void Linux**: XBPS repositories
- **Flathub**: Universal Flatpak applications
- **Snap Store**: Ubuntu Snap packages
- **AppImage Hub**: Portable AppImage applications
- **PyPI**: Python Package Index
- **NPM Registry**: Node.js packages
- **Crates.io**: Rust packages
- **Docker Hub**: Container images

## 🤝 **Community & Support**

- **GitHub Issues**: [Report bugs and request features](https://github.com/nexusos/nexusos/issues)
- **Discord**: [Join our community](https://discord.gg/nexusos)
- **Documentation**: [Complete user guide](https://docs.nexusos.org)
- **Reddit**: [r/NexusOS](https://reddit.com/r/NexusOS)

## 📄 **License**

NexusOS is released under the GNU General Public License v3.0. See [LICENSE](LICENSE) for details.

## 🙏 **Acknowledgments**

- **Awesome Stack**: Complete self-hosting infrastructure integration
- **Garuda Linux**: Inspiration for setup assistant and AI helper
- **ZFS**: Advanced filesystem with data integrity and snapshots  
- **Calamares**: Universal Linux installer framework
- **Qt Framework**: Modern cross-platform GUI toolkit
- **All Linux distributions**: For the amazing packages we can now install universally!

---

**NexusOS - One OS, All Packages, Ultimate Freedom** 🚀