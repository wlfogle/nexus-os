# 🚀 Universal ZFS Installer - Bulletproof Edition

**Professional-grade installer that works from ANY Ubuntu-based live environment**

Install a complete Kubuntu 24.04 system with ZFS root filesystem, desktop environment, gaming stack, AI tools, media server, and development environment - all configured and ready to use after first boot.

---

## ✨ Key Features

### 🛡️ Bulletproof Design
- **Comprehensive error handling** - Fails gracefully with detailed logging
- **Pre-flight validation** - Checks all requirements before making changes
- **Resume support** - Can continue from failure points
- **Automatic rollback** - Cleans up on failure
- **Works from any Ubuntu flavor** - Ubuntu, Kubuntu, Xubuntu, Lubuntu, etc.

### 📦 Complete System
- **Kubuntu 24.04 LTS** with ZFS root filesystem
- **ZFSBootMenu** - Advanced boot management (no GRUB)
- **KDE Plasma Desktop** - Full-featured desktop environment
- **NVIDIA Drivers** (auto-detected)
- **Gaming Stack** - Steam, Lutris, Wine, GameMode, MangoHUD
- **Docker + Compose** - Container platform
- **AI/ML Tools** - Ollama, Python, Jupyter
- **Media Stack** - Jellyfin, qBittorrent, Radarr, Sonarr, Jackett
- **Dev Tools** - Rust, Node.js, Git, and more
- **WireGuard VPN** - Pre-configured
- **Hardware Optimizations** - CPU-specific tuning

---

## 🚀 Quick Start

### Prerequisites
- UEFI boot mode (required)
- 32GB+ disk space (100GB+ recommended)
- 4GB+ RAM (8GB+ recommended)
- Active internet connection
- Ubuntu-based live USB (any flavor)

### Installation

1. **Boot from USB**
   - Boot any Ubuntu flavor live USB in UEFI mode

2. **Download installer**
   ```bash
   wget https://raw.githubusercontent.com/YOUR_USERNAME/universal-zfs-installer/main/install.sh
   chmod +x install.sh
   ```

3. **Set target disk**
   ```bash
   # List available disks
   lsblk -d -o NAME,SIZE,TYPE,MODEL
   
   # Set your target disk
   export TARGET_DISK=/dev/nvme0n1  # Change to your disk!
   ```

4. **Optional: Customize settings**
   ```bash
   export USERNAME="myuser"
   export USER_PASSWORD="securepassword"
   export HOSTNAME="mycomputer"
   ```

5. **Run installer**
   ```bash
   sudo ./install.sh
   ```

6. **Wait and reboot**
   - Installation takes 15-30 minutes
   - Type `YES` when prompted
   - Reboot when complete

---

## ⚙️ Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TARGET_DISK` | *none* | **Required** - Target disk (e.g. `/dev/sda`) |
| `USERNAME` | `kubuntu` | Username for the new system |
| `USER_PASSWORD` | `kubuntu` | Password for user and root |
| `HOSTNAME` | `powerhouse` | System hostname |
| `TARGET_DISTRO` | `kubuntu` | Target distribution (future use) |
| `TARGET_VERSION` | `24.04` | Ubuntu version |

### Example: Custom Installation

```bash
export TARGET_DISK=/dev/nvme0n1
export USERNAME="john"
export USER_PASSWORD="MySecurePass123"
export HOSTNAME="workstation"

sudo -E ./install.sh
```

---

## 📊 What Gets Installed

### Desktop Environment
- KDE Plasma 5.x desktop
- SDDM display manager
- Firefox web browser
- Konsole terminal
- Dolphin file manager
- Kate text editor
- Network Manager with Plasma integration

### Gaming (Optional - based on detection)
- Steam client
- Lutris game manager
- Wine (64-bit and 32-bit)
- Winetricks
- GameMode - Performance optimizer
- MangoHUD - Performance overlay
- Vulkan support

### NVIDIA (Auto-detected)
- NVIDIA driver 550 (or 535 fallback)
- NVIDIA DKMS modules
- CUDA toolkit (optional)

### Docker & Containers
- Docker Engine
- Docker Compose V2
- Pre-configured media stack:
  - **Jellyfin** - Media server (port 8096)
  - **qBittorrent** - Download client (port 8080)
  - **Radarr** - Movie management (port 7878)
  - **Sonarr** - TV management (port 8989)
  - **Jackett** - Indexer proxy (port 9117)

### AI/ML Stack
- Ollama service (port 11434)
- Python 3 with pip and venv
- Jupyter Notebook
- Development libraries

### Development Tools
- **Languages**: Rust (rustup), Node.js, npm
- **Version Control**: Git, Git LFS
- **Editors**: Neovim, Vim, Nano
- **Monitoring**: htop, btop
- **Multiplexers**: tmux, screen
- **Build Tools**: cmake, pkg-config, build-essential

### Security & Networking
- WireGuard VPN (pre-configured)
- UFW firewall (ready to configure)
- SSH server

---

## 🔧 Post-Installation

### First Boot

1. **Login** with your username/password
2. **Start media stack** (optional):
   ```bash
   cd ~/media-stack
   docker compose up -d
   ```

3. **Start Ollama** (if installed):
   ```bash
   sudo systemctl start ollama
   ollama pull codellama  # Download models
   ```

### Access Services

| Service | URL | Notes |
|---------|-----|-------|
| Jellyfin | http://localhost:8096 | Set up on first access |
| qBittorrent | http://localhost:8080 | Default: admin/adminadmin |
| Radarr | http://localhost:7878 | No auth by default |
| Sonarr | http://localhost:8989 | No auth by default |
| Jackett | http://localhost:9117 | No auth by default |
| Ollama API | http://localhost:11434 | REST API |

### File Locations

```
~/media-stack/          # Docker Compose config
~/media-stack/docker-compose.yml
~/.config/              # Application configs
/mnt/media/             # Media storage
/mnt/media/movies/
/mnt/media/tv/
/mnt/media/downloads/
```

---

## 🛠️ Troubleshooting

### Installation Fails

**Check the log:**
```bash
cat install.log
```

**Resume from failure:**
The installer creates a state file (`.install_state`) that tracks progress. Simply re-run the installer to continue from where it failed.

### Common Issues

#### "System must be booted in UEFI mode"
- Reboot and ensure UEFI mode is enabled in BIOS
- Disable Legacy/CSM boot

#### "No internet connection"
- Check network cable or WiFi
- Test: `ping 8.8.8.8`

#### "Failed to load ZFS module"
- Host system needs ZFS support
- On Ubuntu: `sudo apt install zfsutils-linux`

#### "Disk too small"
- Minimum 32GB required
- 100GB+ recommended for full installation

### ZFS Commands

```bash
# List pools
zpool list

# Pool status
zpool status

# List datasets
zfs list

# Check compression ratio
zfs get compressratio rpool

# Create snapshot
zfs snapshot rpool/ROOT/ubuntu@backup

# Rollback snapshot
zfs rollback rpool/ROOT/ubuntu@backup
```

### Docker Issues

```bash
# Check Docker status
sudo systemctl status docker

# View container logs
cd ~/media-stack
docker compose logs

# Restart containers
docker compose restart

# Stop all containers
docker compose down
```

---

## 🎯 Hardware Optimizations

### Automatic Detections

- **NVIDIA GPUs** - Installs appropriate drivers
- **AVX2 CPUs** - Uses ZSTD compression (better than LZ4)
- **Intel i9-13900HX** - Applies specific kernel tuning

### Custom Optimizations

Edit `/etc/sysctl.d/99-custom.conf` after installation to add your own tuning.

---

## 📋 Technical Details

### Disk Layout

| Partition | Size | Type | Filesystem | Mount Point |
|-----------|------|------|------------|-------------|
| 1 | 512MB | EF00 | FAT32 | /boot/efi |
| 2 | 2GB | BF00 | ZFS (bpool) | /boot |
| 3 | Rest | BF00 | ZFS (rpool) | / |

### ZFS Datasets

```
rpool/ROOT/ubuntu       # Root filesystem
rpool/home              # User home directories
rpool/home/root         # Root home
rpool/var-log           # System logs
rpool/var-cache         # Cache directory
rpool/tmp               # Temporary files
bpool/BOOT/default      # Boot files
```

### ZFS Features

- **Compression**: ZSTD (AVX2) or LZ4
- **Deduplication**: Disabled (too memory-intensive)
- **Autotrim**: Enabled (for SSDs)
- **Snapshots**: Ready to use
- **ACL**: POSIX ACLs enabled

---

## 🤝 Contributing

Contributions welcome! Areas for improvement:

- Additional target distributions (Fedora, Arch, Debian)
- More desktop environments (GNOME, XFCE, MATE)
- Additional hardware optimizations
- Enhanced service configurations
- Documentation improvements

---

## ⚠️ Warnings

- **This ERASES the target disk completely**
- **Backup all data before running**
- **Requires active internet connection**
- **Takes 15-30 minutes to complete**
- **UEFI boot mode is mandatory**

---

## 📜 License

MIT License - Use freely, modify as needed.

---

## 🙏 Credits

Built on amazing open source projects:

- **OpenZFS** - Advanced filesystem
- **Ubuntu/Canonical** - Base operating system
- **ZFSBootMenu** - Boot management
- **KDE Project** - Desktop environment
- **Docker** - Containerization
- **LinuxServer.io** - Container images
- **Ollama** - AI model serving

---

## 📞 Support

- **Issues**: Open an issue on GitHub
- **Questions**: Start a discussion
- **Success Stories**: We'd love to hear them!

---

**Built for power users who demand reliability.**

🚀 **One script. Bulletproof installation. Everything works.**
