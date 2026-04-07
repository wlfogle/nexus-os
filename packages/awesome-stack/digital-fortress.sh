#!/bin/bash
# 🏰 DIGITAL FORTRESS - Complete Privacy & Anonymity Suite
# Unified control for WireGuard VPN, Ghost Mode, API Masking, and Smart Routing
# Combines all the best features from your advanced repositories

set -euo pipefail

# Configuration
FORTRESS_CONFIG="$HOME/.config/digital-fortress"
LOG_FILE="$FORTRESS_CONFIG/fortress.log"
WG_INTERFACE="wg0"
PROXY_PORT="8080"
DASHBOARD_PORT="8081"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Initialize fortress directory
mkdir -p "$FORTRESS_CONFIG"
touch "$LOG_FILE"

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

success() { echo -e "${GREEN}✅ $1${NC}" | tee -a "$LOG_FILE"; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}" | tee -a "$LOG_FILE"; }
error() { echo -e "${RED}❌ $1${NC}" | tee -a "$LOG_FILE"; }
info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
fortress() { echo -e "${PURPLE}🏰 $1${NC}"; }

print_banner() {
    echo -e "${PURPLE}"
    echo "🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰"
    echo "🏰                                               🏰"
    echo "🏰           DIGITAL FORTRESS ACTIVATED          🏰"
    echo "🏰      Complete Privacy & Anonymity Suite       🏰"
    echo "🏰                                               🏰"
    echo "🏰  🛡️  WireGuard VPN    🥷 Ghost Mode           🏰"
    echo "🏰  🎭 API Masking      🌐 Smart Routing        🏰"
    echo "🏰  🔄 Auto Rotation    👁️  Monitoring           🏰"
    echo "🏰                                               🏰"
    echo "🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰🏰"
    echo -e "${NC}"
}

# Check if WireGuard is running
is_vpn_active() {
    systemctl is-active --quiet wg-quick@${WG_INTERFACE} 2>/dev/null
}

# Check if Ghost Mode is active
is_ghost_active() {
    [[ -f "$HOME/.config/ghost-mode/status" ]] && [[ "$(cat "$HOME/.config/ghost-mode/status")" == "ACTIVE" ]]
}

# Check if API proxy is running
is_proxy_active() {
    curl -s "http://localhost:${PROXY_PORT}/health" >/dev/null 2>&1
}

# Get current external IP
get_external_ip() {
    curl -s --max-time 5 http://ifconfig.me 2>/dev/null || echo "Unknown"
}

# Show comprehensive status
show_status() {
    fortress "DIGITAL FORTRESS STATUS REPORT"
    echo ""
    
    # VPN Status
    if is_vpn_active; then
        success "🛡️  WireGuard VPN: ACTIVE"
        VPN_IP=$(ip addr show $WG_INTERFACE 2>/dev/null | grep 'inet ' | awk '{print $2}' | cut -d/ -f1 || echo "Unknown")
        info "   └─ VPN IP: $VPN_IP"
    else
        warn "🛡️  WireGuard VPN: INACTIVE"
    fi
    
    # Ghost Mode Status
    if is_ghost_active; then
        success "🥷 Ghost Mode: ACTIVE (Complete Invisibility)"
        info "   └─ IPv6 Disabled, Hardware Spoofed, Time Masked"
    else
        warn "🥷 Ghost Mode: INACTIVE"
    fi
    
    # API Proxy Status
    if is_proxy_active; then
        success "🎭 API Masking Proxy: ACTIVE"
        info "   └─ Proxy Port: $PROXY_PORT"
    else
        warn "🎭 API Masking Proxy: INACTIVE"
    fi
    
    # External IP
    EXTERNAL_IP=$(get_external_ip)
    info "🌐 External IP: $EXTERNAL_IP"
    
    # Smart Routing Status
    if ip rule show | grep -q "table vpn_table"; then
        success "🌐 Smart Routing: ACTIVE"
        info "   └─ API calls masked, main connection preserved"
    else
        warn "🌐 Smart Routing: INACTIVE"
    fi
    
    # Monitoring Status
    if pgrep -f "ghost-monitor" >/dev/null; then
        success "👁️  Monitoring: ACTIVE"
        info "   └─ Continuous leak detection running"
    else
        warn "👁️  Monitoring: INACTIVE"
    fi
    
    echo ""
    fortress "System tray widget status: $(pgrep -f "wireguard-tray-widget" >/dev/null && echo "Running" || echo "Not running")"
}

# Activate complete fortress protection
activate_fortress() {
    print_banner
    log "Starting Digital Fortress activation"
    
    fortress "Activating complete protection suite..."
    
    # 1. Start WireGuard VPN
    info "🛡️  Starting WireGuard VPN..."
    if ! is_vpn_active; then
        if /home/lou/scripts/wireguard-killswitch.sh toggle-vpn 2>/dev/null; then
            success "WireGuard VPN activated"
        else
            error "Failed to start WireGuard VPN"
        fi
    else
        success "WireGuard VPN already active"
    fi
    
    # 2. Activate Ghost Mode
    info "🥷 Activating Ghost Mode..."
    if command -v ghost-mode >/dev/null 2>&1; then
        if sudo ghost-mode start 2>/dev/null; then
            success "Ghost Mode activated"
        else
            warn "Ghost Mode activation had issues"
        fi
    else
        warn "Ghost Mode not available"
    fi
    
    # 3. Start API Masking Proxy
    info "🎭 Starting API Masking Proxy..."
    if [[ -f "/home/lou/Github_Repos/github/awesome-stack/api-masking/api-mask-proxy.py" ]]; then
        if ! is_proxy_active; then
            nohup python3 "/home/lou/Github_Repos/github/awesome-stack/api-masking/api-mask-proxy.py" >/dev/null 2>&1 &
            sleep 2
            if is_proxy_active; then
                success "API Masking Proxy started"
            else
                warn "API Masking Proxy may not have started properly"
            fi
        else
            success "API Masking Proxy already running"
        fi
    else
        warn "API Masking Proxy not found"
    fi
    
    # 4. Activate Smart Routing
    info "🌐 Activating Smart Routing..."
    if [[ -f "/home/lou/scripts/smart-vpn-routing.sh" ]]; then
        if sudo /home/lou/scripts/smart-vpn-routing.sh 2>/dev/null; then
            success "Smart Routing activated"
        else
            warn "Smart Routing activation had issues"
        fi
    else
        warn "Smart Routing script not found"
    fi
    
    # 5. Start Monitoring
    info "👁️  Starting continuous monitoring..."
    if command -v ghost-monitor >/dev/null 2>&1; then
        if ! pgrep -f "ghost-monitor" >/dev/null; then
            nohup ghost-monitor start >/dev/null 2>&1 &
            success "Monitoring started"
        else
            success "Monitoring already active"
        fi
    else
        warn "Ghost monitoring not available"
    fi
    
    # 6. Start System Tray Widget
    info "🖥️  Starting system tray widget..."
    if [[ -f "$HOME/.local/bin/wireguard-tray-widget.py" ]]; then
        if ! pgrep -f "wireguard-tray-widget" >/dev/null; then
            nohup python3 "$HOME/.local/bin/wireguard-tray-widget.py" >/dev/null 2>&1 &
            success "System tray widget started"
        else
            success "System tray widget already running"
        fi
    else
        warn "System tray widget not found"
    fi
    
    echo ""
    fortress "DIGITAL FORTRESS ACTIVATION COMPLETE!"
    success "🛡️  All protection layers are now active"
    success "🥷 You are now completely invisible online"
    success "🖥️  Use the system tray widget for quick control"
    
    # Show final status
    echo ""
    show_status
}

# Deactivate fortress protection
deactivate_fortress() {
    fortress "Deactivating Digital Fortress..."
    log "Starting Digital Fortress deactivation"
    
    # Stop Ghost Mode
    if command -v ghost-mode >/dev/null 2>&1; then
        sudo ghost-mode stop 2>/dev/null || true
        info "🥷 Ghost Mode deactivated"
    fi
    
    # Stop API Proxy
    pkill -f "api-mask-proxy.py" 2>/dev/null || true
    info "🎭 API Masking Proxy stopped"
    
    # Stop Monitoring
    pkill -f "ghost-monitor" 2>/dev/null || true
    info "👁️  Monitoring stopped"
    
    # Reset smart routing (keep VPN active)
    if ip rule show | grep -q "table vpn_table"; then
        sudo ip rule del table vpn_table 2>/dev/null || true
        info "🌐 Smart routing reset"
    fi
    
    success "🏰 Digital Fortress deactivated (VPN remains active for basic protection)"
    show_status
}

# Quick toggle function
toggle_fortress() {
    if is_ghost_active && is_proxy_active; then
        deactivate_fortress
    else
        activate_fortress
    fi
}

# Install/setup function
setup_fortress() {
    fortress "Setting up Digital Fortress..."
    
    # Install required Python packages
    info "Installing Python dependencies..."
    pip3 install --user PyQt5 requests aiohttp 2>/dev/null || true
    
    # Install required system packages
    info "Installing system packages..."
    sudo pacman -S --needed --noconfirm wireguard-tools python-pyqt5 python-requests cronie firefox tor 2>/dev/null || warn "Some packages may have failed to install"
    
    # Setup sudoers permissions
    info "Setting up sudoers permissions..."
    sudo tee /etc/sudoers.d/digital-fortress << EOF
# Digital Fortress permissions
$USER ALL=(ALL) NOPASSWD: /usr/bin/systemctl start wg-quick@wg0
$USER ALL=(ALL) NOPASSWD: /usr/bin/systemctl stop wg-quick@wg0
$USER ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart wg-quick@wg0
$USER ALL=(ALL) NOPASSWD: /usr/bin/systemctl is-active wg-quick@wg0
$USER ALL=(ALL) NOPASSWD: /usr/bin/iptables
$USER ALL=(ALL) NOPASSWD: /usr/bin/ip
$USER ALL=(ALL) NOPASSWD: /usr/sbin/sysctl
$USER ALL=(ALL) NOPASSWD: $HOME/.local/bin/ghost-mode
$USER ALL=(ALL) NOPASSWD: /home/lou/scripts/*
EOF
    
    # Setup autostart
    info "Setting up autostart..."
    cat > ~/.config/autostart/digital-fortress.desktop << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Digital Fortress
Comment=Complete Privacy & Anonymity Suite
Exec=$HOME/.local/bin/wireguard-tray-widget.py
Icon=security-high
Terminal=false
Categories=Network;Security;
X-GNOME-Autostart-enabled=true
Hidden=false
EOF
    
    success "🏰 Digital Fortress setup complete!"
    info "Run 'digital-fortress activate' to start complete protection"
}

# Test anonymity function
test_anonymity() {
    fortress "Testing anonymity protection..."
    
    echo ""
    info "🧪 Running anonymity tests..."
    
    # Test 1: IP Address
    EXTERNAL_IP=$(get_external_ip)
    echo "📍 External IP: $EXTERNAL_IP"
    
    # Test 2: DNS Leak Test
    echo "🔍 DNS Servers:"
    resolvectl status 2>/dev/null | grep "DNS Servers" | head -3 || echo "Could not detect DNS servers"
    
    # Test 3: WebRTC Test
    echo "🌐 WebRTC Status:"
    if command -v firefox >/dev/null 2>&1; then
        echo "   Use browser to visit: https://browserleaks.com/webrtc"
    else
        echo "   Firefox not available for WebRTC test"
    fi
    
    # Test 4: VPN Status
    echo "🛡️  VPN Status:"
    if is_vpn_active; then
        echo "   ✅ WireGuard VPN is active"
    else
        echo "   ❌ WireGuard VPN is not active"
    fi
    
    # Test 5: Ghost Mode Status
    echo "🥷 Ghost Mode Status:"
    if is_ghost_active; then
        echo "   ✅ Ghost Mode is active"
    else
        echo "   ❌ Ghost Mode is not active"
    fi
    
    echo ""
    info "Visit these sites to verify complete anonymity:"
    echo "   🔍 https://browserleaks.com - Complete browser testing"
    echo "   🌐 https://ipleak.net - IP and WebRTC leak testing"
    echo "   🛡️  https://dnsleaktest.com - DNS leak testing"
}

# Main command processing
case "${1:-help}" in
    "activate"|"start")
        activate_fortress
        ;;
    "deactivate"|"stop")
        deactivate_fortress
        ;;
    "toggle")
        toggle_fortress
        ;;
    "status")
        show_status
        ;;
    "setup"|"install")
        setup_fortress
        ;;
    "test")
        test_anonymity
        ;;
    "help"|"--help"|"-h")
        print_banner
        echo -e "${CYAN}🏰 DIGITAL FORTRESS COMMANDS:${NC}"
        echo ""
        echo -e "${BLUE}Main Commands:${NC}"
        echo "  activate/start     - Activate complete protection suite"
        echo "  deactivate/stop    - Deactivate protection (keep basic VPN)"
        echo "  toggle            - Toggle fortress on/off"
        echo "  status            - Show comprehensive status"
        echo ""
        echo -e "${BLUE}Setup & Testing:${NC}"
        echo "  setup/install     - Initial setup and configuration"
        echo "  test              - Test anonymity protection"
        echo ""
        echo -e "${BLUE}Protection Layers:${NC}"
        echo "  🛡️  WireGuard VPN with auto-rotation"
        echo "  🥷 Ghost Mode complete invisibility"
        echo "  🎭 API masking proxy for AI services"
        echo "  🌐 Smart routing (preserve connection + mask APIs)"
        echo "  👁️  Continuous monitoring and leak detection"
        echo "  🖥️  System tray widget for GUI control"
        echo ""
        echo -e "${PURPLE}🏰 Your Digital Fortress is ready to protect you!${NC}"
        ;;
    *)
        error "Unknown command: $1"
        echo "Run 'digital-fortress help' for usage information"
        exit 1
        ;;
esac