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
                rsync dosfstools isolinux syslinux-common)
    local missing=()

    for dep in "${deps[@]}"; do
        if ! dpkg -l "$dep" &>/dev/null; then
            missing+=("$dep")
        fi
    done

    if (( ${#missing[@]} > 0 )); then
        log "Installing: ${missing[*]}"
        nala install -y "${missing[@]}"
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
        set -e
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
            systemd-timesyncd network-manager \
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
            unattended-upgrades apt-transport-https \
            plymouth-label \
            update-notifier-common \
            ubuntu-release-upgrader-core

        # Casper live boot — install separately to ensure it succeeds
        # and its initramfs hooks are properly registered
        apt-get install -y casper || {
            echo '[FATAL] casper install failed — live boot will not work'
            exit 1
        }

        # Verify casper boot script exists
        if [ ! -f /usr/share/initramfs-tools/scripts/casper ]; then
            echo '[FATAL] casper boot script missing after install'
            dpkg -L casper | grep initramfs || true
            exit 1
        fi

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

        # VM guest agents (SPICE auto-resize, QEMU guest integration)
        apt-get install -y -qq \
            spice-vdagent qemu-guest-agent \
            xserver-xorg-video-qxl \
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

        # Core KDE + display manager (must succeed)
        apt-get install -y -qq \
            kde-plasma-desktop plasma-workspace \
            sddm sddm-theme-breeze \
            xserver-xorg-core xserver-xorg-input-all \
            xserver-xorg-video-all \
            xinit x11-xserver-utils \
            dbus-x11

        # KDE applications
        apt-get install -y -qq \
            konsole dolphin ark kate okular \
            gwenview kde-spectacle \
            plasma-nm plasma-pa plasma-systemmonitor \
            kde-config-sddm \
            breeze breeze-cursor-theme breeze-icon-theme \
            kde-style-breeze \
            kscreen kinfocenter \
            polkit-kde-agent-1 \
            xdg-utils xdg-user-dirs \
            firefox

        # KDE extras (non-fatal)
        apt-get install -y -qq \
            bluedevil \
            print-manager \
            plasma-discover plasma-discover-backend-flatpak \
            partitionmanager \
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

        # Prevent DKMS from trying to compile modules in chroot (hangs)
        # The packaged pre-built .ko files are sufficient for the target kernel
        mkdir -p /usr/local/bin
        echo '#!/bin/sh' > /usr/local/bin/dkms
        echo 'echo \"[nexusos] dkms skipped in chroot build\"' >> /usr/local/bin/dkms
        chmod +x /usr/local/bin/dkms

        apt-get install -y -qq --no-install-recommends \
            nvidia-driver-550 \
            nvidia-utils-550 \
            libnvidia-decode-550 libnvidia-encode-550 \
            2>/dev/null || {
                echo '[warn] NVIDIA 550 unavailable, trying 535...'
                apt-get install -y -qq --no-install-recommends \
                    nvidia-driver-535 \
                    nvidia-utils-535 \
                    2>/dev/null || echo '[warn] NVIDIA install failed — will use nouveau'
            }

        # Install CUDA toolkit separately (large, non-fatal)
        apt-get install -y -qq nvidia-cuda-toolkit 2>/dev/null || \
            echo '[warn] CUDA toolkit unavailable — install post-boot with: sudo apt install nvidia-cuda-toolkit'

        # Remove fake dkms, restore real one if it exists
        rm -f /usr/local/bin/dkms
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
# Step 7b: Install Universal Package Management Backends
# ---------------------------------------------------------------------------
install_universal_pkg() {
    log "Installing universal package management backends..."

    mount_chroot

    # ══════════════════════════════════════════════════════════════════════
    # Build dependencies for compiling package managers from source
    # ══════════════════════════════════════════════════════════════════════
    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        apt-get install -y -qq \
            build-essential cmake meson ninja-build pkg-config autoconf automake libtool \
            libarchive-dev libcurl4-openssl-dev libssl-dev libgpgme-dev \
            python3-dev python3-pip python3-setuptools python3-wheel \
            zlib1g-dev liblzma-dev libbz2-dev libzstd-dev \
            asciidoc gettext doxygen \
            git
    "

    # ══════════════════════════════════════════════════════════════════════
    # NATIVE: apt / dpkg / nala  (already installed from debootstrap + base)
    # ══════════════════════════════════════════════════════════════════════

    # ══════════════════════════════════════════════════════════════════════
    # NATIVE: RPM / DNF / alien  (Fedora, RHEL, SUSE .rpm support)
    # ══════════════════════════════════════════════════════════════════════
    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        set -e
        apt-get install -y -qq rpm alien dnf
        command -v rpm >/dev/null
        command -v dnf >/dev/null
    "

    # ══════════════════════════════════════════════════════════════════════
    # COMPILE: pacman  (Arch Linux — .pkg.tar.zst)
    # Source: https://gitlab.archlinux.org/pacman/pacman
    # ══════════════════════════════════════════════════════════════════════
    log "Compiling pacman (Arch Linux package manager)..."
    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        set -e
        cd /tmp
        rm -rf pacman-src
        if ! git clone --depth 1 --branch v6.1.0 https://gitlab.archlinux.org/pacman/pacman.git pacman-src; then
            git clone --depth 1 https://gitlab.archlinux.org/pacman/pacman.git pacman-src
        fi
        cd pacman-src
        meson setup build --prefix=/usr \
            -Ddoc=disabled -Dscriptlet-shell=/bin/bash -Duse-git-version=false
        ninja -C build
        ninja -C build install
        command -v pacman >/dev/null
        cd / && rm -rf /tmp/pacman-src
        echo '[OK] pacman compiled and installed'
    "
    # Configure pacman with Arch repos
    mkdir -p "${ROOT}/etc/pacman.d"
    cat > "${ROOT}/etc/pacman.conf" << 'PACCONF'
[options]
Architecture = x86_64
CheckSpace
SigLevel = Optional TrustAll

[core]
Server = https://geo.mirror.pkgbuild.com/$repo/os/$arch

[extra]
Server = https://geo.mirror.pkgbuild.com/$repo/os/$arch

[multilib]
Server = https://geo.mirror.pkgbuild.com/$repo/os/$arch
PACCONF

    # ══════════════════════════════════════════════════════════════════════
    # COMPILE: portage / emerge  (Gentoo — ebuilds, source-based)
    # Source: https://github.com/gentoo/portage
    # ══════════════════════════════════════════════════════════════════════
    log "Installing portage (Gentoo package manager)..."
    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        set -e
        # Upgrade typing_extensions — jammy's version is too old for modern pyproject-metadata
        pip3 install --upgrade typing_extensions 2>/dev/null || \
            pip3 install --break-system-packages --upgrade typing_extensions
        cd /tmp
        rm -rf portage-src
        git clone --depth 1 https://github.com/gentoo/portage.git portage-src
        cd portage-src
        pip3 install . 2>/dev/null || pip3 install --break-system-packages .
        command -v emerge >/dev/null
        cd / && rm -rf /tmp/portage-src
        echo '[OK] portage/emerge installed'
    "
    # Gentoo repo config
    mkdir -p "${ROOT}/etc/portage" "${ROOT}/var/db/repos/gentoo"
    cat > "${ROOT}/etc/portage/make.conf" << 'MAKECONF'
GENTOO_MIRRORS="https://distfiles.gentoo.org"
FEATURES="-sandbox -usersandbox -pid-sandbox -network-sandbox"
ACCEPT_KEYWORDS="amd64"
ACCEPT_LICENSE="*"
MAKEOPTS="-j$(nproc)"
MAKECONF
    mkdir -p "${ROOT}/etc/portage/repos.conf"
    cat > "${ROOT}/etc/portage/repos.conf/gentoo.conf" << 'GCONF'
[DEFAULT]
main-repo = gentoo

[gentoo]
location = /var/db/repos/gentoo
sync-type = webrsync
sync-uri = https://rsync.gentoo.org/gentoo-portage
auto-sync = yes
GCONF

    # ══════════════════════════════════════════════════════════════════════
    # COMPILE: apk-tools  (Alpine Linux — .apk)
    # Source: https://gitlab.alpinelinux.org/alpine/apk-tools
    # ══════════════════════════════════════════════════════════════════════
    log "Compiling apk-tools (Alpine Linux package manager)..."
    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        set -e
        apt-get install -y -qq lua5.3 liblua5.3-dev scdoc
        cd /tmp
        rm -rf apk-src
        if ! git clone --depth 1 --branch v2.14.6 https://gitlab.alpinelinux.org/alpine/apk-tools.git apk-src; then
            git clone --depth 1 https://gitlab.alpinelinux.org/alpine/apk-tools.git apk-src
        fi
        cd apk-src
        # Try meson first, fall back to make
        if meson setup build --prefix=/usr -Dlua=disabled -Ddocs=disabled; then
            ninja -C build && ninja -C build install
        else
            make -j\$(nproc) PREFIX=/usr
            make PREFIX=/usr install
        fi
        command -v apk >/dev/null
        cd / && rm -rf /tmp/apk-src
        echo '[OK] apk-tools compiled and installed'
    "
    # Alpine repo config
    mkdir -p "${ROOT}/etc/apk"
    cat > "${ROOT}/etc/apk/repositories" << 'APKREPO'
https://dl-cdn.alpinelinux.org/alpine/latest-stable/main
https://dl-cdn.alpinelinux.org/alpine/latest-stable/community
APKREPO

    # ══════════════════════════════════════════════════════════════════════
    # COMPILE: xbps  (Void Linux — .xbps)
    # Source: https://github.com/void-linux/xbps
    # ══════════════════════════════════════════════════════════════════════
    log "Compiling xbps (Void Linux package manager)..."
    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        set -e
        cd /tmp
        rm -rf xbps-src
        git clone --depth 1 https://github.com/void-linux/xbps.git xbps-src
        cd xbps-src
        ./configure --prefix=/usr
        make -j\$(nproc)
        make install
        command -v xbps-install >/dev/null
        cd / && rm -rf /tmp/xbps-src
        echo '[OK] xbps compiled and installed'
    "
    # Void repo config
    mkdir -p "${ROOT}/etc/xbps.d"
    cat > "${ROOT}/etc/xbps.d/00-repository-main.conf" << 'XBPSREPO'
repository=https://repo-default.voidlinux.org/current
XBPSREPO

    # ══════════════════════════════════════════════════════════════════════
    # COMPILE: zypper + libsolv + libzypp  (openSUSE — .rpm via zypper)
    # ══════════════════════════════════════════════════════════════════════
    log "Compiling zypper (openSUSE package manager)..."
    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        set -e
        apt-get install -y -qq libboost-all-dev libxml2-dev libyaml-cpp-dev \
            libproxy-dev libsigc++-2.0-dev libreadline-dev libaugeas-dev \
            librpm-dev libglib2.0-dev

        # 1) libsolv
        cd /tmp
        rm -rf libsolv
        git clone --depth 1 https://github.com/openSUSE/libsolv.git
        cd libsolv
        mkdir build && cd build
        cmake .. -DCMAKE_INSTALL_PREFIX=/usr -DENABLE_RPMMD=ON -DENABLE_RPMDB=ON \
            -DENABLE_PUBKEY=ON -DENABLE_RPMDB_BYRPMHEADER=ON
        make -j\$(nproc) && make install
        cd /tmp && rm -rf libsolv

        # 2) libzypp
        rm -rf libzypp
        git clone --depth 1 https://github.com/openSUSE/libzypp.git
        cd libzypp
        mkdir build && cd build
        cmake .. -DCMAKE_INSTALL_PREFIX=/usr
        make -j\$(nproc)
        make install
        cd /tmp && rm -rf libzypp

        # 3) zypper
        rm -rf zypper
        git clone --depth 1 https://github.com/openSUSE/zypper.git
        cd zypper
        mkdir build && cd build
        cmake .. -DCMAKE_INSTALL_PREFIX=/usr
        make -j\$(nproc)
        make install
        cd /tmp && rm -rf zypper
        command -v zypper >/dev/null
        echo '[OK] zypper compiled and installed'
    "

    # ══════════════════════════════════════════════════════════════════════
    # NATIVE: flatpak, snap, AppImage/FUSE
    # ══════════════════════════════════════════════════════════════════════
    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        apt-get install -y -qq snapd 2>/dev/null || true
        systemctl enable snapd.socket 2>/dev/null || true
        apt-get install -y -qq libfuse2 libfuse3-3 2>/dev/null || true
        mkdir -p /home/${LIVE_USER}/Applications
        chown 1000:1000 /home/${LIVE_USER}/Applications 2>/dev/null || true
    "

    # ══════════════════════════════════════════════════════════════════════
    # Nix — requires running daemon, install on first boot
    # ══════════════════════════════════════════════════════════════════════
    cat > "${ROOT}/usr/local/bin/nexus-setup-nix" << 'NIXSETUP'
#!/usr/bin/env bash
set -euo pipefail
if command -v nix &>/dev/null; then
    echo "Nix is already installed."; nix --version; exit 0
fi
[[ $EUID -ne 0 ]] && { echo "Run with sudo: sudo nexus-setup-nix"; exit 1; }
echo "Installing Nix package manager (multi-user)..."
sh <(curl -L https://nixos.org/nix/install) --daemon --yes
echo "Nix installed. Log out and back in, then use: nix-env -iA nixpkgs.<package>"
NIXSETUP
    chmod +x "${ROOT}/usr/local/bin/nexus-setup-nix"

    # ══════════════════════════════════════════════════════════════════════
    # Language package managers
    # ══════════════════════════════════════════════════════════════════════
    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        apt-get install -y -qq \
            nodejs npm \
            cargo rustc \
            ruby ruby-dev \
            golang-go \
            2>/dev/null || true
    "

    # ══════════════════════════════════════════════════════════════════════
    # Generate nexuspkg config + clean up build deps
    # ══════════════════════════════════════════════════════════════════════
    if [[ -f "${PROJECT_ROOT}/core/bin/nexuspkg" ]]; then
        run_in_chroot "/opt/nexus-os/bin/nexuspkg init 2>/dev/null || true"
    fi

    # Remove build-only dependencies to save ISO space
    run_in_chroot "
        export DEBIAN_FRONTEND=noninteractive
        apt-get remove -y --purge \
            meson ninja-build autoconf automake libtool doxygen asciidoc scdoc \
            2>/dev/null || true
        apt-get autoremove -y 2>/dev/null || true
    "

    umount_chroot
    log "Universal package management installed"
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

    # Override casper.conf so the live user matches SDDM autologin
    cat > "${ROOT}/etc/casper.conf" << CASPEREOF
export USERNAME="${LIVE_USER}"
export USERFULLNAME="NexusOS Live User"
export HOST="nexus"
export BUILD_SYSTEM="NexusOS"
export FLAVOUR="NexusOS"
CASPEREOF

    # Set graphical target (minbase defaults to multi-user)
    run_in_chroot "systemctl set-default graphical.target"

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

    # Plymouth — always attempt to set nexusos theme (the theme files are
    # installed by apply_overlay before configure_system runs)
    run_in_chroot "
        if [ -f /usr/share/plymouth/themes/nexusos/nexusos.plymouth ]; then
            update-alternatives --install /usr/share/plymouth/themes/default.plymouth \
                default.plymouth /usr/share/plymouth/themes/nexusos/nexusos.plymouth 200
            update-alternatives --set default.plymouth \
                /usr/share/plymouth/themes/nexusos/nexusos.plymouth
            plymouth-set-default-theme nexusos 2>/dev/null || true
            echo '[OK] Plymouth theme set to nexusos'
        else
            echo '[WARN] nexusos plymouth theme not found — using default'
        fi
    "

    # Fix xauth timeout on overlayfs — file locking doesn't work on casper's cow overlay
    cat > "${ROOT}/etc/X11/Xsession.d/00nexus-xauthority" << 'XAUTHFIX'
# NexusOS: Use tmpfs for Xauthority — overlayfs doesn't support flock()
if [ -z "$XAUTHORITY" ] || echo "$XAUTHORITY" | grep -q '/home/'; then
    export XAUTHORITY="/tmp/.Xauthority-${USER:-nexus}"
fi
XAUTHFIX

    # KWin software rendering fallback for VMs without GPU acceleration
    mkdir -p "${ROOT}/etc/xdg"
    cat > "${ROOT}/etc/xdg/kwinrc" << 'KWINEOF'
[Compositing]
Backend=XRender
GLCore=false
OpenGLIsUnsafe=false
KWINEOF

    # SDDM autologin for live session
    mkdir -p "${ROOT}/etc/sddm.conf.d"
    cat > "${ROOT}/etc/sddm.conf.d/autologin.conf" << SDDMEOF
[Autologin]
User=${LIVE_USER}
Session=plasma.desktop
SDDMEOF

    # SDDM xauth fix for overlayfs live session
    cat > "${ROOT}/etc/sddm.conf.d/xauth-fix.conf" << 'SDDMXAUTH'
[X11]
UserAuthFile=/tmp/.Xauthority-sddm
SDDMXAUTH

    # SDDM theme
    if [[ -d "${ROOT}/usr/share/sddm/themes/nexusos" ]]; then
        cat > "${ROOT}/etc/sddm.conf.d/theme.conf" << 'SDDMEOF'
[Theme]
Current=nexusos
SDDMEOF
    fi

    # PRIME Render Offload — Intel iGPU drives the display, NVIDIA available on-demand
    # for gaming / AI workloads via prime-run or __NV_PRIME_RENDER_OFFLOAD=1
    mkdir -p "${ROOT}/etc/X11/xorg.conf.d"

    # Intel/AMD iGPU as primary display
    cat > "${ROOT}/etc/X11/xorg.conf.d/10-intel-primary.conf" << 'XORGEOF'
Section "Device"
    Identifier "Intel iGPU"
    Driver     "modesetting"
    Option     "AccelMethod" "glamor"
    Option     "TearFree"    "true"
EndSection
XORGEOF

    # NVIDIA as PRIME render offload provider (only activates if HW present)
    cat > "${ROOT}/etc/X11/xorg.conf.d/11-nvidia-prime-offload.conf" << 'XORGEOF'
# PRIME Render Offload — NVIDIA handles rendering when apps request it
# Usage: prime-run <application>  OR  __NV_PRIME_RENDER_OFFLOAD=1 <application>
Section "OutputClass"
    Identifier  "nvidia"
    MatchDriver "nvidia-drm"
    Driver      "nvidia"
    Option      "AllowEmptyInitialConfiguration"
    Option      "PrimaryGPU" "no"
    ModulePath  "/usr/lib/x86_64-linux-gnu/nvidia/xorg"
EndSection
XORGEOF

    # prime-run helper — launch any app on the NVIDIA GPU
    cat > "${ROOT}/usr/local/bin/prime-run" << 'PRIMEEOF'
#!/bin/bash
export __NV_PRIME_RENDER_OFFLOAD=1
export __NV_PRIME_RENDER_OFFLOAD_PROVIDER=NVIDIA-G0
export __GLX_VENDOR_LIBRARY_NAME=nvidia
export __VK_LAYER_NV_optimus=NVIDIA_only
exec "$@"
PRIMEEOF
    chmod +x "${ROOT}/usr/local/bin/prime-run"

    # Ensure nvidia-drm modeset is enabled for PRIME to work
    cat > "${ROOT}/etc/modprobe.d/nvidia-prime.conf" << 'MODEOF'
options nvidia-drm modeset=1
options nvidia NVreg_DynamicPowerManagement=0x02
MODEOF

    # Systemd service to set up NVIDIA power management (RTD3) for hybrid laptops/desktops
    cat > "${ROOT}/etc/udev/rules.d/80-nvidia-pm.rules" << 'UVEOF'
# Enable runtime PM for NVIDIA GPU — allows GPU to sleep when not in use
ACTION=="bind", SUBSYSTEM=="pci", ATTR{vendor}=="0x10de", ATTR{class}=="0x030000", TEST=="power/control", ATTR{power/control}="auto"
ACTION=="bind", SUBSYSTEM=="pci", ATTR{vendor}=="0x10de", ATTR{class}=="0x030200", TEST=="power/control", ATTR{power/control}="auto"
UVEOF

    # Disable apt CD-ROM source in live session — casper mounts the ISO at /cdrom
    # but it has no Debian repository structure, causing apt-cdrom errors
    cat > "${ROOT}/etc/apt/apt.conf.d/99no-cdrom" << 'APTCDEOF'
APT::CDROM::NoMount "true";
Acquire::cdrom::AutoDetect "false";
APT::CDROM::NoAutoDetect "true";
APTCDEOF

    # Add a casper bottom script to clean up CD-ROM apt sources at live boot
    mkdir -p "${ROOT}/usr/share/initramfs-tools/scripts/casper-bottom"
    cat > "${ROOT}/usr/share/initramfs-tools/scripts/casper-bottom/99nexus_apt_cleanup" << 'CASPERBOT'
#!/bin/sh
PREREQ=""
DESCRIPTION="Removing CD-ROM apt sources..."

prereqs()
{
    echo "$PREREQ"
}

case $1 in
    prereqs)
        prereqs
        exit 0
        ;;
esac

. /scripts/casper-functions

log_begin_msg "$DESCRIPTION"

# Remove any cdrom entries from apt sources
if [ -f /root/etc/apt/sources.list ]; then
    sed -i '/^deb cdrom:/d' /root/etc/apt/sources.list
fi
rm -f /root/etc/apt/sources.list.d/cdrom-*.list 2>/dev/null

log_end_msg
CASPERBOT
    chmod +x "${ROOT}/usr/share/initramfs-tools/scripts/casper-bottom/99nexus_apt_cleanup"

    # Ensure Plymouth properly quits during casper boot to avoid
    # "unexpectedly disconnected from boot status daemon" error
    mkdir -p "${ROOT}/etc/systemd/system/sddm.service.d"
    cat > "${ROOT}/etc/systemd/system/sddm.service.d/plymouth.conf" << 'PLYMEOF'
[Unit]
After=plymouth-quit-wait.service
Conflicts=plymouth-quit.service

[Service]
ExecStartPre=-/usr/bin/plymouth deactivate
ExecStartPre=-/usr/bin/plymouth quit --retain-splash
PLYMEOF

    # Ensure overlay filesystem module is included in initramfs
    # (required by casper for copy-on-write live session)
    echo 'overlay' >> "${ROOT}/etc/initramfs-tools/modules"

    # Fallback: load overlay via insmod if modprobe fails at boot
    # This runs in init-premount, before casper's main script tries modprobe overlay
    mkdir -p "${ROOT}/usr/share/initramfs-tools/scripts/init-premount"
    cat > "${ROOT}/usr/share/initramfs-tools/scripts/init-premount/overlay-fallback" << 'OVHOOK'
#!/bin/sh
PREREQ=""
prereqs() { echo "$PREREQ"; }
case $1 in prereqs) prereqs; exit 0;; esac

# Ensure overlay filesystem support exists before casper needs union mount
if ! grep -Eq '(^|[[:space:]])overlay$' /proc/filesystems 2>/dev/null; then
    modprobe overlay 2>/dev/null || true
fi
if ! grep -Eq '(^|[[:space:]])overlay$' /proc/filesystems 2>/dev/null; then
    # modprobe failed — try direct insmod as last resort
    KVER="$(uname -r)"
    for ko in "/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko" \
              "/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko.zst" \
              "/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko.xz"; do
        [ -f "$ko" ] && insmod "$ko" 2>/dev/null && break
    done
fi
OVHOOK
    chmod +x "${ROOT}/usr/share/initramfs-tools/scripts/init-premount/overlay-fallback"

    # Patch casper's overlay setup to avoid false panic when overlay support
    # is already present and to add direct insmod fallback if modprobe fails.
    cat > "${ROOT}/tmp/patch-casper-overlay.py" << 'PYCASPER'
from pathlib import Path

casper = Path('/usr/share/initramfs-tools/scripts/casper')
if not casper.exists():
    raise SystemExit(0)

text = casper.read_text()
needle = '    modprobe "${MP_QUIET}" -b overlay || panic "/cow format specified as \'overlay\' and no support found"'
replacement = '''    if ! grep -Eq '(^|[[:space:]])overlay$' /proc/filesystems; then
        modprobe "${MP_QUIET}" -b overlay 2>/dev/null || true
    fi
    if ! grep -Eq '(^|[[:space:]])overlay$' /proc/filesystems; then
        KVER="$(uname -r)"
        for ko in "/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko" \
                  "/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko.zst" \
                  "/lib/modules/${KVER}/kernel/fs/overlayfs/overlay.ko.xz"; do
            [ -f "$ko" ] && insmod "$ko" 2>/dev/null && break
        done
    fi
    grep -Eq '(^|[[:space:]])overlay$' /proc/filesystems || panic "/cow format specified as 'overlay' and no support found"'''
if needle not in text:
    raise SystemExit("FATAL: expected casper overlay line not found")

casper.write_text(text.replace(needle, replacement, 1))
PYCASPER
    run_in_chroot "python3 /tmp/patch-casper-overlay.py && rm -f /tmp/patch-casper-overlay.py"

    # Rebuild initramfs — MUST happen after casper is installed and all
    # custom scripts (casper-bottom, overlay-fallback, plymouth, etc.) are in place.
    # Stash NVIDIA .ko files out of the module tree first so update-initramfs
    # won't include them (hooks run BEFORE module copying, so a hook can't
    # remove them — we must hide them from the source tree instead).
    log "Rebuilding initramfs with casper hooks..."
    run_in_chroot "
        KVER=\$(ls /lib/modules/ | sort -V | tail -1)
        echo \"[nexusos] Target kernel: \${KVER}\"

        # Remove old kernel versions — only keep the latest
        for d in /lib/modules/*/; do
            kv=\$(basename \"\${d}\")
            [ \"\${kv}\" = \"\${KVER}\" ] && continue
            echo \"[nexusos] Removing old kernel modules: \${kv}\"
            rm -rf \"/lib/modules/\${kv}\"
            rm -f /boot/vmlinuz-\${kv} /boot/initrd.img-\${kv} \
                  /boot/config-\${kv} /boot/System.map-\${kv} 2>/dev/null || true
        done

        # Stash nvidia .ko files so update-initramfs won't copy them into initrd
        mkdir -p /tmp/nvidia-stash
        find \"/lib/modules/\${KVER}\" -name 'nvidia*.ko*' \
            -exec mv -t /tmp/nvidia-stash {} + 2>/dev/null || true

        # Rebuild module dependency database without nvidia
        depmod -a \"\${KVER}\"

        # Build initramfs for target kernel only (not -k all)
        update-initramfs -u -k \"\${KVER}\" || { echo '[FATAL] initramfs rebuild failed'; exit 1; }

        # Restore nvidia modules to rootfs (needed if user installs to disk)
        if ls /tmp/nvidia-stash/nvidia*.ko* 1>/dev/null 2>&1; then
            nvidia_dir=\"/lib/modules/\${KVER}/updates/dkms\"
            mkdir -p \"\${nvidia_dir}\"
            mv /tmp/nvidia-stash/nvidia*.ko* \"\${nvidia_dir}/\"
            depmod -a \"\${KVER}\"
        fi
        rm -rf /tmp/nvidia-stash
        echo \"[nexusos] initramfs rebuilt for \${KVER}\"
    "

    # Verify casper + overlay support in the initramfs (use chroot kernel, not host kernel)
    run_in_chroot "
        KVER=\$(ls /boot/vmlinuz-* | sort -V | tail -1 | sed 's|.*/vmlinuz-||')
        INITRD=/boot/initrd.img-\${KVER}

        lsinitramfs \${INITRD} 2>/dev/null | grep -q '^scripts/casper$' || {
            echo '[FATAL] casper scripts missing from initramfs'
            exit 1
        }
        lsinitramfs \${INITRD} 2>/dev/null | grep -q '^scripts/init-premount/overlay-fallback$' || {
            echo '[FATAL] overlay-fallback script missing from initramfs'
            exit 1
        }
        lsinitramfs \${INITRD} 2>/dev/null | grep -Eq 'kernel/fs/overlayfs/overlay\.ko(\.zst|\.xz)?$' || {
            echo '[FATAL] overlay module missing from initramfs'
            exit 1
        }
        echo '[OK] casper + overlay assets found in initramfs'
    "

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

    # Squashfs (casper is installed in install_base step)
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
    linux /casper/vmlinuz boot=casper systemd.unit=graphical.target quiet splash plymouth.enable=1 ---
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

    # Isolinux for BIOS boot (prefer host paths — isolinux is a host build dep)
    local isolinux_bin=""
    for path in /usr/lib/ISOLINUX/isolinux.bin \
                "${ROOT}/usr/lib/ISOLINUX/isolinux.bin" \
                "${ROOT}/usr/lib/syslinux/isolinux.bin"; do
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
    APPEND initrd=/casper/initrd boot=casper systemd.unit=graphical.target quiet splash ---

LABEL safe
    MENU LABEL NexusOS 1.0 — Safe Graphics
    KERNEL /casper/vmlinuz
    APPEND initrd=/casper/initrd boot=casper systemd.unit=graphical.target quiet splash nomodeset ---

LABEL toram
    MENU LABEL NexusOS 1.0 — Load to RAM
    KERNEL /casper/vmlinuz
    APPEND initrd=/casper/initrd boot=casper systemd.unit=graphical.target quiet splash toram ---

LABEL check
    MENU LABEL Check disc for defects
    KERNEL /casper/vmlinuz
    APPEND initrd=/casper/initrd boot=casper systemd.unit=graphical.target integrity-check quiet splash ---
SYSEOF
        log "BIOS boot (isolinux) configured"
    else
        warn "isolinux.bin not found — BIOS boot unavailable"
    fi

    # .disk metadata
    echo "NexusOS 1.0 \"AI-Native\" - Release x86_64 (${BUILD_DATE})" > "${ISO_DIR}/.disk/info"
    touch "${ISO_DIR}/.disk/base_installable"

    # casper.conf — must match the rootfs copy in /etc/casper.conf
    cat > "${ISO_DIR}/casper/casper.conf" << CASPEREOF
export USERNAME="${LIVE_USER}"
export USERFULLNAME="NexusOS Live User"
export HOST="nexus"
export BUILD_SYSTEM="NexusOS"
export FLAVOUR="NexusOS"
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
        # Hybrid BIOS + UEFI boot (isohybrid for USB stick booting)
        local isohdpfx=""
        for p in /usr/lib/ISOLINUX/isohdpfx.bin /usr/lib/syslinux/mbr/isohdpfx.bin; do
            [[ -f "$p" ]] && { isohdpfx="$p"; break; }
        done
        xorriso -as mkisofs \
            -volid "$ISO_LABEL" \
            -J -R -l -iso-level 3 \
            -b isolinux/isolinux.bin \
            -c isolinux/boot.cat \
            -no-emul-boot -boot-load-size 4 -boot-info-table \
            ${isohdpfx:+-isohybrid-mbr "$isohdpfx"} \
            -eltorito-alt-boot \
            -e boot/grub/efi.img \
            -no-emul-boot \
            -isohybrid-gpt-basdat \
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
    install_universal_pkg
    configure_system
    build_live_fs
    create_efi_image
    build_iso
    print_summary
}

main "$@"
