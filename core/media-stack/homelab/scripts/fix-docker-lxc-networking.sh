#!/usr/bin/env bash
# Fix Docker port mapping in unprivileged LXC containers
# Changes all docker-compose.yml files to use network_mode: host
set -euo pipefail

PROXMOX=192.168.12.242
CTS=(102 232 240 241 244 245 275 276 277 278)

fix_ct() {
  local CT=$1
  local NAME=$(ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX "pct config $CT | grep '^hostname' | cut -d' ' -f2" 2>/dev/null)
  echo ""
  echo "=== CT-$CT ($NAME) ==="

  # Stop CT
  ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX "pct stop $CT" 2>/dev/null || true
  sleep 3

  # Mount filesystem
  local ROOTFS=$(ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX "pct mount $CT 2>&1 | grep -o \"'/[^']*'\" | tr -d \"'\"")
  if [ -z "$ROOTFS" ]; then
    ROOTFS="/var/lib/lxc/$CT/rootfs"
  fi
  echo "  Mounted at $ROOTFS"

  # Find docker-compose files
  local COMPOSE_FILES=$(ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX \
    "find '$ROOTFS' -name 'docker-compose.yml' -not -path '*/proc/*' -not -path '*/sys/*' 2>/dev/null | head -5")

  if [ -z "$COMPOSE_FILES" ]; then
    echo "  No docker-compose.yml found — skipping"
  else
    echo "  Found: $COMPOSE_FILES"
    # Apply network_mode: host to each compose file
    while IFS= read -r COMPOSE; do
      echo "  Fixing: $COMPOSE"
      ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX "
        python3 - '$COMPOSE' << 'PYEOF'
import sys, re

path = sys.argv[1]
content = open(path).read()

# Remove ports: blocks
content = re.sub(r'\n(\s+)ports:\n(\s+- [^\n]+\n)+', '\n', content)

# Add network_mode: host after container_name or image line if not already present
if 'network_mode' not in content:
    content = re.sub(
        r'(    container_name:[^\n]+\n)',
        r'\1    network_mode: host\n',
        content, count=1
    )
    # fallback: add after image line
    if 'network_mode' not in content:
        content = re.sub(
            r'(    image:[^\n]+\n)',
            r'\1    network_mode: host\n',
            content, count=1
        )

open(path, 'w').write(content)
print('    Updated:', path)
PYEOF"
    done <<< "$COMPOSE_FILES"
  fi

  # Unmount and start
  ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX "pct unmount $CT" 2>/dev/null || true
  ssh -o BatchMode=yes -o StrictHostKeyChecking=no root@$PROXMOX "pct start $CT" && echo "  Started CT-$CT"
  sleep 2
}

for CT in "${CTS[@]}"; do
  fix_ct "$CT"
done

echo ""
echo "=== All done. Waiting 30s for services to start... ==="
sleep 30

echo ""
echo "=== Checking services ==="
declare -A URLS=(
  [102]="http://192.168.12.102:8191"
  [232]="http://192.168.12.232:13378"
  [240]="http://192.168.12.240:6767"
  [241]="http://192.168.12.241:5055"
  [244]="http://192.168.12.169:8181"
  [275]="http://192.168.12.275:7575"
  [276]="http://192.168.12.276:3000"
)
for CT in "${!URLS[@]}"; do
  CODE=$(curl -s -o /dev/null -w "%{http_code}" --max-time 5 "${URLS[$CT]}" 2>/dev/null)
  [ "$CODE" = "200" ] || [ "$CODE" = "302" ] && echo "✓ CT-$CT: $CODE" || echo "✗ CT-$CT: $CODE"
done
