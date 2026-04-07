#!/bin/bash
# VPN NAT Rules for Main System (External IP: 172.59.82.13)
# Run this script on your main system (192.168.12.204) to complete VPN setup

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

echo "========================================"
echo "  VPN NAT Rules Setup"
echo "  For External IP: 172.59.82.13"
echo "========================================"
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   print_error "This script must be run as root (use sudo)"
   exit 1
fi

# Detect external network interface
print_status "Detecting network interfaces..."
EXTERNAL_INTERFACE=$(ip route | grep default | awk '{print $5}' | head -n1)
EXTERNAL_IP=$(ip addr show $EXTERNAL_INTERFACE | grep 'inet ' | awk '{print $2}' | cut -d'/' -f1)

print_success "Detected external interface: $EXTERNAL_INTERFACE"
print_success "Detected external IP: $EXTERNAL_IP"

# Backup current iptables rules
print_status "Backing up current iptables rules..."
iptables-save > /tmp/iptables_backup_$(date +%Y%m%d_%H%M%S).rules
print_success "Backup saved to /tmp/iptables_backup_$(date +%Y%m%d_%H%M%S).rules"

# Enable IP forwarding
print_status "Enabling IP forwarding..."
echo 1 > /proc/sys/net/ipv4/ip_forward

# Make IP forwarding persistent
if ! grep -q "net.ipv4.ip_forward=1" /etc/sysctl.conf; then
    echo "net.ipv4.ip_forward=1" >> /etc/sysctl.conf
    print_success "IP forwarding enabled persistently"
else
    print_success "IP forwarding already configured"
fi

# Add NAT rules for WireGuard VPN subnet (10.0.0.0/24)
print_status "Adding NAT rules for WireGuard subnet (10.0.0.0/24)..."

# MASQUERADE rule for VPN traffic
iptables -t nat -A POSTROUTING -s 10.0.0.0/24 -o $EXTERNAL_INTERFACE -j MASQUERADE

# FORWARD rules for VPN traffic
iptables -A FORWARD -s 10.0.0.0/24 -j ACCEPT
iptables -A FORWARD -d 10.0.0.0/24 -j ACCEPT

# Allow forwarding between VPN and external interface
iptables -A FORWARD -i tun+ -o $EXTERNAL_INTERFACE -j ACCEPT
iptables -A FORWARD -i $EXTERNAL_INTERFACE -o tun+ -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT

print_success "NAT rules added for WireGuard VPN"

# Display current rules
print_status "Current NAT rules:"
iptables -t nat -L POSTROUTING -v --line-numbers | grep -E "(MASQUERADE|10.0.0.0)"

print_status "Current FORWARD rules:"
iptables -L FORWARD -v --line-numbers | grep -E "(10.0.0.0|ACCEPT)"

echo ""
print_success "✅ VPN NAT rules configured successfully!"
echo ""
print_warning "To make these rules persistent across reboots:"
echo "sudo iptables-save > /etc/iptables/rules.v4"
echo "OR"  
echo "sudo netfilter-persistent save"
echo ""
print_status "Testing VPN connection from CT-101..."
echo "Run this command to test: ssh proxmox \"pct exec 101 -- curl -s http://httpbin.org/ip\""
echo ""

# Test the VPN connection
print_status "Attempting to test VPN connection..."
if command -v ssh >/dev/null 2>&1; then
    echo "Testing VPN through proxy..."
    TEST_RESULT=$(timeout 10 curl -s --connect-timeout 5 -x http://192.168.122.101:8888 http://httpbin.org/ip 2>/dev/null || echo "timeout")
    
    if [[ "$TEST_RESULT" != "timeout" && "$TEST_RESULT" != "" ]]; then
        print_success "🎉 VPN is working! External IP via VPN:"
        echo "$TEST_RESULT"
    else
        print_warning "VPN test inconclusive - check CT-101 container status"
    fi
else
    print_warning "SSH not available for remote testing"
fi

echo ""
print_success "🎯 VPN NAT setup complete!"
