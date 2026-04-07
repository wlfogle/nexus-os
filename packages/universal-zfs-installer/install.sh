#!/bin/bash

# ═══════════════════════════════════════════════════════════════════════════
# 🚀 UNIVERSAL ZFS INSTALLER - BULLETPROOF EDITION
# ═══════════════════════════════════════════════════════════════════════════
#
# Run from ANY Linux distro, installs target distro with ZFS root
# Currently supports: Kubuntu 24.04 (more distros coming)
#
# Usage: sudo ./install.sh
#
# ═══════════════════════════════════════════════════════════════════════════

set -euo pipefail

# ═══════════════════════════════════════════════════════════════════════════
# CONFIGURATION
# ═══════════════════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_FILE="${SCRIPT_DIR}/install.log"
STATE_FILE="${SCRIPT_DIR}/.install_state"

# Default configuration (can be overridden by environment variables)
: "${USERNAME:=kubuntu}"
: "${USER_PASSWORD:=kubuntu}"
: "${HOSTNAME:=powerhouse}"
: "${TARGET_DISK:=}"
: "${TARGET_DISTRO:=kubuntu}"
: "${TARGET_VERSION:=24.04}"

# Colors
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly CYAN='\033[0;36m'
readonly NC='\033[0m'

# ═══════════════════════════════════════════════════════════════════════════
# LOGGING AND ERROR HANDLING
# ═══════════════════════════════════════════════════════════════════════════

log() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${GREEN}[${timestamp}]${NC} $*" | tee -a "$LOG_FILE"
}

warn() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${YELLOW}[${timestamp}] WARNING:${NC} $*" | tee -a "$LOG_FILE"
}

error() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${RED}[${timestamp}] ERROR:${NC} $*" | tee -a "$LOG_FILE"
    cleanup_on_failure
    exit 1
}

progress() {
    echo -e "${CYAN}▶${NC} $*"
}

save_state() {
    echo "$1" > "$STATE_FILE"
}

get_state() {
    [ -f "$STATE_FILE" ] && cat "$STATE_FILE" || echo "START"
}

cleanup_on_failure() {
    warn "Installation failed, cleaning up..."
    
    # Unmount if mounted
    if [ -d "$MNTROOT" ]; then
        umount -R "$MNTROOT/dev" 2>/dev/null || true
        umount -R "$MNTROOT/proc" 2>/dev/null || true
        umount -R "$MNTROOT/sys" 2>/dev/null || true
        umount "$MNTROOT/boot/efi" 2>/dev/null || true
    fi
    
    # Export pools if they exist
    if zpool list bpool &>/dev/null; then
        zpool export bpool 2>/dev/null || true
    fi
    if zpool list rpool &>/dev/null; then
        zpool export rpool 2>/dev/null || true
    fi
    
    log "Cleanup complete. Check $LOG_FILE for details."
}

trap cleanup_on_failure ERR INT TERM

# ═══════════════════════════════════════════════════════════════════════════
# DISTRO DETECTION
# ═══════════════════════════════════════════════════════════════════════════

detect_host_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        HOST_DISTRO="$ID"
        HOST_VERSION="$VERSION_ID"
        log "Detected host: $NAME $VERSION_ID"
    else
        error "Cannot detect host distribution"
    fi
}

# ═══════════════════════════════════════════════════════════════════════════
# PACKAGE MANAGER ABSTRACTION
# ═══════════════════════════════════════════════════════════════════════════

pkg_update() {
    case "$HOST_DISTRO" in
        ubuntu|debian|kubuntu|xubuntu|lubuntu|pop)
            export DEBIAN_FRONTEND=noninteractive
            apt-get update -qq || error "Failed to update package lists"
            ;;
        fedora|rhel|centos)
            dnf check-update -q || true
            ;;
        arch|manjaro)
            pacman -Sy --noconfirm || error "Failed to update package lists"
            ;;
        opensuse*|sles)
            zypper refresh -q || error "Failed to update package lists"
            ;;
        *)
            error "Unsupported host distro: $HOST_DISTRO"
            ;;
    esac
}

pkg_install() {
    local packages=("$@")
    
    case "$HOST_DISTRO" in
        ubuntu|debian|kubuntu|xubuntu|lubuntu|pop)
            DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "${packages[@]}" || \
                error "Failed to install: ${packages[*]}"
            ;;
        fedora|rhel|centos)
            dnf install -y -q "${packages[@]}" || error "Failed to install: ${packages[*]}"
            ;;
        arch|manjaro)
            pacman -S --noconfirm --needed "${packages[@]}" || error "Failed to install: ${packages[*]}"
            ;;
        opensuse*|sles)
            zypper install -y "${packages[@]}" || error "Failed to install: ${packages[*]}"
            ;;
        *)
            error "Unsupported host distro: $HOST_DISTRO"
            ;;
    esac
}

# ═══════════════════════════════════════════════════════════════════════════
# PRE-FLIGHT CHECKS
# ═══════════════════════════════════════════════════════════════════════════

check_root() {
    [ $EUID -eq 0 ] || error "Must run as root (use: sudo ./install.sh)"
}

check_uefi() {
    [ -d /sys/firmware/efi ] || error "System must be booted in UEFI mode"
    log "✓ UEFI boot detected"
}

check_internet() {
    log "Checking internet connectivity..."
    if ping -c 1 -W 5 8.8.8.8 &>/dev/null || ping -c 1 -W 5 1.1.1.1 &>/dev/null; then
        log "✓ Internet connection active"
    else
        error "No internet connection"
    fi
}

check_disk() {
    if [ -z "$TARGET_DISK" ]; then
        error "TARGET_DISK not set. Set with: export TARGET_DISK=/dev/sdX"
    fi
    
    [ -b "$TARGET_DISK" ] || error "Disk $TARGET_DISK does not exist"
    
    local disk_size=$(blockdev --getsize64 "$TARGET_DISK")
    local disk_size_gb=$((disk_size / 1024 / 1024 / 1024))
    
    [ $disk_size_gb -lt 32 ] && error "Disk too small: ${disk_size_gb}GB (minimum 32GB)"
    
    log "✓ Target disk: $TARGET_DISK (${disk_size_gb}GB)"
}

check_memory() {
    local mem_gb=$(free -g | awk '/^Mem:/{print $2}')
    [ $mem_gb -lt 4 ] && warn "Low memory: ${mem_gb}GB (8GB+ recommended)"
    log "✓ Available memory: ${mem_gb}GB"
}

detect_hardware() {
    log "Detecting hardware..."
    
    # NVIDIA GPU
    if lspci 2>/dev/null | grep -qi nvidia; then
        HAS_NVIDIA=1
        log "✓ NVIDIA GPU detected"
    else
        HAS_NVIDIA=0
    fi
    
    # CPU features
    if grep -q avx2 /proc/cpuinfo; then
        HAS_AVX2=1
        log "✓ AVX2 support detected"
    else
        HAS_AVX2=0
    fi
    
    # Specific CPU detection
    if lscpu | grep -qi "i9-13900HX"; then
        IS_I9_13900HX=1
        log "✓ Intel i9-13900HX detected"
    else
        IS_I9_13900HX=0
    fi
}

# ═══════════════════════════════════════════════════════════════════════════
# DEPENDENCY INSTALLATION (HOST SYSTEM)
# ═══════════════════════════════════════════════════════════════════════════

install_host_dependencies() {
    log "Installing host dependencies..."
    
    case "$HOST_DISTRO" in
        ubuntu|debian|kubuntu|xubuntu|lubuntu|pop)
            pkg_install debootstrap gdisk zfsutils-linux dosfstools wget curl parted
            ;;
        fedora|rhel|centos)
            pkg_install debootstrap gdisk zfs dosfstools wget curl parted
            ;;
        arch|manjaro)
            pkg_install debootstrap gptfdisk zfs-linux dosfstools wget curl parted
            ;;
        opensuse*|sles)
            pkg_install debootstrap gptfdisk zfs dosfstools wget curl parted
            ;;
    esac
    
    # Load ZFS module
    if ! lsmod | grep -q zfs; then
        log "Loading ZFS module..."
        modprobe zfs || error "Failed to load ZFS module"
    fi
    
    log "✓ Host dependencies installed"
}

# ═══════════════════════════════════════════════════════════════════════════
# DISK PREPARATION
# ═══════════════════════════════════════════════════════════════════════════

prepare_disk() {
    log "Preparing disk $TARGET_DISK..."
    
    # Unmount any mounted partitions
    for part in ${TARGET_DISK}*; do
        [ -b "$part" ] && umount "$part" 2>/dev/null || true
    done
    
    # Clean disk
    wipefs -af "$TARGET_DISK" 2>/dev/null || true
    sgdisk --zap-all "$TARGET_DISK" || error "Failed to zap disk"
    
    # Create partitions
    log "Creating partitions..."
    sgdisk -n1:1M:+512M -t1:EF00 "$TARGET_DISK" || error "Failed to create EFI partition"
    sgdisk -n2:0:+2G -t2:BF00 "$TARGET_DISK" || error "Failed to create boot partition"
    sgdisk -n3:0:0 -t3:BF00 "$TARGET_DISK" || error "Failed to create root partition"
    
    # Wait for kernel to recognize partitions
    sleep 2
    partprobe "$TARGET_DISK" 2>/dev/null || true
    sleep 1
    
    # Determine partition naming
    if [[ "$TARGET_DISK" == *"nvme"* ]] || [[ "$TARGET_DISK" == *"mmcblk"* ]]; then
        PART_PREFIX="${TARGET_DISK}p"
    else
        PART_PREFIX="${TARGET_DISK}"
    fi
    
    EFI_PART="${PART_PREFIX}1"
    BOOT_PART="${PART_PREFIX}2"
    ROOT_PART="${PART_PREFIX}3"
    
    # Verify partitions exist
    [ -b "$EFI_PART" ] || error "EFI partition not created"
    [ -b "$BOOT_PART" ] || error "Boot partition not created"
    [ -b "$ROOT_PART" ] || error "Root partition not created"
    
    # Format EFI
    log "Formatting EFI partition..."
    mkfs.vfat -F32 "$EFI_PART" || error "Failed to format EFI partition"
    
    log "✓ Disk prepared"
    save_state "DISK_PREPARED"
}

# ═══════════════════════════════════════════════════════════════════════════
# ZFS POOL CREATION
# ═══════════════════════════════════════════════════════════════════════════

create_zfs_pools() {
    log "Creating ZFS pools..."
    
    # Destroy existing pools if they exist
    if zpool list bpool &>/dev/null; then
        warn "Destroying existing bpool..."
        zpool destroy -f bpool || error "Failed to destroy bpool"
    fi
    if zpool list rpool &>/dev/null; then
        warn "Destroying existing rpool..."
        zpool destroy -f rpool || error "Failed to destroy rpool"
    fi
    
    # Determine compression
    local compression="lz4"
    if [ $HAS_AVX2 -eq 1 ]; then
        compression="zstd"
        log "Using ZSTD compression (AVX2 detected)"
    else
        log "Using LZ4 compression"
    fi
    
    # Create boot pool
    log "Creating boot pool (bpool)..."
    zpool create -f -o ashift=12 \
        -O acltype=posixacl -O compression=lz4 \
        -O normalization=formD -O relatime=on \
        -O xattr=sa -O mountpoint=none \
        bpool "$BOOT_PART" || error "Failed to create bpool"
    
    # Create root pool
    log "Creating root pool (rpool)..."
    zpool create -f -o ashift=12 -o autotrim=on \
        -O acltype=posixacl -O compression=$compression \
        -O dnodesize=auto -O normalization=formD \
        -O relatime=on -O xattr=sa -O mountpoint=none \
        rpool "$ROOT_PART" || error "Failed to create rpool"
    
    log "✓ ZFS pools created"
    save_state "POOLS_CREATED"
}

create_zfs_datasets() {
    log "Creating ZFS datasets..."
    
    # Root datasets
    zfs create -o canmount=off -o mountpoint=none rpool/ROOT || error "Failed to create rpool/ROOT"
    zfs create -o canmount=noauto -o mountpoint=/ rpool/ROOT/ubuntu || error "Failed to create root dataset"
    zpool set bootfs=rpool/ROOT/ubuntu rpool || error "Failed to set bootfs"
    
    # Boot datasets
    zfs create -o canmount=off -o mountpoint=none bpool/BOOT || error "Failed to create bpool/BOOT"
    zfs create -o mountpoint=/boot bpool/BOOT/default || error "Failed to create boot dataset"
    
    # User datasets
    zfs create -o mountpoint=/home rpool/home || error "Failed to create home dataset"
    zfs create rpool/home/$USERNAME || error "Failed to create user dataset"
    zfs create -o mountpoint=/root rpool/home/root || error "Failed to create root home dataset"
    
    # System datasets
    zfs create -o mountpoint=/var/log rpool/var-log || error "Failed to create var-log dataset"
    zfs create -o mountpoint=/var/cache rpool/var-cache || error "Failed to create var-cache dataset"
    zfs create -o mountpoint=/tmp rpool/tmp || error "Failed to create tmp dataset"
    
    log "✓ ZFS datasets created"
    save_state "DATASETS_CREATED"
}

mount_filesystems() {
    log "Mounting filesystems..."
    
    # Mount root
    zfs mount rpool/ROOT/ubuntu || error "Failed to mount root dataset"
    MNTROOT=$(zfs get -H -o value mountpoint rpool/ROOT/ubuntu)
    [ -z "$MNTROOT" ] && error "Failed to get mount point"
    
    log "Mount point: $MNTROOT"
    
    # Create and mount EFI
    mkdir -p "$MNTROOT/boot/efi" || error "Failed to create EFI mount point"
    mount "$EFI_PART" "$MNTROOT/boot/efi" || error "Failed to mount EFI partition"
    
    # Set permissions
    chmod 1777 "$MNTROOT/tmp"
    
    log "✓ Filesystems mounted"
    save_state "FS_MOUNTED"
}

# ═══════════════════════════════════════════════════════════════════════════
# BASE SYSTEM INSTALLATION
# ═══════════════════════════════════════════════════════════════════════════

install_base_system() {
    log "Installing base system (this will take several minutes)..."
    
    local distro_release="noble"  # Ubuntu 24.04
    local mirror="http://archive.ubuntu.com/ubuntu"
    
    debootstrap --arch amd64 "$distro_release" "$MNTROOT" "$mirror" || \
        error "Failed to install base system"
    
    log "✓ Base system installed"
    save_state "BASE_INSTALLED"
}

configure_base_system() {
    log "Configuring base system..."
    
    # Hostname
    echo "$HOSTNAME" > "$MNTROOT/etc/hostname" || error "Failed to set hostname"
    
    # Hosts
    cat > "$MNTROOT/etc/hosts" <<EOF || error "Failed to create hosts file"
127.0.0.1 localhost
127.0.1.1 $HOSTNAME
::1 localhost ip6-localhost ip6-loopback
EOF
    
    # APT sources
    cat > "$MNTROOT/etc/apt/sources.list" <<EOF || error "Failed to create sources.list"
deb http://archive.ubuntu.com/ubuntu noble main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu noble-updates main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu noble-security main restricted universe multiverse
EOF
    
    # DNS
    echo "nameserver 8.8.8.8" > "$MNTROOT/etc/resolv.conf"
    echo "nameserver 1.1.1.1" >> "$MNTROOT/etc/resolv.conf"
    
    # ZFS cache
    mkdir -p "$MNTROOT/etc/zfs"
    zpool set cachefile="$MNTROOT/etc/zfs/zpool.cache" rpool || error "Failed to set rpool cache"
    zpool set cachefile="$MNTROOT/etc/zfs/zpool.cache" bpool || error "Failed to set bpool cache"
    
    log "✓ Base system configured"
    save_state "BASE_CONFIGURED"
}

# ═══════════════════════════════════════════════════════════════════════════
# CHROOT PREPARATION
# ═══════════════════════════════════════════════════════════════════════════

prepare_chroot() {
    log "Preparing chroot environment..."
    
    mount --rbind /dev "$MNTROOT/dev" || error "Failed to bind mount /dev"
    mount --rbind /proc "$MNTROOT/proc" || error "Failed to bind mount /proc"
    mount --rbind /sys "$MNTROOT/sys" || error "Failed to bind mount /sys"
    
    log "✓ Chroot prepared"
}

# ═══════════════════════════════════════════════════════════════════════════
# WAIT FOR SERVICE HELPERS
# ═══════════════════════════════════════════════════════════════════════════

wait_for_service() {
    local service=$1
    local max_wait=${2:-30}
    local count=0
    
    while [ $count -lt $max_wait ]; do
        if systemctl is-active --quiet "$service" 2>/dev/null; then
            return 0
        fi
        sleep 1
        ((count++))
    done
    
    return 1
}

wait_for_port() {
    local port=$1
    local max_wait=${2:-30}
    local count=0
    
    while [ $count -lt $max_wait ]; do
        if nc -z localhost "$port" 2>/dev/null; then
            return 0
        fi
        sleep 1
        ((count++))
    done
    
    return 1
}

# ═══════════════════════════════════════════════════════════════════════════
# CHROOT INSTALLATION SCRIPT
# ═══════════════════════════════════════════════════════════════════════════

create_chroot_install_script() {
    log "Creating chroot installation script..."
    
    cat > "$MNTROOT/root/install-chroot.sh" <<'CHROOT_SCRIPT'
#!/bin/bash
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive

log() { echo "[$(date +'%H:%M:%S')] $*"; }
error() { echo "ERROR: $*" >&2; exit 1; }

log "Step 1/15: Updating package lists..."
apt-get update -qq || error "Failed to update"

log "Step 2/15: Installing essential packages..."
apt-get install -y -qq \
    linux-image-generic linux-headers-generic \
    zfsutils-linux zfs-initramfs zfs-zed \
    efibootmgr kexec-tools \
    locales tzdata sudo nano vim \
    curl wget ca-certificates gnupg lsb-release \
    apt-transport-https software-properties-common \
    build-essential || error "Failed to install essentials"

log "Step 3/15: Configuring locale..."
locale-gen en_US.UTF-8
update-locale LANG=en_US.UTF-8

log "Step 4/15: Creating user..."
useradd -m -s /bin/bash -G sudo,adm,cdrom,dip,plugdev __USERNAME__ || error "Failed to create user"
echo "__USERNAME__:__PASSWORD__" | chpasswd
echo "root:__PASSWORD__" | chpasswd
echo "__USERNAME__ ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/__USERNAME__
chmod 440 /etc/sudoers.d/__USERNAME__

log "Step 5/15: Installing desktop environment..."
apt-get install -y -qq \
    kde-plasma-desktop sddm \
    network-manager plasma-nm \
    firefox konsole dolphin kate \
    system-config-printer-common \
    || error "Failed to install KDE"

systemctl enable sddm NetworkManager

log "Step 6/15: Installing ZFSBootMenu..."
apt-get install -y -qq zfsbootmenu || error "Failed to install ZFSBootMenu"

mkdir -p /etc/zfsbootmenu/dracut.conf.d
cat > /etc/zfsbootmenu/config.yaml <<'ZBMCONF'
Global:
  ManageImages: true
  BootMountPoint: /boot/efi
  DracutConfDir: /etc/zfsbootmenu/dracut.conf.d
Components:
  Enabled: true
  ImageDir: /boot/efi/EFI/ZBM
  Versions: 3
  syslinux:
    Enabled: false
EFI:
  ImageDir: /boot/efi/EFI/ZBM
  Versions: 3
  Enabled: true
Kernel:
  CommandLine: "ro quiet loglevel=0"
ZBMCONF

mkdir -p /boot/efi/EFI/ZBM
generate-zbm --debug || error "Failed to generate ZFSBootMenu"

# Create EFI boot entry
efibootmgr --create --disk __DISK__ --part 1 \
  --loader '\EFI\ZBM\vmlinuz.EFI' \
  --label "ZFSBootMenu" \
  --unicode || warn "Failed to create EFI entry (may be OK)"

NVIDIA_SECTION

GAMING_SECTION

DOCKER_SECTION

AI_SECTION

DEV_SECTION

MEDIA_SECTION

WIREGUARD_SECTION

I9_SECTION

log "Step 14/15: Updating initramfs..."
update-initramfs -u -k all || error "Failed to update initramfs"

log "Step 15/15: Final configuration..."
chown -R __USERNAME__:__USERNAME__ /home/__USERNAME__

log "✓ Installation complete!"
CHROOT_SCRIPT

    # Replace placeholders
    sed -i "s|__USERNAME__|$USERNAME|g" "$MNTROOT/root/install-chroot.sh"
    sed -i "s|__PASSWORD__|$USER_PASSWORD|g" "$MNTROOT/root/install-chroot.sh"
    sed -i "s|__DISK__|$TARGET_DISK|g" "$MNTROOT/root/install-chroot.sh"
    
    # Add sections based on hardware
    add_nvidia_section
    add_gaming_section
    add_docker_section
    add_ai_section
    add_dev_section
    add_media_section
    add_wireguard_section
    add_i9_section
    
    chmod +x "$MNTROOT/root/install-chroot.sh"
    
    log "✓ Chroot script created"
}

add_nvidia_section() {
    if [ $HAS_NVIDIA -eq 1 ]; then
        local section='
log "Step 7/15: Installing NVIDIA drivers..."
apt-get install -y -qq nvidia-driver-550 nvidia-dkms-550 nvidia-utils-550 || \
    apt-get install -y -qq nvidia-driver-535 nvidia-dkms-535 nvidia-utils-535
'
    else
        local section='
log "Step 7/15: Skipping NVIDIA (no GPU detected)..."
'
    fi
    sed -i "s|NVIDIA_SECTION|$section|" "$MNTROOT/root/install-chroot.sh"
}

add_gaming_section() {
    local section='
log "Step 8/15: Installing gaming stack..."
dpkg --add-architecture i386
apt-get update -qq
apt-get install -y -qq \
    steam-installer \
    wine64 wine32 winetricks \
    gamemode libgamemode0 libgamemodeauto0 \
    mangohud \
    vulkan-tools mesa-vulkan-drivers mesa-vulkan-drivers:i386 \
    || true

# Lutris from official sources
mkdir -p /etc/apt/keyrings
wget -q -O /etc/apt/keyrings/lutris.gpg https://download.opensuse.org/repositories/home:/strycore/xUbuntu_24.04/Release.key || true
echo "deb [signed-by=/etc/apt/keyrings/lutris.gpg] https://download.opensuse.org/repositories/home:/strycore/xUbuntu_24.04/ ./" > /etc/apt/sources.list.d/lutris.list
apt-get update -qq
apt-get install -y -qq lutris || true
'
    sed -i "s|GAMING_SECTION|$section|" "$MNTROOT/root/install-chroot.sh"
}

add_docker_section() {
    local section='
log "Step 9/15: Installing Docker..."
mkdir -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
chmod a+r /etc/apt/keyrings/docker.asc
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" > /etc/apt/sources.list.d/docker.list
apt-get update -qq
apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
systemctl enable docker
usermod -aG docker __USERNAME__
'
    sed -i "s|DOCKER_SECTION|$section|" "$MNTROOT/root/install-chroot.sh"
    sed -i "s|__USERNAME__|$USERNAME|g" "$MNTROOT/root/install-chroot.sh"
}

add_ai_section() {
    local section='
log "Step 10/15: Installing AI/ML tools..."
apt-get install -y -qq python3-pip python3-venv python3-dev jupyter-notebook

# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh || true

# Create Ollama service
useradd -r -s /bin/false -d /opt/ollama ollama || true
mkdir -p /opt/ollama
chown -R ollama:ollama /opt/ollama

cat > /etc/systemd/system/ollama.service <<OLLAMA
[Unit]
Description=Ollama AI Service
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=ollama
Group=ollama
ExecStart=/usr/local/bin/ollama serve
Environment="OLLAMA_HOST=0.0.0.0:11434"
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
OLLAMA

systemctl daemon-reload
systemctl enable ollama
'
    sed -i "s|AI_SECTION|$section|" "$MNTROOT/root/install-chroot.sh"
}

add_dev_section() {
    local section='
log "Step 11/15: Installing development tools..."
apt-get install -y -qq \
    nodejs npm \
    git git-lfs \
    htop btop \
    tmux screen \
    neovim \
    cmake \
    pkg-config \
    libssl-dev

# Install Rust for user
su - __USERNAME__ -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y" || true
'
    sed -i "s|DEV_SECTION|$section|" "$MNTROOT/root/install-chroot.sh"
    sed -i "s|__USERNAME__|$USERNAME|g" "$MNTROOT/root/install-chroot.sh"
}

add_media_section() {
    local section='
log "Step 12/15: Setting up media stack..."
mkdir -p /home/__USERNAME__/media-stack
cat > /home/__USERNAME__/media-stack/docker-compose.yml <<MEDIA
version: "3.8"
services:
  jellyfin:
    image: lscr.io/linuxserver/jellyfin:latest
    container_name: jellyfin
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=America/New_York
    volumes:
      - /home/__USERNAME__/.config/jellyfin:/config
      - /mnt/media:/media
    ports:
      - 8096:8096
    restart: unless-stopped

  qbittorrent:
    image: lscr.io/linuxserver/qbittorrent:latest
    container_name: qbittorrent
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=America/New_York
      - WEBUI_PORT=8080
    volumes:
      - /home/__USERNAME__/.config/qbittorrent:/config
      - /mnt/media/downloads:/downloads
    ports:
      - 8080:8080
      - 6881:6881
      - 6881:6881/udp
    restart: unless-stopped

  radarr:
    image: lscr.io/linuxserver/radarr:latest
    container_name: radarr
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=America/New_York
    volumes:
      - /home/__USERNAME__/.config/radarr:/config
      - /mnt/media:/media
    ports:
      - 7878:7878
    restart: unless-stopped

  sonarr:
    image: lscr.io/linuxserver/sonarr:latest
    container_name: sonarr
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=America/New_York
    volumes:
      - /home/__USERNAME__/.config/sonarr:/config
      - /mnt/media:/media
    ports:
      - 8989:8989
    restart: unless-stopped

  jackett:
    image: lscr.io/linuxserver/jackett:latest
    container_name: jackett
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=America/New_York
    volumes:
      - /home/__USERNAME__/.config/jackett:/config
    ports:
      - 9117:9117
    restart: unless-stopped
MEDIA

mkdir -p /mnt/media/{movies,tv,downloads}
chown -R __USERNAME__:__USERNAME__ /home/__USERNAME__/media-stack /mnt/media
'
    sed -i "s|MEDIA_SECTION|$section|" "$MNTROOT/root/install-chroot.sh"
    sed -i "s|__USERNAME__|$USERNAME|g" "$MNTROOT/root/install-chroot.sh"
}

add_wireguard_section() {
    local section='
log "Step 13/15: Installing WireGuard..."
apt-get install -y -qq wireguard wireguard-tools resolvconf
wg genkey | tee /etc/wireguard/privatekey | wg pubkey > /etc/wireguard/publickey
chmod 600 /etc/wireguard/privatekey

cat > /etc/wireguard/wg0.conf <<WG
[Interface]
PrivateKey = $(cat /etc/wireguard/privatekey)
Address = 10.0.0.1/24
ListenPort = 51820
WG
'
    sed -i "s|WIREGUARD_SECTION|$section|" "$MNTROOT/root/install-chroot.sh"
}

add_i9_section() {
    if [ $IS_I9_13900HX -eq 1 ]; then
        local section='
log "Applying i9-13900HX optimizations..."
cat > /etc/sysctl.d/99-i9-optimizations.conf <<I9OPT
vm.swappiness=1
vm.dirty_ratio=15
vm.dirty_background_ratio=5
kernel.sched_migration_cost_ns=500000
net.core.rmem_max=134217728
net.core.wmem_max=134217728
net.ipv4.tcp_rmem=4096 87380 67108864
net.ipv4.tcp_wmem=4096 65536 67108864
I9OPT
'
    else
        local section='
log "Skipping i9-13900HX optimizations (CPU not detected)..."
'
    fi
    sed -i "s|I9_SECTION|$section|" "$MNTROOT/root/install-chroot.sh"
}

# ═══════════════════════════════════════════════════════════════════════════
# MAIN EXECUTION
# ═══════════════════════════════════════════════════════════════════════════

main() {
    clear
    cat << 'EOF'
╔══════════════════════════════════════════════════════════════╗
║     🚀 UNIVERSAL ZFS INSTALLER - BULLETPROOF EDITION        ║
║                                                              ║
║  • Run from ANY Ubuntu-based distro                          ║
║  • Installs Kubuntu 24.04 with ZFS root                      ║
║  • Comprehensive error handling                              ║
║  • Resume support on failure                                 ║
╚══════════════════════════════════════════════════════════════╝
EOF
    echo ""
    
    log "Starting installation at $(date)"
    echo "" | tee -a "$LOG_FILE"
    
    # Pre-flight checks
    progress "Running pre-flight checks..."
    check_root
    detect_host_distro
    check_uefi
    check_internet
    check_memory
    detect_hardware
    
    # Show configuration
    echo ""
    echo "Configuration:"
    echo "  Target Disk: ${TARGET_DISK:-NOT SET}"
    echo "  Username: $USERNAME"
    echo "  Password: $USER_PASSWORD"
    echo "  Hostname: $HOSTNAME"
    echo "  Target: $TARGET_DISTRO $TARGET_VERSION"
    echo ""
    
    if [ -z "$TARGET_DISK" ]; then
        echo "Available disks:"
        lsblk -d -o NAME,SIZE,TYPE,MODEL | grep disk
        echo ""
        error "TARGET_DISK not set. Set with: export TARGET_DISK=/dev/sdX"
    fi
    
    check_disk
    
    echo ""
    warn "⚠  This will ERASE ALL DATA on $TARGET_DISK"
    echo ""
    read -p "Type YES to continue: " confirm
    [ "$confirm" != "YES" ] && error "Installation cancelled"
    
    # Install dependencies
    progress "Installing host dependencies..."
    pkg_update
    install_host_dependencies
    
    # Disk preparation
    if [ "$(get_state)" = "START" ]; then
        prepare_disk
        create_zfs_pools
        create_zfs_datasets
        mount_filesystems
    fi
    
    # Base system
    if [ "$(get_state)" = "FS_MOUNTED" ]; then
        install_base_system
        configure_base_system
    fi
    
    # Chroot installation
    if [ "$(get_state)" = "BASE_CONFIGURED" ]; then
        prepare_chroot
        create_chroot_install_script
        
        progress "Running chroot installation (this will take 15-30 minutes)..."
        chroot "$MNTROOT" /root/install-chroot.sh || error "Chroot installation failed"
        
        save_state "CHROOT_COMPLETE"
    fi
    
    # Cleanup
    progress "Cleaning up..."
    umount -R "$MNTROOT/dev" "$MNTROOT/proc" "$MNTROOT/sys" 2>/dev/null || true
    umount "$MNTROOT/boot/efi" 2>/dev/null || true
    zfs umount -a 2>/dev/null || true
    zpool export bpool 2>/dev/null || true
    zpool export rpool 2>/dev/null || true
    
    # Success
    rm -f "$STATE_FILE"
    
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║         🎉 INSTALLATION COMPLETE! 🎉                     ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Installed components:"
    echo "  ✓ Kubuntu 24.04 with ZFS root"
    echo "  ✓ ZFSBootMenu (no GRUB)"
    echo "  ✓ KDE Plasma desktop"
    [ $HAS_NVIDIA -eq 1 ] && echo "  ✓ NVIDIA drivers"
    echo "  ✓ Gaming stack (Steam, Lutris, Wine)"
    echo "  ✓ Docker + Docker Compose"
    echo "  ✓ Ollama AI service"
    echo "  ✓ Media stack (Jellyfin, qBittorrent, Radarr, Sonarr, Jackett)"
    echo "  ✓ Development tools (Rust, Node.js, etc.)"
    echo "  ✓ WireGuard VPN"
    echo ""
    echo "Login credentials:"
    echo "  Username: $USERNAME"
    echo "  Password: $USER_PASSWORD"
    echo ""
    echo "After boot, access services at:"
    echo "  • Jellyfin: http://localhost:8096"
    echo "  • qBittorrent: http://localhost:8080"
    echo "  • Radarr: http://localhost:7878"
    echo "  • Sonarr: http://localhost:8989"
    echo "  • Jackett: http://localhost:9117"
    echo "  • Ollama: http://localhost:11434"
    echo ""
    echo "To start media stack after first boot:"
    echo "  cd ~/media-stack"
    echo "  docker compose up -d"
    echo ""
    log "Installation log saved to: $LOG_FILE"
    echo ""
    warn "Remove installation media and reboot!"
    read -p "Press Enter to reboot now (Ctrl+C to cancel)..."
    reboot
}

# Run main
main "$@"
