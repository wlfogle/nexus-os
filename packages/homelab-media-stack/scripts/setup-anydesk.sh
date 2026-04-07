#!/usr/bin/env bash
# =============================================================================
# setup-anydesk.sh — Install AnyDesk on Tiamat (Proxmox VE / Debian x86_64)
# Installs XFCE4 desktop + AnyDesk with unattended access enabled
# Run as root directly on the Proxmox host:
#   bash /opt/homelab-media-stack/scripts/setup-anydesk.sh
# =============================================================================
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
die()   { echo -e "${RED}[ERR]${NC}   $*"; exit 1; }

[[ $EUID -ne 0 ]] && die "Run as root (sudo or on Proxmox host directly)"

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║        AnyDesk Setup — Tiamat (Proxmox x86_64)       ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""

# ── 1. System update ──────────────────────────────────────────────────────────
info "Updating package lists..."
apt-get update -qq

# ── 2. Install XFCE4 (lightweight desktop) ───────────────────────────────────
info "Installing XFCE4 desktop environment..."
DEBIAN_FRONTEND=noninteractive apt-get install -y \
    xfce4 \
    xfce4-terminal \
    xfce4-screensaver \
    lightdm \
    lightdm-gtk-greeter \
    dbus-x11 \
    xauth \
    xvfb \
    --no-install-recommends -qq
ok "XFCE4 installed"

# ── 3. Configure lightdm for auto-login ──────────────────────────────────────
info "Configuring display manager..."
cat > /etc/lightdm/lightdm.conf <<'EOF'
[Seat:*]
autologin-user=root
autologin-user-timeout=0
user-session=xfce
EOF
systemctl enable lightdm 2>/dev/null || true
ok "lightdm configured"

# ── 4. Add AnyDesk apt repository ────────────────────────────────────────────
info "Adding AnyDesk repository..."
install -m 0755 -d /usr/share/keyrings
wget -qO - https://keys.anydesk.com/repos/DEB-GPG-KEY \
    | gpg --dearmor \
    | tee /usr/share/keyrings/anydesk-keyring.gpg > /dev/null
echo "deb [signed-by=/usr/share/keyrings/anydesk-keyring.gpg] http://deb.anydesk.com/ all main" \
    > /etc/apt/sources.list.d/anydesk-stable.list
apt-get update -qq
ok "AnyDesk repo added"

# ── 5. Install AnyDesk ───────────────────────────────────────────────────────
info "Installing AnyDesk..."
DEBIAN_FRONTEND=noninteractive apt-get install -y anydesk -qq
ok "AnyDesk installed"

# ── 6. Enable and start AnyDesk service ──────────────────────────────────────
info "Enabling AnyDesk service..."
systemctl enable anydesk
systemctl start anydesk
sleep 2
ok "AnyDesk service running"

# ── 7. Set unattended access password ────────────────────────────────────────
echo ""
echo -e "${YELLOW}Set an unattended access password for AnyDesk.${NC}"
echo    "This password is used when connecting remotely without user interaction."
echo    "Choose something strong — this is your front door."
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
ANYDESK_ID=$(anydesk --get-id 2>/dev/null || anydesk --info 2>/dev/null | grep -oP 'AnyDesk-ID:\s*\K[0-9 ]+' || echo "Run: anydesk --get-id")
echo -e "${GREEN}  ✓  Tiamat AnyDesk ID: ${CYAN}${ANYDESK_ID}${NC}"
echo "════════════════════════════════════════════════════════"
echo ""
echo "  Save this ID — you'll use it to connect from any device."
echo ""
echo "  Next steps:"
echo "  1. Download AnyDesk on your laptop / phone / Fire TV"
echo "  2. Enter this ID and use the password you just set"
echo "  3. See docs/ANYDESK.md for full instructions"
echo ""

# ── 9. Note about Proxmox ────────────────────────────────────────────────────
warn "Note: XFCE runs alongside Proxmox. The Proxmox web UI (:8006) is unaffected."
warn "To open the Proxmox manager from the XFCE desktop, use the Thunar file manager"
warn "or open Firefox and go to https://localhost:8006"
echo ""
ok "Setup complete. AnyDesk is running and ready for remote connections."
