#!/bin/bash
# Complete WireGuard Management for Garuda Host
# Combines VPN protection, API masking, and dashboard access

PROXMOX_HOST="192.168.122.9"
PROXMOX_USER="root"
DASHBOARD_PORT="10086"
API_PROXY_PORT="8080"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════════════╗"
    echo "║                   🛡️  Garuda WireGuard Manager  🛡️                   ║"
    echo "║                     VPN + API Masking + Dashboard                    ║"
    echo "╚══════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_status() {
    local service=$1
    local status=$2
    if [[ $status == "active" ]]; then
        echo -e "  ${GREEN}✅ $service: ACTIVE${NC}"
    else
        echo -e "  ${RED}❌ $service: INACTIVE${NC}"
    fi
}

check_services() {
    echo -e "${YELLOW}🔍 Checking Services Status...${NC}"
    echo
    
    # Check WireGuard VPN
    if pgrep -f "wg-quick" > /dev/null; then
        print_status "WireGuard VPN" "active"
        # Show current IP
        if command -v wg &> /dev/null; then
            local wg_status=$(wg show 2>/dev/null)
            if [[ -n "$wg_status" ]]; then
                echo "    └─ Interface: $(echo "$wg_status" | grep "interface:" | cut -d: -f2 | tr -d ' ')"
            fi
        fi
    else
        print_status "WireGuard VPN" "inactive"
    fi
    
    # Check API Masking Proxy
    if curl -s --connect-timeout 2 http://127.0.0.1:$API_PROXY_PORT/health >/dev/null 2>&1; then
        print_status "API Masking Proxy" "active"
        echo "    └─ Port: $API_PROXY_PORT"
    else
        print_status "API Masking Proxy" "inactive"
    fi
    
    # Check Dashboard Access
    if curl -s --connect-timeout 2 http://$PROXMOX_HOST:$DASHBOARD_PORT/ >/dev/null 2>&1; then
        print_status "WGDashboard" "active"
        echo "    └─ URL: http://$PROXMOX_HOST:$DASHBOARD_PORT"
    else
        print_status "WGDashboard" "inactive"
    fi
    
    # Check External IP
    echo
    echo -e "${YELLOW}🌐 Network Status:${NC}"
    local ext_ip=$(curl -s --connect-timeout 5 ifconfig.me 2>/dev/null || echo "Unable to detect")
    echo "  📡 External IP: $ext_ip"
    
    # DNS Check
    local dns_status="OK"
    if ! nslookup google.com >/dev/null 2>&1; then
        dns_status="FAILED"
    fi
    echo "  🔍 DNS Resolution: $dns_status"
    
    echo
}

rotate_wireguard() {
    echo -e "${YELLOW}🔄 Rotating WireGuard Configuration...${NC}"
    
    if ssh -o ConnectTimeout=5 "$PROXMOX_USER@$PROXMOX_HOST" "/root/wireguard-rotate.sh garuda-host" 2>/dev/null; then
        echo -e "${GREEN}✅ Server-side rotation completed${NC}"
        
        # Fetch new config
        echo "📥 Fetching new client configuration..."
        local latest_config=$(ssh "$PROXMOX_USER@$PROXMOX_HOST" "ls -1t /etc/wireguard/clients/garuda-host-*.conf 2>/dev/null | head -1")
        
        if [[ -n "$latest_config" ]]; then
            echo "📋 New configuration available: $(basename "$latest_config")"
            echo
            echo -e "${BLUE}📄 Copy this configuration to your Garuda host:${NC}"
            echo "=================================="
            ssh "$PROXMOX_USER@$PROXMOX_HOST" "cat '$latest_config'"
            echo "=================================="
            echo
            echo -e "${YELLOW}💡 To apply on Garuda:${NC}"
            echo "  1. Save the above config to /etc/wireguard/wg0.conf"
            echo "  2. Run: sudo wg-quick down wg0 && sudo wg-quick up wg0"
            echo "  3. Verify with: curl ifconfig.me"
        else
            echo -e "${RED}❌ Could not find new configuration${NC}"
        fi
    else
        echo -e "${RED}❌ Failed to rotate configuration${NC}"
        echo "Make sure SSH access to Proxmox host is working"
    fi
}

start_api_proxy() {
    echo -e "${YELLOW}🚀 Starting API Masking Proxy...${NC}"
    
    # Check if already running
    if curl -s --connect-timeout 2 http://127.0.0.1:$API_PROXY_PORT/health >/dev/null 2>&1; then
        echo -e "${GREEN}✅ API Proxy already running on port $API_PROXY_PORT${NC}"
        return 0
    fi
    
    # Start the proxy
    if [[ -f "/root/api-mask-proxy.py" ]]; then
        echo "Starting proxy server..."
        nohup python3 /root/api-mask-proxy.py > /var/log/api-proxy.log 2>&1 &
        sleep 3
        
        if curl -s --connect-timeout 2 http://127.0.0.1:$API_PROXY_PORT/health >/dev/null 2>&1; then
            echo -e "${GREEN}✅ API Masking Proxy started successfully${NC}"
            echo "  📡 Proxy URL: http://127.0.0.1:$API_PROXY_PORT"
            echo "  📋 Usage: Route AI API calls through this proxy"
            echo "  📝 Example: http://127.0.0.1:$API_PROXY_PORT/api/openai/v1/chat/completions"
        else
            echo -e "${RED}❌ Failed to start API proxy${NC}"
        fi
    else
        echo -e "${RED}❌ API proxy script not found${NC}"
        echo "Copy api-mask-proxy.py from Proxmox host first"
    fi
}

stop_api_proxy() {
    echo -e "${YELLOW}🛑 Stopping API Masking Proxy...${NC}"
    pkill -f "api-mask-proxy.py"
    echo -e "${GREEN}✅ API Proxy stopped${NC}"
}

open_dashboard() {
    echo -e "${YELLOW}🌐 Opening WireGuard Dashboard...${NC}"
    
    local dashboard_url="http://$PROXMOX_HOST:$DASHBOARD_PORT"
    
    if curl -s --connect-timeout 5 "$dashboard_url" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Dashboard is accessible${NC}"
        echo "  🔗 URL: $dashboard_url"
        echo "  📱 You can also access from mobile devices on the same network"
        echo
        echo "  🔐 Dashboard Features:"
        echo "    • View WireGuard server status"
        echo "    • Manage client connections"  
        echo "    • Monitor traffic and usage"
        echo "    • Add/remove clients"
        echo "    • Download client configurations"
        
        # Try to open in browser if available
        if command -v xdg-open &> /dev/null; then
            echo
            echo -e "${BLUE}🚀 Opening in browser...${NC}"
            xdg-open "$dashboard_url" 2>/dev/null &
        elif command -v firefox &> /dev/null; then
            echo
            echo -e "${BLUE}🚀 Opening in Firefox...${NC}"
            firefox "$dashboard_url" 2>/dev/null &
        fi
    else
        echo -e "${RED}❌ Dashboard is not accessible${NC}"
        echo "  Check if WGDashboard is running on Proxmox host"
        echo "  Try: ssh $PROXMOX_USER@$PROXMOX_HOST 'systemctl status wgdashboard'"
    fi
}

test_ai_access() {
    echo -e "${YELLOW}🤖 Testing AI Service Access...${NC}"
    echo
    
    local ai_services=(
        "claude.ai"
        "api.openai.com" 
        "chat.openai.com"
        "api.anthropic.com"
        "api.cohere.ai"
        "api.mistral.ai"
    )
    
    for service in "${ai_services[@]}"; do
        echo -n "  Testing $service ... "
        if timeout 5 curl -s --connect-timeout 3 "https://$service" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ Accessible${NC}"
        else
            echo -e "${RED}❌ Blocked/Unavailable${NC}"
        fi
    done
    
    echo
    echo -e "${BLUE}💡 If services are blocked:${NC}"
    echo "  1. Start API masking proxy: $0 start-proxy"
    echo "  2. Configure applications to use proxy: http://127.0.0.1:$API_PROXY_PORT"
    echo "  3. Rotate WireGuard config: $0 rotate"
}

enable_stealth_mode() {
    echo -e "${YELLOW}🥷 Enabling Stealth Mode...${NC}"
    echo "This will rotate WireGuard configuration every 30 minutes"
    echo -e "${RED}⚠️  This runs continuously - Press Ctrl+C to stop${NC}"
    echo
    
    while true; do
        echo -e "${BLUE}[$(date '+%H:%M:%S')] Rotating configuration...${NC}"
        rotate_wireguard
        echo -e "${GREEN}[$(date '+%H:%M:%S')] Rotation complete. Sleeping for 30 minutes...${NC}"
        echo "──────────────────────────────────────────────────────────"
        sleep 1800  # 30 minutes
    done
}

install_dependencies() {
    echo -e "${YELLOW}📦 Installing Garuda Dependencies...${NC}"
    
    # Install WireGuard tools
    if ! command -v wg &> /dev/null; then
        echo "Installing WireGuard tools..."
        sudo pacman -S --noconfirm wireguard-tools
    fi
    
    # Install Python for API proxy
    if ! command -v python3 &> /dev/null; then
        echo "Installing Python..."
        sudo pacman -S --noconfirm python python-pip
    fi
    
    # Install aiohttp for API proxy
    if ! python3 -c "import aiohttp" 2>/dev/null; then
        echo "Installing Python dependencies..."
        pip install --user aiohttp
    fi
    
    echo -e "${GREEN}✅ Dependencies installed${NC}"
}

copy_files_from_proxmox() {
    echo -e "${YELLOW}📥 Copying files from Proxmox host...${NC}"
    
    # Create local directory
    mkdir -p ~/wireguard-manager
    
    # Copy API masking proxy
    if scp "$PROXMOX_USER@$PROXMOX_HOST:/root/api-mask-proxy.py" ~/wireguard-manager/ 2>/dev/null; then
        echo -e "${GREEN}✅ Copied API masking proxy${NC}"
        cp ~/wireguard-manager/api-mask-proxy.py /tmp/api-mask-proxy.py
    else
        echo -e "${RED}❌ Failed to copy API proxy${NC}"
    fi
    
    # Copy browser masking script
    if scp "$PROXMOX_USER@$PROXMOX_HOST:/root/browser-api-mask.js" ~/wireguard-manager/ 2>/dev/null; then
        echo -e "${GREEN}✅ Copied browser masking script${NC}"
        echo "  📝 Use this script in browser console for client-side masking"
    else
        echo -e "${RED}❌ Failed to copy browser script${NC}"
    fi
    
    echo
    echo -e "${BLUE}📂 Files copied to: ~/wireguard-manager/${NC}"
}

# Main menu
show_menu() {
    print_header
    check_services
    
    echo -e "${BLUE}🎛️  Available Commands:${NC}"
    echo "  status          - Check all services status"
    echo "  rotate          - Rotate WireGuard keys and IP"
    echo "  start-proxy     - Start API masking proxy"
    echo "  stop-proxy      - Stop API masking proxy" 
    echo "  dashboard       - Open WireGuard dashboard"
    echo "  test            - Test AI service accessibility"
    echo "  stealth         - Enable continuous rotation (30min)"
    echo "  install         - Install required dependencies"
    echo "  copy-files      - Copy tools from Proxmox host"
    echo "  help            - Show this menu"
    echo
}

# Command processing
case "${1:-help}" in
    "status")
        print_header
        check_services
        ;;
    "rotate")
        print_header
        rotate_wireguard
        ;;
    "start-proxy")
        print_header
        start_api_proxy
        ;;
    "stop-proxy")
        print_header
        stop_api_proxy
        ;;
    "dashboard")
        print_header
        open_dashboard
        ;;
    "test")
        print_header
        test_ai_access
        ;;
    "stealth")
        print_header
        enable_stealth_mode
        ;;
    "install")
        print_header
        install_dependencies
        ;;
    "copy-files")
        print_header
        copy_files_from_proxmox
        ;;
    "help"|*)
        show_menu
        echo -e "${YELLOW}💡 Usage: $0 [command]${NC}"
        echo -e "${YELLOW}   Example: $0 status${NC}"
        ;;
esac
