#!/usr/bin/env bash
# Convert all Docker-in-unprivileged-LXC containers to privileged
# Uses in-place UID remap + config change (no backup/restore needed)
set -euo pipefail

PROXMOX=192.168.12.242
CTS=(102 232 240 241 244 245 275 276 277 278)

convert_ct() {
  local CT=$1
  local NAME=$(ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX \
    "pct config $CT | grep '^hostname' | cut -d' ' -f2" 2>/dev/null)

  echo ""
  echo "=== CT-$CT ($NAME) ==="

  # Check if already privileged
  local UNPRIV=$(ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX \
    "pct config $CT | grep '^unprivileged' | awk '{print \$2}'" 2>/dev/null)
  if [ "${UNPRIV:-0}" != "1" ]; then
    echo "  Already privileged — skipping"
    return
  fi

  # Stop CT
  echo "  Stopping..."
  ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX "
    if pct status $CT | grep -q running; then pct stop $CT; fi
    for i in \$(seq 1 20); do
      pct status $CT | grep -q stopped && break || sleep 2
    done
  " 2>/dev/null
  echo "  Stopped."

  # Mount filesystem
  echo "  Mounting filesystem..."
  local ROOTFS="/var/lib/lxc/$CT/rootfs"
  ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX "pct mount $CT" 2>/dev/null || true

  # Remap UIDs
  echo "  Remapping UIDs (unprivileged→privileged)..."
  ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX \
    "uidmapshift -b '$ROOTFS' 100000 0 65536" 2>/dev/null && echo "  UID remap done."

  # Unmount
  ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX "pct unmount $CT" 2>/dev/null || true

  # Update config to privileged
  echo "  Updating Proxmox config to privileged..."
  ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX "
    sed -i '/^unprivileged:/d' /etc/pve/lxc/$CT.conf
    sed -i '/^lxc.idmap/d' /etc/pve/lxc/$CT.conf
    echo 'unprivileged: 0' >> /etc/pve/lxc/$CT.conf
  "

  # Start CT
  echo "  Starting CT-$CT (privileged)..."
  ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX "pct start $CT" && echo "  Started."
  sleep 3
}

for CT in "${CTS[@]}"; do
  convert_ct "$CT"
done

echo ""
echo "=== All CTs converted. Waiting 60s for Docker to start services... ==="
sleep 60

echo ""
echo "=== Final service check ==="
declare -A URLS=(
  [102]="http://192.168.12.102:8191"
  [232]="http://192.168.12.232:13378"
  [240]="http://192.168.12.240:6767"
  [241]="http://192.168.12.241:5055"
  [244]="http://192.168.12.169:8181"
  [275]="http://192.168.12.275:7575"
  [276]="http://192.168.12.276:3000"
  [277]="http://192.168.12.277:7474"
  [278]="http://192.168.12.278:8080"
)
for CT in "${!URLS[@]}"; do
  NAME=$(ssh -o BatchMode=yes -o StrictHostKeyChecking=no -o ConnectTimeout=3 root@$PROXMOX \
    "pct config $CT | grep '^hostname' | cut -d' ' -f2" 2>/dev/null)
  CODE=$(curl -s -o /dev/null -w "%{http_code}" --max-time 5 "${URLS[$CT]}" 2>/dev/null)
  [ "$CODE" = "200" ] || [ "$CODE" = "302" ] && STATUS="✓" || STATUS="✗"
  echo "$STATUS CT-$CT ($NAME): $CODE"
done | sort
