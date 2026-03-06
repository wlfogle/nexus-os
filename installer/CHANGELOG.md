# Installer Changelog

All notable changes to the NexusOS installer will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [2.0.0] - 2026-03-06

### Complete Rewrite for Pop!_OS

The installer has been completely rewritten as `nexus-install.sh` — a unified Bash installer supporting both overlay and fresh ZFS-on-root installation modes on Pop!_OS 22.04 LTS NVIDIA.

### Added
- **Dual installation modes**: Overlay (on existing Pop!_OS) and Fresh (ZFS-on-root via debootstrap)
- **Installation profiles**: Gaming, Media Server, Complete, Developer, Custom
- **Hardware detection**: NVIDIA GPU, AVX2, i9-13900HX, memory, storage auto-detection
- **Preflight checks**: Root, Pop!_OS detection, UEFI, internet, nala availability
- **AI services integration**: Ollama, Stella, Max Jr., Orchestrator with systemd units
- **Media stack deployment**: Docker Compose with 65+ services and .env.template
- **Gaming stack**: Steam, Lutris, Wine, GameMode, MangoHUD via nala
- **KDE Plasma desktop**: SDDM display manager with X11 session
- **ZFS data pool**: Optional ZFS pool for media, downloads, Docker, AI models (overlay mode)
- **Fresh install ZFS**: Full ZFS-on-root with bpool/rpool, ZFSBootMenu UEFI bootloader
- **Resume support**: State file (`.install_state`) allows resuming after failure in fresh mode
- **nexuspkg integration**: Universal package manager installed to /usr/local/bin
- **Desktop entries**: .desktop files for Stella, Max Jr., NexusOS Control

### Changed
- **Base system**: Pop!_OS 22.04 LTS NVIDIA (was previously targeting Arch/Garuda)
- **Package manager**: All packages installed via nala (not pacman/apt)
- **Bootloader**: ZFSBootMenu (fresh mode) or existing Pop!_OS bootloader (overlay mode)
- **Installer type**: Single Bash script (was Calamares-based graphical installer)

### Removed
- Calamares graphical installer modules (zfspostcfg, zfs.conf, settings-zfs.conf)
- Arch Linux / Garuda Linux specific configurations
- pacman/mkinitcpio integration
- Qt6 build dependencies

### Technical Details
- **Script size**: 1717 lines of Bash
- **Language**: Bash with shellcheck-clean syntax
- **Dependencies**: nala, zfsutils-linux, docker-ce, debootstrap (fresh only)
- **Compatibility**: Pop!_OS 22.04 LTS NVIDIA (Ubuntu 22.04 base)

## [1.0.0] - 2024-09-17

### Initial Release (Legacy — Calamares-based)

Initial Calamares ZFS integration modules. This version has been fully replaced by the v2.0.0 rewrite.
