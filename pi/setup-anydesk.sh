#!/usr/bin/env bash
# =============================================================================
# setup-anydesk.sh — Install AnyDesk on Ziggy (Raspberry Pi 3B+, ARM64)
# Installs XFCE4 desktop + AnyDesk ARM64 with unattended access enabled
# Run as root on Ziggy:
#   bash /opt/homelab-media-stack/pi/setup-anydesk.sh
# =============================================================================
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
die()   { echo -e "${RED}[ERR]${NC}   $*"; exit 1; }

[[ $EUID -ne 0 ]] && die "Run as root: sudo bash pi/setup-anydesk.sh"

# Detect arch — Pi 3B+ 64-bit OS is aarch64
ARCH=$(uname -m)
[[ "$ARCH" != "aarch64" ]] && die "Expected aarch64 (ARM64), got $ARCH. Is Pi OS Lite 64-bit installed?"

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║        AnyDesk Setup — Ziggy (Raspberry Pi ARM64)    ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""

# ── 1. System update ──────────────────────────────────────────────────────────
info "Updating package lists..."
apt-get update -qq

# ── 2. Install XFCE4 (lightweight — Pi 3B+ has limited RAM) ─────────────────
info "Installing XFCE4 desktop environment (lightweight)..."
DEBIAN_FRONTEND=noninteractive apt-get install -y \
    xfce4 \
    xfce4-terminal \
    lightdm \
    lightdm-gtk-greeter \
    dbus-x11 \
    xauth \
    xvfb \
    --no-install-recommends -qq
ok "XFCE4 installed"

# ── 3. Configure lightdm for auto-login as pi ────────────────────────────────
info "Configuring display manager for auto-login..."
cat > /etc/lightdm/lightdm.conf <<'EOF'
[Seat:*]
autologin-user=pi
autologin-user-timeout=0
user-session=xfce
EOF
systemctl enable lightdm 2>/dev/null || true
ok "lightdm configured (auto-login as pi)"

# ── 4. Download AnyDesk ARM64 ─────────────────────────────────────────────────
# AnyDesk does not have an apt repo for ARM — direct .deb download
ANYDESK_VERSION="6.3.2"
ANYDESK_DEB="anydesk_${ANYDESK_VERSION}-1_arm64.deb"
ANYDESK_URL="https://download.anydesk.com/rpi/${ANYDESK_DEB}"

info "Downloading AnyDesk ${ANYDESK_VERSION} ARM64..."
TMP_DIR=$(mktemp -d)
wget -q --show-progress -O "${TMP_DIR}/${ANYDESK_DEB}" "${ANYDESK_URL}" || {
    # Fallback: try armhf build
    warn "arm64 download failed, trying armhf fallback..."
    ANYDESK_DEB="anydesk_${ANYDESK_VERSION}-1_armhf.deb"
    ANYDESK_URL="https://download.anydesk.com/rpi/${ANYDESK_DEB}"
    wget -q --show-progress -O "${TMP_DIR}/${ANYDESK_DEB}" "${ANYDESK_URL}"
}
ok "AnyDesk downloaded"

# ── 5. Install AnyDesk deb ────────────────────────────────────────────────────
info "Installing AnyDesk..."
dpkg -i "${TMP_DIR}/${ANYDESK_DEB}" 2>/dev/null || true
apt-get install -f -y -qq   # Fix any missing dependencies
rm -rf "$TMP_DIR"
ok "AnyDesk installed"

# ── 6. Enable and start AnyDesk service ──────────────────────────────────────
info "Enabling AnyDesk service..."
systemctl enable anydesk
systemctl start anydesk
sleep 2
ok "AnyDesk service running"

# ── 7. Set unattended access password ────────────────────────────────────────
echo ""
echo -e "${YELLOW}Set an unattended access password for AnyDesk on Ziggy.${NC}"
echo    "Use a different password than Tiamat's."
echo ""
while true; do
    read -rsp "Enter AnyDesk unattended password: " AD_PASS; echo ""
    read -rsp "Confirm password: " AD_PASS2; echo ""
    [[ "$AD_PASS" == "$AD_PASS2" ]] && break
    warn "Passwords do not match. Try again."
done

echo "$AD_PASS" | anydesk --set-password
ok "Unattended password set"

# ── 8. Print AnyDesk ID ──────────────────────────────────────────────────────
echo ""
echo "════════════════════════════════════════════════════════"
ANYDESK_ID=$(anydesk --get-id 2>/dev/null || echo "Run: anydesk --get-id")
echo -e "${GREEN}  ✓  Ziggy AnyDesk ID: ${CYAN}${ANYDESK_ID}${NC}"
echo "════════════════════════════════════════════════════════"
echo ""
echo "  Save this ID — you'll use it to connect remotely."
echo "  See docs/ANYDESK.md for instructions on all devices."
echo ""

ok "Setup complete. AnyDesk is running on Ziggy."
