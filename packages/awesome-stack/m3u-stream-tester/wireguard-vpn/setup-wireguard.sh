#!/bin/bash

# Install WireGuard
sudo apt update && sudo apt install -y wireguard

# Generate server keys
wg genkey | tee server_private_key | wg pubkey > server_public_key
wg genkey | tee client_private_key | wg pubkey > client_public_key

# Prepare server configuration (wg0.conf)
cat << EOF | sudo tee /etc/wireguard/wg0.conf
[Interface]
PrivateKey = $(cat server_private_key)
Address = 10.200.200.1/24
ListenPort = 51820

# Enable IP forwarding
PostUp = iptables -A FORWARD -i wg0 -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE
PostDown = iptables -D FORWARD -i wg0 -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE

[Peer]
# Client
PublicKey = $(cat client_public_key)
AllowedIPs = 10.200.200.2/32
EOF

# Configure client configuration
cat << EOF > client.conf
[Interface]
PrivateKey = $(cat client_private_key)
Address = 10.200.200.2/24

[Peer]
PublicKey = $(cat server_public_key)
Endpoint = YOUR_SERVER_IP:51820
AllowedIPs = 0.0.0.0/0, ::/0
EOF

# Start WireGuard
sudo systemctl start wg-quick@wg0
sudo systemctl enable wg-quick@wg0

