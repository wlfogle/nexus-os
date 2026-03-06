#!/usr/bin/env bash
# =============================================================================
# NexusOS ISO Builder — Standalone Distribution
# Builds NexusOS from a minimal debootstrap root. No base ISO required.
# KDE Plasma X11 + SDDM + NVIDIA + AI Services
#
# Usage: sudo ./build-iso.sh [--output /path] [--mirror URL] [--no-nvidia]
# =============================================================================
set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
readonly BUILD_DATE="$(date +%Y.%m.%d)"
readonly ISO_LABEL="NEXUSOS_1_0"
readonly ISO_NAME="nexusos-1.0-${BUILD_DATE}-x86_64.iso"
readonly WORK_DIR="/tmp/nexusos-build-$$"
readonly ROOT="${WORK_DIR}/rootfs"
readonly ISO_DIR="${WORK_DIR}/iso"
readonly NEXUS_DIR="/opt/nexus-os"
readonly SUITE="jammy"
readonly ARCH="amd64"

OUTPUT_DIR="${PROJECT_ROOT}/build"
MIRROR="http://archive.ubuntu.com/ubuntu"
INCLUDE_NVIDIA=1
LIVE_USER="nexus"

# Colors
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly CYAN='\033[0;36m'
readonly BOLD='\033[1m'
readonly NC='\033[0m'

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
log()  { echo -e "${GREEN}[nexusos]${NC} $*"; }
warn() { echo -e "${YELLOW}[warn]${NC}   $*"; }
err()  { echo -e "${RED}[error]${NC}  $*"; }
die()  { err "$*"; cleanup; exit 1; }

cleanup() {
    log "Cleaning up..."
    for mnt in "${ROOT}/dev/pts" "${ROOT}/dev" "${ROOT}/proc" \
               "${ROOT}/sys" "${ROOT}/run" "${ROOT}/tmp"; do
        umount -lf "$mnt" 2>/dev/null || true
    done
    rm -rf "$WORK_DIR"
}
trap cleanup EXIT

usage() {
    cat <<EOF
${BOLD}NexusOS ISO Builder${NC} — Standalone Distribution

Usage: sudo $0 [OPTIONS]

Options:
  --output PATH    Output directory (default: ${PROJECT_ROOT}/build)
  --mirror URL     APT mirror (default: archive.ubuntu.com)
  --no-nvidia      Skip NVIDIA driver installation
  --help           Show this help

Example:
  sudo $0
  sudo $0 --output ~/iso-out --no-nvidia
EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --output)       OUTPUT_DIR="$2"; shift 2 ;;
            --mirror)       MIRROR="$2"; shift 2 ;;
            --no-nvidia)    INCLUDE_NVIDIA=0; shift ;;
            --help)         usage; exit 0 ;;
            *)              die "Unknown option: $1" ;;
        esac
    done
}

# ---------------------------------------------------------------------------
# Chroot Helpers
# ---------------------------------------------------------------------------
mount_chroot() {
    mount --bind /dev     "${ROOT}/dev"
    mount --bind /dev/pts "${ROOT}/dev/pts"
    mount -t proc proc    "${ROOT}/proc"
    mount -t sysfs sys    "${ROOT}/sys"
    mount -t tmpfs tmpfs  "${ROOT}/run"
    mount -t tmpfs tmpfs  "${ROOT}/tmp"

    local resolv="${ROOT}/etc/resolv.conf"
    [[ -L "$resolv" ]] && rm -f "$resolv"
    cp /etc/resolv.conf "$resolv"
}

umount_chroot() {
    umount -lf "${ROOT}/dev/pts" 2>/dev/null || true
    umount -lf "${ROOT}/dev"     2>/dev/null || true
    umount -lf "${ROOT}/proc"    2>/dev/null || true
    umount -lf "${ROOT}/sys"     2>/dev/null || true
    umount -lf "${ROOT}/run"     2>/dev/null || true
    umount -lf "${ROOT}/tmp"     2>/dev/null || true
}

run_in_chroot() {
    chroot "$ROOT" /bin/bash -c "$1"
}

# ---------------------------------------------------------------------------
# Step 1: Check Build Dependencies (on host)
# ---------------------------------------------------------------------------
check_deps() {
    log "Checking build dependencies..."

    local deps=(debootstrap squashfs-tools xorriso mtools grub-pc-bin
                grub-efi-amd64-bin grub-efi-amd64-signed shim-signed
                rsync dosfstools)
    local missing=()

    for dep in "${deps[@]}"; do
        if ! dpkg -l "$dep" &>/dev/null; then
            missing+=("$dep")
        fi
    done

    if (( ${#missing[@]} > 0 )); then
        log "Installing: ${missing[*]}"
        if command -v nala &>/dev/null; then
            nala install -y "${missing[@]}"
        else
            apt-get install -y "${missing[@]}"
        fi
    fi

    log "Build dependencies satisfied"
}

# ---------------------------------------------------------------------------
# Step 2: Bootstrap Minimal Root
# ---------------------------------------------------------------------------
bootstrap() {
    log "Bootstrapping minimal ${SUITE} root..."
    mkdir -p "$ROOT"

    debootstrap --arch="$ARCH" --variant=minbase \
        --include=apt,apt-utils,locales,sudo,systemd,systemd-sysv,dbus,ca-certificates \
        "$SUITE" "$ROOT" "$MIRROR"

    log "Bootstrap complete"
}

# ---------------------------------------------------------------------------
# Step 3: Configure APT Sources
# ---------------------------------------------------------------------------
configure_apt() {
    log "Configuring APT sources..."

    cat > "${ROOT}/etc/apt/sources.list" << APTEOF
deb ${MIRROR} ${SUITE} main restricted universe multiverse
deb ${MIRROR} ${SUITE}-updates main restricted universe multiverse
deb ${MIRROR} ${SUITE}-security main restricted universe multiverse
deb ${MIRROR} ${SUITE}-backports main restricted universe multiverse
APTEOF

    mount_chroot

    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -qq
    "

    if [[ "$INCLUDE_NVIDIA" -eq 1 ]]; then
        run_in_chroot "
            export DEBIAN_FRONTEND=noninteractive
            apt-get install -y -qq software-properties-common
            add-apt-repository -y ppa:graphics-drivers/ppa
            apt-get update -qq
        "
    fi

    umount_chroot
    log "APT configured"
}

# ---------------------------------------------------------------------------
# Step 4: Install Kernel & Base System
# ---------------------------------------------------------------------------
install_base() {
    log "Installing kernel and base system..."

    mount_chroot

    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive

        apt-get install -y -qq \
            linux-generic linux-firmware

        apt-get install -y -qq \
            systemd-timesyncd networkmanager \
            bash bash-completion \
            sudo adduser passwd \
            nala \
            iproute2 iputils-ping net-tools \
            pciutils usbutils lshw dmidecode \
            e2fsprogs dosfstools parted gdisk \
            btrfs-progs xfsprogs \
            lvm2 cryptsetup \
            grub-efi-amd64-signed grub-pc-bin shim-signed \
            isolinux syslinux syslinux-common syslinux-efi \
            initramfs-tools plymouth plymouth-themes \
            openssh-server \
            ufw fail2ban \
            curl wget rsync git jq htop neofetch tree \
            nano less man-db \
            pipewire pipewire-pulse wireplumber \
            flatpak \
            unattended-upgrades apt-transport-https

        # Bluetooth
        apt-get install -y -qq \
            bluez bluez-tools rfkill

        # Printing
        apt-get install -y -qq \
            cups cups-browsed cups-filters system-config-printer

        # Media codecs
        apt-get install -y -qq \
            ffmpeg gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
            gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
            gstreamer1.0-libav gstreamer1.0-vaapi \
            libavcodec-extra \
            2>/dev/null || echo '[warn] Some codec packages unavailable'

        # Power management
        apt-get install -y -qq \
            power-profiles-daemon thermald \
            2>/dev/null || echo '[warn] Power management packages partially unavailable'

        # Firmware updates & Thunderbolt
        apt-get install -y -qq \
            fwupd udisks2 bolt

        # Zram swap
        apt-get install -y -qq \
            zram-tools 2>/dev/null || true

        # Office
        apt-get install -y -qq \
            libreoffice-calc libreoffice-writer libreoffice-impress \
            libreoffice-gtk3 \
            2>/dev/null || echo '[warn] LibreOffice install had issues'

        # Accessibility
        apt-get install -y -qq \
            at-spi2-core orca speech-dispatcher \
            2>/dev/null || true

        # AMD/Intel GPU support (for non-NVIDIA hardware)
        apt-get install -y -qq \
            mesa-vulkan-drivers mesa-va-drivers mesa-vdpau-drivers \
            libdrm-amdgpu1 libdrm-radeon1 \
            xserver-xorg-video-amdgpu xserver-xorg-video-intel \
            2>/dev/null || true
    "

    umount_chroot
    log "Base system installed"
}

# ---------------------------------------------------------------------------
# Step 5: Install KDE Plasma X11
# ---------------------------------------------------------------------------
install_kde() {
    log "Installing KDE Plasma X11 desktop..."

    mount_chroot

    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive

        apt-get install -y -qq \
            kde-plasma-desktop plasma-workspace \
            sddm sddm-theme-breeze \
            konsole dolphin ark kate okular \
            gwenview spectacle \
            plasma-nm plasma-pa plasma-systemmonitor \
            kde-config-sddm \
            breeze breeze-cursor-theme breeze-icon-theme \
            kde-style-breeze \
            kscreen kinfocenter \
            polkit-kde-agent-1 \
            xdg-utils xdg-user-dirs \
            firefox \
            xserver-xorg-core xserver-xorg-input-all \
            xserver-xorg-video-all \
            xinit x11-xserver-utils

        # KDE Bluetooth, printing, and software center
        apt-get install -y -qq \
            bluedevil \
            print-manager \
            plasma-discover plasma-discover-backend-flatpak \
            kde-spectacle partitionmanager \
            2>/dev/null || true

        echo 'sddm sddm/daemon select sddm' | debconf-set-selections
        dpkg-reconfigure -f noninteractive sddm 2>/dev/null || true
        systemctl enable sddm
        systemctl enable bluetooth 2>/dev/null || true
        systemctl enable cups 2>/dev/null || true

        # Setup Flathub
        flatpak remote-add --if-not-exists flathub \
            https://dl.flathub.org/repo/flathub.flatpakrepo 2>/dev/null || true
    "

    umount_chroot
    log "KDE Plasma X11 installed"
}

# ---------------------------------------------------------------------------
# Step 6: Install NVIDIA Drivers
# ---------------------------------------------------------------------------
install_nvidia() {
    if [[ "$INCLUDE_NVIDIA" -ne 1 ]]; then
        log "Skipping NVIDIA drivers (--no-nvidia)"
        return
    fi

    log "Installing NVIDIA drivers..."

    mount_chroot

    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        apt-get install -y -qq \
            nvidia-driver-550 \
            nvidia-utils-550 \
            libnvidia-decode-550 libnvidia-encode-550 \
            nvidia-cuda-toolkit \
            2>/dev/null || {
                echo '[warn] NVIDIA 550 unavailable, trying 535...'
                apt-get install -y -qq \
                    nvidia-driver-535 \
                    nvidia-utils-535 \
                    2>/dev/null || echo '[warn] NVIDIA install failed — will use nouveau'
            }
    "

    umount_chroot
    log "NVIDIA drivers installed"
}

# ---------------------------------------------------------------------------
# Step 7: Install NexusOS Application Packages
# ---------------------------------------------------------------------------
install_nexus_packages() {
    log "Installing NexusOS application packages..."

    mount_chroot

    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive

        apt-get install -y -qq \
            docker.io docker-compose containerd

        apt-get install -y -qq \
            python3 python3-pip python3-venv \
            build-essential cmake

        pip3 install --break-system-packages \
            fastapi uvicorn psutil 2>/dev/null || \
        pip3 install fastapi uvicorn psutil

        apt-get install -y -qq \
            calamares calamares-settings-debian \
            2>/dev/null || echo '[warn] Calamares not in repos — install manually'
    "

    umount_chroot
    log "NexusOS packages installed"
}

# ---------------------------------------------------------------------------
# Step 8: Apply NexusOS Overlay
# ---------------------------------------------------------------------------
apply_overlay() {
    log "Applying NexusOS overlay..."

    mkdir -p "${ROOT}${NEXUS_DIR}"/{bin,services,configs,branding,docs}
    mkdir -p "${ROOT}/var/log/nexus-os"
    mkdir -p "${ROOT}/etc/nexus-os"
    mkdir -p "${ROOT}/usr/share/applications"

    # ── CLI tools ─────────────────────────────────────────────────────────
    if [[ -d "${PROJECT_ROOT}/core/bin" ]]; then
        cp -a "${PROJECT_ROOT}/core/bin/"* "${ROOT}${NEXUS_DIR}/bin/" 2>/dev/null || true
        for bin in "${ROOT}${NEXUS_DIR}/bin/"*; do
            if [[ -f "$bin" ]]; then
                chmod +x "$bin"
                ln -sf "${NEXUS_DIR}/bin/$(basename "$bin")" \
                       "${ROOT}/usr/local/bin/$(basename "$bin")"
            fi
        done
    fi

    # ── Python services ───────────────────────────────────────────────────
    if [[ -d "${PROJECT_ROOT}/core/services" ]]; then
        for f in "${PROJECT_ROOT}/core/services/"*.py; do
            [[ -f "$f" ]] && cp "$f" "${ROOT}${NEXUS_DIR}/services/"
        done
        for f in "${PROJECT_ROOT}/core/services/"*.sh; do
            [[ -f "$f" ]] && { cp "$f" "${ROOT}${NEXUS_DIR}/services/"; \
                               chmod +x "${ROOT}${NEXUS_DIR}/services/$(basename "$f")"; }
        done
        for f in "${PROJECT_ROOT}/core/services/"*.service "${PROJECT_ROOT}/core/services/"*.timer; do
            [[ -f "$f" ]] && cp "$f" "${ROOT}/etc/systemd/system/"
        done
    fi

    # ── Security ──────────────────────────────────────────────────────────
    if [[ -f "${PROJECT_ROOT}/core/security/nexus-harden.sh" ]]; then
        cp "${PROJECT_ROOT}/core/security/nexus-harden.sh" "${ROOT}${NEXUS_DIR}/bin/"
        chmod +x "${ROOT}${NEXUS_DIR}/bin/nexus-harden.sh"
        ln -sf "${NEXUS_DIR}/bin/nexus-harden.sh" "${ROOT}/usr/local/bin/nexus-harden"
    fi

    # ── AI setup ──────────────────────────────────────────────────────────
    if [[ -f "${PROJECT_ROOT}/core/ai/ollama-setup.sh" ]]; then
        cp "${PROJECT_ROOT}/core/ai/ollama-setup.sh" "${ROOT}${NEXUS_DIR}/bin/"
        chmod +x "${ROOT}${NEXUS_DIR}/bin/ollama-setup.sh"
        ln -sf "${NEXUS_DIR}/bin/ollama-setup.sh" "${ROOT}/usr/local/bin/ollama-setup"
    fi

    # ── Kernel / system configs ───────────────────────────────────────────
    mkdir -p "${ROOT}/etc/sysctl.d" "${ROOT}/etc/modprobe.d" \
             "${ROOT}/etc/udev/rules.d" "${ROOT}/etc/security/limits.d"

    [[ -f "${PROJECT_ROOT}/core/config/sysctl-nexus.conf" ]] && \
        cp "${PROJECT_ROOT}/core/config/sysctl-nexus.conf" "${ROOT}/etc/sysctl.d/90-nexus.conf"
    [[ -f "${PROJECT_ROOT}/core/config/modprobe-nvidia.conf" ]] && \
        cp "${PROJECT_ROOT}/core/config/modprobe-nvidia.conf" "${ROOT}/etc/modprobe.d/nexus-nvidia.conf"
    [[ -f "${PROJECT_ROOT}/core/config/udev-gaming.rules" ]] && \
        cp "${PROJECT_ROOT}/core/config/udev-gaming.rules" "${ROOT}/etc/udev/rules.d/99-nexus-gaming.rules"
    [[ -f "${PROJECT_ROOT}/core/config/limits-nexus.conf" ]] && \
        cp "${PROJECT_ROOT}/core/config/limits-nexus.conf" "${ROOT}/etc/security/limits.d/90-nexus.conf"
    [[ -f "${PROJECT_ROOT}/core/config/zram-nexus.conf" ]] && {
        mkdir -p "${ROOT}/etc/default"
        cp "${PROJECT_ROOT}/core/config/zram-nexus.conf" "${ROOT}/etc/default/zramswap"
    }

    # ── Branding ──────────────────────────────────────────────────────────
    [[ -f "${PROJECT_ROOT}/core/branding/motd" ]] && \
        cp "${PROJECT_ROOT}/core/branding/motd" "${ROOT}/etc/motd"
    [[ -f "${PROJECT_ROOT}/core/branding/issue" ]] && {
        cp "${PROJECT_ROOT}/core/branding/issue" "${ROOT}/etc/issue"
        cp "${PROJECT_ROOT}/core/branding/issue" "${ROOT}/etc/issue.net"
    }

    if [[ -f "${PROJECT_ROOT}/core/branding/neofetch-nexus.conf" ]]; then
        mkdir -p "${ROOT}/etc/skel/.config/neofetch"
        cp "${PROJECT_ROOT}/core/branding/neofetch-nexus.conf" \
           "${ROOT}/etc/skel/.config/neofetch/config.conf"
    fi

    if [[ -d "${PROJECT_ROOT}/core/branding/plymouth" ]]; then
        mkdir -p "${ROOT}/usr/share/plymouth/themes/nexusos"
        cp -a "${PROJECT_ROOT}/core/branding/plymouth/"* \
              "${ROOT}/usr/share/plymouth/themes/nexusos/" 2>/dev/null || true
    fi

    if [[ -d "${PROJECT_ROOT}/core/branding/sddm-theme" ]]; then
        mkdir -p "${ROOT}/usr/share/sddm/themes/nexusos"
        cp -a "${PROJECT_ROOT}/core/branding/sddm-theme/"* \
              "${ROOT}/usr/share/sddm/themes/nexusos/" 2>/dev/null || true
    fi

    if [[ -d "${PROJECT_ROOT}/core/branding/wallpaper" ]]; then
        mkdir -p "${ROOT}/usr/share/wallpapers/NexusOS/contents/images"
        # Copy metadata to wallpaper root, images to images subdir
        [[ -f "${PROJECT_ROOT}/core/branding/wallpaper/metadata.desktop" ]] && \
            cp "${PROJECT_ROOT}/core/branding/wallpaper/metadata.desktop" \
               "${ROOT}/usr/share/wallpapers/NexusOS/metadata.desktop"
        if [[ -d "${PROJECT_ROOT}/core/branding/wallpaper/contents/images" ]]; then
            cp -a "${PROJECT_ROOT}/core/branding/wallpaper/contents/images/"* \
                  "${ROOT}/usr/share/wallpapers/NexusOS/contents/images/" 2>/dev/null || true
        fi
    fi

    # ── Shell configs ─────────────────────────────────────────────────────
    if [[ -f "${PROJECT_ROOT}/core/shell/bashrc-nexus" ]]; then
        cp "${PROJECT_ROOT}/core/shell/bashrc-nexus" "${ROOT}/etc/skel/.bashrc-nexus"
        if [[ -f "${ROOT}/etc/skel/.bashrc" ]]; then
            if ! grep -q 'bashrc-nexus' "${ROOT}/etc/skel/.bashrc" 2>/dev/null; then
                printf '\n# NexusOS\n[ -f ~/.bashrc-nexus ] && source ~/.bashrc-nexus\n' \
                    >> "${ROOT}/etc/skel/.bashrc"
            fi
        fi
    fi
    [[ -f "${PROJECT_ROOT}/core/shell/profile-nexus" ]] && \
        cp "${PROJECT_ROOT}/core/shell/profile-nexus" "${ROOT}/etc/profile.d/nexus.sh"

    # ── KDE Desktop configs ──────────────────────────────────────────────
    mkdir -p "${ROOT}/etc/skel/.config"
    if [[ -f "${PROJECT_ROOT}/core/desktop/konsole-nexus.profile" ]]; then
        mkdir -p "${ROOT}/etc/skel/.local/share/konsole"
        cp "${PROJECT_ROOT}/core/desktop/konsole-nexus.profile" \
           "${ROOT}/etc/skel/.local/share/konsole/NexusOS.profile"
        cat > "${ROOT}/etc/skel/.config/konsolerc" << 'KEOF'
[Desktop Entry]
DefaultProfile=NexusOS.profile
KEOF
    fi

    if [[ -f "${PROJECT_ROOT}/core/desktop/sddm-nexus.conf" ]]; then
        mkdir -p "${ROOT}/etc/sddm.conf.d"
        cp "${PROJECT_ROOT}/core/desktop/sddm-nexus.conf" "${ROOT}/etc/sddm.conf.d/nexus.conf"
    fi

    [[ -f "${PROJECT_ROOT}/core/desktop/plasma-layout.js" ]] && \
        cp "${PROJECT_ROOT}/core/desktop/plasma-layout.js" "${ROOT}${NEXUS_DIR}/configs/plasma-layout.js"

    # ── .desktop application launchers ────────────────────────────────────
    if [[ -d "${PROJECT_ROOT}/core/desktop/applications" ]]; then
        cp "${PROJECT_ROOT}/core/desktop/applications/"*.desktop \
           "${ROOT}/usr/share/applications/" 2>/dev/null || true
    fi

    # ── First-run autostart ───────────────────────────────────────────────
    if [[ -f "${PROJECT_ROOT}/core/desktop/applications/nexus-first-run.desktop" ]]; then
        mkdir -p "${ROOT}/etc/xdg/autostart"
        cp "${PROJECT_ROOT}/core/desktop/applications/nexus-first-run.desktop" \
           "${ROOT}/etc/xdg/autostart/"
    fi

    # ── Media stack ───────────────────────────────────────────────────────
    if [[ -d "${PROJECT_ROOT}/core/media-stack" ]]; then
        mkdir -p "${ROOT}${NEXUS_DIR}/media-stack"
        cp -a "${PROJECT_ROOT}/core/media-stack/"* \
              "${ROOT}${NEXUS_DIR}/media-stack/" 2>/dev/null || true
    fi

    # ── Calamares installer config ────────────────────────────────────────
    if [[ -d "${PROJECT_ROOT}/core/installer" ]]; then
        mkdir -p "${ROOT}/etc/calamares"
        cp -a "${PROJECT_ROOT}/core/installer/"* \
              "${ROOT}/etc/calamares/" 2>/dev/null || true
    fi

    log "Overlay applied"
}

# ---------------------------------------------------------------------------
# Step 9: NexusOS Identity & System Config
# ---------------------------------------------------------------------------
configure_system() {
    log "Configuring NexusOS identity..."

    # /etc/os-release
    cat > "${ROOT}/etc/os-release" << OSEOF
NAME="NexusOS"
VERSION="1.0"
ID=nexusos
ID_LIKE=ubuntu
VERSION_ID="1.0"
VERSION_CODENAME=nexus
PRETTY_NAME="NexusOS 1.0 — AI-Native Operating System"
HOME_URL="https://github.com/wlfogle/nexus-os"
SUPPORT_URL="https://github.com/wlfogle/nexus-os/issues"
BUG_REPORT_URL="https://github.com/wlfogle/nexus-os/issues"
BUILD_DATE="${BUILD_DATE}"
OSEOF

    cp "${ROOT}/etc/os-release" "${ROOT}/etc/nexus-os/os-release"

    # Hostname
    echo "nexus" > "${ROOT}/etc/hostname"
    cat > "${ROOT}/etc/hosts" << 'HOSTSEOF'
127.0.0.1   localhost
127.0.1.1   nexus

::1         localhost ip6-localhost ip6-loopback
ff02::1     ip6-allnodes
ff02::2     ip6-allrouters
HOSTSEOF

    # Locale + live user + services
    mount_chroot

    run_in_chroot "
        sed -i 's/# en_US.UTF-8/en_US.UTF-8/' /etc/locale.gen
        locale-gen
        echo 'LANG=en_US.UTF-8' > /etc/default/locale
    "

    run_in_chroot "
        useradd -m -s /bin/bash -G sudo,docker,audio,video,plugdev,netdev ${LIVE_USER} 2>/dev/null || true
        echo '${LIVE_USER}:${LIVE_USER}' | chpasswd
        echo '${LIVE_USER} ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/${LIVE_USER}
        chmod 440 /etc/sudoers.d/${LIVE_USER}
    "

    run_in_chroot "
        systemctl enable NetworkManager   2>/dev/null || true
        systemctl enable sddm             2>/dev/null || true
        systemctl enable docker            2>/dev/null || true
        systemctl enable ufw               2>/dev/null || true
        systemctl enable fail2ban          2>/dev/null || true
        systemctl enable ssh               2>/dev/null || true
        systemctl enable nexus-update.timer  2>/dev/null || true
        systemctl enable nexus-health.timer  2>/dev/null || true
        systemctl enable bluetooth         2>/dev/null || true
        systemctl enable cups              2>/dev/null || true
        systemctl enable fwupd             2>/dev/null || true
        systemctl enable fstrim.timer      2>/dev/null || true
        systemctl enable power-profiles-daemon 2>/dev/null || true
    "

    # Plymouth
    run_in_chroot "
        if [[ -d /usr/share/plymouth/themes/nexusos ]] && \
           [[ -f /usr/share/plymouth/themes/nexusos/nexusos.plymouth ]]; then
            update-alternatives --install /usr/share/plymouth/themes/default.plymouth \
                default.plymouth /usr/share/plymouth/themes/nexusos/nexusos.plymouth 200 \
                2>/dev/null || true
            update-alternatives --set default.plymouth \
                /usr/share/plymouth/themes/nexusos/nexusos.plymouth 2>/dev/null || true
        fi
        update-initramfs -u 2>/dev/null || true
    "

    # SDDM theme
    if [[ -d "${ROOT}/usr/share/sddm/themes/nexusos" ]]; then
        mkdir -p "${ROOT}/etc/sddm.conf.d"
        cat > "${ROOT}/etc/sddm.conf.d/theme.conf" << 'SDDMEOF'
[Theme]
Current=nexusos
SDDMEOF
    fi

    # Clean caches
    run_in_chroot "
        apt-get clean
        rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*
    "

    umount_chroot
    log "System configured"
}

# ---------------------------------------------------------------------------
# Step 10: Build Live Filesystem
# ---------------------------------------------------------------------------
build_live_fs() {
    log "Building live filesystem..."

    mkdir -p "${ISO_DIR}"/{casper,boot/grub,EFI/boot,isolinux,.disk}

    # Install casper for live boot support
    mount_chroot
    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        apt-get update -qq
        apt-get install -y -qq casper lupin-casper 2>/dev/null || true
        apt-get clean
        rm -rf /var/lib/apt/lists/*
    "
    umount_chroot

    # Squashfs
    log "Compressing root filesystem (this takes several minutes)..."
    mksquashfs "$ROOT" "${ISO_DIR}/casper/filesystem.squashfs" \
        -comp xz -Xbcj x86 -b 1M -Xdict-size 1M -noappend

    # Manifest + size
    mount_chroot
    run_in_chroot "dpkg-query -W --showformat='\${Package} \${Version}\n'" \
        > "${ISO_DIR}/casper/filesystem.manifest" 2>/dev/null || true
    umount_chroot

    du -sx --block-size=1 "$ROOT" | awk '{print $1}' > "${ISO_DIR}/casper/filesystem.size"

    # Kernel + initrd
    local vmlinuz initrd
    vmlinuz=$(ls "${ROOT}/boot/vmlinuz-"* 2>/dev/null | sort -V | tail -1)
    initrd=$(ls "${ROOT}/boot/initrd.img-"* 2>/dev/null | sort -V | tail -1)
    [[ -z "$vmlinuz" || -z "$initrd" ]] && die "Kernel or initrd not found"

    cp "$vmlinuz" "${ISO_DIR}/casper/vmlinuz"
    cp "$initrd"  "${ISO_DIR}/casper/initrd"

    # GRUB config
    cat > "${ISO_DIR}/boot/grub/grub.cfg" << 'GRUBEOF'
set timeout=10
set default=0

insmod all_video
insmod gfxterm

menuentry "NexusOS 1.0 — Live Session (KDE Plasma X11)" {
    linux /casper/vmlinuz boot=casper quiet splash plymouth.enable=1 ---
    initrd /casper/initrd
}

menuentry "NexusOS 1.0 — Safe Graphics" {
    linux /casper/vmlinuz boot=casper quiet splash nomodeset ---
    initrd /casper/initrd
}

menuentry "NexusOS 1.0 — Load to RAM" {
    linux /casper/vmlinuz boot=casper quiet splash toram ---
    initrd /casper/initrd
}

menuentry "Check disc for defects" {
    linux /casper/vmlinuz boot=casper integrity-check quiet splash ---
    initrd /casper/initrd
}
GRUBEOF

    # Isolinux for BIOS boot
    local isolinux_bin=""
    for path in "${ROOT}/usr/lib/ISOLINUX/isolinux.bin" \
                "${ROOT}/usr/lib/syslinux/isolinux.bin" \
                /usr/lib/ISOLINUX/isolinux.bin; do
        [[ -f "$path" ]] && { isolinux_bin="$path"; break; }
    done
    if [[ -n "$isolinux_bin" ]]; then
        cp "$isolinux_bin" "${ISO_DIR}/isolinux/"
        # Copy ldlinux.c32 and other required modules
        for mod_dir in "${ROOT}/usr/lib/syslinux/modules/bios" /usr/lib/syslinux/modules/bios; do
            if [[ -d "$mod_dir" ]]; then
                cp "${mod_dir}/ldlinux.c32"    "${ISO_DIR}/isolinux/" 2>/dev/null || true
                cp "${mod_dir}/libutil.c32"    "${ISO_DIR}/isolinux/" 2>/dev/null || true
                cp "${mod_dir}/libcom32.c32"   "${ISO_DIR}/isolinux/" 2>/dev/null || true
                cp "${mod_dir}/vesamenu.c32"   "${ISO_DIR}/isolinux/" 2>/dev/null || true
                cp "${mod_dir}/menu.c32"       "${ISO_DIR}/isolinux/" 2>/dev/null || true
                break
            fi
        done

        cat > "${ISO_DIR}/isolinux/isolinux.cfg" << 'SYSEOF'
DEFAULT nexusos
TIMEOUT 100
PROMPT 1

UI vesamenu.c32
MENU TITLE NexusOS 1.0 — Boot Menu
MENU COLOR border   30;44 #40ffffff #a0000000 std
MENU COLOR title    1;36;44 #9033ccff #a0000000 std
MENU COLOR sel      7;37;40 #e0ffffff #20ffffff all
MENU COLOR unsel    37;44 #50ffffff #a0000000 std

LABEL nexusos
    MENU LABEL NexusOS 1.0 — Live Session (KDE Plasma X11)
    MENU DEFAULT
    KERNEL /casper/vmlinuz
    APPEND initrd=/casper/initrd boot=casper quiet splash ---

LABEL safe
    MENU LABEL NexusOS 1.0 — Safe Graphics
    KERNEL /casper/vmlinuz
    APPEND initrd=/casper/initrd boot=casper quiet splash nomodeset ---

LABEL toram
    MENU LABEL NexusOS 1.0 — Load to RAM
    KERNEL /casper/vmlinuz
    APPEND initrd=/casper/initrd boot=casper quiet splash toram ---

LABEL check
    MENU LABEL Check disc for defects
    KERNEL /casper/vmlinuz
    APPEND initrd=/casper/initrd boot=casper integrity-check quiet splash ---
SYSEOF
        log "BIOS boot (isolinux) configured"
    else
        warn "isolinux.bin not found — BIOS boot unavailable"
    fi

    # .disk metadata
    echo "NexusOS 1.0 \"AI-Native\" - Release x86_64 (${BUILD_DATE})" > "${ISO_DIR}/.disk/info"
    touch "${ISO_DIR}/.disk/base_installable"

    # casper.conf
    cat > "${ISO_DIR}/casper/casper.conf" << CASPEREOF
export USERNAME="${LIVE_USER}"
export USERFULLNAME="NexusOS Live User"
export HOST="nexus"
export BUILD_SYSTEM="NexusOS"
CASPEREOF

    log "Live filesystem built"
}

# ---------------------------------------------------------------------------
# Step 11: Create EFI boot image
# ---------------------------------------------------------------------------
create_efi_image() {
    log "Creating EFI boot image..."

    mkdir -p "${ISO_DIR}/EFI/boot"

    # Copy shim + grub for UEFI Secure Boot
    [[ -f "${ROOT}/usr/lib/shim/shimx64.efi.signed" ]] && \
        cp "${ROOT}/usr/lib/shim/shimx64.efi.signed" "${ISO_DIR}/EFI/boot/bootx64.efi"
    [[ -f "${ROOT}/usr/lib/grub/x86_64-efi-signed/grubx64.efi.signed" ]] && \
        cp "${ROOT}/usr/lib/grub/x86_64-efi-signed/grubx64.efi.signed" "${ISO_DIR}/EFI/boot/grubx64.efi"

    # Build EFI FAT image
    local efi_img="${ISO_DIR}/boot/grub/efi.img"
    dd if=/dev/zero of="$efi_img" bs=1M count=10 2>/dev/null
    mkfs.vfat "$efi_img" >/dev/null
    local efi_mnt="${WORK_DIR}/efi-mount"
    mkdir -p "$efi_mnt"
    mount -o loop "$efi_img" "$efi_mnt"

    mkdir -p "${efi_mnt}/EFI/boot"
    [[ -f "${ISO_DIR}/EFI/boot/bootx64.efi" ]] && \
        cp "${ISO_DIR}/EFI/boot/bootx64.efi" "${efi_mnt}/EFI/boot/"
    [[ -f "${ISO_DIR}/EFI/boot/grubx64.efi" ]] && \
        cp "${ISO_DIR}/EFI/boot/grubx64.efi" "${efi_mnt}/EFI/boot/"

    mkdir -p "${efi_mnt}/boot/grub"
    cat > "${efi_mnt}/boot/grub/grub.cfg" << 'EFIGRUBEOF'
search --set=root --file /.disk/info
set prefix=($root)/boot/grub
configfile $prefix/grub.cfg
EFIGRUBEOF

    umount "$efi_mnt"
    rmdir "$efi_mnt"

    log "EFI image created"
}

# ---------------------------------------------------------------------------
# Step 12: Build Final ISO
# ---------------------------------------------------------------------------
build_iso() {
    log "Building final ISO..."

    mkdir -p "$OUTPUT_DIR"

    if [[ -f "${ISO_DIR}/isolinux/isolinux.bin" ]] && [[ -f "${ISO_DIR}/boot/grub/efi.img" ]]; then
        # Hybrid BIOS + UEFI boot
        xorriso -as mkisofs \
            -volid "$ISO_LABEL" \
            -J -R -l -iso-level 3 \
            -b isolinux/isolinux.bin \
            -c isolinux/boot.cat \
            -no-emul-boot -boot-load-size 4 -boot-info-table \
            -eltorito-alt-boot \
            -e boot/grub/efi.img \
            -no-emul-boot \
            -append_partition 2 0xEF "${ISO_DIR}/boot/grub/efi.img" \
            -o "${OUTPUT_DIR}/${ISO_NAME}" \
            "$ISO_DIR"
    elif [[ -f "${ISO_DIR}/boot/grub/efi.img" ]]; then
        # UEFI only
        xorriso -as mkisofs \
            -volid "$ISO_LABEL" \
            -J -R -l -iso-level 3 \
            -eltorito-boot boot/grub/efi.img \
            -no-emul-boot \
            -eltorito-catalog boot/grub/boot.cat \
            -append_partition 2 0xEF "${ISO_DIR}/boot/grub/efi.img" \
            -o "${OUTPUT_DIR}/${ISO_NAME}" \
            "$ISO_DIR"
    else
        die "No bootloader images found — cannot build ISO"
    fi

    sha256sum "${OUTPUT_DIR}/${ISO_NAME}" > "${OUTPUT_DIR}/${ISO_NAME}.sha256"

    log "ISO built: ${OUTPUT_DIR}/${ISO_NAME}"
}

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
print_summary() {
    local iso_path="${OUTPUT_DIR}/${ISO_NAME}"
    local iso_size="N/A"
    [[ -f "$iso_path" ]] && iso_size="$(du -h "$iso_path" | awk '{print $1}')"

    local kernel_ver="unknown"
    local kfile
    kfile=$(ls "${ROOT}/boot/vmlinuz-"* 2>/dev/null | sort -V | tail -1)
    [[ -n "$kfile" ]] && kernel_ver="${kfile##*vmlinuz-}"

    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║              NexusOS ISO Build Complete                  ║${NC}"
    echo -e "${CYAN}╠══════════════════════════════════════════════════════════╣${NC}"
    echo -e "${CYAN}║${NC}  ISO:      ${ISO_NAME}"
    echo -e "${CYAN}║${NC}  Size:     ${iso_size}"
    echo -e "${CYAN}║${NC}  Path:     ${OUTPUT_DIR}/"
    echo -e "${CYAN}║${NC}  Desktop:  KDE Plasma X11"
    echo -e "${CYAN}║${NC}  Kernel:   ${kernel_ver}"
    echo -e "${CYAN}║${NC}  Built:    ${BUILD_DATE}"
    echo -e "${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  Write to USB:"
    echo -e "${CYAN}║${NC}    sudo dd if=${iso_path} of=/dev/sdX bs=4M status=progress oflag=sync"
    echo -e "${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}  Test in QEMU:"
    echo -e "${CYAN}║${NC}    qemu-system-x86_64 -m 4G -enable-kvm -cdrom ${iso_path}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    if [[ $EUID -ne 0 ]]; then
        die "This script must be run as root (sudo $0)"
    fi

    parse_args "$@"

    echo -e "${CYAN}"
    echo " ███╗   ██╗███████╗██╗  ██╗██╗   ██╗███████╗ ██████╗ ███████╗"
    echo " ████╗  ██║██╔════╝╚██╗██╔╝██║   ██║██╔════╝██╔═══██╗██╔════╝"
    echo " ██╔██╗ ██║█████╗   ╚███╔╝ ██║   ██║███████╗██║   ██║███████╗"
    echo " ██║╚██╗██║██╔══╝   ██╔██╗ ██║   ██║╚════██║██║   ██║╚════██║"
    echo " ██║ ╚████║███████╗██╔╝ ╚██╗╚██████╔╝███████║╚██████╔╝███████║"
    echo " ╚═╝  ╚═══╝╚══════╝╚═╝   ╚═╝ ╚═════╝ ╚══════╝ ╚═════╝ ╚══════╝"
    echo "              Standalone Distribution Builder"
    echo -e "${NC}"

    log "Output:     $OUTPUT_DIR"
    log "Mirror:     $MIRROR"
    log "NVIDIA:     $([ $INCLUDE_NVIDIA -eq 1 ] && echo 'yes' || echo 'no')"
    log "Build date: $BUILD_DATE"
    echo ""

    check_deps
    bootstrap
    configure_apt
    install_base
    install_kde
    install_nvidia
    install_nexus_packages
    apply_overlay
    configure_system
    build_live_fs
    create_efi_image
    build_iso
    print_summary
}

main "$@"
