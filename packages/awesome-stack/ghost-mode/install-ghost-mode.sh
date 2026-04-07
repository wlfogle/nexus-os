#!/bin/bash

# 🥷 Ghost Mode Installation Script
# Complete setup for ultimate online invisibility

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/ghost-mode"

echo "🥷 Installing Ghost Mode - Ultimate Online Invisibility Suite"
echo "=============================================================="

# Create directories
echo "📁 Creating directories..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$CONFIG_DIR"
mkdir -p "$HOME/.config/autostart"
mkdir -p "$HOME/Desktop"

# Check dependencies
echo "🔍 Checking dependencies..."
MISSING_DEPS=()

if ! command -v firefox >/dev/null 2>&1; then
    MISSING_DEPS+=("firefox")
fi

if ! command -v python3 >/dev/null 2>&1; then
    MISSING_DEPS+=("python3")
fi

if ! python3 -c "import PyQt5" >/dev/null 2>&1; then
    MISSING_DEPS+=("python-pyqt5 or python3-pyqt5")
fi

if ! command -v wg >/dev/null 2>&1; then
    MISSING_DEPS+=("wireguard-tools")
fi

if [ ${#MISSING_DEPS[@]} -ne 0 ]; then
    echo "❌ Missing dependencies: ${MISSING_DEPS[*]}"
    echo ""
    echo "Install on Arch/Garuda:"
    echo "sudo pacman -S firefox wireguard-tools python-pyqt5 python-requests cronie"
    echo ""
    echo "Install on Ubuntu/Debian:"
    echo "sudo apt install firefox wireguard-tools python3-pyqt5 python3-requests cron"
    echo ""
    echo "Please install missing dependencies and run this script again."
    exit 1
fi

echo "✅ All dependencies satisfied"

# Copy scripts to install directory
echo "📋 Installing scripts..."
SCRIPTS=(
    "ghost-mode"
    "ghost-toggle"
    "ghost-tray-widget"
    "ghost-browser"
    "ghost-exec" 
    "ghost-time"
    "setup-ghost-firefox"
    "spoof-hardware"
    "secure-dns"
    "mask-time"
    "traffic-obfuscation"
    "clock-jitter"
    "ghost-monitor"
    "dns-leak-test"
    "ghost-help"
    "ghost-widget-info"
    "prevent-timing-attacks"
)

# Find script files in parent directories
for script in "${SCRIPTS[@]}"; do
    script_path=""
    
    # Check in ~/.local/bin first (already installed)
    if [ -f "$HOME/.local/bin/$script" ]; then
        echo "  ✅ $script (already installed)"
        continue
    fi
    
    # Check various possible locations
    locations=(
        "$SCRIPT_DIR/../scripts/$script"
        "$SCRIPT_DIR/$script"
        "$HOME/.local/bin/$script"
    )
    
    found=false
    for location in "${locations[@]}"; do
        if [ -f "$location" ]; then
            cp "$location" "$INSTALL_DIR/"
            chmod +x "$INSTALL_DIR/$script"
            echo "  ✅ $script"
            found=true
            break
        fi
    done
    
    if [ "$found" = false ]; then
        echo "  ⚠️  $script (not found, may need manual setup)"
    fi
done

# Set up auto-start
echo "🚀 Setting up auto-start..."
cat > "$HOME/.config/autostart/ghost-mode-tray.desktop" << EOF
[Desktop Entry]
Name=Ghost Mode Tray Widget
Comment=System tray widget for complete online anonymity control
Exec=$INSTALL_DIR/ghost-tray-widget
Icon=security-high
Terminal=false
Type=Application
Categories=Security;Privacy;
StartupNotify=false
X-GNOME-Autostart-enabled=true
Hidden=false
EOF

# Create desktop shortcut
echo "🖥️ Creating desktop shortcut..."
cat > "$HOME/Desktop/Ghost-Mode.desktop" << EOF
[Desktop Entry]
Name=🥷 Ghost Mode
Comment=Toggle complete online invisibility
Exec=$INSTALL_DIR/ghost-toggle
Icon=security-high
Terminal=true
Type=Application
Categories=Security;Privacy;Network;
Keywords=privacy;anonymity;vpn;ghost;invisible;security;
EOF

chmod +x "$HOME/.config/autostart/ghost-mode-tray.desktop"
chmod +x "$HOME/Desktop/Ghost-Mode.desktop"

# Initialize configuration
echo "⚙️ Initializing configuration..."
echo "inactive" > "$CONFIG_DIR/status"

# Add to PATH if not already there
if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
    echo "🔧 Adding $HOME/.local/bin to PATH..."
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
    echo "  (Restart terminal or run: source ~/.bashrc)"
fi

# Create quick reference card
echo "📖 Creating quick reference..."
cat > "$CONFIG_DIR/quick-reference.txt" << 'EOF'
🥷 GHOST MODE QUICK REFERENCE

One-Click Controls:
  ghost-toggle          # Toggle on/off
  ghost-mode            # Full control
  ghost-browser         # Anonymous browser

System Tray Widget:
  🟢 Green = Invisible online
  🟡 Yellow = Needs attention  
  🔴 Red = Visible online
  
  Left Click = Toggle
  Right Click = Menu
  Double Click = Control Panel

Testing:
  ghost-mode test       # Run anonymity tests
  dns-leak-test        # Check for DNS leaks

Visit: https://browserleaks.com to verify anonymity

Commands are installed in: ~/.local/bin/
Configuration stored in: ~/.config/ghost-mode/
EOF

echo ""
echo "✅ Ghost Mode installation complete!"
echo ""
echo "🎯 Quick Start:"
echo "   ghost-toggle          # Toggle on/off"
echo "   ghost-tray-widget &   # Start tray widget"
echo "   ghost-help            # Complete guide"
echo ""
echo "🔥 System tray widget will auto-start on next login"
echo "🔥 Desktop shortcut created: ~/Desktop/Ghost-Mode.desktop"
echo ""
echo "To start the tray widget now:"
echo "   $INSTALL_DIR/ghost-tray-widget &"
echo ""
echo "🥷 You now have one-click invisibility! 🥷"
