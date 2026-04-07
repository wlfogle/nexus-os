#!/bin/bash

# Garuda Homelab Toolbar Installation Script
# This script sets up the homelab management toolbar for Garuda Linux

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLBAR_DIR="$HOME/.local/share/homelab-toolbar"
PLASMOID_DIR="$HOME/.local/share/plasma/plasmoids/org.kde.plasma.homelab"
AUTOSTART_DIR="$HOME/.config/autostart"
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"

echo "🏠 Installing Garuda Homelab Toolbar..."

# Create directories
mkdir -p "$TOOLBAR_DIR"
mkdir -p "$PLASMOID_DIR"
mkdir -p "$AUTOSTART_DIR"
mkdir -p "$SYSTEMD_USER_DIR"

# Install dependencies
echo "📦 Installing dependencies..."
if command -v pamac >/dev/null 2>&1; then
    # Garuda Linux with pamac
    pamac install --no-confirm python python-requests plasma-framework5
elif command -v pacman > /dev/null 2>&1; then
    # Arch-based system
    sudo pacman -S --noconfirm python python-requests plasma-framework5
elif command -v apt >/dev/null 2>&1; then
    # Debian-based system
    sudo apt update && sudo apt install -y python3 python3-requests plasma-framework
else
    echo "⚠️  Could not detect package manager. Please install manually:"
    echo "   - Python 3"
    echo "   - python-requests"
    echo "   - plasma-framework"
fi

# Copy toolbar files
echo "📁 Copying toolbar files..."
cp "$SCRIPT_DIR/homelab-toolbar.html" "$TOOLBAR_DIR/"
cp "$SCRIPT_DIR/homelab-api.py" "$TOOLBAR_DIR/"
chmod +x "$TOOLBAR_DIR/homelab-api.py"

# Install KDE Plasmoid
echo "🔌 Installing KDE Plasmoid..."
cp -r "$SCRIPT_DIR/kde-plasmoid/"* "$PLASMOID_DIR/"

# Create systemd service for API server
echo "⚙️  Creating API service..."
cat > "$SYSTEMD_USER_DIR/homelab-api.service" << EOF
[Unit]
Description=Homelab API Server
After=network.target

[Service]
Type=simple
ExecStart=/usr/bin/python3 $TOOLBAR_DIR/homelab-api.py
Restart=always
RestartSec=10
Environment=PYTHONUNBUFFERED=1

[Install]
WantedBy=default.target
EOF

# Create desktop entry for toolbar
echo "🖥️  Creating desktop entries..."
cat > "$AUTOSTART_DIR/homelab-toolbar.desktop" << EOF
[Desktop Entry]
Type=Application
Name=Homelab Toolbar
Comment=Homelab Management Toolbar
Icon=network-server
Exec=firefox file://$TOOLBAR_DIR/homelab-toolbar.html
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
StartupNotify=false
EOF

# Create launcher script
cat > "$TOOLBAR_DIR/launch-toolbar.sh" << 'EOF'
#!/bin/bash

# Start API server if not running
if ! pgrep -f homelab-api.py >/dev/null; then
    systemctl --user start homelab-api.service
fi

# Open toolbar in browser
if command -v firefox >/dev/null 2>&1; then
    firefox --new-window "file://$HOME/.local/share/homelab-toolbar/homelab-toolbar.html" &
elif command -v chromium >/dev/null 2>&1; then
    chromium --app="file://$HOME/.local/share/homelab-toolbar/homelab-toolbar.html" &
elif command -v google-chrome >/dev/null 2>&1; then
    google-chrome --app="file://$HOME/.local/share/homelab-toolbar/homelab-toolbar.html" &
else
    echo "No suitable browser found. Please install Firefox or Chromium."
fi
EOF

chmod +x "$TOOLBAR_DIR/launch-toolbar.sh"

# Enable and start services
echo "🚀 Starting services..."
systemctl --user daemon-reload
systemctl --user enable homelab-api.service
systemctl --user start homelab-api.service

# Install plasmoid
echo "🎨 Installing Plasmoid..."
if command -v plasmapkg2 >/dev/null 2>&1; then
    plasmapkg2 -i "$PLASMOID_DIR" || plasmapkg2 -u "$PLASMOID_DIR"
else
    echo "⚠️  plasmapkg2 not found. Manual plasmoid installation required."
fi

# Create desktop shortcut
cat > "$HOME/Desktop/Homelab-Toolbar.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Homelab Toolbar
Comment=Open Homelab Management Toolbar
Exec=$TOOLBAR_DIR/launch-toolbar.sh
Icon=network-server
Terminal=false
Categories=System;Network;
EOF

chmod +x "$HOME/Desktop/Homelab-Toolbar.desktop"

# Create configuration file
echo "⚙️  Creating configuration..."
cat > "$TOOLBAR_DIR/config.json" << EOF
{
    "proxmox": {
        "ip": "192.168.122.9",
        "port": "8006",
        "user": "root"
    },
    "services": {
        "plex": "http://192.168.122.230:32400/web",
        "jellyfin": "http://192.168.122.231:8096",
        "audiobookshelf": "http://192.168.122.232:13378",
        "sonarr": "http://192.168.122.214:8989",
        "radarr": "http://192.168.122.215:7878",
        "readarr": "http://192.168.122.217:8787",
        "bazarr": "http://192.168.122.240:6767",
        "prowlarr": "http://192.168.122.210:9696",
        "qbittorrent": "http://192.168.122.212:8080",
        "deluge": "http://192.168.122.224:8112",
        "overseerr": "http://192.168.122.241:5055",
        "jellyseerr": "http://192.168.122.242:5055",
        "ombi": "http://192.168.122.243:3579",
        "organizr": "http://192.168.122.274:80",
        "homarr": "http://192.168.122.275:7575",
        "homepage": "http://192.168.122.276:3000",
        "tautulli": "http://192.168.122.244:8181",
        "grafana": "http://192.168.122.261:3000",
        "prometheus": "http://192.168.122.260:9090",
        "authentik": "http://192.168.122.107:9000",
        "vaultwarden": "http://192.168.122.104:80",
        "traefik": "http://192.168.122.103:8080",
        "wireguard": "http://192.168.122.100:51820",
        "xvfb": "http://192.168.122.9:6080"
    },
    "refresh_interval": 30,
    "auto_minimize": false,
    "containers": {
        "100": "wireguard",
        "230": "plex",
        "231": "jellyfin",
        "214": "sonarr",
        "215": "radarr",
        "210": "prowlarr",
        "212": "qbittorrent",
        "241": "overseerr",
        "261": "grafana",
        "103": "traefik",
        "107": "authentik",
        "104": "vaultwarden"
    }
}
EOF

echo ""
echo "✅ Installation Complete!"
echo ""
echo "🎯 What's been installed:"
echo "   • Homelab Toolbar (Web interface)"
echo "   • KDE Plasmoid widget for system tray"
echo "   • API server for service monitoring"
echo "   • Desktop shortcuts and autostart entries"
echo ""
echo "🚀 To get started:"
echo "   1. Add the 'Homelab Manager' widget to your KDE panel"
echo "   2. Or run: $TOOLBAR_DIR/launch-toolbar.sh"
echo "   3. Or double-click the desktop shortcut"
echo ""
echo "⚙️  Configuration file: $TOOLBAR_DIR/config.json"
echo "📊 API Server: http://localhost:8082"
echo ""
echo "🔧 Keyboard shortcuts (when toolbar is active):"
echo "   Alt+Shift+H: Toggle toolbar"
echo "   Alt+Shift+P: Open Proxmox"
echo "   Alt+Shift+J: Open Jellyfin"
echo "   Alt+Shift+G: Open Grafana"
echo ""
echo "🎉 Enjoy your new homelab management toolbar!"

# Optional: Open the toolbar immediately
read -p "🚀 Launch the toolbar now? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    "$TOOLBAR_DIR/launch-toolbar.sh"
fi
