#!/usr/bin/env bash
# Creates CT-242 as a PRIVILEGED LXC and installs Jellyseerr natively
# Run this on the Proxmox host (192.168.12.242)
set -euo pipefail

CT_ID=242
CT_HOSTNAME=jellyseerr
CT_IP=192.168.12.151/24
CT_GW=192.168.12.1
CT_DNS=192.168.12.244
CT_STORAGE=local-lvm
CT_DISK=8
CT_RAM=4096
CT_CORES=4
CT_TEMPLATE=local:vztmpl/debian-12-standard_12.7-1_amd64.tar.zst

echo "==> Checking if template exists..."
if ! pveam list local | grep -q "debian-12-standard"; then
  echo "   Downloading Debian 12 template..."
  pveam download local debian-12-standard_12.7-1_amd64.tar.zst
fi

echo "==> Creating CT-$CT_ID (privileged)..."
pct create "$CT_ID" "$CT_TEMPLATE" \
  --hostname "$CT_HOSTNAME" \
  --unprivileged 0 \
  --features nesting=1 \
  --net0 name=eth0,bridge=vmbr0,gw="$CT_GW",ip="$CT_IP",type=veth \
  --nameserver "$CT_DNS" \
  --storage "$CT_STORAGE" \
  --rootfs "$CT_STORAGE:$CT_DISK" \
  --memory "$CT_RAM" \
  --cores "$CT_CORES" \
  --onboot 1 \
  --start 1
echo "   CT-$CT_ID created and started."

echo "==> Waiting for CT to boot..."
sleep 10

echo "==> Running Jellyseerr install script inside CT..."
pct exec "$CT_ID" -- bash -c "$(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/install/jellyseerr-install.sh)"

echo ""
echo "========================================="
echo "  DONE: CT-$CT_ID (privileged) running"
echo "  Jellyseerr: http://192.168.12.151:5055"
echo "  First-run setup required in browser"
echo "  Jellyfin:  192.168.12.231:8096"
echo "  Radarr:    192.168.12.225:7878"
echo "  Sonarr:    192.168.12.214:8989"
echo "========================================="
