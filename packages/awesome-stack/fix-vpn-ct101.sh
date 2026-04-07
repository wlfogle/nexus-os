#!/bin/bash

# CT-101 VPN Diagnostic and Recovery Script
# Based on WIREGUARD-GLUETUN-SOLUTION.md documentation

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

CT_101_IP="192.168.122.101"
PROXMOX_ALIAS="proxmox"
PROXY_PORT="8888"

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

# Function to test proxy connectivity
test_proxy() {
    print_status "Testing TinyProxy connectivity on CT-101..."
    if nc -z -w5 $CT_101_IP $PROXY_PORT; then
        print_success "TinyProxy is responding on port $PROXY_PORT"
        return 0
    else
        print_error "Cannot connect to TinyProxy on port $PROXY_PORT"
        return 1
    fi
}

# Function to test VPN connectivity through proxy
test_vpn_connection() {
    print_status "Testing VPN connectivity through proxy..."
    
    # Test external IP through proxy
    EXTERNAL_IP=$(curl -s --connect-timeout 10 -x http://$CT_101_IP:$PROXY_PORT http://httpbin.org/ip 2>/dev/null | grep -o '"origin": "[^"]*' | cut -d'"' -f4)
    
    if [ -n "$EXTERNAL_IP" ]; then
        print_success "VPN is working - External IP: $EXTERNAL_IP"
        return 0
    else
        print_error "VPN connection is DOWN - proxy cannot reach external sites"
        return 1
    fi
}

# Function to generate SSH commands for Proxmox host
generate_fix_commands() {
    print_warning "CT-101 VPN appears to be down. Here are commands to run on Proxmox host:"
    echo ""
    echo -e "${YELLOW}# Connect to Proxmox host and run these commands:${NC}"
    echo "ssh $PROXMOX_ALIAS"
    echo ""
    echo -e "${YELLOW}# 1. Check container status:${NC}"
    echo "pct status 101"
    echo ""
    echo -e "${YELLOW}# 2. If container is running, restart WireGuard:${NC}"
    echo "pct exec 101 -- wg-quick down wg0"
    echo "pct exec 101 -- wg-quick up wg0"
    echo ""
    echo -e "${YELLOW}# 3. Restart TinyProxy:${NC}"
    echo "pct exec 101 -- rc-service tinyproxy restart"
    echo ""
    echo -e "${YELLOW}# 4. Check WireGuard status:${NC}"
    echo "pct exec 101 -- wg show"
    echo ""
    echo -e "${YELLOW}# 5. Test VPN connection from container:${NC}"
    echo "pct exec 101 -- curl -s http://httpbin.org/ip"
    echo ""
    echo -e "${YELLOW}# 6. If container won't start:${NC}"
    echo "pct start 101"
    echo ""
}

# Function to create a remote restart script
create_remote_script() {
    print_status "Creating remote restart script..."
    
cat > /tmp/restart-ct101-vpn.sh << 'EOF'
#!/bin/bash
# CT-101 VPN restart script for Proxmox host

echo "Checking CT-101 status..."
pct status 101

echo "Stopping WireGuard..."
pct exec 101 -- wg-quick down wg0 2>/dev/null

echo "Starting WireGuard..."
pct exec 101 -- wg-quick up wg0

echo "Restarting TinyProxy..."
pct exec 101 -- rc-service tinyproxy restart

echo "Checking WireGuard status..."
pct exec 101 -- wg show

echo "Testing VPN connection..."
pct exec 101 -- curl -s http://httpbin.org/ip

echo "CT-101 VPN restart completed"
EOF

    chmod +x /tmp/restart-ct101-vpn.sh
    print_success "Remote script created at /tmp/restart-ct101-vpn.sh"
    print_status "Copy this script to your Proxmox host and run it as root"
}

# Main diagnostic flow
echo "========================================"
echo "  CT-101 VPN Diagnostic Tool"
echo "========================================"
echo ""

# Test 1: Proxy connectivity
if test_proxy; then
    # Test 2: VPN connectivity
    if test_vpn_connection; then
        print_success "CT-101 VPN is working correctly!"
        exit 0
    else
        print_warning "Proxy is running but VPN connection is down"
        generate_fix_commands
        create_remote_script
        exit 1
    fi
else
    print_error "Cannot reach CT-101 proxy service"
    print_warning "Container may be stopped or network issues"
    generate_fix_commands
    create_remote_script
    exit 1
fi
