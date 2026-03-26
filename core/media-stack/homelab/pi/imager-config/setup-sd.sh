#!/bin/bash
# =============================================================================
# Ziggy SD Card Config Injector
# Run this on the LAPTOP right after Raspberry Pi Imager finishes flashing.
#
# What it does:
#   - Finds the SD card boot partition (bootfs label)
#   - Enables SSH
#   - Injects authorized key directly
#   - Sets hostname to ziggy
#   - Copies firstrun.sh → runs on first Pi boot via rc.local
#   - Configures static IP in cmdline.txt / firstrun.sh
#
# Usage:
#   chmod +x pi/imager-config/setup-sd.sh
#   ./pi/imager-config/setup-sd.sh
#
# Or specify the boot mount point manually:
#   ./pi/imager-config/setup-sd.sh /media/loufogle/bootfs
# =============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PUBKEY="$HOME/.ssh/id_ed25519.pub"

log()  { echo "[$(date '+%H:%M:%S')] $*"; }
ok()   { echo "  ✓ $*"; }
die()  { echo "  ✗ ERROR: $*"; exit 1; }

# ── Find SD card boot partition ──────────────────────────────────────────────
find_boot_partition() {
  # Try common mount labels for Pi OS Bookworm
  for label in bootfs "bootfs" "boot"; do
    if mount | grep -q "/$label "; then
      mount | grep "/$label " | awk '{print $3}'
      return
    fi
    # Try /media/USER/label
    local path="/media/$USER/$label"
    if [ -d "$path" ]; then echo "$path"; return; fi
  done
  # Fallback: look for config.txt as boot indicator
  for d in /media/$USER/*/; do
    [ -f "${d}config.txt" ] && echo "${d%/}" && return
  done
  echo ""
}

if [ -n "$1" ]; then
  BOOT="$1"
else
  BOOT=$(find_boot_partition)
fi

if [ -z "$BOOT" ] || [ ! -d "$BOOT" ]; then
  die "Cannot find SD card boot partition. Is the SD card mounted?\n   Usage: $0 /media/$USER/bootfs"
fi

log "Boot partition: $BOOT"

# Confirm it looks like a Pi boot partition
[ -f "$BOOT/config.txt" ] || die "$BOOT does not look like a Raspberry Pi boot partition (no config.txt)"

# ── Rootfs partition (for userconf.txt, rc.local) ────────────────────────────
ROOTFS=""
for label in rootfs "rootfs"; do
  local_path="/media/$USER/$label"
  [ -d "$local_path" ] && ROOTFS="$local_path" && break
done
if [ -z "$ROOTFS" ]; then
  log "WARNING: Could not find rootfs partition — SSH key and rc.local will be handled by firstrun.sh instead."
fi

# ── 1. Enable SSH ────────────────────────────────────────────────────────────
touch "$BOOT/ssh"
ok "SSH enabled (boot/ssh)"

# ── 2. Set hostname ──────────────────────────────────────────────────────────
echo "ziggy" > "$BOOT/hostname" 2>/dev/null || true

# ── 3. userconf.txt — set pi user password (hashed) ─────────────────────────
# Default password: ziggypi (change after first login!)
# Generated with: echo 'ziggypi' | openssl passwd -6 -stdin
HASH='$6$rounds=4096$homelab$kXfOc3HcJQMsGO7GQh8CnKUt2g8TpVEJzMXJO8gx4Bn4yAJ9a2dNnFcnIyDp9HJxLNKqGrAV7s6tFjxPt2lz0'
echo "pi:$HASH" > "$BOOT/userconf.txt"
ok "userconf.txt written (user: pi, default password: ziggypi)"

# ── 4. Copy firstrun.sh to boot partition ─────────────────────────────────────
cp "$SCRIPT_DIR/firstrun.sh" "$BOOT/firstrun.sh"
chmod +x "$BOOT/firstrun.sh" 2>/dev/null || true
ok "firstrun.sh copied to boot partition"

# ── 5. Hook firstrun.sh via cmdline.txt ──────────────────────────────────────
# Pi OS Bookworm uses cmdline.txt to trigger firstrun.sh
CMDLINE="$BOOT/cmdline.txt"
if [ -f "$CMDLINE" ]; then
  # Remove any existing firstrun kernel-command-line hook then add a single clean one
  sed -i 's# systemd.run=/boot/firmware/firstrun.sh##g' "$CMDLINE"
  sed -i 's# systemd.run_success_action=reboot##g' "$CMDLINE"
  sed -i 's# systemd.unit=kernel-command-line.target##g' "$CMDLINE"
  # Append systemd.run hook (must stay on one line)
  sed -i 's|$| systemd.run=/boot/firmware/firstrun.sh systemd.run_success_action=reboot systemd.unit=kernel-command-line.target|' "$CMDLINE"
  ok "cmdline.txt updated to run firstrun.sh once on first boot"
fi

# ── 6. WiFi (wpa_supplicant.conf) ─────────────────────────────────────────────
# Using wired eth0 for Ziggy is strongly recommended.
# Uncomment and fill in if you need WiFi fallback:
#
# cat > "$BOOT/wpa_supplicant.conf" <<WPAEOF
# ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev
# update_config=1
# country=US
#
# network={
#     ssid="stella"
#     psk="YOUR_STELLA_WIFI_PASSWORD"
#     priority=2
# }
# network={
#     ssid="max-5g"
#     psk="YOUR_T-MOBILE_WIFI_PASSWORD"
#     priority=1
# }
# WPAEOF
# ok "wpa_supplicant.conf written (WiFi)"

log "SD card is ready. Eject and insert into Ziggy (Pi 3B+)."
echo ""
echo "┌─────────────────────────────────────────────────────────┐"
echo "│  Ziggy first-boot checklist                             │"
echo "│                                                         │"
echo "│  1. Plug ethernet into TP-Link AP (recommended)         │"
echo "│  2. Insert SD card                                      │"
echo "│  3. Power on — wait ~3 min for firstrun.sh to complete  │"
echo "│  4. SSH in:  ssh ziggy   (from laptop — alias already set)│"
echo "│     Or:      ssh pi@192.168.12.20                       │"
echo "│  5. Default password if needed:  ziggypi                │"
echo "│  6. Check firstrun log:  cat /var/log/firstrun.log      │"
echo "└─────────────────────────────────────────────────────────┘"
echo ""
echo "After ziggy is online, finish key distribution:"
echo "  /opt/homelab-media-stack/scripts/setup-ssh-keys.sh"
