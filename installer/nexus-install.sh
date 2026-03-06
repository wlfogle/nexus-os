#!/bin/bash

# ═══════════════════════════════════════════════════════════════════════════
# NexusOS System Installer v2025.1
# ═══════════════════════════════════════════════════════════════════════════
#
# Installs NexusOS on Pop!_OS 22.04 LTS NVIDIA (X11)
#
# Two installation modes:
#   Mode 1 — Overlay Install (default)
#       Installs NexusOS components on top of existing Pop!_OS
#       Optional ZFS data pool for media/docker/AI storage
#
#   Mode 2 — Fresh Install with ZFS-on-root
#       Debootstraps Ubuntu 22.04, adds System76 packages
#       Full ZFS root with ZFSBootMenu as bootloader
#       Requires a dedicated target disk
#
# Usage:
#   sudo ./nexus-install.sh              # Interactive mode
#   sudo INSTALL_MODE=overlay ./nexus-install.sh
#   sudo INSTALL_MODE=fresh TARGET_DISK=/dev/sdX ./nexus-install.sh
#
# ═══════════════════════════════════════════════════════════════════════════

# ═══════════════════════════════════════════════════════════════════════════
# CONFIGURATION
# ═══════════════════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
LOG_DIR="/var/log/nexus-os"
LOG_FILE="$LOG_DIR/nexus-install.log"
STATE_FILE="$SCRIPT_DIR/.install_state"

NEXUS_VERSION="2025.1"
NEXUS_CODENAME="Stellar"
BASE_DIR="/opt/nexus-os"
CONFIG_DIR="/etc/nexus-os"

: "${INSTALL_MODE:=}"
: "${TARGET_DISK:=}"
: "${INSTALL_USERNAME:=${SUDO_USER:-nexus}}"
: "${INSTALL_HOSTNAME:=nexus-powerhouse}"
: "${INSTALL_PASSWORD:=nexus}"

INSTALL_KDE=true
INSTALL_GAMING=false
INSTALL_MEDIA=false
INSTALL_AI=false
INSTALL_DEV=false
INSTALL_ZFS_DATA=false
ZFS_DATA_DEVICE=""
INSTALLATION_PROFILE=""
PROFILE_NAME=""

HAS_NVIDIA=0
HAS_AVX2=0
IS_I9_13900HX=0
HOST_RAM_GB=0
HOST_STORAGE_GB=0
MNTROOT=""
EFI_PART=""
BOOT_PART=""
ROOT_PART=""

# ═══════════════════════════════════════════════════════════════════════════
# COLORS AND BRANDING
# ═══════════════════════════════════════════════════════════════════════════

readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly PURPLE='\033[0;35m'
readonly CYAN='\033[0;36m'
readonly WHITE='\033[1;37m'
readonly NC='\033[0m'

NEXUS_LOGO="
    ███╗   ██╗███████╗██╗  ██╗██╗   ██╗███████╗ ██████╗ ███████╗
    ████╗  ██║██╔════╝╚██╗██╔╝██║   ██║██╔════╝██╔═══██╗██╔════╝
    ██╔██╗ ██║█████╗   ╚███╔╝ ██║   ██║███████╗██║   ██║███████╗
    ██║╚██╗██║██╔══╝   ██╔██╗ ██║   ██║╚════██║██║   ██║╚════██║
    ██║ ╚████║███████╗██╔╝ ██╗╚██████╔╝███████║╚██████╔╝███████║
    ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝ ╚═════╝ ╚══════╝
"

# ═══════════════════════════════════════════════════════════════════════════
# LOGGING AND ERROR HANDLING
# ═══════════════════════════════════════════════════════════════════════════

init_logging() {
    mkdir -p "$LOG_DIR"
    touch "$LOG_FILE"
    exec > >(tee -a "$LOG_FILE") 2>&1
}

log() {
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${GREEN}[${timestamp}]${NC} $*"
}

warn() {
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${YELLOW}[${timestamp}] WARNING:${NC} $*"
}

die() {
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${RED}[${timestamp}] FATAL:${NC} $*"
    cleanup_on_failure
    exit 1
}

save_state() {
    echo "$1" > "$STATE_FILE"
}

get_state() {
    [[ -f "$STATE_FILE" ]] && cat "$STATE_FILE" || echo "START"
}

cleanup_on_failure() {
    warn "Installation interrupted, cleaning up..."

    if [[ "$INSTALL_MODE" == "fresh" && -n "$MNTROOT" && -d "$MNTROOT" ]]; then
        umount -R "$MNTROOT/dev" 2>/dev/null || true
        umount -R "$MNTROOT/proc" 2>/dev/null || true
        umount -R "$MNTROOT/sys" 2>/dev/null || true
        umount "$MNTROOT/boot/efi" 2>/dev/null || true
    fi

    if [[ "$INSTALL_MODE" == "fresh" ]]; then
        zpool list bpool &>/dev/null && zpool export bpool 2>/dev/null || true
        zpool list rpool &>/dev/null && zpool export rpool 2>/dev/null || true
    fi

    log "Cleanup complete. Check $LOG_FILE for details."
}

trap cleanup_on_failure ERR INT TERM

# ═══════════════════════════════════════════════════════════════════════════
# UI FUNCTIONS
# ═══════════════════════════════════════════════════════════════════════════

print_header() {
    clear
    echo -e "${PURPLE}${NEXUS_LOGO}${NC}"
    echo -e "${CYAN}                  AI-Native Operating System for Pop!_OS${NC}"
    echo -e "${WHITE}                    Version: $NEXUS_VERSION ($NEXUS_CODENAME)${NC}"
    echo ""
    echo "==================================================================================="
}

print_status()  { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[ OK ]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error()   { echo -e "${RED}[FAIL]${NC} $1"; }
print_stella()  { echo -e "${PURPLE}[STELLA]${NC} $1"; }
print_maxjr()   { echo -e "${YELLOW}[MAX JR]${NC} $1"; }

confirm_prompt() {
    local message="$1"
    local default="${2:-N}"
    local prompt reply

    if [[ "$default" == "Y" ]]; then
        prompt="(Y/n)"
    else
        prompt="(y/N)"
    fi

    read -rp "$message $prompt: " reply
    reply="${reply:-$default}"
    [[ "$reply" =~ ^[Yy] ]]
}

# ═══════════════════════════════════════════════════════════════════════════
# HARDWARE DETECTION
# ═══════════════════════════════════════════════════════════════════════════

detect_hardware() {
    log "Detecting hardware..."

    # NVIDIA GPU
    if lspci 2>/dev/null | grep -qi nvidia; then
        HAS_NVIDIA=1
        local gpu_name
        gpu_name=$(lspci | grep -i nvidia | head -1 | sed 's/.*: //')
        print_success "NVIDIA GPU: $gpu_name"
    else
        print_warning "No NVIDIA GPU detected"
    fi

    # CPU features
    if grep -q avx2 /proc/cpuinfo 2>/dev/null; then
        HAS_AVX2=1
        log "AVX2 support detected"
    fi

    # i9-13900HX detection
    if lscpu 2>/dev/null | grep -qi "i9-13900HX"; then
        IS_I9_13900HX=1
        print_success "Intel i9-13900HX detected — Alderlake optimizations available"
    fi

    # Memory
    HOST_RAM_GB=$(free -g | awk '/^Mem:/{print $2}')
    print_status "RAM: ${HOST_RAM_GB}GB"

    # Root filesystem available storage
    HOST_STORAGE_GB=$(df -BG / | awk 'NR==2{print int($4)}')
    print_status "Available storage: ${HOST_STORAGE_GB}GB"
}

# ═══════════════════════════════════════════════════════════════════════════
# PREFLIGHT CHECKS
# ═══════════════════════════════════════════════════════════════════════════

check_root_user() {
    if [[ $EUID -ne 0 ]]; then
        die "This script must be run with sudo: sudo ./nexus-install.sh"
    fi

    if [[ "$INSTALL_MODE" == "overlay" ]]; then
        if [[ -z "${SUDO_USER:-}" || "$SUDO_USER" == "root" ]]; then
            die "Overlay mode requires running via sudo from a regular user account."
        fi
    fi
}

check_popos() {
    if [[ ! -f /etc/os-release ]]; then
        die "Cannot detect operating system"
    fi

    # shellcheck source=/dev/null
    . /etc/os-release

    if [[ "$ID" == "pop" ]]; then
        print_success "Detected Pop!_OS $VERSION_ID"
        if [[ "$VERSION_ID" != "22.04" ]]; then
            warn "NexusOS is optimized for Pop!_OS 22.04. Running on $VERSION_ID may have issues."
        fi
    elif [[ "$ID" == "ubuntu" || "${ID_LIKE:-}" == *"ubuntu"* ]]; then
        warn "Running on $NAME $VERSION_ID (Ubuntu-based). Some Pop!_OS features may be unavailable."
    else
        die "NexusOS requires Pop!_OS 22.04 or an Ubuntu-based distribution."
    fi
}

check_uefi() {
    if [[ -d /sys/firmware/efi ]]; then
        print_success "UEFI boot detected"
    else
        if [[ "$INSTALL_MODE" == "fresh" ]]; then
            die "Fresh install requires UEFI boot mode"
        fi
        warn "Legacy BIOS mode detected. ZFSBootMenu will not be available."
    fi
}

check_internet() {
    log "Checking internet connectivity..."
    if ping -c 1 -W 5 8.8.8.8 &>/dev/null || ping -c 1 -W 5 1.1.1.1 &>/dev/null; then
        print_success "Internet connection active"
    else
        die "No internet connection. NexusOS installer requires internet access."
    fi
}

check_resources() {
    if [[ $HOST_RAM_GB -lt 4 ]]; then
        warn "Low RAM (${HOST_RAM_GB}GB). 8GB+ recommended for full NexusOS experience."
    fi
    if [[ $HOST_STORAGE_GB -lt 30 ]]; then
        warn "Low disk space (${HOST_STORAGE_GB}GB free). 50GB+ recommended."
    fi
}

ensure_nala() {
    if ! command -v nala &>/dev/null; then
        log "Installing nala package manager..."
        apt-get update -qq
        apt-get install -y -qq nala || die "Failed to install nala"
    fi
    print_success "nala package manager available"
}

run_preflight() {
    log "Running preflight checks..."
    check_root_user
    check_popos
    detect_hardware
    check_uefi
    check_internet
    check_resources
    ensure_nala
    echo ""
}

# ═══════════════════════════════════════════════════════════════════════════
# USER INTERACTION
# ═══════════════════════════════════════════════════════════════════════════

select_install_mode() {
    if [[ -n "$INSTALL_MODE" ]]; then
        return
    fi

    print_header
    echo -e "${WHITE}Select Installation Mode:${NC}"
    echo ""
    echo "  1) Overlay Install (recommended)"
    echo "     Install NexusOS components on your existing Pop!_OS system."
    echo "     Safe, reversible, keeps your current setup intact."
    echo ""
    echo "  2) Fresh Install with ZFS-on-root (advanced)"
    echo "     Full installation to a target disk with ZFS root filesystem"
    echo "     and ZFSBootMenu. ERASES the target disk completely."
    echo ""

    while true; do
        read -rp "Enter your choice (1-2): " mode_choice
        case $mode_choice in
            1) INSTALL_MODE="overlay"; break ;;
            2) INSTALL_MODE="fresh"; break ;;
            *) print_error "Invalid selection." ;;
        esac
    done

    print_success "Selected: $INSTALL_MODE install"
}

select_profile() {
    print_header
    echo -e "${WHITE}Select NexusOS Installation Profile:${NC}"
    echo ""
    echo "  1) Gaming Focused      — Gaming stack with basic media"
    echo "  2) Media Server         — Complete 65+ service media stack"
    echo "  3) Complete Experience  — Everything: Gaming + Media + AI + Dev"
    echo "  4) Developer Workstation — Dev tools + AI assistants"
    echo "  5) Custom               — Choose individual components"
    echo ""

    while true; do
        read -rp "Enter your choice (1-5): " profile_choice
        case $profile_choice in
            1)
                INSTALLATION_PROFILE="gaming"
                PROFILE_NAME="Gaming Focused"
                INSTALL_GAMING=true
                ;;
            2)
                INSTALLATION_PROFILE="media"
                PROFILE_NAME="Media Server"
                INSTALL_MEDIA=true
                INSTALL_AI=true
                ;;
            3)
                INSTALLATION_PROFILE="complete"
                PROFILE_NAME="Complete Experience"
                INSTALL_GAMING=true
                INSTALL_MEDIA=true
                INSTALL_AI=true
                INSTALL_DEV=true
                ;;
            4)
                INSTALLATION_PROFILE="developer"
                PROFILE_NAME="Developer Workstation"
                INSTALL_AI=true
                INSTALL_DEV=true
                ;;
            5)
                INSTALLATION_PROFILE="custom"
                PROFILE_NAME="Custom"
                ;;
            *)
                print_error "Invalid selection."
                continue
                ;;
        esac
        break
    done

    print_success "Selected profile: $PROFILE_NAME"
}

select_custom_components() {
    if [[ "$INSTALLATION_PROFILE" != "custom" ]]; then
        return
    fi

    echo ""
    echo -e "${WHITE}Select components to install:${NC}"
    echo ""

    confirm_prompt "  Install Gaming packages (Steam, Lutris, Wine, GameMode)?" && INSTALL_GAMING=true
    confirm_prompt "  Install Media Stack (Docker + 65+ media services)?" && INSTALL_MEDIA=true
    confirm_prompt "  Install AI Assistants (Stella, Max Jr., Ollama)?" && INSTALL_AI=true
    confirm_prompt "  Install Development Tools (build tools, Rust, Node.js)?" && INSTALL_DEV=true
    echo ""
}

select_zfs_data_pool() {
    if [[ "$INSTALL_MODE" != "overlay" ]]; then
        return
    fi

    echo ""
    if ! confirm_prompt "  Create a ZFS data pool for media/docker/AI storage?"; then
        return
    fi

    INSTALL_ZFS_DATA=true
    echo ""
    echo "Available block devices:"
    lsblk -d -o NAME,SIZE,TYPE,MODEL | grep disk
    echo ""
    echo "Enter the device for the ZFS data pool (e.g., /dev/sdb)."
    echo "WARNING: This will FORMAT the selected device. Do NOT use your root disk."
    echo ""

    while true; do
        read -rp "Device path (or 'skip' to cancel): " ZFS_DATA_DEVICE
        if [[ "$ZFS_DATA_DEVICE" == "skip" ]]; then
            INSTALL_ZFS_DATA=false
            return
        fi
        if [[ -b "$ZFS_DATA_DEVICE" ]]; then
            # Prevent selecting the root disk
            local root_disk
            root_disk=$(findmnt -no SOURCE / 2>/dev/null | sed 's/[0-9]*$//' | sed 's/p[0-9]*$//')
            if [[ "$ZFS_DATA_DEVICE" == "$root_disk" ]]; then
                print_error "That is your root disk. Choose a different device."
                continue
            fi
            break
        fi
        print_error "Device $ZFS_DATA_DEVICE does not exist."
    done

    echo ""
    warn "ALL DATA on $ZFS_DATA_DEVICE will be ERASED."
    if ! confirm_prompt "  Proceed with ZFS data pool on $ZFS_DATA_DEVICE?"; then
        INSTALL_ZFS_DATA=false
    fi
}

select_fresh_disk() {
    if [[ "$INSTALL_MODE" != "fresh" ]]; then
        return
    fi

    if [[ -n "$TARGET_DISK" && -b "$TARGET_DISK" ]]; then
        return
    fi

    echo ""
    echo "Available disks for fresh install:"
    lsblk -d -o NAME,SIZE,TYPE,MODEL | grep disk
    echo ""

    while true; do
        read -rp "Enter target disk (e.g., /dev/sda): " TARGET_DISK
        if [[ -b "$TARGET_DISK" ]]; then
            local disk_size_gb
            disk_size_gb=$(( $(blockdev --getsize64 "$TARGET_DISK") / 1024 / 1024 / 1024 ))
            if [[ $disk_size_gb -lt 32 ]]; then
                print_error "Disk too small (${disk_size_gb}GB). Minimum 32GB required."
                continue
            fi
            print_success "Target disk: $TARGET_DISK (${disk_size_gb}GB)"
            break
        fi
        print_error "Device $TARGET_DISK does not exist."
    done

    echo ""
    warn "ALL DATA on $TARGET_DISK will be PERMANENTLY ERASED."
    read -rp "Type YES to confirm: " confirm
    [[ "$confirm" == "YES" ]] || die "Installation cancelled by user"
}

show_install_summary() {
    print_header
    echo -e "${WHITE}Installation Summary:${NC}"
    echo ""
    echo "  Mode:     $INSTALL_MODE"
    echo "  Profile:  $PROFILE_NAME"
    echo "  KDE:      yes"
    [[ "$INSTALL_GAMING" == true ]] && echo "  Gaming:   yes"
    [[ "$INSTALL_MEDIA" == true ]]  && echo "  Media:    yes"
    [[ "$INSTALL_AI" == true ]]     && echo "  AI:       yes"
    [[ "$INSTALL_DEV" == true ]]    && echo "  Dev:      yes"

    if [[ "$INSTALL_MODE" == "overlay" ]]; then
        [[ "$INSTALL_ZFS_DATA" == true ]] && echo "  ZFS Pool: $ZFS_DATA_DEVICE"
    else
        echo "  Disk:     $TARGET_DISK"
        echo "  User:     $INSTALL_USERNAME"
        echo "  Hostname: $INSTALL_HOSTNAME"
    fi

    echo ""
    if ! confirm_prompt "Proceed with installation?" "Y"; then
        die "Installation cancelled by user"
    fi
}

# ═══════════════════════════════════════════════════════════════════════════
# ZFS FUNCTIONS (shared)
# ═══════════════════════════════════════════════════════════════════════════

install_zfs_packages() {
    log "Installing ZFS packages..."
    nala install --assume-yes zfsutils-linux || die "Failed to install ZFS packages"

    if ! lsmod | grep -q zfs; then
        log "Loading ZFS kernel module..."
        modprobe zfs || die "Failed to load ZFS module. Ensure ZFS DKMS is built for your kernel."
    fi

    print_success "ZFS packages installed and module loaded"
}

create_zfs_data_pool() {
    if [[ "$INSTALL_ZFS_DATA" != true ]]; then
        return
    fi

    log "Creating ZFS data pool on $ZFS_DATA_DEVICE..."

    # Unmount any mounted partitions on the device
    for part in "${ZFS_DATA_DEVICE}"*; do
        [[ -b "$part" ]] && umount "$part" 2>/dev/null || true
    done

    # Wipe and create single partition
    wipefs -af "$ZFS_DATA_DEVICE" 2>/dev/null || true
    sgdisk --zap-all "$ZFS_DATA_DEVICE" || die "Failed to wipe $ZFS_DATA_DEVICE"
    sgdisk -n1:0:0 -t1:BF00 "$ZFS_DATA_DEVICE" || die "Failed to create partition"
    sleep 2
    partprobe "$ZFS_DATA_DEVICE" 2>/dev/null || true
    sleep 1

    # Determine partition path
    local data_part
    if [[ "$ZFS_DATA_DEVICE" == *"nvme"* ]] || [[ "$ZFS_DATA_DEVICE" == *"mmcblk"* ]]; then
        data_part="${ZFS_DATA_DEVICE}p1"
    else
        data_part="${ZFS_DATA_DEVICE}1"
    fi

    [[ -b "$data_part" ]] || die "Data partition $data_part not found"

    # Destroy existing pool if present
    if zpool list nexus-data &>/dev/null; then
        warn "Destroying existing nexus-data pool..."
        zpool destroy -f nexus-data || die "Failed to destroy existing pool"
    fi

    # Choose compression based on CPU
    local compression="lz4"
    if [[ $HAS_AVX2 -eq 1 ]]; then
        compression="zstd"
        log "Using ZSTD compression (AVX2 detected)"
    fi

    # Create pool
    zpool create -f -o ashift=12 -o autotrim=on \
        -O acltype=posixacl -O compression=$compression \
        -O dnodesize=auto -O normalization=formD \
        -O relatime=on -O xattr=sa -O mountpoint=none \
        nexus-data "$data_part" || die "Failed to create nexus-data pool"

    local user_home="/home/$INSTALL_USERNAME"

    # Create datasets
    zfs create -o mountpoint="$user_home/nexus-media/media" nexus-data/media \
        || die "Failed to create media dataset"
    zfs create -o mountpoint="$user_home/nexus-media/downloads" nexus-data/downloads \
        || die "Failed to create downloads dataset"
    zfs create -o mountpoint=/var/lib/docker nexus-data/docker \
        || die "Failed to create docker dataset"
    zfs create -o mountpoint="$BASE_DIR/models" nexus-data/ai-models \
        || die "Failed to create ai-models dataset"

    # Fix ownership
    chown -R "$INSTALL_USERNAME:$INSTALL_USERNAME" "$user_home/nexus-media" 2>/dev/null || true

    # Enable auto-import on boot
    zpool set cachefile=/etc/zfs/zpool.cache nexus-data || true

    print_success "ZFS data pool created with datasets: media, downloads, docker, ai-models"
}

# ═══════════════════════════════════════════════════════════════════════════
# OVERLAY MODE: SYSTEM PREPARATION
# ═══════════════════════════════════════════════════════════════════════════

overlay_prepare_system() {
    print_header
    log "Preparing system for NexusOS overlay installation..."

    # Create NexusOS directory structure
    mkdir -p "$BASE_DIR"/{bin,lib,share,etc,models,services}
    mkdir -p "$CONFIG_DIR"/{gaming,media,desktop,services}
    mkdir -p "$LOG_DIR"

    local user_home="/home/$INSTALL_USERNAME"
    mkdir -p "$user_home/.config/nexus-os"
    chown -R "$INSTALL_USERNAME:$INSTALL_USERNAME" "$user_home/.config/nexus-os"

    # Update system
    print_status "Updating system packages..."
    nala update || die "Failed to update package lists"
    nala upgrade --assume-yes || warn "Some packages could not be upgraded"

    # Install base dependencies
    print_status "Installing base dependencies..."
    nala install --assume-yes \
        git wget curl ca-certificates gnupg lsb-release \
        software-properties-common apt-transport-https \
        build-essential python3-pip python3-venv \
        || die "Failed to install base dependencies"

    print_success "System prepared for NexusOS"
}

# ═══════════════════════════════════════════════════════════════════════════
# OVERLAY MODE: KDE PLASMA
# ═══════════════════════════════════════════════════════════════════════════

overlay_install_kde() {
    log "Installing KDE Plasma Desktop..."

    nala install --assume-yes \
        kde-plasma-desktop sddm plasma-nm \
        konsole dolphin kate ark kde-spectacle gwenview okular \
        plasma-systemmonitor kde-config-sddm kde-config-screenlocker \
        || die "Failed to install KDE Plasma"

    # Configure SDDM for X11
    mkdir -p /etc/sddm.conf.d
    cat > /etc/sddm.conf.d/nexusos.conf <<'SDDMCONF'
[General]
InputMethod=

[Theme]
Current=breeze

[Users]
MaximumUid=60000
MinimumUid=1000

[X11]
SessionDir=/usr/share/xsessions
SDDMCONF

    # Switch display manager from gdm3 to sddm
    if systemctl is-active --quiet gdm3 2>/dev/null; then
        log "Switching display manager from GDM to SDDM..."
        systemctl disable gdm3 2>/dev/null || true
    fi
    systemctl enable sddm || die "Failed to enable SDDM"

    # Verify X11 session file exists
    if [[ ! -f /usr/share/xsessions/plasma.desktop ]]; then
        warn "KDE X11 session file not found. KDE may default to Wayland."
    fi

    # Note system76-power CLI availability
    if command -v system76-power &>/dev/null; then
        print_success "system76-power available for GPU switching (CLI: system76-power graphics)"
    fi

    print_success "KDE Plasma Desktop installed. SDDM enabled."
}

# ═══════════════════════════════════════════════════════════════════════════
# OVERLAY MODE: GAMING
# ═══════════════════════════════════════════════════════════════════════════

overlay_install_gaming() {
    if [[ "$INSTALL_GAMING" != true ]]; then
        return
    fi

    log "Installing gaming packages..."
    print_maxjr "Setting up gaming optimizations..."

    # Enable 32-bit architecture for Wine and Steam
    dpkg --add-architecture i386
    nala update

    # Core gaming packages
    nala install --assume-yes \
        steam-installer \
        gamemode libgamemode0 libgamemodeauto0 \
        mangohud \
        wine64 wine32 winetricks \
        vulkan-tools mesa-vulkan-drivers mesa-vulkan-drivers:i386 \
        || warn "Some gaming packages could not be installed"

    # Lutris from official PPA
    print_status "Adding Lutris PPA..."
    if ! grep -rq "lutris" /etc/apt/sources.list.d/ 2>/dev/null; then
        add-apt-repository -y ppa:lutris-team/lutris 2>/dev/null || true
        nala update
    fi
    nala install --assume-yes lutris || warn "Lutris install failed — try: flatpak install lutris"

    # Bottles via Flatpak
    print_status "Installing Bottles via Flatpak..."
    if ! command -v flatpak &>/dev/null; then
        nala install --assume-yes flatpak
    fi
    flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo 2>/dev/null || true
    flatpak install -y --noninteractive flathub com.usebottles.bottles 2>/dev/null \
        || warn "Bottles flatpak install failed"

    # Performance tuning packages
    nala install --assume-yes \
        irqbalance cpufrequtils \
        || warn "Some performance tools could not be installed"

    # Enable gaming services
    systemctl enable gamemode 2>/dev/null || true
    systemctl enable irqbalance 2>/dev/null || true

    # Gaming sysctl optimizations
    cat > /etc/sysctl.d/99-nexus-gaming.conf <<'GAMINGCONF'
# NexusOS Gaming Optimizations
vm.swappiness=10
vm.dirty_ratio=15
vm.dirty_background_ratio=5
vm.max_map_count=2147483642
kernel.sched_autogroup_enabled=1
GAMINGCONF
    sysctl --system &>/dev/null || true

    # i9-13900HX specific optimizations
    if [[ $IS_I9_13900HX -eq 1 ]]; then
        cat > /etc/sysctl.d/99-nexus-i9.conf <<'I9CONF'
# i9-13900HX Specific Optimizations
kernel.sched_migration_cost_ns=500000
net.core.rmem_max=134217728
net.core.wmem_max=134217728
net.ipv4.tcp_rmem=4096 87380 67108864
net.ipv4.tcp_wmem=4096 65536 67108864
I9CONF
        sysctl --system &>/dev/null || true
        print_success "i9-13900HX optimizations applied"
    fi

    print_maxjr "Gaming stack installed and optimized"
}

# ═══════════════════════════════════════════════════════════════════════════
# OVERLAY MODE: MEDIA STACK (DOCKER)
# ═══════════════════════════════════════════════════════════════════════════

overlay_install_media() {
    if [[ "$INSTALL_MEDIA" != true ]]; then
        return
    fi

    log "Installing Docker and media stack..."

    # Install Docker from official repo if not already present
    if ! command -v docker &>/dev/null; then
        print_status "Adding Docker repository..."
        install -m 0755 -d /etc/apt/keyrings
        curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
        chmod a+r /etc/apt/keyrings/docker.asc

        local codename
        codename=$(. /etc/os-release && echo "${UBUNTU_CODENAME:-${VERSION_CODENAME:-jammy}}")
        echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] \
https://download.docker.com/linux/ubuntu $codename stable" > /etc/apt/sources.list.d/docker.list

        nala update
        nala install --assume-yes \
            docker-ce docker-ce-cli containerd.io \
            docker-buildx-plugin docker-compose-plugin \
            || die "Failed to install Docker"
    fi

    systemctl enable docker
    systemctl start docker
    usermod -aG docker "$INSTALL_USERNAME"

    # Create media directories (ZFS data pool may have already created mount targets)
    local user_home="/home/$INSTALL_USERNAME"
    local media_base="$user_home/nexus-media"
    mkdir -p "$media_base"/{media,downloads,config}
    mkdir -p "$media_base"/media/{movies,tv,music,books,audiobooks,comics}
    chown -R "$INSTALL_USERNAME:$INSTALL_USERNAME" "$media_base"

    # Copy media stack compose file from repo if available
    if [[ -f "$REPO_DIR/core/media-stack/docker-compose.yml" ]]; then
        cp "$REPO_DIR/core/media-stack/docker-compose.yml" "$media_base/docker-compose.yml"
        [[ -f "$REPO_DIR/core/media-stack/.env.template" ]] && \
            cp "$REPO_DIR/core/media-stack/.env.template" "$media_base/.env.template"
    fi

    # Generate .env file with system-specific values
    cat > "$media_base/.env" <<MEDIAENV
# NexusOS Media Stack Configuration
# Generated by nexus-install.sh on $(date)
PUID=$(id -u "$INSTALL_USERNAME")
PGID=$(id -g "$INSTALL_USERNAME")
TZ=$(timedatectl show --property=Timezone --value 2>/dev/null || echo "UTC")

# Paths
MEDIA_ROOT=$media_base/media
DOWNLOADS_ROOT=$media_base/downloads
CONFIG_ROOT=$media_base/config

# Database
POSTGRES_DB=nexusdb
POSTGRES_USER=nexus
POSTGRES_PASSWORD=nexus_$(openssl rand -hex 8)

# Optional: Plex claim token (get from https://plex.tv/claim)
PLEX_CLAIM=
MEDIAENV

    chown -R "$INSTALL_USERNAME:$INSTALL_USERNAME" "$media_base"

    print_success "Docker installed. Media directories ready at $media_base"
    print_status "Start the media stack: cd $media_base && docker compose up -d"
}

# ═══════════════════════════════════════════════════════════════════════════
# OVERLAY MODE: AI SERVICES
# ═══════════════════════════════════════════════════════════════════════════

overlay_install_ai() {
    if [[ "$INSTALL_AI" != true ]]; then
        return
    fi

    log "Installing AI services..."
    print_stella "Initializing AI security and monitoring systems..."

    # Python dependencies
    nala install --assume-yes \
        python3-pip python3-venv python3-dev \
        python3-psutil python3-aiohttp \
        || die "Failed to install Python dependencies"

    # FastAPI and uvicorn for AI services
    pip3 install --break-system-packages fastapi uvicorn httpx 2>/dev/null \
        || pip3 install fastapi uvicorn httpx 2>/dev/null \
        || warn "FastAPI pip install failed — AI services may need manual dependency setup"

    # Ollama AI runtime
    if ! command -v ollama &>/dev/null; then
        print_status "Installing Ollama AI runtime..."
        curl -fsSL https://ollama.ai/install.sh | sh || warn "Ollama installation failed"
    fi

    if command -v ollama &>/dev/null; then
        # Create Ollama service user
        if ! id ollama &>/dev/null; then
            useradd -r -s /bin/false -d /opt/ollama ollama 2>/dev/null || true
        fi
        mkdir -p /opt/ollama
        chown -R ollama:ollama /opt/ollama 2>/dev/null || true

        cat > /etc/systemd/system/ollama.service <<'OLLSVC'
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
OLLSVC

        systemctl daemon-reload
        systemctl enable ollama
        print_success "Ollama AI service installed and enabled"
    fi

    # Copy NexusOS AI service files from the repo
    local services_dir="$BASE_DIR/services"
    mkdir -p "$services_dir"

    for svc in stella.py maxjr.py orchestrator.py; do
        if [[ -f "$REPO_DIR/core/services/$svc" ]]; then
            cp "$REPO_DIR/core/services/$svc" "$services_dir/$svc"
            log "Installed $svc to $services_dir"
        fi
    done

    # Copy and enable systemd units for NexusOS AI services
    for unit in nexus-stella.service nexus-maxjr.service nexus-orchestrator.service; do
        if [[ -f "$REPO_DIR/core/services/$unit" ]]; then
            cp "$REPO_DIR/core/services/$unit" /etc/systemd/system/"$unit"
            systemctl daemon-reload
            systemctl enable "$unit" 2>/dev/null || true
            log "Enabled $unit"
        fi
    done

    print_stella "AI services ready — Stella (8601), Max Jr. (8602), Orchestrator (8600)"
}

# ═══════════════════════════════════════════════════════════════════════════
# OVERLAY MODE: DEVELOPMENT TOOLS
# ═══════════════════════════════════════════════════════════════════════════

overlay_install_dev() {
    if [[ "$INSTALL_DEV" != true ]]; then
        return
    fi

    log "Installing development tools..."

    nala install --assume-yes \
        build-essential cmake pkg-config libssl-dev \
        git git-lfs \
        nodejs npm \
        htop btop \
        tmux screen \
        neovim \
        jq \
        || warn "Some dev packages could not be installed"

    # Rust toolchain for the user
    local user_home="/home/$INSTALL_USERNAME"
    if [[ ! -d "$user_home/.rustup" ]]; then
        print_status "Installing Rust toolchain for $INSTALL_USERNAME..."
        sudo -u "$INSTALL_USERNAME" bash -c \
            "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y" \
            || warn "Rust installation failed"
    fi

    print_success "Development tools installed"
}

# ═══════════════════════════════════════════════════════════════════════════
# FRESH MODE: DISK PREPARATION
# ═══════════════════════════════════════════════════════════════════════════

fresh_prepare_disk() {
    log "Preparing disk $TARGET_DISK for fresh install..."

    # Unmount any mounted partitions
    for part in "${TARGET_DISK}"*; do
        [[ -b "$part" ]] && umount "$part" 2>/dev/null || true
    done

    # Wipe disk
    wipefs -af "$TARGET_DISK" 2>/dev/null || true
    sgdisk --zap-all "$TARGET_DISK" || die "Failed to wipe disk"

    # Partition layout: EFI (512MB) + bpool (2GB) + rpool (rest)
    log "Creating partition layout..."
    sgdisk -n1:1M:+512M -t1:EF00 "$TARGET_DISK" || die "Failed to create EFI partition"
    sgdisk -n2:0:+2G    -t2:BF00 "$TARGET_DISK" || die "Failed to create boot pool partition"
    sgdisk -n3:0:0      -t3:BF00 "$TARGET_DISK" || die "Failed to create root pool partition"

    sleep 2
    partprobe "$TARGET_DISK" 2>/dev/null || true
    sleep 1

    # Determine partition naming convention (NVMe vs SATA)
    local part_prefix
    if [[ "$TARGET_DISK" == *"nvme"* ]] || [[ "$TARGET_DISK" == *"mmcblk"* ]]; then
        part_prefix="${TARGET_DISK}p"
    else
        part_prefix="${TARGET_DISK}"
    fi

    EFI_PART="${part_prefix}1"
    BOOT_PART="${part_prefix}2"
    ROOT_PART="${part_prefix}3"

    [[ -b "$EFI_PART"  ]] || die "EFI partition $EFI_PART not created"
    [[ -b "$BOOT_PART" ]] || die "Boot partition $BOOT_PART not created"
    [[ -b "$ROOT_PART" ]] || die "Root partition $ROOT_PART not created"

    # Format EFI partition
    mkfs.vfat -F32 "$EFI_PART" || die "Failed to format EFI partition"

    print_success "Disk partitioned: EFI=$EFI_PART, bpool=$BOOT_PART, rpool=$ROOT_PART"
    save_state "DISK_PREPARED"
}

# ═══════════════════════════════════════════════════════════════════════════
# FRESH MODE: ZFS ROOT POOLS AND DATASETS
# ═══════════════════════════════════════════════════════════════════════════

fresh_create_zfs_pools() {
    log "Creating ZFS pools..."

    # Destroy pre-existing pools if found
    zpool list bpool &>/dev/null && zpool destroy -f bpool 2>/dev/null || true
    zpool list rpool &>/dev/null && zpool destroy -f rpool 2>/dev/null || true

    local compression="lz4"
    if [[ $HAS_AVX2 -eq 1 ]]; then
        compression="zstd"
        log "Using ZSTD compression (AVX2 detected)"
    fi

    # Boot pool — limited features for broad compatibility
    log "Creating boot pool (bpool)..."
    zpool create -f -o ashift=12 \
        -O acltype=posixacl -O compression=lz4 \
        -O normalization=formD -O relatime=on \
        -O xattr=sa -O mountpoint=none \
        bpool "$BOOT_PART" || die "Failed to create bpool"

    # Root pool — full feature set
    log "Creating root pool (rpool)..."
    zpool create -f -o ashift=12 -o autotrim=on \
        -O acltype=posixacl -O compression=$compression \
        -O dnodesize=auto -O normalization=formD \
        -O relatime=on -O xattr=sa -O mountpoint=none \
        rpool "$ROOT_PART" || die "Failed to create rpool"

    print_success "ZFS pools created"
    save_state "POOLS_CREATED"
}

fresh_create_datasets() {
    log "Creating ZFS datasets..."

    # Root datasets
    zfs create -o canmount=off -o mountpoint=none rpool/ROOT || die "Failed: rpool/ROOT"
    zfs create -o canmount=noauto -o mountpoint=/ rpool/ROOT/nexusos || die "Failed: rpool/ROOT/nexusos"
    zpool set bootfs=rpool/ROOT/nexusos rpool || die "Failed to set bootfs"

    # Boot datasets
    zfs create -o canmount=off -o mountpoint=none bpool/BOOT || die "Failed: bpool/BOOT"
    zfs create -o mountpoint=/boot bpool/BOOT/default || die "Failed: bpool/BOOT/default"

    # Home datasets
    zfs create -o mountpoint=/home rpool/home || die "Failed: rpool/home"
    zfs create rpool/home/"$INSTALL_USERNAME" || die "Failed: rpool/home/$INSTALL_USERNAME"
    zfs create -o mountpoint=/root rpool/home/root || die "Failed: rpool/home/root"

    # System datasets
    zfs create -o mountpoint=/var/log rpool/var-log || die "Failed: rpool/var-log"
    zfs create -o mountpoint=/var/cache rpool/var-cache || die "Failed: rpool/var-cache"
    zfs create -o mountpoint=/tmp rpool/tmp || die "Failed: rpool/tmp"

    # NexusOS dataset
    zfs create -o mountpoint=/opt/nexus-os rpool/opt-nexus || die "Failed: rpool/opt-nexus"

    print_success "ZFS datasets created"
    save_state "DATASETS_CREATED"
}

fresh_mount_filesystems() {
    log "Mounting filesystems..."

    zfs mount rpool/ROOT/nexusos || die "Failed to mount root dataset"
    MNTROOT=$(zfs get -H -o value mountpoint rpool/ROOT/nexusos)
    [[ -n "$MNTROOT" ]] || die "Failed to determine mount point"

    log "Root mounted at: $MNTROOT"

    # Mount EFI
    mkdir -p "$MNTROOT/boot/efi"
    mount "$EFI_PART" "$MNTROOT/boot/efi" || die "Failed to mount EFI partition"

    # Set /tmp permissions
    chmod 1777 "$MNTROOT/tmp"

    print_success "Filesystems mounted at $MNTROOT"
    save_state "FS_MOUNTED"
}

# ═══════════════════════════════════════════════════════════════════════════
# FRESH MODE: BASE SYSTEM
# ═══════════════════════════════════════════════════════════════════════════

fresh_install_base() {
    log "Installing base system via debootstrap (this takes several minutes)..."

    nala install --assume-yes debootstrap gdisk dosfstools || die "Failed to install debootstrap"

    debootstrap --arch amd64 jammy "$MNTROOT" http://archive.ubuntu.com/ubuntu \
        || die "debootstrap failed"

    print_success "Base system installed"
    save_state "BASE_INSTALLED"
}

fresh_configure_base() {
    log "Configuring base system..."

    # Hostname
    echo "$INSTALL_HOSTNAME" > "$MNTROOT/etc/hostname"

    # Hosts file
    cat > "$MNTROOT/etc/hosts" <<HOSTS
127.0.0.1 localhost
127.0.1.1 $INSTALL_HOSTNAME

::1 localhost ip6-localhost ip6-loopback
HOSTS

    # APT sources (Ubuntu 22.04 jammy)
    cat > "$MNTROOT/etc/apt/sources.list" <<SOURCES
deb http://archive.ubuntu.com/ubuntu jammy main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu jammy-updates main restricted universe multiverse
deb http://archive.ubuntu.com/ubuntu jammy-security main restricted universe multiverse
SOURCES

    # DNS resolution
    cat > "$MNTROOT/etc/resolv.conf" <<DNS
nameserver 8.8.8.8
nameserver 1.1.1.1
DNS

    # ZFS pool cache for boot
    mkdir -p "$MNTROOT/etc/zfs"
    zpool set cachefile="$MNTROOT/etc/zfs/zpool.cache" rpool || warn "Failed to set rpool cache"
    zpool set cachefile="$MNTROOT/etc/zfs/zpool.cache" bpool || warn "Failed to set bpool cache"

    print_success "Base system configured"
    save_state "BASE_CONFIGURED"
}

fresh_prepare_chroot() {
    log "Preparing chroot environment..."
    mount --rbind /dev  "$MNTROOT/dev"  || die "Failed to bind /dev"
    mount --rbind /proc "$MNTROOT/proc" || die "Failed to bind /proc"
    mount --rbind /sys  "$MNTROOT/sys"  || die "Failed to bind /sys"
    print_success "Chroot environment ready"
}

# ═══════════════════════════════════════════════════════════════════════════
# FRESH MODE: CHROOT INSTALLATION SCRIPT GENERATION
# ═══════════════════════════════════════════════════════════════════════════

fresh_generate_chroot_script() {
    log "Generating chroot installation script..."

    local script="$MNTROOT/root/nexus-chroot-install.sh"

    # ---- Header and base packages ----
    cat > "$script" <<CHROOT_HEADER
#!/bin/bash
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive

log() { echo "[\$(date +'%H:%M:%S')] \$*"; }
warn() { echo "[\$(date +'%H:%M:%S')] WARNING: \$*"; }
die() { echo "[\$(date +'%H:%M:%S')] FATAL: \$*" >&2; exit 1; }

log "=== NexusOS Fresh Install: Chroot Phase ==="

log "Step 1: Updating packages and installing nala..."
apt-get update -qq || die "apt-get update failed"
apt-get install -y -qq nala || die "nala install failed"

log "Step 2: Installing kernel, ZFS, and base packages..."
nala install --assume-yes \
    linux-generic linux-headers-generic \
    zfsutils-linux zfs-initramfs zfs-zed \
    efibootmgr kexec-tools \
    locales tzdata sudo nano curl wget ca-certificates gnupg lsb-release \
    software-properties-common apt-transport-https \
    build-essential python3-pip python3-venv python3-dev \
    || die "Base package install failed"

log "Step 3: Configuring locale..."
locale-gen en_US.UTF-8
update-locale LANG=en_US.UTF-8

log "Step 4: Creating user ${INSTALL_USERNAME}..."
useradd -m -s /bin/bash -G sudo,adm,cdrom,dip,plugdev ${INSTALL_USERNAME} || die "useradd failed"
echo "${INSTALL_USERNAME}:${INSTALL_PASSWORD}" | chpasswd
echo "root:${INSTALL_PASSWORD}" | chpasswd
echo "${INSTALL_USERNAME} ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/${INSTALL_USERNAME}
chmod 440 /etc/sudoers.d/${INSTALL_USERNAME}
CHROOT_HEADER

    # ---- System76 PPA ----
    cat >> "$script" <<'CHROOT_S76'

log "Step 5: Adding System76 PPA..."
add-apt-repository -y ppa:system76-dev/stable 2>/dev/null || warn "System76 PPA failed"
nala update
nala install --assume-yes system76-power 2>/dev/null || warn "system76-power not available"
systemctl enable system76-power 2>/dev/null || true
CHROOT_S76

    # ---- NVIDIA drivers (conditional) ----
    if [[ $HAS_NVIDIA -eq 1 ]]; then
        cat >> "$script" <<'CHROOT_NVIDIA'

log "Step 6: Installing NVIDIA drivers..."
nala install --assume-yes nvidia-driver-535 nvidia-dkms-535 nvidia-utils-535 2>/dev/null || \
    nala install --assume-yes nvidia-driver-530 nvidia-dkms-530 nvidia-utils-530 2>/dev/null || \
    warn "NVIDIA driver install failed — install manually after boot"
CHROOT_NVIDIA
    else
        cat >> "$script" <<'CHROOT_NO_NVIDIA'

log "Step 6: Skipping NVIDIA drivers (no GPU detected)..."
CHROOT_NO_NVIDIA
    fi

    # ---- KDE Plasma ----
    cat >> "$script" <<'CHROOT_KDE'

log "Step 7: Installing KDE Plasma Desktop..."
nala install --assume-yes \
    kde-plasma-desktop sddm plasma-nm \
    konsole dolphin kate ark kde-spectacle gwenview okular \
    plasma-systemmonitor kde-config-sddm kde-config-screenlocker \
    || die "KDE install failed"

systemctl enable sddm
systemctl enable NetworkManager

mkdir -p /etc/sddm.conf.d
cat > /etc/sddm.conf.d/nexusos.conf <<'SDDMCFG'
[General]
InputMethod=

[Theme]
Current=breeze

[Users]
MaximumUid=60000
MinimumUid=1000

[X11]
SessionDir=/usr/share/xsessions
SDDMCFG
CHROOT_KDE

    # ---- ZFSBootMenu ----
    cat >> "$script" <<CHROOT_ZBM

log "Step 8: Installing ZFSBootMenu..."
mkdir -p /boot/efi/EFI/ZBM
curl -fsSL -o /boot/efi/EFI/ZBM/VMLINUZ.EFI https://get.zfsbootmenu.org/efi \\
    || die "Failed to download ZFSBootMenu EFI binary"

zfs set org.zfsbootmenu:commandline="ro quiet loglevel=0" rpool/ROOT/nexusos \\
    || warn "ZBM commandline property failed"

efibootmgr --create --disk ${TARGET_DISK} --part 1 \\
    --loader '\\EFI\\ZBM\\VMLINUZ.EFI' \\
    --label "NexusOS (ZFSBootMenu)" \\
    --unicode || warn "EFI boot entry creation failed — configure manually after boot"
CHROOT_ZBM

    # ---- Gaming (conditional) ----
    if [[ "$INSTALL_GAMING" == true ]]; then
        cat >> "$script" <<'CHROOT_GAMING'

log "Step 9: Installing gaming stack..."
dpkg --add-architecture i386
nala update
nala install --assume-yes \
    steam-installer \
    gamemode libgamemode0 libgamemodeauto0 \
    mangohud \
    wine64 wine32 winetricks \
    vulkan-tools mesa-vulkan-drivers mesa-vulkan-drivers:i386 \
    || warn "Some gaming packages failed"

add-apt-repository -y ppa:lutris-team/lutris 2>/dev/null || true
nala update
nala install --assume-yes lutris 2>/dev/null || warn "Lutris install failed"

nala install --assume-yes irqbalance cpufrequtils 2>/dev/null || true
systemctl enable gamemode 2>/dev/null || true
systemctl enable irqbalance 2>/dev/null || true

cat > /etc/sysctl.d/99-nexus-gaming.conf <<'GAMESYS'
vm.swappiness=10
vm.dirty_ratio=15
vm.dirty_background_ratio=5
vm.max_map_count=2147483642
kernel.sched_autogroup_enabled=1
GAMESYS
CHROOT_GAMING
    else
        cat >> "$script" <<'CHROOT_NO_GAMING'

log "Step 9: Skipping gaming packages..."
CHROOT_NO_GAMING
    fi

    # ---- Docker + Media (conditional) ----
    if [[ "$INSTALL_MEDIA" == true ]]; then
        cat >> "$script" <<CHROOT_MEDIA

log "Step 10: Installing Docker and media stack..."
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
chmod a+r /etc/apt/keyrings/docker.asc
echo "deb [arch=\$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] \\
https://download.docker.com/linux/ubuntu jammy stable" > /etc/apt/sources.list.d/docker.list
nala update
nala install --assume-yes docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin \\
    || die "Docker install failed"

systemctl enable docker
usermod -aG docker ${INSTALL_USERNAME}

mkdir -p /home/${INSTALL_USERNAME}/nexus-media/{media,downloads,config}
mkdir -p /home/${INSTALL_USERNAME}/nexus-media/media/{movies,tv,music,books,audiobooks,comics}
chown -R ${INSTALL_USERNAME}:${INSTALL_USERNAME} /home/${INSTALL_USERNAME}/nexus-media
CHROOT_MEDIA
    else
        cat >> "$script" <<'CHROOT_NO_MEDIA'

log "Step 10: Skipping media stack..."
CHROOT_NO_MEDIA
    fi

    # ---- AI services (conditional) ----
    if [[ "$INSTALL_AI" == true ]]; then
        cat >> "$script" <<'CHROOT_AI'

log "Step 11: Installing AI services..."
nala install --assume-yes python3-pip python3-venv python3-dev python3-psutil python3-aiohttp || true
pip3 install --break-system-packages fastapi uvicorn httpx 2>/dev/null || \
    pip3 install fastapi uvicorn httpx 2>/dev/null || warn "FastAPI install failed"

curl -fsSL https://ollama.ai/install.sh | sh 2>/dev/null || warn "Ollama install failed"
if command -v ollama &>/dev/null; then
    useradd -r -s /bin/false -d /opt/ollama ollama 2>/dev/null || true
    mkdir -p /opt/ollama
    chown -R ollama:ollama /opt/ollama 2>/dev/null || true
    cat > /etc/systemd/system/ollama.service <<'OLLSRV'
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
OLLSRV
    systemctl daemon-reload
    systemctl enable ollama
fi
CHROOT_AI
    else
        cat >> "$script" <<'CHROOT_NO_AI'

log "Step 11: Skipping AI services..."
CHROOT_NO_AI
    fi

    # ---- Dev tools (conditional) ----
    if [[ "$INSTALL_DEV" == true ]]; then
        cat >> "$script" <<CHROOT_DEV

log "Step 12: Installing development tools..."
nala install --assume-yes \\
    build-essential cmake pkg-config libssl-dev \\
    git git-lfs nodejs npm \\
    htop btop tmux screen neovim jq \\
    || warn "Some dev packages failed"

su - ${INSTALL_USERNAME} -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y" 2>/dev/null || warn "Rust install failed"
CHROOT_DEV
    else
        cat >> "$script" <<'CHROOT_NO_DEV'

log "Step 12: Skipping dev tools..."
CHROOT_NO_DEV
    fi

    # ---- i9-13900HX tuning (conditional) ----
    if [[ $IS_I9_13900HX -eq 1 ]]; then
        cat >> "$script" <<'CHROOT_I9'

log "Applying i9-13900HX optimizations..."
cat > /etc/sysctl.d/99-nexus-i9.conf <<'I9OPT'
kernel.sched_migration_cost_ns=500000
net.core.rmem_max=134217728
net.core.wmem_max=134217728
net.ipv4.tcp_rmem=4096 87380 67108864
net.ipv4.tcp_wmem=4096 65536 67108864
I9OPT
CHROOT_I9
    fi

    # ---- NexusOS directories and finalization ----
    cat >> "$script" <<CHROOT_FINAL

log "Step 13: Setting up NexusOS directories..."
mkdir -p /opt/nexus-os/{bin,lib,share,etc,models,services}
mkdir -p /etc/nexus-os/{gaming,media,desktop,services}
mkdir -p /var/log/nexus-os

log "Step 14: Updating initramfs..."
update-initramfs -u -k all || die "initramfs update failed"

log "Step 15: Final ownership fixes..."
chown -R ${INSTALL_USERNAME}:${INSTALL_USERNAME} /home/${INSTALL_USERNAME}

log "=== NexusOS chroot installation complete ==="
CHROOT_FINAL

    chmod +x "$script"
    print_success "Chroot installation script generated"
}

fresh_run_chroot() {
    log "Running chroot installation (this takes 15-30 minutes)..."
    chroot "$MNTROOT" /root/nexus-chroot-install.sh || die "Chroot installation failed"
    print_success "Chroot installation complete"
    save_state "CHROOT_COMPLETE"
}

fresh_cleanup() {
    log "Cleaning up fresh install..."

    # Copy NexusOS repo files into the new root
    if [[ -f "$REPO_DIR/core/bin/nexuspkg" ]]; then
        cp "$REPO_DIR/core/bin/nexuspkg" "$MNTROOT/opt/nexus-os/bin/nexuspkg"
        chmod +x "$MNTROOT/opt/nexus-os/bin/nexuspkg"
        mkdir -p "$MNTROOT/usr/local/bin"
        ln -sf /opt/nexus-os/bin/nexuspkg "$MNTROOT/usr/local/bin/nexuspkg"
    fi

    for svc in stella.py maxjr.py orchestrator.py; do
        if [[ -f "$REPO_DIR/core/services/$svc" ]]; then
            cp "$REPO_DIR/core/services/$svc" "$MNTROOT/opt/nexus-os/services/$svc"
        fi
    done

    for unit in nexus-stella.service nexus-maxjr.service nexus-orchestrator.service; do
        if [[ -f "$REPO_DIR/core/services/$unit" ]]; then
            cp "$REPO_DIR/core/services/$unit" "$MNTROOT/etc/systemd/system/$unit"
        fi
    done

    # Unmount chroot binds
    umount -R "$MNTROOT/dev"  2>/dev/null || true
    umount -R "$MNTROOT/proc" 2>/dev/null || true
    umount -R "$MNTROOT/sys"  2>/dev/null || true
    umount "$MNTROOT/boot/efi" 2>/dev/null || true

    # Unmount and export ZFS pools
    zfs umount -a 2>/dev/null || true
    zpool export bpool 2>/dev/null || true
    zpool export rpool 2>/dev/null || true

    rm -f "$STATE_FILE"
    print_success "Fresh install cleanup complete"
}

# ═══════════════════════════════════════════════════════════════════════════
# SHARED: NEXUSOS DESKTOP INTEGRATION
# ═══════════════════════════════════════════════════════════════════════════

setup_nexus_integration() {
    log "Setting up NexusOS integration..."

    local user_home="/home/$INSTALL_USERNAME"

    # Install nexuspkg from the repo
    if [[ -f "$REPO_DIR/core/bin/nexuspkg" ]]; then
        cp "$REPO_DIR/core/bin/nexuspkg" "$BASE_DIR/bin/nexuspkg"
        chmod +x "$BASE_DIR/bin/nexuspkg"
        ln -sf "$BASE_DIR/bin/nexuspkg" /usr/local/bin/nexuspkg
        print_success "nexuspkg installed to /usr/local/bin/nexuspkg"
    fi

    # Desktop entry files
    local apps_dir="$user_home/.local/share/applications"
    mkdir -p "$apps_dir"

    cat > "$apps_dir/nexus-system-services.desktop" <<DESKSVC
[Desktop Entry]
Name=NexusOS System Services
Comment=Unified system management with Stella and Max Jr.
Exec=xdg-open http://localhost:8600
Icon=utilities-system-monitor
Type=Application
Categories=System;Settings;
DESKSVC

    cat > "$apps_dir/nexus-media.desktop" <<DESKMEDIA
[Desktop Entry]
Name=NexusOS Media Center
Comment=Access your media stack dashboard
Exec=xdg-open http://localhost:8500
Icon=applications-multimedia
Type=Application
Categories=AudioVideo;Video;
DESKMEDIA

    chown -R "$INSTALL_USERNAME:$INSTALL_USERNAME" "$apps_dir"

    # Autostart entry
    local autostart_dir="$user_home/.config/autostart"
    mkdir -p "$autostart_dir"
    cp "$apps_dir/nexus-system-services.desktop" "$autostart_dir/"
    chown -R "$INSTALL_USERNAME:$INSTALL_USERNAME" "$user_home/.config"

    # NexusOS master configuration
    cat > "$CONFIG_DIR/nexusos.conf" <<NEXUSCFG
[Desktop]
Theme=breeze-dark
Environment=KDE
DisplayServer=X11

[AI]
StellaPort=8601
MaxJrPort=8602
OrchestratorPort=8600
OllamaPort=11434

[Gaming]
GameModeEnabled=$INSTALL_GAMING
PerformanceOverlay=$INSTALL_GAMING

[MediaStack]
Enabled=$INSTALL_MEDIA
DashboardPort=8500

[System]
Version=$NEXUS_VERSION
Codename=$NEXUS_CODENAME
BaseDistro=Pop!_OS 22.04
PackageManager=nala
NEXUSCFG

    print_success "NexusOS desktop integration configured"
}

# ═══════════════════════════════════════════════════════════════════════════
# INSTALLATION SUMMARY
# ═══════════════════════════════════════════════════════════════════════════

installation_summary() {
    print_header
    echo -e "${GREEN}  NexusOS Installation Complete!${NC}"
    echo ""
    echo -e "${WHITE}Installation Summary:${NC}"
    echo "  Version:  $NEXUS_VERSION ($NEXUS_CODENAME)"
    echo "  Mode:     $INSTALL_MODE"
    echo "  Profile:  $PROFILE_NAME"
    echo "  User:     $INSTALL_USERNAME"
    echo ""

    echo -e "${CYAN}Installed Components:${NC}"
    echo "  + KDE Plasma Desktop (SDDM, X11)"

    [[ "$INSTALL_GAMING" == true ]] && echo "  + Gaming: Steam, Lutris, Wine, GameMode, MangoHUD"
    [[ "$INSTALL_MEDIA" == true ]]  && echo "  + Media: Docker + media stack services"
    [[ "$INSTALL_AI" == true ]]     && echo "  + AI: Ollama, Stella (8601), Max Jr. (8602), Orchestrator (8600)"
    [[ "$INSTALL_DEV" == true ]]    && echo "  + Dev: Rust, Node.js, build tools, neovim"
    [[ "$INSTALL_ZFS_DATA" == true ]] && echo "  + ZFS data pool: nexus-data"

    if [[ "$INSTALL_MODE" == "fresh" ]]; then
        echo "  + ZFS-on-root with ZFSBootMenu"
    fi

    echo ""
    echo -e "${WHITE}Next Steps:${NC}"

    if [[ "$INSTALL_MODE" == "overlay" ]]; then
        echo "  1. Reboot to switch to KDE Plasma (SDDM login screen)"
        echo "  2. Select 'Plasma (X11)' session at login"
        [[ "$INSTALL_MEDIA" == true ]] && echo "  3. Start media stack: cd ~/nexus-media && docker compose up -d"
        [[ "$INSTALL_AI" == true ]]    && echo "  4. AI services start automatically via systemd"
    else
        echo "  1. Remove installation media and reboot"
        echo "  2. ZFSBootMenu will load — select NexusOS"
        echo "  3. Log in as: $INSTALL_USERNAME"
        [[ "$INSTALL_MEDIA" == true ]] && echo "  4. Start media stack: cd ~/nexus-media && docker compose up -d"
    fi

    echo ""
    echo "  nexuspkg is available system-wide for universal package management."
    echo "  Logs saved to: $LOG_FILE"
    echo ""
    print_stella "Security monitoring systems ready."
    print_maxjr "Performance optimization active."
    echo ""
}

# ═══════════════════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════════════════

main() {
    init_logging

    # Preflight
    run_preflight

    # Mode selection
    select_install_mode

    # Profile and component selection
    select_profile
    select_custom_components

    # Mode-specific disk selection
    if [[ "$INSTALL_MODE" == "overlay" ]]; then
        select_zfs_data_pool
    else
        select_fresh_disk
    fi

    # Confirmation
    show_install_summary

    # ── OVERLAY INSTALL ──
    if [[ "$INSTALL_MODE" == "overlay" ]]; then
        overlay_prepare_system

        if [[ "$INSTALL_ZFS_DATA" == true ]]; then
            install_zfs_packages
            create_zfs_data_pool
        fi

        overlay_install_kde
        overlay_install_gaming
        overlay_install_media
        overlay_install_ai
        overlay_install_dev
        setup_nexus_integration

    # ── FRESH INSTALL ──
    elif [[ "$INSTALL_MODE" == "fresh" ]]; then
        install_zfs_packages

        # Resume support — pick up from last completed step
        local state
        state=$(get_state)

        if [[ "$state" == "START" ]]; then
            fresh_prepare_disk
            state="DISK_PREPARED"
        fi

        if [[ "$state" == "DISK_PREPARED" ]]; then
            fresh_create_zfs_pools
            state="POOLS_CREATED"
        fi

        if [[ "$state" == "POOLS_CREATED" ]]; then
            fresh_create_datasets
            state="DATASETS_CREATED"
        fi

        if [[ "$state" == "DATASETS_CREATED" ]]; then
            fresh_mount_filesystems
            state="FS_MOUNTED"
        fi

        if [[ "$state" == "FS_MOUNTED" ]]; then
            fresh_install_base
            state="BASE_INSTALLED"
        fi

        if [[ "$state" == "BASE_INSTALLED" ]]; then
            fresh_configure_base
            state="BASE_CONFIGURED"
        fi

        if [[ "$state" == "BASE_CONFIGURED" ]]; then
            fresh_prepare_chroot
            fresh_generate_chroot_script
            fresh_run_chroot
        fi

        fresh_cleanup
    fi

    # Summary
    installation_summary

    if [[ "$INSTALL_MODE" == "fresh" ]]; then
        echo ""
        if confirm_prompt "Reboot now?"; then
            reboot
        fi
    fi
}

main "$@"
