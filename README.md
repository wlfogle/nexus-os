# NexusOS — AI-Native Operating System

[![Desktop](https://img.shields.io/badge/Desktop-KDE%20Plasma%20X11-blue)](https://kde.org)
[![AI](https://img.shields.io/badge/AI-Stella%20%26%20Max%20Jr.-purple)](https://github.com/wlfogle/nexus-os)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

NexusOS is a standalone Linux distribution built from the ground up with AI at its core. It features KDE Plasma X11, NVIDIA GPU optimization, a universal package manager, and two AI companions that keep your system secure and fast.

```
 ███╗   ██╗███████╗██╗  ██╗██╗   ██╗███████╗ ██████╗ ███████╗
 ████╗  ██║██╔════╝╚██╗██╔╝██║   ██║██╔════╝██╔═══██╗██╔════╝
 ██╔██╗ ██║█████╗   ╚███╔╝ ██║   ██║███████╗██║   ██║███████╗
 ██║╚██╗██║██╔══╝   ██╔██╗ ██║   ██║╚════██║██║   ██║╚════██║
 ██║ ╚████║███████╗██╔╝ ╚██╗╚██████╔╝███████║╚██████╔╝███████║
 ╚═╝  ╚═══╝╚══════╝╚═╝   ╚═╝ ╚═════╝ ╚══════╝ ╚═════╝ ╚══════╝
              The AI-Native Operating System
```

## Meet the AI Companions

```
🐕 Stella — Security Guardian             🐱 Max Jr. — Performance Optimizer

    ╭─────────╮                                ╭───────╮
   ╱   ◕   ◕  ╲                              ╱ ◉   ◉ ╲
  ╱     ▽     ╲                              ╱    △    ╲
 ╱   ╭─────╮   ╲                            ╱  ╭─────╮  ╲
╱    │░░░░░│    ╲                           ╱   │▓▓▓▓▓│   ╲
     ╰─────╯                                    ╰─────╯
   Golden Coat                                Cream Coat
   ~~~TAIL~~~                                  purring

 Wags tail when secure!                    Purrs when optimized!
```

- **Stella** (Golden Retriever) — Firewall management, security scanning, package validation, SSH hardening, system backup, Digital Fortress mode
- **Max Jr.** (Cat) — Real-time performance monitoring, GPU optimization, gaming mode, temperature checks, system tuning

## Features

- **KDE Plasma X11** desktop with NexusOS dark theme, custom SDDM login, Plymouth boot splash
- **nexuspkg** — Universal package manager supporting 20+ backends (nala, apt, flatpak, snap, pip, npm, cargo, AppImage, GitHub/GitLab releases, and more)
- **NVIDIA CUDA** out of the box with optimized drivers and Ollama for local LLMs
- **Docker** pre-installed with optional media stack (65+ self-hosted services)
- **Kernel tuning** — BBR congestion control, gaming scheduler, NVIDIA modprobe, controller udev rules
- **Security** — UFW firewall, fail2ban, SSH hardening, automated health checks
- **Calamares installer** for installing to disk from the live session

## Quick Start

### Build the ISO

```bash
git clone https://github.com/wlfogle/nexus-os.git
cd nexus-os
sudo ./scripts/build-iso.sh
```

The build script bootstraps a complete NexusOS root filesystem from scratch — no base ISO needed. Options:

```bash
sudo ./scripts/build-iso.sh --no-nvidia       # Skip NVIDIA drivers
sudo ./scripts/build-iso.sh --output ~/iso     # Custom output dir
sudo ./scripts/build-iso.sh --mirror http://...  # Custom APT mirror
```

### Test in QEMU

```bash
qemu-system-x86_64 -m 4G -enable-kvm \
    -cdrom build/nexusos-1.0-*.iso
```

### Write to USB

```bash
sudo dd if=build/nexusos-1.0-*.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

## Daily Usage

```bash
# System management
nexus-control status          # System overview
nexus-control health          # Full health check
nexus-control gpu             # GPU stats
nexus-control services        # Service status
sudo nexus-control update     # Update everything

# Security (Stella)
stella --status               # Security overview
stella --scan                 # Full security scan
sudo stella --digital-fortress  # Enable full hardening
sudo stella --backup-system   # Backup NexusOS config

# Performance (Max Jr.)
maxjr --monitor               # Live performance dashboard
maxjr --temperature           # System temps
sudo maxjr --optimize         # Full optimization
sudo maxjr --gaming-mode      # Gaming performance profile

# Package management
nexuspkg install firefox      # Auto-detect best source
nexuspkg install --backend flatpak discord
nexuspkg search neovim        # Search all 20 backends
nexuspkg update               # Update all backends
nexuspkg repos                # Show available backends
```

## Architecture

```
nexus-os/
├── scripts/
│   └── build-iso.sh          # ISO builder (debootstrap-based)
├── core/
│   ├── bin/                   # CLI tools
│   │   ├── nexus-control      # System management
│   │   ├── nexuspkg           # Universal package manager
│   │   ├── stella             # Security guardian
│   │   ├── maxjr              # Performance optimizer
│   │   └── nexus-first-run    # First-boot wizard
│   ├── services/              # AI services (FastAPI)
│   │   ├── stella.py          # Security API (:8601)
│   │   ├── maxjr.py           # Performance API (:8602)
│   │   ├── orchestrator.py    # Central coordinator (:8600)
│   │   └── *.service/timer    # systemd units
│   ├── config/                # System tuning
│   │   ├── sysctl-nexus.conf  # Kernel parameters
│   │   ├── modprobe-nvidia.conf
│   │   ├── udev-gaming.rules
│   │   └── limits-nexus.conf
│   ├── branding/              # Visual identity
│   │   ├── motd, issue        # Terminal branding
│   │   ├── neofetch-nexus.conf
│   │   ├── plymouth/          # Boot splash
│   │   ├── sddm-theme/        # Login screen (QML)
│   │   └── wallpaper/         # Desktop wallpaper
│   ├── desktop/               # KDE Plasma config
│   │   ├── plasma-layout.js   # Panel/desktop layout
│   │   ├── konsole-nexus.profile
│   │   ├── sddm-nexus.conf
│   │   └── applications/      # .desktop launchers
│   ├── shell/                 # Shell customizations
│   │   ├── bashrc-nexus       # Aliases, prompt, PATH
│   │   └── profile-nexus      # Login environment
│   ├── security/              # Hardening scripts
│   ├── ai/                    # Ollama setup
│   ├── installer/             # Calamares config
│   └── media-stack/           # Docker Compose (65+ services)
└── build/                     # Built ISO output
```

## AI Services

| Service | Port | Purpose |
|---------|------|---------|
| Orchestrator | 8600 | Central coordination, API gateway |
| Stella | 8601 | Security scanning, firewall, login monitoring |
| Max Jr. | 8602 | CPU/GPU/memory metrics, gaming detection |

## Hardware Optimized For

- **CPU**: Intel i9-13900HX (scheduler tuning for P-core/E-core)
- **GPU**: NVIDIA RTX 4080 (CUDA, persistence mode, power management)
- **RAM**: 64GB DDR5 (low swappiness, huge pages)
- **Desktop**: KDE Plasma X11

Works on any x86_64 hardware with or without NVIDIA GPU.

## Build Dependencies

Installed automatically by the build script:

```
debootstrap squashfs-tools xorriso mtools
grub-efi-amd64-bin grub-efi-amd64-signed shim-signed
grub-pc-bin rsync dosfstools
```

## License

GPL-3.0 — See [LICENSE](LICENSE).
