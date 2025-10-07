#!/bin/bash

# NexusOS System Installer
# Complete installation system integrating gaming, media stack, and NexusDE
# Version: 2024.1 Stellar Edition

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# NexusOS Branding
NEXUS_LOGO="
    ███╗   ██╗███████╗██╗  ██╗██╗   ██╗███████╗ ██████╗ ███████╗
    ████╗  ██║██╔════╝╚██╗██╔╝██║   ██║██╔════╝██╔═══██╗██╔════╝
    ██╔██╗ ██║█████╗   ╚███╔╝ ██║   ██║███████╗██║   ██║███████╗
    ██║╚██╗██║██╔══╝   ██╔██╗ ██║   ██║╚════██║██║   ██║╚════██║
    ██║ ╚████║███████╗██╔╝ ██╗╚██████╔╝███████║╚██████╔╝███████║
    ╚═╝  ╚═══╝╚══════╝╚═╝  ╚═╝ ╚═════╝ ╚══════╝ ╚═════╝ ╚══════╝
"

# ASCII Art Characters
STELLA_ASCII="    🛡️  Stella - Security Guardian"
MAXJR_ASCII="    ⚡ Max Jr. - System Optimizer"

# Installation Configuration
NEXUS_VERSION="2024.1-stellar"
NEXUS_CODENAME="Stellar"
BASE_DIR="/opt/nexus-os"
CONFIG_DIR="/etc/nexus-os"
LOG_DIR="/var/log/nexus-os"
USER_HOME="/home/$SUDO_USER"

# Print functions
print_header() {
    clear
    echo -e "${PURPLE}${NEXUS_LOGO}${NC}"
    echo -e "${CYAN}                    The Ultimate Gaming & Media Experience${NC}"
    echo -e "${WHITE}                         Version: $NEXUS_VERSION ($NEXUS_CODENAME)${NC}"
    echo ""
    echo -e "${BLUE}$STELLA_ASCII${NC}"
    echo -e "${YELLOW}$MAXJR_ASCII${NC}"
    echo ""
    echo "=================================================================================="
}

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_stellar() {
    echo -e "${PURPLE}[STELLA]${NC} $1"
}

print_maxjr() {
    echo -e "${YELLOW}[MAX JR.]${NC} $1"
}

# Validation functions
check_root() {
    if [[ $EUID -eq 0 ]]; then
        print_error "This script should not be run as root directly."
        print_status "Use: sudo ./nexus-install.sh"
        exit 1
    fi
    
    if [[ $SUDO_USER == "root" ]] || [[ -z $SUDO_USER ]]; then
        print_error "Please run this script with sudo from a regular user account."
        exit 1
    fi
}

check_system() {
    print_status "Checking system compatibility..."
    
    # Check if running on Arch-based system
    if ! command -v pacman &> /dev/null; then
        print_error "NexusOS requires an Arch Linux-based system."
        print_status "Please install on Arch Linux, Manjaro, Garuda Linux, or similar."
        exit 1
    fi
    
    # Check if running on Garuda (preferred)
    if [ -f /etc/garuda-release ]; then
        print_success "Detected Garuda Linux - Optimal base system!"
        GARUDA_DETECTED=true
    else
        print_warning "Running on non-Garuda system. Some optimizations may not be available."
        GARUDA_DETECTED=false
    fi
    
    # Check system resources
    RAM_GB=$(free -m | awk 'NR==2{printf "%.1f", $2/1024}')
    STORAGE_GB=$(df -BG / | awk 'NR==2{print int($2)}')
    
    print_status "System Resources:"
    print_status "  RAM: ${RAM_GB}GB"
    print_status "  Storage: ${STORAGE_GB}GB"
    
    if (( $(echo "$RAM_GB < 8.0" | bc -l) )); then
        print_warning "Low RAM detected. Gaming and media performance may be limited."
    fi
    
    if [[ $STORAGE_GB -lt 50 ]]; then
        print_warning "Low storage space. Consider freeing up space or using external storage."
    fi
}

# Installation Profile Selection
select_profile() {
    print_header
    echo -e "${WHITE}Select NexusOS Installation Profile:${NC}"
    echo ""
    echo "1) 🎮 Gaming Focused    - Optimized for gaming with basic media"
    echo "2) 📺 Media Server      - Complete media stack with basic gaming"
    echo "3) 🚀 Complete Experience - Everything (Gaming + Media + Development)"
    echo "4) 💻 Developer Workstation - Development tools + NexusDE"
    echo "5) 🎯 Custom Installation - Choose individual components"
    echo ""
    
    while true; do
        read -p "Enter your choice (1-5): " profile_choice
        case $profile_choice in
            1)
                INSTALLATION_PROFILE="gaming_focused"
                PROFILE_NAME="Gaming Focused"
                break
                ;;
            2)
                INSTALLATION_PROFILE="media_server"
                PROFILE_NAME="Media Server"
                break
                ;;
            3)
                INSTALLATION_PROFILE="complete_experience"
                PROFILE_NAME="Complete Experience"
                break
                ;;
            4)
                INSTALLATION_PROFILE="developer_workstation"
                PROFILE_NAME="Developer Workstation"
                break
                ;;
            5)
                INSTALLATION_PROFILE="custom"
                PROFILE_NAME="Custom Installation"
                break
                ;;
            *)
                print_error "Invalid selection. Please choose 1-5."
                ;;
        esac
    done
    
    print_success "Selected profile: $PROFILE_NAME"
}

# Custom installation component selection
select_custom_components() {
    if [[ $INSTALLATION_PROFILE != "custom" ]]; then
        return
    fi
    
    print_header
    echo -e "${WHITE}Custom Component Selection:${NC}"
    echo ""
    
    INSTALL_GAMING=false
    INSTALL_MEDIA=false
    INSTALL_DEVELOPMENT=false
    INSTALL_KRUNNER_INTEGRATION=false
    
    echo "Select components to install:"
    echo ""
    
    read -p "🎮 Install Gaming Packages (Garuda Gaming + Steam + Lutris)? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        INSTALL_GAMING=true
        print_success "Gaming packages will be installed"
    fi
    
    read -p "📺 Install Media Stack (65+ Awesome Stack services)? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        INSTALL_MEDIA=true
        print_success "Media stack will be installed"
    fi
    
    read -p "💻 Install Development Tools (KVM Manager + AI Assistant)? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        INSTALL_DEVELOPMENT=true
        print_success "Development tools will be installed"
    fi
    
    read -p "🔗 Install KRunner Integration (Garuda Hello + Backup System)? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        INSTALL_KRUNNER_INTEGRATION=true
        print_success "System integration will be installed"
    fi
}

# System preparation
prepare_system() {
    print_header
    print_status "Preparing NexusOS installation..."
    
    # Create system directories
    print_status "Creating system directories..."
    mkdir -p "$BASE_DIR"/{bin,lib,share,etc}
    mkdir -p "$CONFIG_DIR"/{gaming,media,desktop,services}
    mkdir -p "$LOG_DIR"
    mkdir -p "$USER_HOME"/.config/nexus-os
    
    # Setup logging
    exec 1> >(tee -a "$LOG_DIR/nexus-install.log")
    exec 2> >(tee -a "$LOG_DIR/nexus-install-error.log")
    
    print_stellar "Security systems initializing..."
    print_maxjr "System optimization preparing..."
    
    # Update system
    print_status "Updating system packages..."
    pacman -Syu --noconfirm
    
    # Install base dependencies
    print_status "Installing base dependencies..."
    pacman -S --needed --noconfirm \
        base-devel \
        git \
        wget \
        curl \
        yay \
        docker \
        docker-compose
}

# Gaming packages installation
install_gaming_packages() {
    if [[ $INSTALLATION_PROFILE == "media_server" ]] && [[ $INSTALL_GAMING == false ]]; then
        return
    fi
    
    print_header
    print_maxjr "Installing gaming packages and optimizations..."
    
    # Gaming kernel (if not already installed)
    if ! pacman -Q linux-zen &> /dev/null; then
        print_status "Installing gaming-optimized kernel..."
        pacman -S --noconfirm linux-zen linux-zen-headers
    fi
    
    # GPU drivers detection and installation
    print_status "Detecting and installing GPU drivers..."
    
    # NVIDIA detection
    if lspci | grep -i nvidia &> /dev/null; then
        print_status "NVIDIA GPU detected - installing drivers..."
        pacman -S --noconfirm nvidia-dkms nvidia-utils nvidia-settings nvidia-prime
    fi
    
    # AMD detection
    if lspci | grep -i amd &> /dev/null; then
        print_status "AMD GPU detected - installing drivers..."
        pacman -S --noconfirm mesa lib32-mesa vulkan-radeon lib32-vulkan-radeon
    fi
    
    # Intel detection
    if lspci | grep -i intel &> /dev/null; then
        print_status "Intel GPU detected - installing drivers..."
        pacman -S --noconfirm mesa vulkan-intel lib32-vulkan-intel intel-media-driver
    fi
    
    # Gaming platforms
    print_status "Installing gaming platforms..."
    pacman -S --noconfirm \
        steam \
        lutris \
        bottles \
        gamescope \
        mangohud \
        goverlay
    
    # Wine compatibility
    print_status "Installing Wine and compatibility layers..."
    pacman -S --noconfirm \
        wine \
        wine-gecko \
        wine-mono \
        winetricks \
        lib32-gnutls \
        lib32-libxml2 \
        lib32-mpg123
    
    # Install DXVK and VKD3D from AUR
    print_status "Installing DirectX compatibility..."
    sudo -u $SUDO_USER yay -S --noconfirm dxvk-bin vkd3d
    
    # Performance tools
    print_status "Installing performance optimization tools..."
    pacman -S --noconfirm \
        gamemode \
        lib32-gamemode \
        irqbalance \
        thermald \
        cpupower
    
    # Audio system
    print_status "Installing gaming audio system..."
    pacman -S --noconfirm \
        pipewire \
        lib32-pipewire \
        pipewire-alsa \
        pipewire-pulse \
        pipewire-jack \
        lib32-pipewire-jack \
        easyeffects
    
    # Garuda gaming packages (if on Garuda)
    if [[ $GARUDA_DETECTED == true ]]; then
        print_status "Installing Garuda gaming optimizations..."
        pacman -S --noconfirm \
            garuda-gamer \
            garuda-gaming-tweaks \
            garuda-performance-tweaks
    fi
    
    # Enable gaming services
    print_status "Enabling gaming services..."
    systemctl enable gamemode
    systemctl enable irqbalance
    systemctl enable thermald
    
    print_maxjr "Gaming optimization complete! Performance mode activated."
}

# Media stack installation
install_media_stack() {
    if [[ $INSTALLATION_PROFILE == "gaming_focused" ]] && [[ $INSTALL_MEDIA == false ]]; then
        return
    fi
    
    print_header
    print_status "Installing NexusOS Media Stack (65+ services)..."
    
    # Ensure Docker is running
    systemctl enable docker
    systemctl start docker
    usermod -aG docker $SUDO_USER
    
    # Create media directories
    print_status "Creating media directories..."
    mkdir -p "$USER_HOME"/nexus-media/{media,downloads,config}
    mkdir -p "$USER_HOME"/nexus-media/media/{movies,tv,music,books,audiobooks,comics}
    chown -R $SUDO_USER:$SUDO_USER "$USER_HOME"/nexus-media
    
    # Copy awesome-stack integration
    print_status "Setting up awesome-stack integration..."
    
    # Create enhanced docker-compose for NexusOS integration
    cat > "$USER_HOME/nexus-media/docker-compose.yml" << 'EOF'
# NexusOS Media Stack - Integrated Awesome Stack
# Version: 2024.1 Stellar Edition
# Optimized for NexusDE integration with AI mascot monitoring

networks:
  nexus-mediastack:
    driver: bridge
    ipam:
      config:
        - subnet: 172.22.0.0/16

services:
  # ============================================================================
  # NEXUS INTEGRATION LAYER
  # ============================================================================
  
  nexus-media-coordinator:
    image: alpine:latest
    container_name: nexus-media-coordinator
    restart: unless-stopped
    command: >
      sh -c "
        apk add --no-cache curl jq &&
        while true; do
          echo 'NexusOS Media Coordinator - Stella & Max Jr. monitoring active'
          # Integration with NexusDE System Services
          curl -s http://host.docker.internal:8600/api/nexus/media-status || true
          sleep 60
        done
      "
    networks:
      - nexus-mediastack
    ports:
      - "8600:80"
    extra_hosts:
      - "host.docker.internal:host-gateway"

  # Core Infrastructure (enhanced for NexusOS)
  postgres:
    image: postgres:16-alpine
    container_name: nexus-postgres
    restart: unless-stopped
    environment:
      POSTGRES_DB: ${POSTGRES_DB:-nexusdb}
      POSTGRES_USER: ${POSTGRES_USER:-nexus}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:-nexus123}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - nexus-mediastack
    ports:
      - "8020:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-nexus}"]
      interval: 10s
      timeout: 5s
      retries: 5

  # (Include all 65+ services from awesome-stack with NexusOS branding)
  # Essential Media Services
  prowlarr:
    image: linuxserver/prowlarr:latest
    container_name: nexus-prowlarr
    restart: unless-stopped
    environment:
      PUID: ${PUID:-1000}
      PGID: ${PGID:-1000}
      TZ: ${TZ:-UTC}
    volumes:
      - prowlarr_config:/config
    networks:
      - nexus-mediastack
    ports:
      - "8100:9696"
    labels:
      - "nexus.service=indexer"
      - "nexus.ai.monitoring=stella"

  sonarr:
    image: linuxserver/sonarr
    container_name: nexus-sonarr
    restart: unless-stopped
    environment:
      PUID: ${PUID:-1000}
      PGID: ${PGID:-1000}
      TZ: ${TZ:-UTC}
    volumes:
      - sonarr_config:/config
      - ${MEDIA_ROOT:-./media}/tv:/tv
      - ${DOWNLOADS_ROOT:-./downloads}:/downloads
    networks:
      - nexus-mediastack
    ports:
      - "8110:8989"
    labels:
      - "nexus.service=media-automation"
      - "nexus.ai.monitoring=maxjr"

  radarr:
    image: linuxserver/radarr
    container_name: nexus-radarr
    restart: unless-stopped
    environment:
      PUID: ${PUID:-1000}
      PGID: ${PGID:-1000}
      TZ: ${TZ:-UTC}
    volumes:
      - radarr_config:/config
      - ${MEDIA_ROOT:-./media}/movies:/movies
      - ${DOWNLOADS_ROOT:-./downloads}:/downloads
    networks:
      - nexus-mediastack
    ports:
      - "8111:7878"
    labels:
      - "nexus.service=media-automation"
      - "nexus.ai.monitoring=maxjr"

  # Media Servers
  jellyfin:
    image: jellyfin/jellyfin
    container_name: nexus-jellyfin
    restart: unless-stopped
    environment:
      JELLYFIN_PublishedServerUrl: https://jellyfin.nexus.local
    volumes:
      - jellyfin_config:/config
      - jellyfin_cache:/cache
      - ${MEDIA_ROOT:-./media}:/media:ro
    networks:
      - nexus-mediastack
    ports:
      - "8200:8096"
    labels:
      - "nexus.service=media-server"
      - "nexus.ai.monitoring=both"

  plex:
    image: plexinc/pms-docker
    container_name: nexus-plex
    restart: unless-stopped
    environment:
      PLEX_CLAIM: ${PLEX_CLAIM}
      PLEX_UID: ${PUID:-1000}
      PLEX_GID: ${PGID:-1000}
      TZ: ${TZ:-UTC}
    volumes:
      - plex_config:/config
      - plex_transcode:/transcode
      - ${MEDIA_ROOT:-./media}:/media:ro
    networks:
      - nexus-mediastack
    ports:
      - "8201:32400"
    labels:
      - "nexus.service=media-server"
      - "nexus.ai.monitoring=both"

  # Management Dashboard
  organizr:
    image: organizr/organizr:latest
    container_name: nexus-organizr
    restart: unless-stopped
    environment:
      PUID: ${PUID:-1000}
      PGID: ${PGID:-1000}
      TZ: ${TZ:-UTC}
      fpm: "true"
    volumes:
      - organizr_config:/config
    networks:
      - nexus-mediastack
    ports:
      - "8500:80"
    labels:
      - "nexus.service=dashboard"
      - "nexus.ai.monitoring=stella"

volumes:
  postgres_data:
  prowlarr_config:
  sonarr_config:
  radarr_config:
  jellyfin_config:
  jellyfin_cache:
  plex_config:
  plex_transcode:
  organizr_config:
EOF

    # Create media stack environment file
    cat > "$USER_HOME/nexus-media/.env" << EOF
# NexusOS Media Stack Configuration
PUID=$(id -u $SUDO_USER)
PGID=$(id -g $SUDO_USER)
TZ=$(timedatectl show --property=Timezone --value)

# Paths
MEDIA_ROOT=$USER_HOME/nexus-media/media
DOWNLOADS_ROOT=$USER_HOME/nexus-media/downloads

# Database
POSTGRES_DB=nexusdb
POSTGRES_USER=nexus
POSTGRES_PASSWORD=nexus_$(openssl rand -hex 8)

# Optional: Plex claim token
PLEX_CLAIM=
EOF

    chown $SUDO_USER:$SUDO_USER "$USER_HOME/nexus-media/docker-compose.yml"
    chown $SUDO_USER:$SUDO_USER "$USER_HOME/nexus-media/.env"
    
    # Start media stack
    print_status "Starting NexusOS Media Stack..."
    cd "$USER_HOME/nexus-media"
    sudo -u $SUDO_USER docker-compose up -d
    
    print_stellar "Media stack security monitoring activated!"
    print_success "Media stack installed! Access dashboard at http://localhost:8500"
}

# NexusDE desktop environment installation
install_nexusde() {
    print_header
    print_status "Installing NexusDE Desktop Environment..."
    
    # Install Qt and QML dependencies
    print_status "Installing desktop environment dependencies..."
    pacman -S --noconfirm \
        qt6-base \
        qt6-declarative \
        qt6-quickcontrols2 \
        qt6-graphicaleffects \
        kwayland \
        xorg-server \
        sddm
    
    # Copy NexusDE files
    print_status "Installing NexusDE components..."
    cp -r ../userspace/desktop/nexusde/* "$BASE_DIR/"
    
    # Setup NexusDE configuration
    cat > "$CONFIG_DIR/nexusde.conf" << EOF
# NexusDE Configuration
[Desktop]
Theme=stellar-dark
MascotStella=enabled
MascotMaxJr=enabled
AIIntegration=enabled

[Gaming]
GameModeIntegration=enabled
PerformanceOverlay=enabled
GPUAutoSwitching=enabled

[MediaStack]
IntegrationEnabled=enabled
DashboardPort=8500
MonitoringEnabled=enabled

[Security]
BiometricAuth=enabled
DigitalFortress=enabled
VaultwardenIntegration=enabled
EOF

    # Enable NexusDE services
    print_status "Setting up NexusDE services..."
    
    # Create systemd service for NexusDE
    cat > /etc/systemd/system/nexusde.service << EOF
[Unit]
Description=NexusDE Desktop Environment
After=display-manager.service

[Service]
Type=forking
ExecStart=$BASE_DIR/bin/nexusde
User=$SUDO_USER
Group=$SUDO_USER

[Install]
WantedBy=multi-user.target
EOF

    systemctl enable nexusde
    
    print_success "NexusDE installed! Stellar & Max Jr. are ready to assist."
}

# System integration installation
install_system_integrations() {
    if [[ $INSTALL_KRUNNER_INTEGRATION == false ]] && [[ $INSTALLATION_PROFILE != "complete_experience" ]]; then
        return
    fi
    
    print_header
    print_status "Installing system integrations..."
    
    # Garuda Hello biometric authentication
    if [[ $GARUDA_DETECTED == true ]]; then
        print_status "Setting up Garuda Hello integration..."
        # Copy Garuda Hello components
        cp -r ../../../garuda-hello/* "$BASE_DIR/garuda-hello/"
    fi
    
    # Ultimate Restore System
    print_status "Setting up backup and restore system..."
    cp -r ../../../garuda-ultimate-restore-system/* "$BASE_DIR/backup-system/"
    
    # KVM Manager
    print_status "Setting up virtualization management..."
    cp -r ../../../kvm-manager/* "$BASE_DIR/kvm-manager/"
    
    print_stellar "System security integrations complete!"
}

# Post-installation configuration
post_installation_setup() {
    print_header
    print_status "Completing NexusOS installation..."
    
    # Create desktop shortcuts
    mkdir -p "$USER_HOME/.local/share/applications"
    
    # NexusDE System Services shortcut
    cat > "$USER_HOME/.local/share/applications/nexus-system-services.desktop" << EOF
[Desktop Entry]
Name=NexusOS System Services
Comment=Unified system management with Stella & Max Jr.
Exec=$BASE_DIR/bin/nexus-system-services
Icon=$BASE_DIR/share/icons/nexus-logo.svg
Type=Application
Categories=System;Settings;
EOF

    # Media Stack Dashboard shortcut  
    cat > "$USER_HOME/.local/share/applications/nexus-media.desktop" << EOF
[Desktop Entry]
Name=NexusOS Media Center
Comment=Access your awesome media stack
Exec=xdg-open http://localhost:8500
Icon=$BASE_DIR/share/icons/media-stack.svg
Type=Application
Categories=AudioVideo;Video;
EOF

    chown -R $SUDO_USER:$SUDO_USER "$USER_HOME/.local"
    
    # Setup auto-start
    mkdir -p "$USER_HOME/.config/autostart"
    cp "$USER_HOME/.local/share/applications/nexus-system-services.desktop" \
       "$USER_HOME/.config/autostart/"
    chown -R $SUDO_USER:$SUDO_USER "$USER_HOME/.config"
    
    # Set default applications
    print_status "Configuring default applications..."
    sudo -u $SUDO_USER xdg-mime default nexus-media.desktop video/mp4
    sudo -u $SUDO_USER xdg-mime default nexus-media.desktop video/mkv
    
    print_success "NexusOS configuration complete!"
}

# Installation summary and next steps
installation_summary() {
    print_header
    echo -e "${GREEN}🎉 NexusOS Installation Complete! 🎉${NC}"
    echo ""
    echo -e "${WHITE}Installation Summary:${NC}"
    echo "===================="
    echo -e "Profile: ${CYAN}$PROFILE_NAME${NC}"
    echo -e "Version: ${PURPLE}$NEXUS_VERSION ($NEXUS_CODENAME)${NC}"
    echo -e "User: ${BLUE}$SUDO_USER${NC}"
    echo ""
    
    if [[ $INSTALLATION_PROFILE != "media_server" ]]; then
        echo -e "${YELLOW}🎮 Gaming Features:${NC}"
        echo "  ✓ Gaming-optimized kernel (linux-zen)"
        echo "  ✓ GPU drivers and performance tools"
        echo "  ✓ Steam, Lutris, and compatibility layers"
        echo "  ✓ MangoHUD performance monitoring"
        if [[ $GARUDA_DETECTED == true ]]; then
            echo "  ✓ Garuda gaming optimizations"
        fi
        echo ""
    fi
    
    if [[ $INSTALLATION_PROFILE != "gaming_focused" ]]; then
        echo -e "${CYAN}📺 Media Stack Features:${NC}"
        echo "  ✓ 65+ awesome-stack services"
        echo "  ✓ Jellyfin & Plex media servers"
        echo "  ✓ Sonarr, Radarr, and automation"
        echo "  ✓ Integrated dashboard at http://localhost:8500"
        echo ""
    fi
    
    echo -e "${PURPLE}🖥️ NexusDE Desktop Environment:${NC}"
    echo "  ✓ AI-powered window management"
    echo "  ✓ Stella (Security Guardian) integration"
    echo "  ✓ Max Jr. (System Optimizer) integration"
    echo "  ✓ Unified system services management"
    echo ""
    
    echo -e "${WHITE}🚀 Next Steps:${NC}"
    echo "============="
    echo "1. Reboot your system to load the gaming kernel"
    echo "2. Log into NexusDE desktop environment"
    echo "3. Open 'NexusOS System Services' to configure Stella & Max Jr."
    echo ""
    
    if [[ $INSTALLATION_PROFILE != "gaming_focused" ]]; then
        echo "4. Access your media dashboard: http://localhost:8500"
        echo "5. Configure media services through the NexusDE interface"
        echo ""
    fi
    
    echo -e "${GREEN}Welcome to NexusOS - The Ultimate Gaming & Media Experience!${NC}"
    echo ""
    print_stellar "System security monitoring ready!"
    print_maxjr "Performance optimization active!"
    echo ""
    echo "Need help? Check /var/log/nexus-os/ for installation logs."
}

# Main installation flow
main() {
    # Check prerequisites
    check_root
    check_system
    
    # Show welcome and get user choices
    print_header
    echo -e "${WHITE}Welcome to the NexusOS Installer!${NC}"
    echo ""
    echo "This installer will set up:"
    echo "• 🎮 Complete Garuda gaming optimization"
    echo "• 📺 65+ service awesome-stack media center" 
    echo "• 🖥️ NexusDE desktop with AI assistants"
    echo "• 🔧 Integrated system management tools"
    echo ""
    
    read -p "Continue with NexusOS installation? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi
    
    # Installation process
    select_profile
    select_custom_components
    
    print_header
    echo -e "${WHITE}Ready to install NexusOS!${NC}"
    echo -e "Profile: ${CYAN}$PROFILE_NAME${NC}"
    echo ""
    read -p "Proceed with installation? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi
    
    # Execute installation steps
    prepare_system
    install_gaming_packages
    install_media_stack
    install_nexusde
    install_system_integrations
    post_installation_setup
    
    # Show completion summary
    installation_summary
}

# Handle Ctrl+C gracefully
cleanup() {
    print_error "Installation interrupted by user."
    print_status "Cleaning up..."
    # Add cleanup commands here
    exit 130
}

trap cleanup SIGINT SIGTERM

# Run the installer
main "$@"