#!/bin/bash

# 🔧 HOME ASSISTANT CONNECTIVITY FIX SCRIPT
# ========================================
# Fixes network connectivity issues between Home Assistant and media stack services

echo "🏠 Home Assistant Connectivity Fix Script"
echo "=========================================="

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to test connectivity
test_connectivity() {
    local service=$1
    local ip=$2
    local port=$3
    
    echo -e "${BLUE}Testing $service at $ip:$port...${NC}"
    
    if timeout 5 bash -c "</dev/tcp/$ip/$port"; then
        echo -e "${GREEN}✅ $service is reachable${NC}"
        return 0
    else
        echo -e "${RED}❌ $service is NOT reachable${NC}"
        return 1
    fi
}

# Function to ping IP address
ping_test() {
    local service=$1
    local ip=$2
    
    echo -e "${BLUE}Pinging $service at $ip...${NC}"
    
    if ping -c 2 -W 3 $ip > /dev/null 2>&1; then
        echo -e "${GREEN}✅ $service IP is reachable${NC}"
        return 0
    else
        echo -e "${RED}❌ $service IP is NOT reachable${NC}"
        return 1
    fi
}

echo "🔍 Testing connectivity to media stack services..."
echo ""

# Test network connectivity to each service
declare -A services=(
    ["Plex Media Server"]="192.168.122.230:32400"
    ["Jellyfin Media Server"]="192.168.122.231:8096"
    ["Ollama AI Services"]="192.168.122.172:11434"
    ["Traefik Load Balancer"]="192.168.122.103:9080"
)

failed_services=()

for service in "${!services[@]}"; do
    IFS=':' read -r ip port <<< "${services[$service]}"
    
    # First test ping
    if ! ping_test "$service" "$ip"; then
        failed_services+=("$service")
        continue
    fi
    
    # Then test port connectivity
    if ! test_connectivity "$service" "$ip" "$port"; then
        failed_services+=("$service")
    fi
    
    echo ""
done

echo "📊 CONNECTIVITY SUMMARY"
echo "======================"

if [ ${#failed_services[@]} -eq 0 ]; then
    echo -e "${GREEN}🎉 All services are reachable! Network connectivity is good.${NC}"
else
    echo -e "${RED}⚠️  Failed services: ${#failed_services[@]}${NC}"
    for service in "${failed_services[@]}"; do
        echo -e "${RED}   • $service${NC}"
    done
fi

echo ""
echo "🔧 TROUBLESHOOTING ACTIONS"
echo "========================="

# Check if running from Home Assistant container
if [ -f "/.dockerenv" ] || [ -f "/config/configuration.yaml" ]; then
    echo -e "${YELLOW}ℹ️  Running inside Home Assistant container${NC}"
    
    # Check network routes
    echo -e "${BLUE}Checking network configuration...${NC}"
    ip route show | grep 192.168.122
    
    # Check DNS resolution
    echo -e "${BLUE}Testing DNS resolution...${NC}"
    nslookup 192.168.122.103 > /dev/null 2>&1 && echo -e "${GREEN}✅ DNS working${NC}" || echo -e "${RED}❌ DNS issues${NC}"
    
else
    echo -e "${YELLOW}ℹ️  Running on host system${NC}"
    
    # Check if we can access the Proxmox network
    echo -e "${BLUE}Testing Proxmox network access...${NC}"
    if ip route | grep -q "192.168.122.0/24"; then
        echo -e "${GREEN}✅ Proxmox network route exists${NC}"
    else
        echo -e "${RED}❌ No route to Proxmox network${NC}"
        echo -e "${YELLOW}💡 Suggestion: Check Proxmox bridge configuration${NC}"
    fi
    
    # Check if Home Assistant container is running
    echo -e "${BLUE}Checking Home Assistant container status...${NC}"
    if pct list | grep -q "500.*running"; then
        echo -e "${GREEN}✅ Home Assistant VM/CT 500 is running${NC}"
    else
        echo -e "${RED}❌ Home Assistant VM/CT 500 is not running${NC}"
        echo -e "${YELLOW}💡 Suggestion: Start the container with 'pct start 500'${NC}"
    fi
fi

echo ""
echo "🛠️  POTENTIAL FIXES"
echo "=================="

if [ ${#failed_services[@]} -gt 0 ]; then
    echo "1. Restart services in order:"
    echo "   sudo pct start 103  # Traefik first"
    echo "   sudo pct start 230  # Plex"
    echo "   sudo pct start 231  # Jellyfin"
    echo "   sudo pct start 900  # AI Services"
    echo ""
    echo "2. Check firewall rules:"
    echo "   sudo iptables -L -n | grep 192.168.122"
    echo ""
    echo "3. Restart Home Assistant:"
    echo "   sudo pct restart 500"
    echo ""
    echo "4. Check Proxmox network bridge:"
    echo "   ip addr show vmbr1"
    echo ""
    echo "5. Test from inside Home Assistant container:"
    echo "   pct enter 500"
    echo "   curl -I http://192.168.122.103:9080"
fi

echo ""
echo "📋 Home Assistant Configuration Status"
echo "====================================="

# Check if fixed configuration files exist
config_path="/home/lou/awesome_stack/homeassistant-configs"

if [ -f "$config_path/binary_sensor.yaml" ]; then
    echo -e "${GREEN}✅ Fixed binary_sensor.yaml exists${NC}"
else
    echo -e "${RED}❌ Missing binary_sensor.yaml${NC}"
fi

if [ -f "$config_path/sensors.yaml" ]; then
    echo -e "${GREEN}✅ Fixed sensors.yaml exists${NC}"
else
    echo -e "${RED}❌ Missing sensors.yaml${NC}"
fi

if [ -f "$config_path/rest_commands.yaml" ]; then
    echo -e "${GREEN}✅ Fixed rest_commands.yaml exists${NC}"
else
    echo -e "${RED}❌ Missing rest_commands.yaml${NC}"
fi

echo ""
echo "🚀 Next Steps:"
echo "1. Copy fixed config files to Home Assistant: sudo cp -r $config_path/* /path/to/homeassistant/config/"
echo "2. Restart Home Assistant: sudo pct restart 500"
echo "3. Check logs: sudo pct exec 500 -- tail -f /config/home-assistant.log"
echo ""
echo "✅ Script completed!"
