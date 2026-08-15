#!/usr/bin/env bash
# install.sh - install/update the OriginPC Control Center suite
#
# Idempotent: safe to re-run after pulling updates to this package.
# Requires sudo only for: the udev rule, and building/loading the
# clevo-hotkeys kernel module via DKMS.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="$HOME/.local/share/originpc-control-center"
BIN_DIR="$HOME/.local/bin"
APPS_DIR="$HOME/.local/share/applications"
USER_UNIT_DIR="$HOME/.config/systemd/user"

echo "==> Installing OriginPC Control Center from $SCRIPT_DIR"

# 1. Application files
mkdir -p "$INSTALL_DIR"
cp -f "$SCRIPT_DIR"/src/*.py "$INSTALL_DIR"/
chmod +x "$INSTALL_DIR"/*.py
echo "  - App files installed to $INSTALL_DIR"

# 2. udev rule for RGB hidraw access (needs sudo once)
UDEV_RULE_SRC="$SCRIPT_DIR/packaging/99-originpc-rgb.rules"
UDEV_RULE_DST="/etc/udev/rules.d/99-originpc-rgb.rules"
if ! sudo cmp -s "$UDEV_RULE_SRC" "$UDEV_RULE_DST" 2>/dev/null; then
    sudo install -m 644 "$UDEV_RULE_SRC" "$UDEV_RULE_DST"
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    echo "  - udev rule installed and reloaded"
else
    echo "  - udev rule already up to date"
fi

# 3. clevo-hotkeys kernel module via DKMS (needs sudo)
KMOD_SRC="$SCRIPT_DIR/kernel/clevo-hotkeys"
KMOD_VERSION="1.0"
KMOD_DEST="/usr/src/clevo-hotkeys-$KMOD_VERSION"
if command -v dkms >/dev/null 2>&1; then
    if ! dkms status clevo-hotkeys/$KMOD_VERSION 2>/dev/null | grep -q installed; then
        sudo rm -rf "$KMOD_DEST"
        sudo mkdir -p "$KMOD_DEST"
        sudo cp "$KMOD_SRC"/clevo-hotkeys.c "$KMOD_SRC"/Makefile "$KMOD_SRC"/dkms.conf "$KMOD_DEST"/
        sudo dkms add -m clevo-hotkeys -v "$KMOD_VERSION"
        sudo dkms build -m clevo-hotkeys -v "$KMOD_VERSION"
        sudo dkms install -m clevo-hotkeys -v "$KMOD_VERSION"
        echo "  - clevo-hotkeys kernel module built and installed via DKMS"
    else
        echo "  - clevo-hotkeys DKMS module already installed"
    fi
    sudo modprobe clevo-hotkeys || echo "  ! modprobe clevo-hotkeys failed - check 'dmesg | grep clevo'"
else
    echo "  ! dkms not found - skipping Fn-hotkey kernel module (Flexikey/RGB/lid-monitor are unaffected)"
fi

# 4. Desktop entries
mkdir -p "$APPS_DIR"
for f in originpc-control-center.desktop originpc-flexikey.desktop; do
    cp -f "$SCRIPT_DIR/packaging/$f" "$APPS_DIR/$f"
done
echo "  - Desktop entries installed"

# 5. Launcher wrappers on PATH
mkdir -p "$BIN_DIR"
cat > "$BIN_DIR/originpc-control-center" <<EOF
#!/usr/bin/env bash
cd "$INSTALL_DIR"
exec python3 enhanced-professional-control-center.py "\$@"
EOF
cat > "$BIN_DIR/originpc-flexikey" <<EOF
#!/usr/bin/env bash
cd "$INSTALL_DIR"
exec python3 flexikey.py --gui "\$@"
EOF
chmod +x "$BIN_DIR/originpc-control-center" "$BIN_DIR/originpc-flexikey"
case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) echo "  ! $BIN_DIR is not on your PATH - add it to your fish config (fish_add_path $BIN_DIR)" ;;
esac
echo "  - Launchers installed to $BIN_DIR"

# 6. systemd --user units
mkdir -p "$USER_UNIT_DIR"
for unit in originpc-lid-monitor.service originpc-flexikey.service originpc-hotkey-osd.service; do
    cp -f "$SCRIPT_DIR/packaging/$unit" "$USER_UNIT_DIR/$unit"
done
systemctl --user daemon-reload
echo "  - systemd --user units installed (not yet enabled)"

echo ""
echo "==> Install complete."
echo "Next steps:"
echo "  originpc-control-center                                   # launch the GUI"
echo "  systemctl --user enable --now originpc-lid-monitor         # clear RGB on lid close"
echo "  systemctl --user enable --now originpc-hotkey-osd          # Fn-hotkey on-screen display"
echo "  originpc-flexikey                                          # configure key remaps/macros,"
echo "                                                              then: systemctl --user enable --now originpc-flexikey"
echo ""
echo "Verify RGB device permissions: ls -la /dev/hidraw0   (expect crw-rw-rw-)"
echo "Verify Fn-hotkey driver bound: ls -la /sys/bus/wmi/devices/ABBC0F6B-8EA1-11D1-00A0-C90629100000/driver"
