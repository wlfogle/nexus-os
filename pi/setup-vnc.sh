#!/usr/bin/env bash
# =============================================================================
# pi/setup-vnc.sh — Ziggy (Raspberry Pi 3B+) VNC + noVNC Desktop Setup
# Installs XFCE4 + TigerVNC + noVNC web interface
# View from: native VNC client, any browser, Android, Fire TV (Silk browser)
# =============================================================================
set -euo pipefail

VNC_PORT="5900"
NOVNC_PORT="6080"
VNC_RESOLUTION="1280x720"
VNC_DEPTH="24"

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║   Ziggy VNC Desktop Setup                ║"
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

# ── Install ──────────────────────────────────────────────────────────────────
echo "[1/5] Installing XFCE4 + TigerVNC + noVNC..."
apt-get update -y
apt-get install -y \
    xfce4 \
    xfce4-goodies \
    tigervnc-standalone-server \
    tigervnc-common \
    dbus-x11 \
    x11-xserver-utils \
    xfonts-base \
    novnc \
    websockify \
    python3 \
    --no-install-recommends

# ── VNC password ─────────────────────────────────────────────────────────────
echo "[2/5] Setting VNC password..."
mkdir -p /home/pi/.vnc
printf '%s\n%s\n\n' "$VNC_PASS" "$VNC_PASS" | sudo -u pi vncpasswd /home/pi/.vnc/passwd
chmod 600 /home/pi/.vnc/passwd
chown pi:pi /home/pi/.vnc/passwd
unset VNC_PASS VNC_PASS2

# ── xstartup ─────────────────────────────────────────────────────────────────
echo "[3/5] Writing xstartup for XFCE4..."
cat > /home/pi/.vnc/xstartup << 'EOF'
#!/usr/bin/env bash
unset SESSION_MANAGER
unset DBUS_SESSION_BUS_ADDRESS
export XKL_XMODMAP_DISABLE=1
if [ -z "$DBUS_SESSION_BUS_ADDRESS" ]; then
    eval "$(dbus-launch --sh-syntax)"
fi
exec startxfce4
EOF
chmod +x /home/pi/.vnc/xstartup
chown pi:pi /home/pi/.vnc/xstartup

# ── VNC config ───────────────────────────────────────────────────────────────
cat > /home/pi/.vnc/config << EOF
geometry=${VNC_RESOLUTION}
depth=${VNC_DEPTH}
localhost=no
alwaysshared
EOF
chown pi:pi /home/pi/.vnc/config

# ── Systemd service ──────────────────────────────────────────────────────────
echo "[4/5] Creating systemd service..."
cat > /etc/systemd/system/vncserver@.service << 'EOF'
[Unit]
Description=TigerVNC Server (display %i)
After=syslog.target network.target

[Service]
Type=forking
User=pi
WorkingDirectory=/home/pi
PIDFile=/home/pi/.vnc/%H%i.pid
ExecStartPre=/usr/bin/vncserver -kill %i > /dev/null 2>&1 || true
ExecStart=/usr/bin/vncserver %i -depth 24 -geometry 1280x720 -localhost no -alwaysshared
ExecStop=/usr/bin/vncserver -kill %i
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable vncserver@:0.service
systemctl start  vncserver@:0.service

# ── noVNC websockify service ──────────────────────────────────────────────────
echo "[5/5] Setting up noVNC on port ${NOVNC_PORT}..."
NOVNC_DIR=$(find /usr/share -maxdepth 1 -name 'novnc' -o -name 'noVNC' 2>/dev/null | head -1)
[[ -z "$NOVNC_DIR" ]] && NOVNC_DIR="/usr/share/novnc"

cat > /etc/systemd/system/novnc.service << EOF
[Unit]
Description=noVNC Web Interface for Ziggy Desktop
After=network.target vncserver@:0.service
Requires=vncserver@:0.service

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
systemctl start  novnc.service

# ── Done ─────────────────────────────────────────────────────────────────────
ZIGGY_IP=$(hostname -I | awk '{print $1}')
echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║   VNC + noVNC is LIVE on Ziggy                                   ║"
echo "╠══════════════════════════════════════════════════════════════════╣"
printf "║   VNC (native):  %-46s ║\n" "${ZIGGY_IP}:${VNC_PORT}"
printf "║   noVNC (web):   %-46s ║\n" "http://${ZIGGY_IP}:${NOVNC_PORT}/vnc.html"
echo "╠══════════════════════════════════════════════════════════════════╣"
echo "║   Connect from anywhere:                                         ║"
echo "║   • Laptop:      vncviewer ${ZIGGY_IP}:5900              ║"
echo "║   • Android:     VNC Viewer app → ${ZIGGY_IP}:5900       ║"
printf "║   • Fire TV:     Silk browser → http://%-26s ║\n" "${ZIGGY_IP}:6080/vnc.html"
echo "║   • Any browser: same noVNC URL above                            ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
