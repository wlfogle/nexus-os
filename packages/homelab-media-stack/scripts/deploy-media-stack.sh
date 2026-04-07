#!/bin/bash
# ============================================================
# Deploy Full Media Stack to Proxmox LXC
# Creates containers/VMs for:
#   CT-100  WireGuard Server (Alpine)
#   CT-101  Gluetun/TinyProxy VPN proxy (Alpine, privileged)
#   CT-102  AdGuard Home DNS (Debian)
#   CT-110  Media Stack — Jellyfin/Plex/Sonarr/Radarr/etc (Debian)
#   CT-150  Fire TV ADB Controller (Ubuntu)
#   VM-200  Windows 10 for PlayOn Desktop (manual setup — see proxmox/vm-windows-playon.md)
# Run on Proxmox host as root after setup-proxmox.sh
# ============================================================
set -e

# ── Collect secrets up front ─────────────────────────────────────────────────
echo ""
echo "========================================================="
echo "  Tiamat Media Stack — Pre-flight Configuration"
echo "========================================================="
echo ""

# Plex claim token (get from https://plex.tv/claim — expires in 4 min)
if [ -z "$PLEX_CLAIM" ]; then
  echo "Get your Plex claim token now: https://plex.tv/claim"
  read -rp "PLEX_CLAIM token: " PLEX_CLAIM
fi

# DuckDNS token
if [ -z "$DUCKDNS_TOKEN" ]; then
  echo ""
  echo "Get your DuckDNS token from: https://www.duckdns.org"
  read -rp "DuckDNS token: " DUCKDNS_TOKEN
fi

# Timezone
if [ -z "$TZ" ]; then
  read -rp "Timezone [America/New_York]: " TZ
  TZ=${TZ:-America/New_York}
fi

# Email (for Let's Encrypt / alerts)
if [ -z "$ACME_EMAIL" ]; then
  read -rp "Email address (for alerts/TLS): " ACME_EMAIL
fi

echo ""
echo "Config collected. Starting deployment..."
echo ""

DEBIAN_TEMPLATE=$(pveam list local | grep debian-12 | awk '{print $1}' | head -1)
if [ -z "$DEBIAN_TEMPLATE" ]; then
  echo "==> Downloading Debian 12 template..."
  pveam update
  pveam download local debian-12-standard_12.7-1_amd64.tar.zst
  DEBIAN_TEMPLATE=$(pveam list local | grep debian-12 | awk '{print $1}' | head -1)
fi

echo "==> Using template: $DEBIAN_TEMPLATE"

# ── CT-100: WireGuard Server ─────────────────────────────
echo "==> Creating CT-100 (WireGuard Server)..."
pct create 100 local:vztmpl/alpine-3.19-default_20240207_amd64.tar.xz \
  --hostname wg-server \
  --memory 256 --cores 1 \
  --net0 name=eth0,bridge=vmbr0,ip=192.168.12.100/24,gw=192.168.12.1 \
  --storage local-lvm --rootfs local-lvm:4 \
  --unprivileged 0 --features nesting=1 \
  --onboot 1

# Add TUN device access
cat >> /etc/pve/lxc/100.conf <<EOF
lxc.cgroup.devices.allow: c 10:200 rwm
lxc.mount.entry: /dev/net dev/net none bind,create=dir
EOF

# ── CT-101: Gluetun Proxy ───────────────────────────────
echo "==> Creating CT-101 (Gluetun/TinyProxy)..."
pct create 101 local:vztmpl/alpine-3.19-default_20240207_amd64.tar.xz \
  --hostname gluetun-proxy \
  --memory 256 --cores 1 \
  --net0 name=eth0,bridge=vmbr0,ip=192.168.12.101/24,gw=192.168.12.1 \
  --storage local-lvm --rootfs local-lvm:4 \
  --unprivileged 0 --features nesting=1,keyctl=1 \
  --onboot 1

cat >> /etc/pve/lxc/101.conf <<EOF
lxc.cgroup.devices.allow: c 10:200 rwm
lxc.mount.entry: /dev/net dev/net none bind,create=dir
lxc.apparmor.profile: unconfined
lxc.mount.auto: proc:rw sys:rw
EOF

# ── CT-102: AdGuard Home ────────────────────────────────
echo "==> Creating CT-102 (AdGuard Home)..."
pct create 102 $DEBIAN_TEMPLATE \
  --hostname adguardhome \
  --memory 512 --cores 1 \
  --net0 name=eth0,bridge=vmbr0,ip=192.168.12.102/24,gw=192.168.12.1 \
  --storage local-lvm --rootfs local-lvm:8 \
  --unprivileged 1 --features nesting=1 \
  --onboot 1

# ── CT-110: Media Stack ─────────────────────────────────
echo "==> Creating CT-110 (Media Stack)..."
pct create 110 $DEBIAN_TEMPLATE \
  --hostname media-stack \
  --memory 4096 --cores 4 \
  --net0 name=eth0,bridge=vmbr0,ip=192.168.12.110/24,gw=192.168.12.1 \
  --storage local-lvm --rootfs local-lvm:32 \
  --unprivileged 1 --features nesting=1 \
  --onboot 1

# Bind mount media and downloads from host
cat >> /etc/pve/lxc/110.conf <<EOF
mp0: /mnt/media,mp=/mnt/media
mp1: /mnt/downloads,mp=/mnt/downloads
mp2: /opt/appdata,mp=/opt/appdata
lxc.cgroup.devices.allow: c 226:0 rwm
lxc.cgroup.devices.allow: c 226:128 rwm
lxc.mount.entry: /dev/dri dev/dri none bind,optional,create=dir
EOF

# ── CT-150: Fire TV ADB Controller ─────────────────────────
UBUNTU_TEMPLATE=$(pveam list local | grep ubuntu-22 | awk '{print $1}' | head -1)
if [ -z "$UBUNTU_TEMPLATE" ]; then
  echo "==> Downloading Ubuntu 22.04 template..."
  pveam update
  pveam download local ubuntu-22.04-standard_22.04-1_amd64.tar.zst
  UBUNTU_TEMPLATE=$(pveam list local | grep ubuntu-22 | awk '{print $1}' | head -1)
fi

echo "==> Creating CT-150 (Fire TV ADB Controller)..."
pct create 150 "$UBUNTU_TEMPLATE" \
  --hostname firetv-controller \
  --memory 1024 --cores 1 \
  --net0 name=eth0,bridge=vmbr0,ip=192.168.12.150/24,gw=192.168.12.1 \
  --storage local-lvm --rootfs local-lvm:8 \
  --unprivileged 1 --features nesting=1 \
  --onboot 1

echo "==> Starting containers..."
pct start 100
pct start 101
pct start 102
pct start 110
pct start 150

sleep 5

echo "==> Installing Docker in CT-102 (AdGuard Home)..."
pct exec 102 -- bash -c "apt update && apt install -y curl && curl -fsSL https://get.docker.com | sh"

echo "==> Installing Docker in CT-110 (Media Stack)..."
pct exec 110 -- bash -c "apt update && apt install -y curl git && curl -fsSL https://get.docker.com | sh"

echo "==> Cloning repo into CT-110..."
pct exec 110 -- bash -c "git clone https://github.com/wlfogle/homelab-media-stack.git /opt/homelab-media-stack"

echo "==> Writing .env with secrets into CT-110..."
pct exec 110 -- bash -c "cp /opt/homelab-media-stack/media-stack/.env.example /opt/homelab-media-stack/media-stack/.env"

# Inject all collected secrets + config into the .env
pct exec 110 -- bash -c "sed -i \
  -e 's|^PLEX_CLAIM=.*|PLEX_CLAIM=${PLEX_CLAIM}|' \
  -e 's|^DUCKDNS_TOKEN=.*|DUCKDNS_TOKEN=${DUCKDNS_TOKEN}|' \
  -e 's|^TZ=.*|TZ=${TZ}|' \
  -e 's|^ACME_EMAIL=.*|ACME_EMAIL=${ACME_EMAIL}|' \
  /opt/homelab-media-stack/media-stack/.env"

echo "==> Deploying media stack in CT-110..."
pct exec 110 -- bash -c "cd /opt/homelab-media-stack/media-stack && docker compose up -d"

echo "==> Setting up DuckDNS dynamic DNS..."
pct exec 110 -- bash -c "
  DUCKDNS_TOKEN='${DUCKDNS_TOKEN}' \
  bash /opt/homelab-media-stack/scripts/setup-duckdns.sh
"

echo "==> Setting up CT-150 (Fire TV ADB Controller)..."
pct exec 150 -- bash -c "
  apt update -qq && apt install -y python3 python3-pip adb curl git
  pip3 install flask flask-cors
  mkdir -p /opt/firetv-controller
  git clone https://github.com/wlfogle/homelab-media-stack.git /tmp/hms
  cp -r /tmp/hms/infrastructure/firetv-controller/* /opt/firetv-controller/
  cat > /etc/systemd/system/firetv-controller.service <<'SVCEOF'
[Unit]
Description=Fire TV ADB Controller
After=network.target
[Service]
WorkingDirectory=/opt/firetv-controller
ExecStart=/usr/bin/python3 /opt/firetv-controller/api.py
Restart=always
RestartSec=5
[Install]
WantedBy=multi-user.target
SVCEOF
  systemctl daemon-reload
  systemctl enable --now firetv-controller
"

echo ""
echo "========================================================="
echo "  Tiamat Media Stack — Deployment Complete"
echo "========================================================="
echo ""
echo "CONTAINERS:"
echo "  CT-100  WireGuard VPN Server   192.168.12.100"
echo "  CT-101  Gluetun/TinyProxy      192.168.12.101  (qBit VPN proxy :8888)"
echo "  CT-102  AdGuard Home DNS       192.168.12.102"
echo "  CT-110  Media Stack            192.168.12.110"
echo "  CT-150  Fire TV ADB Ctrl       192.168.12.150  (Flask API :5000)"
echo ""
echo "NOTE — VM-200 (Windows 10 / PlayOn Desktop):"
echo "  Create manually via Proxmox web UI — see proxmox/vm-windows-playon.md"
echo "  Target IP: 192.168.12.200, 4 cores, 4GB RAM, 60GB disk"
echo ""
echo "MEDIA SERVICES  (all at 192.168.12.110):"
echo "  Homarr Dashboard  http://192.168.12.110:7575"
echo "  Jellyfin          http://192.168.12.110:8096"
echo "  Plex              http://192.168.12.110:32400/web"
echo "  Overseerr         http://192.168.12.110:5055"
echo "  Sonarr            http://192.168.12.110:8989"
echo "  Radarr            http://192.168.12.110:7878"
echo "  Prowlarr          http://192.168.12.110:9696"
echo "  qBittorrent       http://192.168.12.110:9090"
echo "  Bazarr            http://192.168.12.110:6767"
echo "  Tautulli          http://192.168.12.110:8181"
echo "  AdGuard Home      http://192.168.12.102:3000"
echo ""
echo "NEXT STEPS:"
echo "  1. Run infrastructure/wireguard-server/setup-wg-server.sh in CT-100"
echo "  2. See docs/INDEXERS.md to configure Sonarr/Radarr indexers"
echo "  3. Sideload android-app/ to Fire TVs: ./android-app/build-app.sh install-firetv"
echo "  4. Set up Ziggy (Pi 3B+) — see pi/imager-config/README.md"
echo ""
echo "All secrets already injected into CT-110 .env:"  
echo "  PLEX_CLAIM, DUCKDNS_TOKEN, TZ, ACME_EMAIL"
