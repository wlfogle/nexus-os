# 🚀 NexusOS - Universal AI-Native Operating System

[![Development Status](https://img.shields.io/badge/Status-Phase%201%20Development-yellow)](https://github.com/nexusos/nexusos)
[![Base](https://img.shields.io/badge/Base-Pop!_OS%2022.04--blue)](https://pop.system76.com)
[![AI Assistants](https://img.shields.io/badge/AI-Stella%20%26%20Max%20Jr.-purple)](https://github.com/nexusos/nexusos)
[![Gaming](https://img.shields.io/badge/Gaming-linux--zen-green)](https://github.com/nexusos/nexusos)
[![Media](https://img.shields.io/badge/Media-65%2B%20Services-orange)](https://github.com/nexusos/nexusos)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

> **The world's first truly universal Linux distribution with AI mascot companions**  
> Built on Pop!_OS 22.04 LTS NVIDIA • Universal Package Compatibility • Complete Media Center

---

## 🌟 **Meet Your AI Companions**

<div align="center">

### 🐕 **Stella** (Golden Retriever)
**Security Guardian & Package Manager**
```
    ╭─────────╮
   ╱   ◕   ◕  ╲
  ╱     ▽     ╲
 ╱   ╭─────╮   ╲
╱    │░░░░░│    ╲
     ╰─────╯
   Golden Coat
   ~~~TAIL~~~
    wagging
```
*Wags tail when installing packages!*

### 🐱 **Max Jr.** (Cat)  
**Performance Optimizer & System Monitor**
```
    ╭───────╮
   ╱ ◉   ◉ ╲
  ╱    △    ╲
 ╱  ╭─────╮  ╲
╱   │▓▓▓▓▓│   ╲
    ╰─────╯
  Cream Coat
   purring
```
*Purrs when system is optimized!*

</div>

---

## 🎯 **What Makes NexusOS Revolutionary?**

### 🎮 **Gaming Excellence**
Built on the acclaimed **Pop!_OS 22.04 LTS NVIDIA** with the Pop!_OS kernel — all gaming optimizations included plus universal package access.

### 📦 **Universal Package Compatibility**
**Install from ANY Linux distribution:**
- **Ubuntu/Debian** → `nexuspkg deb install firefox.deb`
- **Fedora/RedHat** → `nexuspkg rpm install package.rpm`  
- **Flatpak** → `nexuspkg install discord --format=flatpak`
- **Flatpak** → `nexuspkg flatpak com.spotify.Client`
- **Snap** → `nexuspkg snap install code`
- **Python** → `nexuspkg pip numpy`
- **Node.js** → `nexuspkg npm typescript`
- **Rust** → `nexuspkg cargo ripgrep`

### 🖥️ **NexusDE Desktop Environment**
Hybrid X11/Wayland desktop with AI-integrated features, built on KDE Plasma foundation.

### 📺 **Complete Media Center**
**65+ services ready to deploy:**
- **Media Servers**: Jellyfin, Plex, Audiobookshelf
- **Automation**: Sonarr, Radarr, Lidarr, Readarr, Mylar3
- **Indexers**: Prowlarr, Jackett, Autobrr
- **Monitoring**: Grafana, Prometheus, Tautulli
- **Management**: Organizr, Homarr, Portainer
- **[See full service list →](core/package-management/nexus-packages.yml)**

---

## 🏗️ **System Architecture**

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
│  │   DEB    ││   RPM    ││ Flatpak  ││   Snap   ││ AppImage │  │
│  │ (dpkg)   ││ (rpm)    ││(flatpak) ││ (snapd)  ││ (FUSE)   │  │
│  └──────────┘└──────────┘└──────────┘└──────────┘└──────────┘  │
│  ┌──────────┐┌──────────┐┌──────────┐┌──────────┐┌──────────┐  │
│  │AppImage  ││ Python   ││  Node.js ││   Rust   ││  Docker  │  │
│  │ (FUSE)   ││  (pip)   ││  (npm)   ││ (cargo)  ││(containerd)│ │
│  └──────────┘└──────────┘└──────────┘└──────────┘└──────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                      nexuspkg Manager                          │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  🔍 Auto-detect  📊 Dependency  🔄 Convert  ✅ Install    │ │
│  └────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    Base System Services                        │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐   │
│  │Digital Fortress │ │   Vaultwarden   │ │ Awesome Stack   │   │
│  │  (Ghost Mode)   │ │  (Passwords)    │ │ (Self-hosting)  │   │
│  └─────────────────┘ ┌─────────────────┐ └─────────────────┘   │
│                      │  Stella & Max   │                       │
│                      │  (AI Assistants)│                       │
│                      └─────────────────┘                       │
├─────────────────────────────────────────────────────────────────┤
│                   Pop!_OS 22.04 NVIDIA                     │
│      🗃️ ZFS Root  💾 Pop!_OS kernel  🎮 Gaming Optimized          │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🚀 **Development Status & Roadmap**

### **Current Phase: 1 - Core Foundation** 🟢 *In Progress*

| Component | Status | Description |
|-----------|--------|-------------|
| **Pop!_OS Base** | ✅ 100% | Pop!_OS 22.04 NVIDIA foundation |
| **nexuspkg** | 🔄 85% | Universal package manager (C implementation) |
| **Stella AI** | 🔄 70% | Security & package management assistant |
| **Max Jr. AI** | 🔄 70% | Performance optimization assistant |
| **NexusDE** | 🔄 80% | Desktop environment (QML/C++ components) |
| **Media Stack** | 🔄 90% | 65+ service configurations ready |
| **AI Orchestrator** | 🔄 75% | FastAPI service coordinator |

### **Upcoming Phases**

| Phase | Timeline | Focus |
|-------|----------|-------|
| **Phase 2** | 3-6 weeks | Desktop Environment Implementation |
| **Phase 3** | 2-3 weeks | Live ISO & Installation System |
| **Phase 4** | 4-8 weeks | Hardware Compatibility & Polish |

---

## 🛠️ **Quick Start Guide**

### **Prerequisites**
- Pop!_OS 22.04 LTS NVIDIA (recommended base)
- 16GB+ RAM (for full media stack)
- 500GB+ storage (for gaming + media)

### **Development Setup**

```bash
# Clone the repository
git clone https://github.com/nexusos/nexusos.git
cd nexusos

# Set up development environment
sudo mkdir -p /opt/nexusos
sudo chown $USER:$USER /opt/nexusos

# Build nexuspkg universal package manager
make -C userspace/system/nexuspkg deps
make -C userspace/system/nexuspkg
sudo make -C userspace/system/nexuspkg install

# Deploy AI service orchestrator
cd core/services
pip install -r requirements.txt
python nexus-orchestrator.py &

# Test universal package installation
nexuspkg install firefox                    # Auto-detects best source
nexuspkg flatpak com.spotify.Client        # Install Spotify via Flatpak
nexuspkg deb install ./package.deb         # Install local DEB package
```

### **Media Stack Deployment**

```bash
# Deploy complete 65+ service media stack
cd media
docker compose up -d

# Access services
# Primary Dashboard: http://localhost:8540 (Organizr)
# Jellyfin: http://localhost:8200
# Plex: http://localhost:8201
# Sonarr: http://localhost:8110
# Radarr: http://localhost:8111
```

---

## 🤖 **AI Assistants Usage**

### **Stella 🐕 (Security Guardian)**

```bash
# Interactive security assistant
stella --status                        # Security status check
stella --scan-packages                 # Scan installed packages
stella --digital-fortress              # Enable privacy mode
stella --backup-system                 # Initiate system backup

# Package management with security
nexuspkg install --secure firefox      # Stella validates package
stella --monitor-install               # Watch installations
```

### **Max Jr. 🐱 (Performance Optimizer)**

```bash
# Interactive performance assistant  
maxjr --optimize                       # System optimization
maxjr --gaming-mode                    # Enable gaming optimizations
maxjr --gpu-switch                     # Manage hybrid GPU switching
maxjr --recommendations               # Get performance suggestions

# Real-time monitoring
maxjr --monitor                        # Performance monitoring
maxjr --temperature                    # System temperature check
```

### **AI Service Orchestrator**

```bash
# Web interface (FastAPI)
curl http://localhost:8600/api/status           # System status
curl http://localhost:8600/api/services         # Service health
curl http://localhost:8600/api/recommendations  # AI recommendations

# Toggle AI assistants
curl -X POST http://localhost:8600/api/stella/toggle
curl -X POST http://localhost:8600/api/maxjr/toggle
```

---

## 💻 **Technical Specifications**

| Specification | Details |
|---------------|---------|
| **Base System** | Pop!_OS 22.04 LTS NVIDIA |
| **Kernel** | Pop!_OS kernel (NVIDIA-optimized) |
| **Desktop** | NexusDE (hybrid X11/Wayland) on KDE Plasma |
| **Package Managers** | nexuspkg (universal) + nala/flatpak (Pop!_OS base) |
| **AI Framework** | Python with FastAPI coordination |
| **Media Stack** | Docker containers with 65+ services |
| **Init System** | systemd with AI service orchestration |
| **Security** | Digital Fortress suite + Vaultwarden integration |

### **Supported Package Formats**
- **DEB** (Ubuntu/Debian) via dpkg
- **RPM** (Fedora/RedHat) via rpm/dnf  
- **DEB** (Pop!_OS/Ubuntu) via nala/apt + PPAs
- **APK** (Alpine Linux) via apk
- **Flatpak** (Universal apps) via flatpak
- **Snap** (Ubuntu universal) via snapd
- **AppImage** (Portable apps) via FUSE
- **Python** (PyPI) via pip
- **Node.js** (NPM) via npm
- **Rust** (Crates.io) via cargo
- **Docker** (Containers) via containerd

---

## 🤝 **Contributing**

We welcome contributions from the community! NexusOS is built with:

### **Technologies Used**
- **C/C++**: Core system components and nexuspkg
- **Python**: AI assistants and service orchestration  
- **QML/Qt**: NexusDE desktop environment
- **Docker**: Media stack and service containers
- **Shell Scripts**: Installation and automation

### **How to Contribute**

1. **Fork the Repository**
   ```bash
   gh repo fork nexusos/nexusos
   ```

2. **Choose Your Area**
   - 🔧 **Core Systems**: nexuspkg, system integration
   - 🤖 **AI Development**: Stella & Max Jr. assistants
   - 🖥️ **Desktop Environment**: NexusDE components
   - 📺 **Media Stack**: Service configurations
   - 📚 **Documentation**: Guides and tutorials
   - 🎨 **Design**: UI/UX, mascot artwork

3. **Development Setup**
   ```bash
   git clone https://github.com/YOUR_USERNAME/nexusos.git
   cd nexusos
   
   # Create feature branch
   git checkout -b feature/amazing-feature
   
   # Make changes, test, commit
   git commit -m 'Add amazing feature'
   git push origin feature/amazing-feature
   
   # Create Pull Request
   ```

### **Contribution Guidelines**
- Follow existing code style and conventions
- Test changes thoroughly before submitting
- Update documentation for new features
- Be respectful and constructive in discussions
- Have fun building the future of Linux! 🚀

---

## 🌐 **Community & Support**

### **Join the Community**
- 🐙 **GitHub**: [Issues & Discussions](https://github.com/nexusos/nexusos/issues)
- 💬 **Discord**: [Join our server](https://discord.gg/nexusos) *(coming soon)*
- 🐦 **Twitter**: [@NexusOS_Linux](https://twitter.com/NexusOS_Linux) *(coming soon)*
- 📺 **YouTube**: Development vlogs *(planned)*
- 📧 **Email**: nexusos@example.com *(setup needed)*

### **Get Help**
- 📖 **Documentation**: [docs.nexusos.org](https://docs.nexusos.org) *(coming soon)*
- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/nexusos/nexusos/issues)
- 💡 **Feature Requests**: [GitHub Discussions](https://github.com/nexusos/nexusos/discussions)
- ❓ **Questions**: Discord community chat

### **Follow Development**
- ⚡ **Real-time Updates**: GitHub commits and releases
- 📋 **Weekly Progress**: Development blog *(planned)*
- 🎥 **Video Updates**: YouTube development vlogs *(planned)*
- 📢 **Announcements**: Twitter and Discord

---

## 📄 **License & Acknowledgments**

### **License**
NexusOS is released under the **GNU General Public License v3.0**. See [LICENSE](LICENSE) for details.

### **Built With Love & Thanks To**
- 🎮 **[Pop!_OS](https://pop.system76.com)** - Excellent gaming foundation
- 🐧 **Pop!_OS Kernel** - NVIDIA-optimized gaming performance
- 🎨 **[Qt Framework](https://qt.io)** - Beautiful desktop environment  
- 📺 **[Awesome Stack](https://github.com/awesome-selfhosted/awesome-selfhosted)** - Media center inspiration
- 🐳 **[Docker](https://docker.com)** - Container orchestration
- 🔒 **[Vaultwarden](https://github.com/dani-garcia/vaultwarden)** - Security integration
- 🛠️ **[ZFSBootMenu](https://zfsbootmenu.org)** - ZFS-on-root bootloader

### **Special Recognition**
- The Linux community for endless innovation
- Pop!_OS team for gaming excellence  
- All package maintainers across distributions
- Open source contributors worldwide

---

<div align="center">

## 🎊 **The Future of Linux is Here**

### **NexusOS - One OS, All Packages, Ultimate Freedom** 

*With Stella 🐕 & Max Jr. 🐱 as your AI companions*

**[⭐ Star this repo](https://github.com/nexusos/nexusos)** • **[🍴 Fork & contribute](https://github.com/nexusos/nexusos/fork)** • **[💬 Join discussions](https://github.com/nexusos/nexusos/discussions)**

---

*Built with ❤️ by the NexusOS community*

**Status**: 🔄 Phase 1 Development | **Next Release**: Alpha Preview in 2-4 weeks

</div>