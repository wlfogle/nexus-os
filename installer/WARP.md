# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in the installer directory.

## Project Overview

The NexusOS Installer (`nexus-install.sh`) transforms a Pop!_OS 22.04 LTS NVIDIA system into a full NexusOS environment, or performs a fresh ZFS-on-root install via debootstrap with ZFSBootMenu.

## Installation Modes

### Mode 1: Overlay Install (default)
Installs NexusOS components on top of an existing Pop!_OS system:
- KDE Plasma Desktop with SDDM (X11)
- Gaming stack (Steam, Lutris, Wine, GameMode, MangoHUD)
- Media stack (Docker + 65+ services)
- AI services (Ollama, Stella, Max Jr., Orchestrator)
- Development tools (Rust, Node.js, build tools)
- Optional ZFS data pool for media/docker/AI storage
- nexuspkg universal package manager

### Mode 2: Fresh Install with ZFS-on-root (advanced)
Full installation to a target disk:
- Partitions: EFI (512MB) + bpool (2GB) + rpool (rest)
- debootstrap Ubuntu 22.04 (jammy) base
- System76 PPA for system76-power
- ZFSBootMenu as UEFI bootloader (downloaded from get.zfsbootmenu.org/efi)
- All NexusOS components installed in chroot
- Resume support via state file on failure

## Quick Start

```bash
# Overlay install (on existing Pop!_OS)
sudo ./nexus-install.sh

# Fresh install with ZFS root
sudo INSTALL_MODE=fresh TARGET_DISK=/dev/sdX ./nexus-install.sh
```

## Development Commands

### Syntax Check
```bash
bash -n nexus-install.sh
```

### Testing in QEMU
Use mobalivecd-linux from `reference/mobalivecd-linux/` to test ISOs built from fresh installs.

## Code Architecture

### Script Structure
- **Configuration** (lines 1-65): Variables, defaults, environment overrides
- **Logging & Error Handling** (lines 89-145): log(), warn(), die(), state management, cleanup trap
- **UI Functions** (lines 147-181): print helpers, confirm_prompt()
- **Hardware Detection** (lines 183-219): NVIDIA, AVX2, i9-13900HX, memory, storage
- **Preflight Checks** (lines 221-305): root, Pop!_OS detection, UEFI, internet, nala
- **User Interaction** (lines 307-514): mode, profile, component, disk selection
- **ZFS Functions** (lines 516-601): shared ZFS package install, data pool creation
- **Overlay Mode** (lines 603-957): prepare, KDE, gaming, media, AI, dev
- **Fresh Mode** (lines 959-1482): disk prep, ZFS pools, debootstrap, chroot script generation
- **Desktop Integration** (lines 1484-1562): nexuspkg, desktop entries, NexusOS config
- **Summary & Main** (lines 1564-1717): summary display, main flow with resume support

### Key Patterns
- All package management uses `nala` (apt frontend), never raw `apt` or `pacman`
- Conditional installation based on profile/component selection flags
- Fresh install generates a self-contained chroot script via heredoc appending
- Resume support in fresh mode via state file (`.install_state`)
- Hardware-adaptive: ZSTD compression if AVX2, i9-13900HX sysctl tuning
- Error handling: `die()` triggers cleanup trap, `warn()` for non-fatal issues

### Environment Variables
- `INSTALL_MODE`: "overlay" or "fresh"
- `TARGET_DISK`: Device path for fresh install (e.g., /dev/sda)
- `INSTALL_USERNAME`: User account name (defaults to SUDO_USER)
- `INSTALL_HOSTNAME`: Hostname for fresh install
- `INSTALL_PASSWORD`: User password for fresh install

### ZFS Dataset Layout (Fresh Install)
```
rpool/
├── ROOT/
│   └── nexusos/       (mountpoint: /)
├── home/              (mountpoint: /home)
├── var-log/           (mountpoint: /var/log)
├── var-cache/         (mountpoint: /var/cache)
├── tmp/               (mountpoint: /tmp)
└── opt-nexus/         (mountpoint: /opt/nexus-os)

bpool/
└── BOOT/
    └── default/       (mountpoint: /boot)
```

### ZFS Data Pool Layout (Overlay Mode, Optional)
```
nexus-data/
├── media/             (mountpoint: ~/nexus-media/media)
├── downloads/         (mountpoint: ~/nexus-media/downloads)
├── docker/            (mountpoint: /var/lib/docker)
└── ai-models/         (mountpoint: /opt/nexus-os/models)
```

## Dependencies

**Pop!_OS/Ubuntu packages** (installed via nala):
- `zfsutils-linux` — ZFS tools
- `kde-plasma-desktop sddm` — KDE desktop
- `docker-ce` — Docker (from official Docker repo)
- `steam-installer lutris gamemode mangohud wine64` — Gaming
- `python3-pip fastapi uvicorn` — AI service runtime
- `debootstrap` — Fresh install only

## Testing Strategy

### Syntax Validation
```bash
bash -n nexus-install.sh
```

### Overlay Mode Testing
Run on a Pop!_OS 22.04 VM or test system. The overlay install is reversible — packages can be removed.

### Fresh Mode Testing
Use QEMU with a virtual disk:
```bash
qemu-img create -f qcow2 test-disk.qcow2 64G
# Boot a live Pop!_OS ISO in QEMU, then run the installer targeting the virtual disk
```

### ISO Testing with MobaLiveCD
```bash
cd ../reference/mobalivecd-linux
python3 main.py /path/to/nexus-os.iso
```
