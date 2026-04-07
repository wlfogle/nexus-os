#!/bin/bash
# ============================================================
# Ziggy (Raspberry Pi 3B+) Setup Script
# Raspberry Pi OS Lite 64-bit (Bookworm)
# Services: AdGuard Home (replica), WireGuard (wg-easy), Vaultwarden, Caddy
# Run as: sudo bash setup-pi.sh
# ============================================================
set -e

echo "==> Updating system..."
apt update && apt upgrade -y

echo "==> Installing dependencies..."
apt install -y curl git ca-certificates gnupg lsb-release ufw

echo "==> Installing Docker..."
curl -fsSL https://get.docker.com | sh
usermod -aG docker $SUDO_USER
systemctl enable docker
systemctl start docker

echo "==> Setting static IP (edit /etc/dhcpcd.conf if needed)..."
cat >> /etc/dhcpcd.conf <<EOF

# Static IP for homelab
interface eth0
static ip_address=192.168.12.20/24
static routers=192.168.12.1
static domain_name_servers=192.168.12.10 1.1.1.1
EOF

echo "==> Creating appdata directories..."
mkdir -p /opt/appdata/{adguardhome/{work,conf},wg-easy,vaultwarden,caddy}

echo "==> Configuring UFW firewall..."
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow 53/tcp     # AdGuard DNS
ufw allow 53/udp     # AdGuard DNS
ufw allow 80/tcp     # AdGuard Web UI / Caddy
ufw allow 443/tcp    # Caddy HTTPS
ufw allow 51820/udp  # WireGuard
ufw allow 51821/tcp  # wg-easy Web UI (LAN only)
ufw --force enable

echo "==> Deploying services..."
cd /opt/appdata

# Clone configs from repo
git clone https://github.com/wlfogle/homelab-media-stack.git /opt/homelab-media-stack

# Deploy AdGuard Home replica
cd /opt/homelab-media-stack/pi/adguardhome
docker compose up -d

# Deploy WireGuard (wg-easy)
cd /opt/homelab-media-stack/pi/wireguard
docker compose up -d

# Deploy Vaultwarden + Caddy
cd /opt/homelab-media-stack/pi/vaultwarden
docker compose up -d

echo ""
echo "=== Pi Setup Complete ==="
echo ""
echo "Services:"
echo "  AdGuard Home:  http://192.168.12.20:80"
echo "  wg-easy:       http://192.168.12.20:51821"
echo "  Vaultwarden:   https://192.168.12.20"
echo ""
echo "IMPORTANT: Set router DNS 1 = 192.168.12.10, DNS 2 = 192.168.12.20, DNS 3 = 1.1.1.1"
echo "A reboot is recommended to apply static IP."
