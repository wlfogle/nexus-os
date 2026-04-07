#!/usr/bin/env bash
# Standalone Seerr install script for CT-242 (no build framework needed)
# Run INSIDE CT-242 via: pct exec 242 -- bash -s < this_script.sh
set -euo pipefail

echo "==> Updating OS..."
apt-get update -qq && apt-get upgrade -y -qq

echo "==> Installing dependencies..."
apt-get install -y -qq build-essential python3-setuptools curl git jq

echo "==> Installing Node.js 22..."
curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
apt-get install -y nodejs
node --version

echo "==> Getting latest Seerr release..."
RELEASE_URL=$(curl -fsSL https://api.github.com/repos/seerr-team/seerr/releases/latest | jq -r '.tarball_url')
RELEASE_TAG=$(curl -fsSL https://api.github.com/repos/seerr-team/seerr/releases/latest | jq -r '.tag_name')
echo "   Installing Seerr $RELEASE_TAG"

mkdir -p /opt/seerr
curl -fsSL "$RELEASE_URL" | tar xz -C /opt/seerr --strip-components=1

echo "==> Installing pnpm..."
PNPM_VERSION=$(grep -Po '"pnpm":\s*"\K[^"]+' /opt/seerr/package.json)
npm install -g "pnpm@$PNPM_VERSION"
pnpm --version

echo "==> Building Seerr (this takes several minutes)..."
cd /opt/seerr
export CYPRESS_INSTALL_BINARY=0
pnpm install --frozen-lockfile
export NODE_OPTIONS="--max-old-space-size=3072"
pnpm build

echo "==> Creating config..."
mkdir -p /etc/seerr
cat > /etc/seerr/seerr.conf << 'EOF'
PORT=5055
HOST=0.0.0.0
EOF

echo "==> Creating systemd service..."
cat > /etc/systemd/system/seerr.service << 'EOF'
[Unit]
Description=Seerr Service
Wants=network-online.target
After=network-online.target

[Service]
EnvironmentFile=/etc/seerr/seerr.conf
Environment=NODE_ENV=production
Type=exec
Restart=on-failure
WorkingDirectory=/opt/seerr
ExecStart=/usr/bin/node dist/index.js

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable --now seerr
sleep 5
systemctl is-active seerr && echo "Seerr is running!" || echo "Check: systemctl status seerr"

echo ""
echo "========================================="
echo "  Seerr $RELEASE_TAG installed on CT-242"
echo "  Access: http://192.168.12.151:5055"
echo "  Connect to Jellyfin: 192.168.12.231:8096"
echo "  Radarr: 192.168.12.225:7878"
echo "  Sonarr: 192.168.12.214:8989"
echo "========================================="
