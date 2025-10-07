#!/bin/bash
#
# NexusOS ISO Builder
# Builds a custom ISO based on Garuda Dr460nized Gaming with NexusOS enhancements
#
# Usage: sudo ./build-iso.sh [output-directory]
#

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="${1:-${PROJECT_ROOT}/build}"
ISO_NAME="nexusos-1.0.0-alpha-x86_64.iso"
WORK_DIR="/tmp/nexusos-build"
BUILD_DATE="$(date +%Y.%m.%d)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════════════════╗"
    echo "║              NexusOS ISO Builder                 ║"
    echo "║        Universal Linux Distribution              ║" 
    echo "║                v1.0.0-alpha                      ║"
    echo "╚══════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    print_status "Checking build dependencies..."
    
    local deps=(
        "archiso"
        "git"
        "squashfs-tools"
        "libisoburn"
        "dosfstools"
        "lynx"
        "arch-install-scripts"
        "pacman-contrib"
    )
    
    local missing=()
    
    for dep in "${deps[@]}"; do
        if ! pacman -Q "$dep" &>/dev/null; then
            missing+=("$dep")
        fi
    done
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        print_error "Missing dependencies: ${missing[*]}"
        print_status "Installing missing dependencies..."
        pacman -S --needed "${missing[@]}"
    else
        print_status "All dependencies satisfied"
    fi
}

prepare_workspace() {
    print_status "Preparing workspace..."
    
    # Clean previous builds
    if [[ -d "$WORK_DIR" ]]; then
        rm -rf "$WORK_DIR"
    fi
    
    # Create working directories
    mkdir -p "$WORK_DIR"
    mkdir -p "$OUTPUT_DIR"
    
    # Copy archiso profile
    cp -r /usr/share/archiso/configs/releng "$WORK_DIR/airootfs"
    
    print_status "Workspace prepared at: $WORK_DIR"
}

configure_packages() {
    print_status "Configuring NexusOS package list..."
    
    cat > "$WORK_DIR/airootfs/packages.x86_64" << 'EOF'
# Base Garuda packages
linux-zen
linux-zen-headers
garuda-dr460nized
garuda-common-settings
garuda-fish-config
garuda-gamer
garuda-bash-config
garuda-zsh-config

# NexusOS Core
nexuspkg
stella-ai
maxjr-ai

# Desktop Environment (KDE Plasma)
plasma-meta
plasma-wayland-session
kde-applications-meta
plasma-browser-integration
bluedevil
kde-gtk-config
kdeplasma-addons
kscreen
kwallet-pam
powerdevil
sddm-kcm

# Universal Package Support
flatpak
snapd
appimage-launcher
docker
containerd

# Package Managers from other distros
dpkg
rpm-tools
xbps
nix

# Development Tools
git
base-devel
cmake
ninja
python
python-pip
nodejs
npm
rust
cargo
ruby
gem

# Gaming (Garuda Gaming)
steam
lutris
heroic-games-launcher-bin
gamemode
goverlay
mangohud
lib32-gamemode
lib32-mangohud

# Media Center Components
jellyfin-server
jellyfin-web
plex-media-server
docker-compose
portainer-bin

# System Tools
htop
neofetch
tree
wget
curl
jq
unzip
zip
rsync

# Network Tools
networkmanager
networkmanager-openconnect
networkmanager-openvpn
openssh
firefox

# Graphics Drivers
mesa
lib32-mesa
nvidia-dkms
lib32-nvidia-utils
vulkan-icd-loader
lib32-vulkan-icd-loader

# Audio
pipewire
pipewire-alsa
pipewire-pulse
pipewire-jack
lib32-pipewire
wireplumber

# Fonts
ttf-dejavu
ttf-liberation
noto-fonts
noto-fonts-emoji
ttf-hack

# File System Support
ntfs-3g
exfat-utils
f2fs-tools
btrfs-progs
dosfstools

# Calamares Installer
calamares
calamares-config-nexusos

# NexusOS Specific
nexusos-branding
nexusos-wallpapers
nexusos-plasma-theme
nexusos-sddm-theme
EOF

    print_status "Package list configured"
}

customize_airootfs() {
    print_status "Customizing root filesystem..."
    
    local airootfs="$WORK_DIR/airootfs/airootfs"
    
    # Create necessary directories
    mkdir -p "$airootfs/etc"
    mkdir -p "$airootfs/usr/share/pixmaps"
    mkdir -p "$airootfs/usr/bin"
    mkdir -p "$airootfs/home/nexus"
    
    # Install os-release
    cp "$PROJECT_ROOT/distro/os-release" "$airootfs/etc/os-release"
    
    # Create NexusOS issue files
    cat > "$airootfs/etc/issue" << 'EOF'
NexusOS 1.0.0-alpha (\l) - Universal Linux Distribution

Welcome to NexusOS - The world's first truly universal Linux distribution!

🌍 Universal Package Management - Install from ANY Linux distro
🤖 AI Companions - Meet Stella (🐕) and Max Jr. (🐱)  
🎮 Gaming Excellence - Built on Garuda Dr460nized Gaming
📺 Media Center Ready - 65+ services ready to deploy

Website: https://nexusos.org
GitHub:  https://github.com/nexusos/nexus-os

EOF
    
    cat > "$airootfs/etc/motd" << 'EOF'
╔══════════════════════════════════════════════════════════════════╗
║                          Welcome to NexusOS!                    ║
║                                                                  ║
║  🌍 Universal Package Manager: nexuspkg                         ║
║  🤖 AI Companions: stella & maxjr                               ║
║  🎮 Gaming Ready: Garuda optimizations included                 ║
║  📺 Media Stack: 65+ services ready for deployment              ║
║                                                                  ║
║  Quick Start:                                                    ║
║    nexuspkg install firefox     # Install from any distro       ║
║    nexuspkg search "video editor" # Search all repositories     ║
║    stella --help                # Meet your security AI         ║
║    maxjr --optimize             # Optimize system performance   ║
║                                                                  ║
║  Support: https://github.com/nexusos/nexus-os/issues           ║
╚══════════════════════════════════════════════════════════════════╝
EOF
    
    # Copy nexuspkg binary (placeholder - would copy actual binary)
    if [[ -f "$PROJECT_ROOT/opt/nexusos/bin/nexuspkg" ]]; then
        cp "$PROJECT_ROOT/opt/nexusos/bin/nexuspkg" "$airootfs/usr/bin/"
        chmod +x "$airootfs/usr/bin/nexuspkg"
    fi
    
    # Create default user
    cat > "$airootfs/etc/passwd" << 'EOF'
root:x:0:0:root:/root:/bin/bash
nexus:x:1000:1000:NexusOS User:/home/nexus:/bin/zsh
EOF
    
    # Set up sudo access
    cat > "$airootfs/etc/sudoers.d/nexus" << 'EOF'
nexus ALL=(ALL) NOPASSWD: ALL
EOF
    
    # Desktop configuration
    mkdir -p "$airootfs/home/nexus/.config/plasma-org.kde.plasma.desktop-appletsrc"
    
    print_status "Root filesystem customized"
}

build_iso() {
    print_status "Building NexusOS ISO..."
    
    cd "$WORK_DIR/airootfs"
    
    # Set build information
    cat > "profiledef.sh" << EOF
#!/usr/bin/env bash
iso_name="nexusos"
iso_label="NEXUSOS_$(date +%Y%m)"
iso_publisher="NexusOS Team <https://nexusos.org>"
iso_application="NexusOS Live/Rescue CD"
iso_version="$BUILD_DATE"
install_dir="arch"
buildmodes=('iso')
bootmodes=('bios.syslinux.mbr' 'bios.syslinux.eltorito' 'uefi-ia32.grub.esp' 'uefi-x64.grub.esp' 'uefi-ia32.grub.eltorito' 'uefi-x64.grub.eltorito')
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'xz' '-Xbcj' 'x86' '-b' '1M' '-Xdict-size' '1M')
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/root"]="0:0:750"
  ["/home/nexus"]="1000:1000:755"
  ["/usr/bin/nexuspkg"]="0:0:755"
  ["/usr/bin/stella"]="0:0:755"
  ["/usr/bin/maxjr"]="0:0:755"
)
EOF
    
    # Build the ISO
    print_status "Running mkarchiso..."
    mkarchiso -v -w "$WORK_DIR/work" -o "$OUTPUT_DIR" "$WORK_DIR/airootfs"
    
    # Rename to our naming convention
    if [[ -f "$OUTPUT_DIR/nexusos-$(date +%Y.%m.%d)-x86_64.iso" ]]; then
        mv "$OUTPUT_DIR/nexusos-$(date +%Y.%m.%d)-x86_64.iso" "$OUTPUT_DIR/$ISO_NAME"
    fi
}

generate_checksums() {
    print_status "Generating checksums..."
    
    cd "$OUTPUT_DIR"
    
    if [[ -f "$ISO_NAME" ]]; then
        sha256sum "$ISO_NAME" > "$ISO_NAME.sha256"
        md5sum "$ISO_NAME" > "$ISO_NAME.md5"
        
        print_status "Checksums generated:"
        echo "  SHA256: $(cat "$ISO_NAME.sha256")"
        echo "  MD5:    $(cat "$ISO_NAME.md5")"
    fi
}

cleanup() {
    print_status "Cleaning up build directory..."
    rm -rf "$WORK_DIR"
    print_status "Build directory cleaned"
}

main() {
    print_banner
    
    # Check if running as root
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root (use sudo)"
        exit 1
    fi
    
    print_status "Starting NexusOS ISO build process..."
    print_status "Project root: $PROJECT_ROOT"
    print_status "Output directory: $OUTPUT_DIR"
    print_status "Build date: $BUILD_DATE"
    
    check_dependencies
    prepare_workspace
    configure_packages
    customize_airootfs
    build_iso
    generate_checksums
    cleanup
    
    print_status "Build completed successfully!"
    
    if [[ -f "$OUTPUT_DIR/$ISO_NAME" ]]; then
        local iso_size=$(du -h "$OUTPUT_DIR/$ISO_NAME" | cut -f1)
        echo
        echo -e "${GREEN}╔══════════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║              BUILD SUCCESSFUL!                   ║${NC}"
        echo -e "${GREEN}╠══════════════════════════════════════════════════╣${NC}"
        echo -e "${GREEN}║${NC} ISO File: $ISO_NAME"
        echo -e "${GREEN}║${NC} Location: $OUTPUT_DIR"
        echo -e "${GREEN}║${NC} Size: $iso_size"
        echo -e "${GREEN}║${NC} "
        echo -e "${GREEN}║${NC} Ready for:"
        echo -e "${GREEN}║${NC}   • USB creation (dd/Rufus/Etcher)"
        echo -e "${GREEN}║${NC}   • Virtual machine testing"
        echo -e "${GREEN}║${NC}   • DistroWatch submission"
        echo -e "${GREEN}║${NC}   • Public release"
        echo -e "${GREEN}╚══════════════════════════════════════════════════╝${NC}"
        echo
        print_status "Next steps:"
        echo "  1. Test ISO in virtual machine"
        echo "  2. Create USB bootable media: dd if=$OUTPUT_DIR/$ISO_NAME of=/dev/sdX bs=4M status=progress"
        echo "  3. Submit to DistroWatch using DISTROWATCH_SUBMISSION.md"
        echo "  4. Announce release on GitHub"
    else
        print_error "ISO build failed - file not found"
        exit 1
    fi
}

# Run main function
main "$@"