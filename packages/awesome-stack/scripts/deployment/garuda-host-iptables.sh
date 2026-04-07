#!/bin/bash

# Garuda Host iptables Port Forwarding Script
# Forwards traffic from Garuda host to Proxmox VM services

echo "Setting up iptables port forwarding on Garuda host..."

# Proxmox VM IP
PROXMOX_VM_IP="192.168.122.9"

# Enable IP forwarding
echo "Enabling IP forwarding..."
echo 1 > /proc/sys/net/ipv4/ip_forward

# Make IP forwarding persistent
if ! grep -q "net.ipv4.ip_forward=1" /etc/sysctl.conf; then
    echo "net.ipv4.ip_forward=1" >> /etc/sysctl.conf
fi

# Clear existing FORWARD rules for clean setup
echo "Clearing existing FORWARD rules..."
iptables -t nat -F PREROUTING
iptables -t nat -F POSTROUTING

# HTTP traffic (port 80) -> Proxmox VM
echo "Setting up HTTP port forwarding (80 -> ${PROXMOX_VM_IP}:80)..."
iptables -t nat -A PREROUTING -p tcp --dport 80 -j DNAT --to-destination ${PROXMOX_VM_IP}:80
iptables -A FORWARD -p tcp -d ${PROXMOX_VM_IP} --dport 80 -j ACCEPT

# HTTPS traffic (port 443) -> Proxmox VM  
echo "Setting up HTTPS port forwarding (443 -> ${PROXMOX_VM_IP}:443)..."
iptables -t nat -A PREROUTING -p tcp --dport 443 -j DNAT --to-destination ${PROXMOX_VM_IP}:443
iptables -A FORWARD -p tcp -d ${PROXMOX_VM_IP} --dport 443 -j ACCEPT

# Traefik Dashboard (port 8080) -> Proxmox VM
echo "Setting up Traefik Dashboard forwarding (8080 -> ${PROXMOX_VM_IP}:8080)..."
iptables -t nat -A PREROUTING -p tcp --dport 8080 -j DNAT --to-destination ${PROXMOX_VM_IP}:8080
iptables -A FORWARD -p tcp -d ${PROXMOX_VM_IP} --dport 8080 -j ACCEPT

# MASQUERADE for return traffic
echo "Setting up MASQUERADE for return traffic..."
iptables -t nat -A POSTROUTING -o virbr0 -j MASQUERADE
iptables -A FORWARD -i virbr0 -j ACCEPT

echo "✅ iptables port forwarding configured successfully!"
echo ""
echo "🎯 Test your setup:"
# Get the primary IP address more reliably
HOST_IP=$(ip route get 1.1.1.1 | grep -oP 'src \K\S+' 2>/dev/null || ip addr show | grep 'inet ' | grep -v '127.0.0.1' | head -1 | awk '{print $2}' | cut -d'/' -f1)
echo "   HTTP:  curl -I http://${HOST_IP}"
echo "   HTTPS: curl -k -I https://${HOST_IP}"
echo ""
echo "📝 Current NAT rules:"
iptables -t nat -L -n --line-numbers | grep -E "(DNAT|REDIRECT)"
