#!/bin/bash
# WireGuard Tray Widget Installation Script for Garuda Linux

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROXMOX_HOST="${1:-192.168.122.9}"
PROXMOX_USER="${2:-root}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════════════╗"
    echo "║                🛡️  WireGuard Tray Widget Installer  🛡️                ║"
    echo "║                        For Garuda Linux                             ║"
    echo "╚══════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

log() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

check_requirements() {
    log "Checking system requirements..."
    
    # Check if running on Garuda Linux
    if ! grep -q "garuda" /etc/os-release 2>/dev/null; then
        warn "This script is designed for Garuda Linux but will attempt to continue"
    fi
    
    # Check Python
    if ! command -v python3 &> /dev/null; then
        log "Installing Python..."
        sudo pacman -S --noconfirm python
    fi
    
    # Check PyQt5
    if ! python3 -c "import PyQt5" 2>/dev/null; then
        log "Installing PyQt5..."
        sudo pacman -S --noconfirm python-pyqt5
    fi
    
    # Check requests
    if ! python3 -c "import requests" 2>/dev/null; then
        log "Installing Python requests..."
        sudo pacman -S --noconfirm python-requests
    fi
    
    # Check WireGuard tools
    if ! command -v wg &> /dev/null; then
        log "Installing WireGuard tools..."
        sudo pacman -S --noconfirm wireguard-tools
    fi
    
    log "✅ All requirements satisfied"
}

install_widget() {
    log "Installing WireGuard tray widget..."
    
    # Create directories
    mkdir -p ~/.local/bin
    mkdir -p ~/.config/autostart
    mkdir -p ~/.local/share/applications
    
    # Copy widget script
    log "Installing widget script..."
    cp "$SCRIPT_DIR/wireguard-tray-widget.py" ~/.local/bin/
    chmod +x ~/.local/bin/wireguard-tray-widget.py
    
    # Update script with correct Proxmox host
    if [[ "$PROXMOX_HOST" != "192.168.122.9" ]]; then
        log "Updating Proxmox host configuration to $PROXMOX_HOST"
        sed -i "s/192.168.122.9/$PROXMOX_HOST/g" ~/.local/bin/wireguard-tray-widget.py
    fi
    
    # Install desktop entry for applications menu
    log "Installing desktop entry..."
    cat > ~/.local/share/applications/wireguard-manager.desktop << EOF
[Desktop Entry]
Name=WireGuard Manager
Comment=WireGuard VPN and API masking system tray widget
Exec=python3 %HOME%/.local/bin/wireguard-tray-widget.py
Icon=network-vpn
Type=Application
StartupNotify=false
Terminal=false
Hidden=false
Categories=Network;Security;
EOF
    
    # Install autostart entry
    log "Setting up autostart..."
    cat > ~/.config/autostart/wireguard-manager.desktop << EOF
[Desktop Entry]
Name=WireGuard Manager
Comment=WireGuard VPN and API masking system tray widget
Exec=python3 %HOME%/.local/bin/wireguard-tray-widget.py
Icon=network-vpn
Type=Application
StartupNotify=false
Terminal=false
Hidden=false
Categories=Network;Security;
X-GNOME-Autostart-enabled=true
X-KDE-autostart-after=panel
EOF
    
    log "✅ Widget installed successfully"
}

copy_support_files() {
    log "Copying support files from Proxmox host..."
    
    # Create local directory
    mkdir -p ~/wireguard-tools
    
    # Copy API masking proxy
    if scp -o ConnectTimeout=10 "$PROXMOX_USER@$PROXMOX_HOST:/root/api-mask-proxy.py" ~/wireguard-tools/ 2>/dev/null; then
        log "✅ Copied API masking proxy"
        # Make it available system-wide for the widget
        sudo cp ~/wireguard-tools/api-mask-proxy.py /usr/local/bin/
        sudo chmod +x /usr/local/bin/api-mask-proxy.py
        # Create symlink for compatibility
        ln -sf /usr/local/bin/api-mask-proxy.py ~/.local/bin/api-mask-proxy.py
    else
        warn "❌ Failed to copy API masking proxy"
        warn "You may need to copy it manually later"
    fi
    
    # Copy browser masking script
    if scp -o ConnectTimeout=10 "$PROXMOX_USER@$PROXMOX_HOST:/root/browser-api-mask.js" ~/wireguard-tools/ 2>/dev/null; then
        log "✅ Copied browser masking script"
    else
        warn "❌ Failed to copy browser masking script"
    fi
    
    # Copy management scripts
    if scp -o ConnectTimeout=10 "$PROXMOX_USER@$PROXMOX_HOST:/root/garuda-wg-manager.sh" ~/wireguard-tools/ 2>/dev/null; then
        log "✅ Copied CLI management script"
        chmod +x ~/wireguard-tools/garuda-wg-manager.sh
        ln -sf ~/wireguard-tools/garuda-wg-manager.sh ~/.local/bin/wg-manager
    else
        warn "❌ Failed to copy CLI management script"
    fi
}

setup_ssh_key() {
    log "Setting up SSH key for passwordless access to Proxmox..."
    
    # Generate SSH key if it doesn't exist
    if [[ ! -f ~/.ssh/id_rsa ]]; then
        log "Generating SSH key..."
        ssh-keygen -t rsa -b 4096 -f ~/.ssh/id_rsa -N "" -C "garuda-wireguard-$(whoami)@$(hostname)"
    fi
    
    # Copy public key to Proxmox host
    log "Copying SSH key to Proxmox host..."
    if ssh-copy-id -o ConnectTimeout=10 "$PROXMOX_USER@$PROXMOX_HOST" 2>/dev/null; then
        log "✅ SSH key setup successful"
    else
        warn "❌ SSH key setup failed"
        warn "You may need to enter passwords when using the widget"
        echo -e "${YELLOW}To set up SSH key manually, run:${NC}"
        echo "  ssh-copy-id $PROXMOX_USER@$PROXMOX_HOST"
    fi
}

create_wireguard_config() {
    log "Creating WireGuard configuration directory..."
    
    # Create WireGuard config directory
    sudo mkdir -p /etc/wireguard
    sudo chown root:root /etc/wireguard
    sudo chmod 700 /etc/wireguard
    
    # Get latest client config from Proxmox
    log "Fetching latest WireGuard client configuration..."
    if latest_config=$(ssh -o ConnectTimeout=10 "$PROXMOX_USER@$PROXMOX_HOST" "ls -1t /etc/wireguard/clients/garuda-host-*.conf 2>/dev/null | head -1" 2>/dev/null); then
        if [[ -n "$latest_config" ]]; then
            log "Found configuration: $(basename "$latest_config")"
            if ssh -o ConnectTimeout=10 "$PROXMOX_USER@$PROXMOX_HOST" "cat '$latest_config'" > /tmp/wg0.conf 2>/dev/null; then
                sudo mv /tmp/wg0.conf /etc/wireguard/wg0.conf
                sudo chmod 600 /etc/wireguard/wg0.conf
                log "✅ WireGuard configuration installed"
                
                # Enable WireGuard service
                log "Enabling WireGuard service..."
                sudo systemctl enable wg-quick@wg0
                log "✅ WireGuard service enabled"
                
                return 0
            fi
        fi
    fi
    
    warn "❌ Could not fetch WireGuard configuration"
    warn "You'll need to manually configure WireGuard before using the widget"
    echo -e "${YELLOW}To get your configuration:${NC}"
    echo "  1. SSH to Proxmox: ssh $PROXMOX_USER@$PROXMOX_HOST"
    echo "  2. Rotate config: /root/wireguard-rotate.sh garuda-host"
    echo "  3. Copy config to /etc/wireguard/wg0.conf on this machine"
}

test_installation() {
    log "Testing installation..."
    
    # Test widget script
    if python3 ~/.local/bin/wireguard-tray-widget.py --help &>/dev/null; then
        log "✅ Widget script is executable"
    else
        warn "❌ Widget script test failed"
    fi
    
    # Test SSH connection
    if ssh -o ConnectTimeout=5 -o BatchMode=yes "$PROXMOX_USER@$PROXMOX_HOST" echo "Connection test" &>/dev/null; then
        log "✅ SSH connection to Proxmox is working"
    else
        warn "❌ SSH connection to Proxmox failed"
    fi
    
    # Test WireGuard configuration
    if [[ -f /etc/wireguard/wg0.conf ]]; then
        log "✅ WireGuard configuration exists"
    else
        warn "❌ WireGuard configuration missing"
    fi
}

start_widget() {
    log "Starting WireGuard tray widget..."
    
    # Kill any existing instances
    pkill -f "wireguard-tray-widget.py" 2>/dev/null || true
    
    # Start in background
    nohup python3 ~/.local/bin/wireguard-tray-widget.py &>/dev/null &
    sleep 2
    
    if pgrep -f "wireguard-tray-widget.py" &>/dev/null; then
        log "✅ Tray widget started successfully"
        log "🎯 Look for the WireGuard 'W' icon in your system tray!"
    else
        warn "❌ Failed to start tray widget"
        warn "You can start it manually: python3 ~/.local/bin/wireguard-tray-widget.py"
    fi
}

show_usage_info() {
    echo
    echo -e "${BLUE}🎉 Installation Complete!${NC}"
    echo
    echo -e "${GREEN}📍 What was installed:${NC}"
    echo "  • WireGuard tray widget in system tray"
    echo "  • Auto-start on login"
    echo "  • CLI management tools in ~/wireguard-tools/"
    echo "  • API masking proxy (if available)"
    echo
    echo -e "${GREEN}🎛️  How to use:${NC}"
    echo "  • Right-click the 'W' icon in system tray for menu"
    echo "  • Double-click to open control panel"
    echo "  • Icon changes color based on VPN status:"
    echo "    - 🔴 Red: VPN disconnected"
    echo "    - 🟢 Green: VPN connected"
    echo
    echo -e "${GREEN}🔧 Additional tools:${NC}"
    echo "  • CLI manager: wg-manager status"
    echo "  • Manual start: python3 ~/.local/bin/wireguard-tray-widget.py"
    echo "  • Config files: ~/wireguard-tools/"
    echo
    echo -e "${YELLOW}💡 Next steps:${NC}"
    echo "  1. The widget should appear in your system tray"
    echo "  2. Right-click it to access controls"
    echo "  3. Use 'Quick Rotate' to get a fresh VPN identity"
    echo "  4. Enable auto-rotation if desired"
    echo
}

main() {
    print_header
    
    log "Installing WireGuard Tray Widget for Garuda Linux"
    log "Proxmox host: $PROXMOX_HOST"
    log "Proxmox user: $PROXMOX_USER"
    echo
    
    check_requirements
    setup_ssh_key
    copy_support_files
    install_widget
    create_wireguard_config
    test_installation
    start_widget
    show_usage_info
    
    echo -e "${GREEN}🎉 Installation completed successfully!${NC}"
}

# Run main function with all arguments
main "$@"
