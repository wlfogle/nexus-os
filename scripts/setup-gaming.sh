#!/bin/bash
set -euo pipefail

# Install RetroArch on the laptop (Pop!_OS) for local retro gaming
# ROMs: /media/loufogle/Games/roms (89GB, NFS-exported to Tiamat)
# For Fire TV setup, see docs/RETRO-GAMING.md

echo "Installing RetroArch on laptop..."
sudo nala install -y retroarch retroarch-assets libretro-*

# Create ROM directory symlink if not exists
ROM_DIR="/media/loufogle/Games/roms"
if [ -d "$ROM_DIR" ]; then
    echo "ROM directory found: $ROM_DIR"
else
    echo "Warning: ROM directory not found at $ROM_DIR"
    echo "Mount the Games partition or adjust the path."
fi

# Switch emulator (Ryujinx discontinued, use Ryubing or Suyu)
if command -v snap &>/dev/null; then
    echo "Installing Ryubing (Switch emulator) via snap..."
    sudo snap install ryubing-emulator
else
    echo "Snap not available — install Ryubing manually from https://github.com/Ryubing/Ryujinx"
fi

echo ""
echo "Done! Launch RetroArch for retro games, Ryubing for Switch titles."
echo "ROMs: $ROM_DIR"
echo "Fire TV: see docs/RETRO-GAMING.md for NFS + RetroArch setup"
