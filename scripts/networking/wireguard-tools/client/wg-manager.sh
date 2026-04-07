#!/bin/bash
# WireGuard Management for Pop!_OS
# Local Docker-based VPN management, API masking, and dashboard access

WG_CONTAINER="wireguard"
TOOLS_DIR="$HOME/wireguard-tools"
API_PROXY_PORT="8080"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════════════╗"
    echo "║                  🛡️  Pop!_OS WireGuard Manager  🛡️                   ║"
    echo "║                   VPN + API Masking + Dashboard                     ║"
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

    # Check WireGuard Docker container
    if docker ps --format '{{.Names}}' 2>/dev/null | grep -q "^${WG_CONTAINER}$"; then
        print_status "WireGuard Container" "active"
        local wg_status
        wg_status=$(docker exec "$WG_CONTAINER" wg show 2>/dev/null)
        if [[ -n "$wg_status" ]]; then
            local peers
            peers=$(echo "$wg_status" | grep -c "peer:" || echo "0")
            echo "    └─ Active peers: $peers"
        fi
    else
        print_status "WireGuard Container" "inactive"
    fi

    # Check API Masking Proxy
    if curl -s --connect-timeout 2 http://127.0.0.1:$API_PROXY_PORT/health >/dev/null 2>&1; then
        print_status "API Masking Proxy" "active"
        echo "    └─ Port: $API_PROXY_PORT"
    else
        print_status "API Masking Proxy" "inactive"
    fi

    # Check wg-easy dashboard
    if docker ps --format '{{.Names}}' 2>/dev/null | grep -q "^wg-easy$"; then
        print_status "WG-Easy Dashboard" "active"
        echo "    └─ URL: http://localhost:52821"
    else
        print_status "WG-Easy Dashboard" "inactive"
    fi

    # External IP
    echo
    echo -e "${YELLOW}🌐 Network Status:${NC}"
    local ext_ip
    ext_ip=$(curl -s --connect-timeout 5 ifconfig.me 2>/dev/null || echo "Unable to detect")
    echo "  📡 External IP: $ext_ip"

    local dns_status="OK"
    if ! nslookup google.com >/dev/null 2>&1; then
        dns_status="FAILED"
    fi
    echo "  🔍 DNS Resolution: $dns_status"
    echo
}

rotate_wireguard() {
    echo -e "${YELLOW}🔄 Rotating WireGuard Configuration...${NC}"

    if bash "$TOOLS_DIR/server/wireguard-rotate.sh" rotate "$(hostname)-client" 2>/dev/null; then
        echo -e "${GREEN}✅ Rotation completed${NC}"
        echo
        echo -e "${YELLOW}💡 To apply on a remote client:${NC}"
        local latest
        latest=$(ls -1t "$TOOLS_DIR/clients/"*.conf 2>/dev/null | head -1)
        if [[ -n "$latest" ]]; then
            echo "  Config: $latest"
            echo "  Copy to client and import into WireGuard"
        fi
    else
        echo -e "${RED}❌ Rotation failed${NC}"
        echo "Make sure the wireguard Docker container is running"
    fi
}

start_api_proxy() {
    echo -e "${YELLOW}🚀 Starting API Masking Proxy...${NC}"

    if curl -s --connect-timeout 2 http://127.0.0.1:$API_PROXY_PORT/health >/dev/null 2>&1; then
        echo -e "${GREEN}✅ API Proxy already running on port $API_PROXY_PORT${NC}"
        return 0
    fi

    local proxy_script="$TOOLS_DIR/api-masking/api-mask-proxy.py"
    if [[ -f "$proxy_script" ]]; then
        nohup python3 "$proxy_script" > "$TOOLS_DIR/api-proxy.log" 2>&1 &
        sleep 3

        if curl -s --connect-timeout 2 http://127.0.0.1:$API_PROXY_PORT/health >/dev/null 2>&1; then
            echo -e "${GREEN}✅ API Masking Proxy started successfully${NC}"
            echo "  📡 Proxy URL: http://127.0.0.1:$API_PROXY_PORT"
        else
            echo -e "${RED}❌ Failed to start API proxy${NC}"
        fi
    else
        echo -e "${RED}❌ API proxy script not found at $proxy_script${NC}"
    fi
}

stop_api_proxy() {
    echo -e "${YELLOW}🛑 Stopping API Masking Proxy...${NC}"
    pkill -f "api-mask-proxy.py" 2>/dev/null
    echo -e "${GREEN}✅ API Proxy stopped${NC}"
}

open_dashboard() {
    echo -e "${YELLOW}🌐 Opening WG-Easy Dashboard...${NC}"

    local dashboard_url="http://localhost:52821"

    if docker ps --format '{{.Names}}' 2>/dev/null | grep -q "^wg-easy$"; then
        echo -e "${GREEN}✅ Dashboard is accessible${NC}"
        echo "  🔗 URL: $dashboard_url"

        if command -v xdg-open &> /dev/null; then
            xdg-open "$dashboard_url" 2>/dev/null &
        fi
    else
        echo -e "${RED}❌ WG-Easy container is not running${NC}"
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
    echo "  2. Rotate WireGuard config: $0 rotate"
}

enable_stealth_mode() {
    echo -e "${YELLOW}🥷 Enabling Stealth Mode...${NC}"
    echo "This will rotate WireGuard configuration every 30 minutes"
    echo -e "${RED}⚠️  Press Ctrl+C to stop${NC}"
    echo

    while true; do
        echo -e "${BLUE}[$(date '+%H:%M:%S')] Rotating configuration...${NC}"
        rotate_wireguard
        echo -e "${GREEN}[$(date '+%H:%M:%S')] Rotation complete. Sleeping for 30 minutes...${NC}"
        echo "──────────────────────────────────────────────────────────"
        sleep 1800
    done
}

install_dependencies() {
    echo -e "${YELLOW}📦 Installing Dependencies...${NC}"

    if ! command -v wg &> /dev/null; then
        echo "Installing WireGuard tools..."
        sudo nala install -y wireguard-tools
    else
        echo -e "${GREEN}✅ WireGuard tools already installed${NC}"
    fi

    if ! command -v python3 &> /dev/null; then
        echo "Installing Python..."
        sudo nala install -y python3 python3-pip
    else
        echo -e "${GREEN}✅ Python3 already installed${NC}"
    fi

    if ! python3 -c "import aiohttp" 2>/dev/null; then
        echo "Installing Python aiohttp..."
        pip3 install --user aiohttp
    else
        echo -e "${GREEN}✅ aiohttp already installed${NC}"
    fi

    if ! python3 -c "import PyQt5" 2>/dev/null; then
        echo "Installing PyQt5..."
        sudo nala install -y python3-pyqt5
    else
        echo -e "${GREEN}✅ PyQt5 already installed${NC}"
    fi

    if ! python3 -c "import requests" 2>/dev/null; then
        echo "Installing Python requests..."
        sudo nala install -y python3-requests
    else
        echo -e "${GREEN}✅ requests already installed${NC}"
    fi

    echo -e "${GREEN}✅ All dependencies installed${NC}"
}

show_menu() {
    print_header
    check_services

    echo -e "${BLUE}🎛️  Available Commands:${NC}"
    echo "  status          - Check all services status"
    echo "  rotate          - Rotate WireGuard keys and IP"
    echo "  start-proxy     - Start API masking proxy"
    echo "  stop-proxy      - Stop API masking proxy"
    echo "  dashboard       - Open WG-Easy dashboard"
    echo "  test            - Test AI service accessibility"
    echo "  stealth         - Enable continuous rotation (30min)"
    echo "  install         - Install required dependencies"
    echo "  help            - Show this menu"
    echo
}

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
    "help"|*)
        show_menu
        echo -e "${YELLOW}💡 Usage: $0 [command]${NC}"
        ;;
esac
