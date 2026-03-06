# NexusOS Installer

The NexusOS installer (`nexus-install.sh`) transforms a Pop!_OS 22.04 LTS NVIDIA system into a full NexusOS environment, or performs a fresh ZFS-on-root installation via debootstrap with ZFSBootMenu.

## Installation Modes

### Mode 1: Overlay Install (Default)

Installs NexusOS components on top of an existing Pop!_OS system:

- KDE Plasma Desktop with SDDM (X11)
- Gaming stack (Steam, Lutris, Wine, GameMode, MangoHUD)
- Media stack (Docker + 65+ services)
- AI services (Ollama, Stella, Max Jr., Orchestrator)
- Development tools (Rust, Node.js, build tools)
- Optional ZFS data pool for media/docker/AI storage
- nexuspkg universal package manager

```bash
sudo ./nexus-install.sh
```

### Mode 2: Fresh Install with ZFS-on-Root (Advanced)

Full installation to a target disk:

- Partitions: EFI (512MB) + bpool (2GB) + rpool (rest)
- debootstrap Ubuntu 22.04 (jammy) base
- System76 PPA for system76-power
- ZFSBootMenu as UEFI bootloader
- All NexusOS components installed in chroot
- Resume support via state file on failure

```bash
sudo INSTALL_MODE=fresh TARGET_DISK=/dev/sdX ./nexus-install.sh
```

## Installation Profiles

The installer offers multiple profiles during setup:

| Profile | Description |
|---------|-------------|
| **Gaming** | Steam, Lutris, Wine, GameMode, MangoHUD, performance tweaks |
| **Media Server** | Docker + 65+ media services (Jellyfin, Sonarr, Radarr, etc.) |
| **Complete** | Everything — gaming, media, AI, development tools |
| **Developer** | Build tools, Rust, Node.js, Python, Docker |
| **Custom** | Pick individual components |

## System Requirements

### Minimum
- Pop!_OS 22.04 LTS NVIDIA (for overlay mode)
- 8GB RAM (16GB recommended for media stack)
- 100GB free storage (500GB recommended)
- Internet connection

### Recommended (Target Hardware)
- Intel i9-13900HX + NVIDIA RTX 4080
- 64GB DDR5 RAM
- NVMe SSD with ZFS support

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `INSTALL_MODE` | `overlay` | `overlay` or `fresh` |
| `TARGET_DISK` | — | Device path for fresh install (e.g., `/dev/sda`) |
| `INSTALL_USERNAME` | `$SUDO_USER` | User account name |
| `INSTALL_HOSTNAME` | `nexus` | Hostname for fresh install |
| `INSTALL_PASSWORD` | — | User password for fresh install |

## ZFS Dataset Layouts

### Fresh Install
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

### Overlay Mode (Optional Data Pool)
```
nexus-data/
├── media/             (mountpoint: ~/nexus-media/media)
├── downloads/         (mountpoint: ~/nexus-media/downloads)
├── docker/            (mountpoint: /var/lib/docker)
└── ai-models/         (mountpoint: /opt/nexus-os/models)
```

## Dependencies

All packages installed via nala (Pop!_OS/Ubuntu):

- `zfsutils-linux` — ZFS tools
- `kde-plasma-desktop sddm` — KDE desktop
- `docker-ce` — Docker (from official Docker repo)
- `steam-installer lutris gamemode mangohud wine64` — Gaming
- `python3-pip fastapi uvicorn` — AI service runtime
- `debootstrap` — Fresh install only

## Troubleshooting

### Overlay Install Issues

**Installer fails preflight checks:**
- Ensure running on Pop!_OS 22.04 with NVIDIA drivers
- Run as root: `sudo ./nexus-install.sh`
- Check internet connectivity

**Package installation failures:**
```bash
# Update package lists
sudo nala update

# Fix broken packages
sudo nala install -f
```

### Fresh Install Issues

**ZFS pool creation fails:**
- Verify target disk is not mounted: `lsblk`
- Check ZFS modules loaded: `lsmod | grep zfs`
- Install ZFS if needed: `sudo nala install zfsutils-linux`

**Resume after failure:**
The installer saves state to `.install_state`. Re-running will resume from the last successful step.

### Testing in QEMU

```bash
# Create virtual disk
qemu-img create -f qcow2 test-disk.qcow2 64G

# Boot a live Pop!_OS ISO, then run installer targeting the virtual disk
```

## License

GPL-3.0+ — Same as the NexusOS project.
