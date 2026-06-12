#!/usr/bin/env bash
set -euo pipefail

SVG="$(dirname "$0")/../src-tauri/icons/nexus-codex.svg"
SVG="$(realpath "$SVG")"
ICONS="$(dirname "$SVG")"
BINARY="$(realpath "$(dirname "$0")/../src-tauri/target/debug/nexus-codex")"

echo "SVG:    $SVG"
echo "ICONS:  $ICONS"
echo "BINARY: $BINARY"

# ── Generate PNGs ─────────────────────────────────────────────────────────────
for SIZE in 16 32 48 64 128 256 512; do
    rsvg-convert -w "$SIZE" -h "$SIZE" "$SVG" -o "$ICONS/${SIZE}x${SIZE}.png"
    echo "  ✓ ${SIZE}x${SIZE}.png"
done

rsvg-convert -w 256 -h 256 "$SVG" -o "$ICONS/128x128@2x.png"
cp "$ICONS/512x512.png" "$ICONS/icon.png"
echo "  ✓ Tauri icon files updated"

# ── Install to hicolor theme ──────────────────────────────────────────────────
for SIZE in 16 32 48 64 128 256 512; do
    DIR="$HOME/.local/share/icons/hicolor/${SIZE}x${SIZE}/apps"
    mkdir -p "$DIR"
    cp "$ICONS/${SIZE}x${SIZE}.png" "$DIR/nexus-codex.png"
done
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
echo "  ✓ Icons installed to hicolor theme"

# ── Write .desktop entry ──────────────────────────────────────────────────────
DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"
cat > "$DESKTOP_DIR/nexus-codex.desktop" <<EOF
[Desktop Entry]
Name=Nexus Codex
GenericName=Documentation Intelligence
Comment=AI-powered doc scanning, classification and reporting for NexusOS
Exec=$BINARY
Icon=nexus-codex
Type=Application
Categories=Utility;Development;Science;
Keywords=nexus;codex;docs;ai;ollama;documentation;
StartupNotify=true
StartupWMClass=nexus-codex
EOF

chmod +x "$DESKTOP_DIR/nexus-codex.desktop"
echo "  ✓ .desktop entry written: $DESKTOP_DIR/nexus-codex.desktop"

# ── Refresh launcher ──────────────────────────────────────────────────────────
update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
echo ""
echo "Done. Nexus Codex is now in your app launcher."
