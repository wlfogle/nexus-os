# NexusOS — AI-Native Operating System

[![Desktop](https://img.shields.io/badge/Desktop-KDE%20Plasma%20X11-blue)](https://kde.org)
[![Base](https://img.shields.io/badge/Base-Ubuntu%20Jammy%2022.04-orange)](https://ubuntu.com)
[![AI](https://img.shields.io/badge/AI-Stella%20%26%20Max%20Jr.-purple)](https://github.com/wlfogle/nexus-os)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Build](https://img.shields.io/badge/ISO-4.8G%20Bootable-green)](https://github.com/wlfogle/nexus-os)

NexusOS is a standalone Linux distribution bootstrapped from Ubuntu Jammy (22.04) via debootstrap with AI at its core. It features KDE Plasma X11, NVIDIA PRIME render offload (Intel iGPU + NVIDIA dGPU), every major Linux package manager compiled from source, and two AI companions that keep your system secure and fast.

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

### Build the ISO (full rebuild ~70 min)

```bash
git clone https://github.com/wlfogle/nexus-os.git
cd nexus-os
sudo ./scripts/build-iso.sh
```

The build script debootstraps a complete NexusOS root filesystem from Ubuntu Jammy — no base ISO needed. Options:

```bash
sudo ./scripts/build-iso.sh --no-nvidia       # Skip NVIDIA drivers
sudo ./scripts/build-iso.sh --output ~/iso     # Custom output dir
sudo ./scripts/build-iso.sh --mirror http://...  # Custom APT mirror
```

### Patch an existing ISO (faster)

Apply fixes to an already-built ISO without a full rebuild:

```bash
sudo ./scripts/patch-iso.sh                    # Auto-finds latest ISO
sudo ./scripts/patch-iso.sh path/to/nexusos.iso  # Specific ISO
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
│   ├── build-iso.sh          # Full ISO builder (debootstrap + compile)
│   └── patch-iso.sh          # Delta patcher for existing ISOs
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

## GPU Configuration (PRIME Render Offload)

NexusOS uses PRIME render offload — Intel iGPU drives the desktop, NVIDIA activates on demand:

```bash
prime-run glxgears              # Run app on NVIDIA GPU
prime-run steam                 # Launch Steam on dGPU
__NV_PRIME_RENDER_OFFLOAD=1 app # Manual env var method
```

Config files: `10-intel-primary.conf`, `11-nvidia-prime-offload.conf`, `nvidia-prime.conf`

## Hardware Optimized For

- **CPU**: Intel i9-13900HX (scheduler tuning for P-core/E-core)
- **iGPU**: Intel UHD (modesetting, desktop rendering)
- **dGPU**: NVIDIA RTX 4080 (PRIME offload for gaming/AI, CUDA, RTD3 power mgmt)
- **RAM**: 64GB DDR5 (low swappiness, huge pages)
- **Desktop**: KDE Plasma X11 + SDDM

Works on any x86_64 hardware with or without NVIDIA GPU.

## Universal Package Managers (Compiled from Source)

NexusOS ships every major Linux package manager, compiled natively:

- **apt/dpkg/nala** — Debian/Ubuntu (.deb) — native
- **rpm/dnf/alien** — Fedora/RHEL (.rpm) — from repos
- **pacman** — Arch Linux (.pkg.tar.zst) — compiled from source
- **portage/emerge** — Gentoo (ebuilds) — compiled from source
- **apk-tools** — Alpine Linux (.apk) — compiled from source
- **xbps** — Void Linux (.xbps) — compiled from source
- **zypper/libzypp** — openSUSE (.rpm) — compiled from source
- **flatpak/snap/AppImage** — universal formats
- **Nix** — available via `sudo nexus-setup-nix`
- **pip, npm, cargo, gem, go** — language managers

## Build Dependencies

Installed automatically by the build script:

```
debootstrap squashfs-tools xorriso mtools
grub-efi-amd64-bin grub-efi-amd64-signed shim-signed
grub-pc-bin rsync dosfstools
```

## License

GPL-3.0 — See [LICENSE](LICENSE).
