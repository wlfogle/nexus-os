# NexusOS Installation Guide

## Prerequisites

### Base System
- **Pop!_OS 22.04 LTS NVIDIA** (for overlay mode)
- UEFI boot enabled
- Internet connection
- At least 100GB free storage

### Install Dependencies
```bash
sudo nala install zfsutils-linux debootstrap git curl wget
```

## Quick Start (Overlay Mode)

The simplest path — install NexusOS on top of your existing Pop!_OS system:

```bash
cd /home/loufogle/nexus-os/installer
sudo ./nexus-install.sh
```

The installer will:
1. Detect your hardware (NVIDIA GPU, CPU features, memory)
2. Run preflight checks (root, Pop!_OS detection, UEFI, internet, nala)
3. Prompt for installation profile (Gaming, Media, Complete, Developer, Custom)
4. Install selected components via nala and Docker
5. Configure AI services (Stella, Max Jr., Orchestrator)
6. Set up desktop environment (KDE Plasma + SDDM)
7. Install nexuspkg universal package manager

## Fresh Install (ZFS-on-Root)

For a clean installation to a dedicated disk:

```bash
sudo INSTALL_MODE=fresh TARGET_DISK=/dev/sdX ./nexus-install.sh
```

**Warning**: This will erase all data on the target disk.

### What Fresh Install Does
1. Partitions the target disk (EFI + bpool + rpool)
2. Creates ZFS pools with ZSTD compression and autotrim
3. Bootstraps Ubuntu 22.04 (jammy) via debootstrap
4. Adds System76 PPA for hardware support
5. Installs ZFSBootMenu as the UEFI bootloader
6. Installs all NexusOS components in chroot
7. Configures users, networking, and services

### Fresh Install Environment Variables
```bash
# Required
export INSTALL_MODE=fresh
export TARGET_DISK=/dev/sdX

# Optional (prompted interactively if not set)
export INSTALL_USERNAME=myuser
export INSTALL_HOSTNAME=nexus
export INSTALL_PASSWORD=mypassword
```

## Post-Installation

### Verify Services
```bash
# Check AI services
systemctl status nexus-orchestrator
systemctl status nexus-stella
systemctl status nexus-maxjr

# Check media stack (if installed)
docker compose -f /opt/nexus-os/media/docker-compose.yml ps

# Test nexuspkg
nexuspkg status
```

### Access Points
- **AI Orchestrator**: http://localhost:8600
- **Stella AI**: http://localhost:8601
- **Max Jr. AI**: http://localhost:8602
- **Organizr Dashboard**: http://localhost:8540 (if media stack installed)
- **Jellyfin**: http://localhost:8200 (if media stack installed)

### CLI Tools
```bash
nexus-control status    # System status overview
nexus-control health    # Health check all services
stella --status         # Security status
maxjr --optimize        # Performance optimization
nexuspkg search firefox # Universal package search
```

## Troubleshooting

### Installer Won't Start
- Must be run as root: `sudo ./nexus-install.sh`
- Must be on Pop!_OS 22.04: `lsb_release -a`
- Must have internet: `ping -c1 google.com`

### Package Installation Fails
```bash
sudo nala update
sudo nala install -f
sudo nala clean
```

### Docker Services Not Starting
```bash
sudo systemctl start docker
sudo systemctl enable docker
docker compose -f /opt/nexus-os/media/docker-compose.yml up -d
```

### Resume After Failure
If the installer crashes during fresh install, re-run it — it will resume from the last successful step using the `.install_state` file.

```bash
# To force a clean restart instead
rm -f .install_state
sudo INSTALL_MODE=fresh TARGET_DISK=/dev/sdX ./nexus-install.sh
```

## Uninstallation (Overlay Mode)

The overlay installation can be reversed by removing installed packages:

```bash
# Remove NexusOS-specific packages
sudo nala remove kde-plasma-desktop sddm
sudo nala autoremove

# Stop and remove Docker containers
docker compose -f /opt/nexus-os/media/docker-compose.yml down -v

# Remove NexusOS files
sudo rm -rf /opt/nexus-os
```

## Syntax Validation

```bash
bash -n nexus-install.sh
```
