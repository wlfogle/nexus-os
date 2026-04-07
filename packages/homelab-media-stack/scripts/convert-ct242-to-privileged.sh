#!/usr/bin/env bash
# Converts CT-242 (jellyseerr) from unprivileged to privileged IN PLACE
# No backup/restore needed - directly flips privilege flag and remaps UIDs
set -euo pipefail

CONTAINER_ID=242

echo "==> Checking CT-242 is unprivileged..."
pct config "$CONTAINER_ID" | grep -q 'unprivileged: 1' || { echo "ERROR: CT-242 is already privileged or not found"; exit 1; }

echo "==> Stopping CT-242..."
if pct status "$CONTAINER_ID" | grep -q running; then
  pct stop "$CONTAINER_ID"
fi
echo "   Stopped."

echo "==> Mounting CT-242 filesystem..."
pct mount "$CONTAINER_ID"
ROOTFS="/var/lib/lxc/${CONTAINER_ID}/rootfs"
echo "   Mounted at $ROOTFS"

echo "==> Remapping UIDs: 100000->0 (unprivileged->privileged)..."
uidmapshift -b "$ROOTFS" 100000 0 65536
echo "   UID remap complete."

echo "==> Unmounting..."
pct unmount "$CONTAINER_ID"

echo "==> Updating Proxmox config to privileged..."
sed -i '/^unprivileged:/d' "/etc/pve/lxc/${CONTAINER_ID}.conf"
sed -i '/^lxc.idmap/d' "/etc/pve/lxc/${CONTAINER_ID}.conf"
echo "unprivileged: 0" >> "/etc/pve/lxc/${CONTAINER_ID}.conf"
echo "   Config updated."

echo "==> Starting CT-242 (now privileged)..."
pct start "$CONTAINER_ID"
echo "   Started."

echo ""
echo "========================================="
echo "  DONE: CT-242 is now PRIVILEGED"
echo "  Docker port mapping and networking will work correctly"
echo "  Jellyseerr: http://192.168.12.151:5055"
echo "========================================="
