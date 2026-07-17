#!/usr/bin/env bash
# MobaLiveCD launcher — ensures correct env for Tauri/GTK

export PATH="/usr/local/bin:/usr/bin:/bin:/home/loufogle/.cargo/bin:$PATH"
export HOME="/home/loufogle"
# Set these to avoid the GTK/DISPLAY panic
export DISPLAY=:1
export GDK_BACKEND=x11

# Update these paths to point to your mobalivecd-linux folder
RELEASE="/home/loufogle/nexus-os/packages/mobalivecd-linux/mobalivecd-linux/src-tauri/target/release/mobalivecd-linux"
DIR="/home/loufogle/nexus-os/packages/mobalivecd-linux/mobalivecd-linux"

if [ -f "$RELEASE" ]; then
  # Launch the optimized binary
  cd "$DIR" && exec "$RELEASE"
else
  # Launch the development environment
  cd "$DIR" && exec npm run tauri dev
fi
