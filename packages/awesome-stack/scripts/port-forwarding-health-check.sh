#!/bin/bash
# Port Forwarding Health Check Script
# Location: /home/lou/awesome_stack/scripts/port-forwarding-health-check.sh
# Usage: bash /home/lou/awesome_stack/scripts/port-forwarding-health-check.sh

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Port Forwarding Health Check ===${NC}"
echo "Date: $(date)"
echo "Host: $(hostname)"
echo ""

# Check external IP
echo -e "${BLUE}🌍 External IP Check${NC}"
EXTERNAL_IP=$(curl -4 -s --max-time 10 ifconfig.me)
if [ ! -z "$EXTERNAL_IP" ]; then
    echo -e "Current external IP: ${GREEN}$EXTERNAL_IP${NC}"
else
    echo -e "${RED}❌ Could not determine external IP${NC}"
fi
echo ""

# Check IP forwarding
echo -e "${BLUE}🔀 IP Forwarding Status${NC}"
IP_FORWARD=$(cat /proc/sys/net/ipv4/ip_forward)
if [ "$IP_FORWARD" = "1" ]; then
    echo -e "IP Forwarding: ${GREEN}✅ Enabled${NC}"
else
    echo -e "IP Forwarding: ${RED}❌ Disabled${NC}"
fi
echo ""

# Check iptables rules
echo -e "${BLUE}🔧 iptables NAT Rules${NC}"
RULE_COUNT=$(sudo iptables -t nat -L PREROUTING | grep -c DNAT)
echo "Active DNAT rules: $RULE_COUNT"

# Check specific DNAT rules
services=("32400:Plex" "8096:Jellyfin" "8123:Home Assistant" "11434:Ollama AI" "9080:Traefik")
for service in "${services[@]}"; do
    port=$(echo $service | cut -d: -f1)
    name=$(echo $service | cut -d: -f2)
    
    if sudo iptables -t nat -L PREROUTING | grep -q "dpt:$port"; then
        echo -e "$name ($port): ${GREEN}✅ Rule exists${NC}"
    else
        echo -e "$name ($port): ${RED}❌ Rule missing${NC}"
    fi
done
echo ""

# Test internal services
echo -e "${BLUE}🏠 Internal Service Tests${NC}"
internal_services=(
    "192.168.122.230:32400:Plex"
    "192.168.122.231:8096:Jellyfin" 
    "192.168.122.172:11434:Ollama"
    "192.168.122.103:9080:Traefik"
    "192.168.12.204:8123:Home Assistant"
)

for service in "${internal_services[@]}"; do
    ip_port=$(echo $service | cut -d: -f1,2)
    name=$(echo $service | cut -d: -f3)
    
    if timeout 5 curl -s -I "http://$ip_port" >/dev/null 2>&1; then
        echo -e "$name: ${GREEN}✅ Responding${NC}"
    else
        echo -e "$name: ${RED}❌ Not responding${NC}"
    fi
done
echo ""

# Test external access (if external IP is available)
if [ ! -z "$EXTERNAL_IP" ]; then
    echo -e "${BLUE}🌍 External Access Tests${NC}"
    external_services=("32400:Plex" "8096:Jellyfin" "8123:Home Assistant" "11434:Ollama" "9080:Traefik")
    
    for service in "${external_services[@]}"; do
        port=$(echo $service | cut -d: -f1)
        name=$(echo $service | cut -d: -f2)
        
        if timeout 10 curl -s -I "http://$EXTERNAL_IP:$port" >/dev/null 2>&1; then
            echo -e "$name ($EXTERNAL_IP:$port): ${GREEN}✅ Accessible${NC}"
        else
            echo -e "$name ($EXTERNAL_IP:$port): ${YELLOW}⚠️ Not accessible (may be normal)${NC}"
        fi
    done
    echo ""
fi

# Network interface check
echo -e "${BLUE}🔌 Network Interface Status${NC}"
if ip addr show enp4s0 >/dev/null 2>&1; then
    ip_addr=$(ip addr show enp4s0 | grep 'inet ' | awk '{print $2}')
    echo -e "Interface enp4s0: ${GREEN}✅ Up${NC} ($ip_addr)"
else
    echo -e "Interface enp4s0: ${RED}❌ Down or missing${NC}"
fi
echo ""

# Proxmox connectivity check
echo -e "${BLUE}🖥️ Proxmox Connectivity${NC}"
if timeout 5 ping -c 1 192.168.122.9 >/dev/null 2>&1; then
    echo -e "Proxmox (192.168.122.9): ${GREEN}✅ Reachable${NC}"
else
    echo -e "Proxmox (192.168.122.9): ${RED}❌ Unreachable${NC}"
fi
echo ""

# System resources
echo -e "${BLUE}📊 System Resources${NC}"
CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | sed 's/%us,//')
MEM_USAGE=$(free | grep Mem | awk '{printf "%.1f", $3/$2 * 100.0}')
echo "CPU Usage: ${CPU_USAGE}%"
echo "Memory Usage: ${MEM_USAGE}%"
echo ""

# Log summary
echo -e "${BLUE}📝 Summary${NC}"
echo "Health check completed at $(date)"
if [ "$IP_FORWARD" = "1" ] && [ "$RULE_COUNT" -gt 0 ]; then
    echo -e "Overall Status: ${GREEN}✅ Port forwarding operational${NC}"
else
    echo -e "Overall Status: ${YELLOW}⚠️ Issues detected - check above${NC}"
fi

# Optional: Log to file
LOG_FILE="/home/lou/logs/port-forwarding-health.log"
mkdir -p "$(dirname "$LOG_FILE")"
echo "$(date): Health check completed - IP Forward: $IP_FORWARD, Rules: $RULE_COUNT, External IP: $EXTERNAL_IP" >> "$LOG_FILE"

echo ""
echo -e "${BLUE}💡 Quick Commands:${NC}"
echo "• Apply rules: sudo bash /home/lou/awesome_stack/scripts/port-forwarding-rules.sh"
echo "• View NAT rules: sudo iptables -t nat -L -n"
echo "• Check external IP: curl -4 ifconfig.me"
echo "• View this log: tail -f $LOG_FILE"
