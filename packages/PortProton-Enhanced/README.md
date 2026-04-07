# PortProton Enhanced

**A Native Ubuntu Build with Custom Game Launcher Autoinstallers**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Ubuntu](https://img.shields.io/badge/Ubuntu-25.10-orange.svg)](https://ubuntu.com)
[![KDE Plasma](https://img.shields.io/badge/KDE-Plasma-blue.svg)](https://kde.org/plasma-desktop/)
[![RTX 4080](https://img.shields.io/badge/RTX-4080-green.svg)](https://www.nvidia.com)

## 🎯 Overview

**PortProton Enhanced** is a custom-built native Ubuntu version of PortProton, specifically optimized for:
- **Ubuntu 25.10** with **KDE Plasma** desktop environment
- **NVIDIA RTX 4080** hybrid graphics systems  
- **Gaming optimization** with GameMode + MangoHUD
- **25+ custom game launcher autoinstallers**
- **Steam library integration**
- **Direct game launchers** for installed Windows games

This project extends the original [PortProton](https://github.com/Castro-Fidel/PortWINE) with enhanced autoinstall capabilities and system-specific optimizations.

## ✨ Enhanced Features

### 🎮 **25 Custom Game Launcher Autoinstallers**
- **EA App** (successor to Origin)  
- **Riot Games Client** (Valorant, League of Legends)
- **Amazon Games App**
- **Xbox App** (Microsoft Gaming)
- **GeForce Now**
- **Humble Bundle App**
- **Discord**  
- **Unity Hub**
- **Arc** (Perfect World)
- **Bethesda Launcher**
- **Activision Launcher** 
- **Twitch Desktop**
- **CurseForge**
- **MultiMC** (Minecraft)
- And more...

### 🚀 **Direct Game Launchers**
Pre-configured launchers for your installed games:
- Dead Island 2 Deluxe Edition
- Mafia III Definitive Edition  
- Last Epoch
- Dark Deity Complete Edition
- Battlezone Combat Commander
- Star Wars Empire at War
- Phoenix Point

### 🔍 **Smart Game Detection**
- **Auto-detect** all Windows games in `/media/lou/Games/Games`
- **Steam library parsing** from `/media/lou/Games/SteamLibrary`
- **Bulk shortcut creation** for discovered games
- **Steam appmanifest reading** for proper game identification

### ⚡ **System Optimizations**
- **NVIDIA RTX 4080** shader caching and threading
- **Hybrid graphics** prime render offloading
- **32-thread CPU topology** optimization 
- **KDE Plasma** desktop integration
- **Ubuntu 25.10** compatibility tweaks
- **GameMode + MangoHUD** pre-configured

## 🛠️ Installation

### Prerequisites
- Ubuntu 25.10 (other versions may work but are untested)
- NVIDIA drivers installed (tested with 580.82.07)
- KDE Plasma desktop environment
- At least 4GB free space

### Quick Install
```bash
git clone https://github.com/YourUsername/PortProton-Enhanced.git
cd PortProton-Enhanced
chmod +x install-enhanced-portproton.sh
./install-enhanced-portproton.sh
```

The installer will:
1. Install all required system dependencies
2. Download and configure PortProton core
3. Install all 25+ custom autoinstall scripts
4. Create optimized system configuration
5. Set up desktop integration
6. Configure game library paths

## 🎮 Usage

### Launch Methods
- **Desktop**: Search for "PortProton Enhanced" in your application menu
- **Terminal**: `portproton-enhanced`
- **Direct**: `~/PortProton-Enhanced/bin/portproton`

### Quick Start Guide
1. **Launch PortProton Enhanced**
2. **Click "AutoInstall"** to see all custom launcher options
3. **Use "Auto-Detect ALL Games"** to scan your game libraries
4. **Install game launchers** or create direct game shortcuts
5. **Enjoy optimized gaming** with RTX 4080 + hybrid graphics

### Game Library Paths
The installer automatically configures these paths:
- **Regular Games**: `/media/lou/Games/Games`
- **Steam Library**: `/media/lou/Games/SteamLibrary`

## 🔧 Advanced Configuration

### Custom Game Paths
Edit the configuration file after installation:
```bash
nano ~/PortProton-Enhanced/data/user.conf
```

### NVIDIA Optimizations
The following optimizations are automatically applied:
```bash
export __GL_SHADER_DISK_CACHE=1
export __GL_THREADED_OPTIMIZATIONS=1  
export __NV_PRIME_RENDER_OFFLOAD=1
export __GLX_VENDOR_LIBRARY_NAME=nvidia
```

### Wine Configuration  
- **Default Wine**: PROTON_LG (optimized for gaming)
- **Vulkan**: Enabled by default
- **DXVK/VKD3D**: Pre-configured for RTX 4080
- **ESYNC/FSYNC**: Enabled for better performance

## 🤝 Contributing

We welcome contributions! Please:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)  
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

## 📝 License

This project is licensed under the **GNU General Public License v3.0** - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **[PortWINE Team](https://github.com/Castro-Fidel/PortWINE)** - Original PortProton developers
- **[Flathub Community](https://flathub.org/)** - Flatpak packaging and distribution
- **[NVIDIA](https://www.nvidia.com)** - GPU driver support and optimization guides
- **[Ubuntu Community](https://ubuntu.com)** - Excellent Linux distribution and documentation
- **[KDE Community](https://kde.org)** - Beautiful and functional desktop environment

---

**Enjoy your enhanced gaming experience on Linux!** 🎮✨
