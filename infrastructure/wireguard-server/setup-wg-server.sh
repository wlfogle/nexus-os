#!/bin/sh
# ============================================================
# WireGuard Server Setup — CT-100 (Alpine Linux)
# Self-hosted VPN server for protecting qBittorrent traffic
# Run inside Proxmox LXC container 100
# ============================================================
set -e

echo "==> Installing WireGuard..."
apk update && apk add wireguard-tools iptables ip6tables

echo "==> Enabling IP forwarding..."
echo "net.ipv4.ip_forward=1" >> /etc/sysctl.conf
sysctl -p

echo "==> Generating server keys..."
mkdir -p /etc/wireguard
wg genkey | tee /etc/wireguard/server_private.key | wg pubkey > /etc/wireguard/server_public.key
chmod 600 /etc/wireguard/server_private.key

SERVER_PRIVATE=$(cat /etc/wireguard/server_private.key)
SERVER_PUBLIC=$(cat /etc/wireguard/server_public.key)

echo "==> Generating client keys (for CT-101 WG proxy)..."
wg genkey | tee /etc/wireguard/client_private.key | wg pubkey > /etc/wireguard/client_public.key
chmod 600 /etc/wireguard/client_private.key

CLIENT_PRIVATE=$(cat /etc/wireguard/client_private.key)
CLIENT_PUBLIC=$(cat /etc/wireguard/client_public.key)

# Detect network interface
IFACE=$(ip route | grep default | awk '{print $5}' | head -1)

echo "==> Writing server config /etc/wireguard/wg0.conf..."
cat > /etc/wireguard/wg0.conf <<EOF
[Interface]
PrivateKey = ${SERVER_PRIVATE}
Address = 10.0.0.1/24
ListenPort = 51820
SaveConfig = false
PostUp = iptables -A FORWARD -i wg0 -j ACCEPT; iptables -t nat -A POSTROUTING -o ${IFACE} -j MASQUERADE
PostDown = iptables -D FORWARD -i wg0 -j ACCEPT; iptables -t nat -D POSTROUTING -o ${IFACE} -j MASQUERADE

# CT-101 WireGuard client + TinyProxy
[Peer]
PublicKey = ${CLIENT_PUBLIC}
AllowedIPs = 10.0.0.2/32
PersistentKeepalive = 25
EOF

chmod 600 /etc/wireguard/wg0.conf

echo "==> Writing client config for CT-101..."
mkdir -p /etc/wireguard/clients
cat > /etc/wireguard/clients/ct101-wg-proxy.conf <<EOF
[Interface]
PrivateKey = ${CLIENT_PRIVATE}
Address = 10.0.0.2/32
DNS = 192.168.12.242

[Peer]
PublicKey = ${SERVER_PUBLIC}
Endpoint = 192.168.12.100:51820
AllowedIPs = 0.0.0.0/0
PersistentKeepalive = 25
EOF

echo "==> Starting WireGuard..."
wg-quick up wg0
rc-update add wg-quick.wg0 default

echo ""
echo "=== WireGuard Server Setup Complete ==="
echo "Server Public Key: ${SERVER_PUBLIC}"
echo "Client config saved to: /etc/wireguard/clients/ct101-wg-proxy.conf"
echo ""
echo "Next: Copy ct101-wg-proxy.conf to CT-101 and run setup-gluetun-client.sh"
