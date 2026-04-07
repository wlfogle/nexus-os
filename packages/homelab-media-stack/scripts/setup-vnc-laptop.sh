#!/usr/bin/env bash
# =============================================================================
# setup-vnc-laptop.sh — Pop!_OS Laptop VNC + noVNC Setup
# KDE Plasma on X11, hybrid Intel iGPU (primary) + NVIDIA RTX 4080 PRIME
# GDM session manager — Xauthority at /run/user/<uid>/gdm/Xauthority
# x11vnc mirrors the live X11 session on the Intel display
# View from: Tiamat, Ziggy, Android, Fire TV (Silk browser), any browser
# Run as your normal user (not root)
# =============================================================================
set -euo pipefail

VNC_PORT="5900"
NOVNC_PORT="6080"
LAPTOP_USER="${USER}"
LAPTOP_HOME="${HOME}"
USER_UID=$(id -u)
XAUTH_PATH="/run/user/${USER_UID}/gdm/Xauthority"

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║   Laptop VNC Setup                        ║"
echo "║   KDE Plasma / X11 / Intel+NVIDIA PRIME  ║"
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

# ── Install x11vnc + noVNC ───────────────────────────────────────────────────
echo "[1/5] Installing x11vnc + noVNC..."
sudo nala install -y \
    x11vnc \
    novnc \
    websockify \
    python3

# ── Store VNC password ───────────────────────────────────────────────────────
echo "[2/5] Storing VNC password..."
mkdir -p "${LAPTOP_HOME}/.vnc"
x11vnc -storepasswd "${VNC_PASS}" "${LAPTOP_HOME}/.vnc/passwd"
chmod 600 "${LAPTOP_HOME}/.vnc/passwd"
unset VNC_PASS VNC_PASS2

# ── Systemd user service for x11vnc ─────────────────────────────────────────
echo "[3/5] Creating x11vnc systemd user service..."
mkdir -p "${LAPTOP_HOME}/.config/systemd/user"

# Verify Xauthority exists
if [[ ! -f "${XAUTH_PATH}" ]]; then
    echo "WARNING: Expected Xauthority at ${XAUTH_PATH} not found."
    echo "Falling back to -auth guess (may need manual adjustment)."
    XAUTH_FLAG="-auth guess"
else
    XAUTH_FLAG="-auth ${XAUTH_PATH}"
fi

cat > "${LAPTOP_HOME}/.config/systemd/user/x11vnc.service" << EOF
[Unit]
Description=x11vnc — Mirror live KDE Plasma / Intel+NVIDIA X11 session
After=graphical-session.target

[Service]
Type=simple
Environment=DISPLAY=:0
Environment=XAUTHORITY=${XAUTH_PATH}
ExecStart=/usr/bin/x11vnc \\
    -display :0 \\
    -auth ${XAUTH_PATH} \\
    -rfbauth ${LAPTOP_HOME}/.vnc/passwd \\
    -rfbport ${VNC_PORT} \\
    -shared \\
    -forever \\
    -loop \\
    -noxdamage \\
    -repeat \\
    -xkb
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF
systemctl --user enable x11vnc.service
systemctl --user start  x11vnc.service

# ── noVNC systemd user service ───────────────────────────────────────────────
echo "[4/5] Creating noVNC systemd user service..."
NOVNC_DIR=$(find /usr/share -maxdepth 1 -name 'novnc' -o -name 'noVNC' 2>/dev/null | head -1)
[[ -z "$NOVNC_DIR" ]] && NOVNC_DIR="/usr/share/novnc"

cat > "${LAPTOP_HOME}/.config/systemd/user/novnc.service" << EOF
[Unit]
Description=noVNC Web Interface for Laptop Desktop
After=x11vnc.service
Requires=x11vnc.service

[Service]
Type=simple
ExecStart=/usr/bin/websockify --web=${NOVNC_DIR} ${NOVNC_PORT} localhost:${VNC_PORT}
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable novnc.service
systemctl --user start  novnc.service

# ── Enable lingering so services survive logout ──────────────────────────────
echo "[5/5] Enabling user lingering..."
sudo loginctl enable-linger "${LAPTOP_USER}"

# ── Done ─────────────────────────────────────────────────────────────────────
LAPTOP_IP=$(hostname -I | awk '{print $1}')
echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║   VNC + noVNC is LIVE on Laptop                                  ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
printf "║   VNC (native):  %-46s ║\n" "${LAPTOP_IP}:${VNC_PORT}"
printf "║   noVNC (web):   %-46s ║\n" "http://${LAPTOP_IP}:${NOVNC_PORT}/vnc.html"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo "║   Connect from anywhere on your LAN:                             ║"
echo "║   • Tiamat:      vncviewer ${LAPTOP_IP}:5900              ║"
echo "║   • Ziggy:       vncviewer ${LAPTOP_IP}:5900              ║"
echo "║   • Android:     VNC Viewer app → ${LAPTOP_IP}:5900       ║"
printf "║   • Fire TV:     Silk browser → http://%-26s ║\n" "${LAPTOP_IP}:6080/vnc.html"
echo "║   • Any browser: same noVNC URL above                            ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "NOTE: x11vnc mirrors your live KDE Plasma session. Lock screen when away."
echo ""
