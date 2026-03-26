#!/usr/bin/env bash
# =============================================================================
# setup-dietpi.sh — Ziggy (DietPi) post-flash bootstrap
# Run as root on the Pi after first boot.
# =============================================================================
set -euo pipefail

if [[ "${EUID}" -ne 0 ]]; then
  echo "Run as root: sudo bash pi/setup-dietpi.sh"
  exit 1
fi

TARGET_USER="${SUDO_USER:-dietpi}"
id "${TARGET_USER}" >/dev/null 2>&1 || TARGET_USER="root"

PM="apt-get"
if command -v nala >/dev/null 2>&1; then
  PM="nala"
fi

echo "==> Updating system..."
if [[ "${PM}" == "nala" ]]; then
  nala update -y
  DEBIAN_FRONTEND=noninteractive nala upgrade -y
else
  apt-get update -y
  DEBIAN_FRONTEND=noninteractive apt-get upgrade -y
fi

echo "==> Installing base packages..."
if ! command -v nala >/dev/null 2>&1; then
  apt-get install -y nala || true
  command -v nala >/dev/null 2>&1 && PM="nala"
fi

BASE_PKGS=(
  curl wget git ca-certificates gnupg lsb-release
  ufw avahi-daemon dbus-x11 x11-xserver-utils xfonts-base
  python3 python3-pip
  openbox lxpanel xterm
  tigervnc-standalone-server tigervnc-common
  novnc websockify
)

if [[ "${PM}" == "nala" ]]; then
  DEBIAN_FRONTEND=noninteractive nala install -y "${BASE_PKGS[@]}"
else
  DEBIAN_FRONTEND=noninteractive apt-get install -y "${BASE_PKGS[@]}"
fi

echo "==> Installing Docker..."
if ! command -v docker >/dev/null 2>&1; then
  curl -fsSL https://get.docker.com | sh
fi
systemctl enable docker
systemctl start docker
id "${TARGET_USER}" >/dev/null 2>&1 && usermod -aG docker "${TARGET_USER}" || true

echo "==> Configuring static IP (idempotent)..."
if ! grep -q "Ziggy static LAN IP" /etc/dhcpcd.conf 2>/dev/null; then
  cat >> /etc/dhcpcd.conf <<'EOF'

# Ziggy static LAN IP
interface eth0
static ip_address=192.168.12.20/24
static routers=192.168.12.1
static domain_name_servers=192.168.12.10 1.1.1.1
EOF
fi

echo "==> Enabling SSH + mDNS..."
systemctl enable ssh || true
systemctl restart ssh || true
systemctl enable avahi-daemon
systemctl restart avahi-daemon || true

echo "==> Installing Warp terminal..."
install -d -m 0755 /usr/share/keyrings
curl -fsSL https://releases.warp.dev/linux/keys/warp.asc \
  | gpg --dearmor -o /usr/share/keyrings/warpdotdev.gpg
cat > /etc/apt/sources.list.d/warpdotdev.list <<'EOF'
deb [signed-by=/usr/share/keyrings/warpdotdev.gpg] https://releases.warp.dev/linux/deb stable main
EOF
if [[ "${PM}" == "nala" ]]; then
  nala update -y
  DEBIAN_FRONTEND=noninteractive nala install -y warp-terminal
else
  apt-get update -y
  DEBIAN_FRONTEND=noninteractive apt-get install -y warp-terminal
fi

echo "==> Installing Opera..."
wget -qO- https://deb.opera.com/archive.key | gpg --dearmor > /usr/share/keyrings/opera-browser.gpg
cat > /etc/apt/sources.list.d/opera-stable.list <<'EOF'
deb [signed-by=/usr/share/keyrings/opera-browser.gpg] https://deb.opera.com/opera-stable/ stable non-free
EOF
if [[ "${PM}" == "nala" ]]; then
  nala update -y
  DEBIAN_FRONTEND=noninteractive nala install -y opera-stable
else
  apt-get update -y
  DEBIAN_FRONTEND=noninteractive apt-get install -y opera-stable
fi

echo "==> Creating appdata directories..."
mkdir -p /opt/appdata/{adguardhome/{work,conf},wg-easy,vaultwarden,caddy}

echo "==> Applying firewall rules..."
ufw --force reset
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow 53/tcp
ufw allow 53/udp
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 51820/udp
ufw allow 51821/tcp
ufw --force enable

echo "==> Cloning repo..."
if [[ ! -d /opt/homelab-media-stack/.git ]]; then
  git clone https://github.com/wlfogle/homelab-media-stack.git /opt/homelab-media-stack
else
  git -C /opt/homelab-media-stack pull --ff-only
fi

echo ""
echo "DietPi bootstrap complete."
echo "Next:"
echo "  1) reboot"
echo "  2) run: sudo bash /opt/homelab-media-stack/pi/setup-vnc.sh"
echo "  3) run: sudo bash /opt/homelab-media-stack/pi/setup-pi.sh"
