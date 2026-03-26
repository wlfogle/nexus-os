#!/bin/bash
# Port Forwarding Rules for Media Stack
# Run with: sudo bash /home/lou/port-forwarding-rules.sh

echo "Setting up port forwarding for media stack..."

# Enable IP forwarding
echo 1 > /proc/sys/net/ipv4/ip_forward
echo "net.ipv4.ip_forward=1" >> /etc/sysctl.conf

# Clear existing rules (optional)
# iptables -t nat -F PREROUTING
# iptables -F FORWARD

# DNAT Rules (Port Forwarding)
echo "Setting up DNAT rules..."
iptables -t nat -A PREROUTING -p tcp --dport 32400 -j DNAT --to-destination 192.168.122.230:32400  # Plex
iptables -t nat -A PREROUTING -p tcp --dport 8096 -j DNAT --to-destination 192.168.122.231:8096    # Jellyfin
iptables -t nat -A PREROUTING -p tcp --dport 8123 -j DNAT --to-destination 192.168.12.204:8123     # Home Assistant
iptables -t nat -A PREROUTING -p tcp --dport 11434 -j DNAT --to-destination 192.168.122.172:11434  # Ollama AI
iptables -t nat -A PREROUTING -p tcp --dport 9080 -j DNAT --to-destination 192.168.122.103:9080    # Traefik

# Forward Rules
echo "Setting up FORWARD rules..."
iptables -A FORWARD -p tcp --dport 32400 -d 192.168.122.230 -j ACCEPT
iptables -A FORWARD -p tcp --dport 8096 -d 192.168.122.231 -j ACCEPT
iptables -A FORWARD -p tcp --dport 8123 -d 192.168.12.204 -j ACCEPT
iptables -A FORWARD -p tcp --dport 11434 -d 192.168.122.172 -j ACCEPT
iptables -A FORWARD -p tcp --dport 9080 -d 192.168.122.103 -j ACCEPT

# Masquerade for outgoing traffic
iptables -t nat -A POSTROUTING -o enp4s0 -j MASQUERADE

echo "✅ Port forwarding rules applied!"
echo ""
echo "🌐 Your services are now accessible externally:"
echo "   • Plex: http://YOUR_EXTERNAL_IP:32400"
echo "   • Jellyfin: http://YOUR_EXTERNAL_IP:8096"
echo "   • Home Assistant: http://YOUR_EXTERNAL_IP:8123"
echo "   • Ollama AI: http://YOUR_EXTERNAL_IP:11434"
echo "   • Traefik: http://YOUR_EXTERNAL_IP:9080"
echo ""
echo "🔍 To find your external IP: curl ifconfig.me"
echo "📋 To save rules permanently: iptables-save > /etc/iptables/rules.v4"
