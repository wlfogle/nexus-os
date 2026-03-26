#!/bin/sh
# ============================================================
# WireGuard Client + TinyProxy Setup — CT-101 (Alpine Linux, PRIVILEGED)
# Connects to WireGuard server in CT-100
# Runs TinyProxy so media services route through VPN
# Run inside Proxmox LXC container 101
# ============================================================
set -e

echo "==> Installing WireGuard + TinyProxy..."
apk update && apk add wireguard-tools tinyproxy iptables

echo "==> Copying client WireGuard config..."
echo "    Paste the contents of /etc/wireguard/clients/ct101-wg-proxy.conf from CT-100:"
echo "    (copy it via: pct exec 100 -- cat /etc/wireguard/clients/ct101-wg-proxy.conf)"
echo ""
echo "    Then save it to /etc/wireguard/wg0.conf on this container and re-run this script."
echo ""

if [ ! -f /etc/wireguard/wg0.conf ]; then
  echo "ERROR: /etc/wireguard/wg0.conf not found. Copy from CT-100 first."
  exit 1
fi

chmod 600 /etc/wireguard/wg0.conf

echo "==> Starting WireGuard client tunnel..."
wg-quick up wg0
rc-update add wg-quick.wg0 default

echo "==> Verifying VPN connection..."
sleep 3
curl -s ifconfig.me && echo " (this should NOT be your home IP)"

echo "==> Configuring TinyProxy..."
cat > /etc/tinyproxy/tinyproxy.conf <<EOF
User tinyproxy
Group tinyproxy
Port 8888
Timeout 600
DefaultErrorFile "/usr/share/tinyproxy/default.html"
StatFile "/usr/share/tinyproxy/stats.html"
LogFile "/var/log/tinyproxy/tinyproxy.log"
LogLevel Info
MaxClients 100
MinSpareServers 5
MaxSpareServers 20
StartServers 10
MaxRequestsPerChild 0
# Allow all LAN clients
Allow 192.168.12.0/24
Allow 10.0.0.0/24
Allow 127.0.0.1
ViaProxyName "tinyproxy"
EOF

rc-service tinyproxy start
rc-update add tinyproxy default

echo ""
echo "=== CT-101 WireGuard Client + TinyProxy Setup Complete ==="
echo ""
echo "HTTP Proxy available at: 192.168.12.101:8888"
echo ""
echo "Configure media services to use this proxy:"
echo "  qBittorrent: Settings → Connection → Proxy Type: HTTP"
echo "              Host: 192.168.12.101  Port: 8888"
echo "  Prowlarr:    Settings → General → Proxy → HTTP"
echo "              Host: 192.168.12.101  Port: 8888"
echo ""
echo "Test proxy: curl -x http://192.168.12.101:8888 ifconfig.me"
