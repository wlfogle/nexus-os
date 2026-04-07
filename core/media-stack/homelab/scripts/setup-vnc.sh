#!/usr/bin/env bash
# =============================================================================
# setup-vnc.sh — Tiamat (Proxmox host) VNC + noVNC Desktop Setup
# Installs LXDE + TigerVNC 1.15+ + noVNC web interface
# Desktop: LXDE (lightweight, matches Bahamut)
# TigerVNC 1.15+: config in ~/.config/tigervnc, built-in systemd service
# View from: native VNC client, any browser, Android, Fire TV (Silk browser)
# =============================================================================
set -euo pipefail

VNC_DISPLAY=":1"         # display :1 = port 5901
VNC_PORT="5901"
NOVNC_PORT="6080"
VNC_RESOLUTION="1920x1080"
VNC_DEPTH="24"
VNC_CFG="/root/.config/tigervnc"

# ── Prompt for VNC password ──────────────────────────────────────────────────
echo ""
echo "╔══════════════════════════════════════════╗"
echo "║   Tiamat VNC Desktop Setup               ║"
echo "╚══════════════════════════════════════════╝"
echo ""
read -rsp "Enter VNC password (min 6 chars): " VNC_PASS
echo ""
read -rsp "Confirm VNC password: " VNC_PASS2
echo ""
if [[ "$VNC_PASS" != "$VNC_PASS2" ]]; then
    echo "ERROR: Passwords do not match." >&2
    exit 1
fi
if [[ ${#VNC_PASS} -lt 6 ]]; then
    echo "ERROR: Password must be at least 6 characters." >&2
    exit 1
fi

# ── Install dependencies ─────────────────────────────────────────────────────
echo "[1/6] Installing LXDE + TigerVNC + noVNC + Warp + Opera..."
apt-get update -y
apt-get install -y \
    lxde \
    xterm \
    tigervnc-standalone-server \
    tigervnc-common \
    dbus-x11 \
    xfonts-base \
    x11-xserver-utils \
    novnc \
    websockify \
    python3 \
    curl wget ca-certificates gnupg \
    --no-install-recommends

# Warp terminal
echo "  Installing Warp terminal..."
curl -fsSL https://releases.warp.dev/linux/keys/warp.asc \
    | gpg --dearmor -o /usr/share/keyrings/warpdotdev.gpg
echo "deb [signed-by=/usr/share/keyrings/warpdotdev.gpg] https://releases.warp.dev/linux/deb stable main" \
    > /etc/apt/sources.list.d/warpdotdev.list
apt-get update -qq && apt-get install -y warp-terminal

# Opera browser
echo "  Installing Opera browser..."
wget -qO- https://deb.opera.com/archive.key \
    | gpg --dearmor > /usr/share/keyrings/opera-browser.gpg
echo "deb [signed-by=/usr/share/keyrings/opera-browser.gpg] https://deb.opera.com/opera-stable/ stable non-free" \
    > /etc/apt/sources.list.d/opera-stable.list
apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y opera-stable

# ── Set VNC password (TigerVNC 1.15 — config in ~/.config/tigervnc) ──────────
echo "[2/6] Setting VNC password..."
# Remove legacy ~/.vnc to avoid migration errors in TigerVNC 1.15
rm -rf /root/.vnc
mkdir -p "${VNC_CFG}"
# Generate VNC passwd file: DES-encrypt 8 null bytes with bit-reversed password key
python3 - "$VNC_PASS" <<'PYEOF'
import sys, subprocess, os
p = (sys.argv[1].encode() + b'\x00'*8)[:8]
key = bytes(int('{:08b}'.format(b)[::-1],2) for b in p)
# OpenSSL 3.x requires -provider legacy for DES
out = subprocess.run(
    ['openssl','enc','-des-ecb','-provider','legacy','-provider','default',
     '-nosalt','-nopad','-K',key.hex()],
    input=b'\x00'*8, capture_output=True, check=True).stdout[:8]
path = '/root/.config/tigervnc/passwd'
open(path,'wb').write(out)
os.chmod(path, 0o600)
PYEOF
unset VNC_PASS VNC_PASS2

# ── Write xstartup ───────────────────────────────────────────────────────────
echo "[3/6] Writing VNC xstartup for LXDE..."
cat > "${VNC_CFG}/xstartup" << 'EOF'
#!/bin/bash
xrdb $HOME/.Xresources 2>/dev/null || true
# Start LXDE desktop environment (matches Bahamut)
exec startlxde
EOF
chmod +x "${VNC_CFG}/xstartup"

# ── Write VNC config ─────────────────────────────────────────────────────────
echo "[4/6] Writing VNC server config..."
cat > "${VNC_CFG}/config" << EOF
geometry=${VNC_RESOLUTION}
depth=${VNC_DEPTH}
localhost=no
alwaysshared
EOF

# ── Configure vncserver.users and enable built-in service ────────────────────
echo "[5/6] Configuring TigerVNC systemd service..."
# Map display :1 to root
mkdir -p /etc/tigervnc
echo ":1=root" > /etc/tigervnc/vncserver.users

systemctl daemon-reload
systemctl enable tigervncserver@:1.service
systemctl restart tigervncserver@:1.service

# ── noVNC websockify service ─────────────────────────────────────────────────
echo "[6/6] Setting up noVNC web interface on port ${NOVNC_PORT}..."

NOVNC_DIR=$(find /usr/share -maxdepth 1 \( -name 'novnc' -o -name 'noVNC' \) 2>/dev/null | head -1)
[[ -z "$NOVNC_DIR" ]] && NOVNC_DIR="/usr/share/novnc"

cat > /etc/systemd/system/novnc.service << EOF
[Unit]
Description=noVNC Web Interface for Tiamat Desktop
After=network.target tigervncserver@:1.service
Requires=tigervncserver@:1.service

[Service]
Type=simple
ExecStart=/usr/bin/websockify --web=${NOVNC_DIR} ${NOVNC_PORT} localhost:${VNC_PORT}
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable novnc.service
systemctl restart novnc.service

# ── Open firewall ports ───────────────────────────────────────────────────────
apt-get install -y iptables-persistent
for PORT in "${VNC_PORT}" "${NOVNC_PORT}"; do
    iptables -C INPUT -p tcp --dport "${PORT}" -j ACCEPT 2>/dev/null || \
        iptables -I INPUT -p tcp --dport "${PORT}" -j ACCEPT
done
netfilter-persistent save

# ── Done ─────────────────────────────────────────────────────────────────────
TIAMAT_IP=$(hostname -I | awk '{print $1}')
echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║   VNC + noVNC is LIVE on Tiamat                                  ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
printf "║   VNC (native):  %-46s ║\n" "${TIAMAT_IP}:${VNC_PORT}"
printf "║   noVNC (web):   %-46s ║\n" "http://${TIAMAT_IP}:${NOVNC_PORT}/vnc.html"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo "║   Connect from anywhere:                                         ║"
printf "║   • Laptop:    vncviewer %-40s ║\n" "${TIAMAT_IP}:${VNC_PORT}"
printf "║   • Android:   VNC Viewer app → %-33s ║\n" "${TIAMAT_IP}:${VNC_PORT}"
printf "║   • Fire TV:   Silk browser → http://%-28s ║\n" "${TIAMAT_IP}:${NOVNC_PORT}/vnc.html"
echo "║   • Any browser: same noVNC URL above                            ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
