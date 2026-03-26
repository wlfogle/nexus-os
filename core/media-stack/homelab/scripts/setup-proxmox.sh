#!/bin/bash
# ============================================================
# Proxmox VE 9.x Post-Install Setup
# Run on Proxmox host as root immediately after first boot
# ============================================================
set -e

echo "==> Disabling enterprise repo (requires subscription)..."
[ -f /etc/apt/sources.list.d/pve-enterprise.list ] && sed -i 's/^deb/#deb/' /etc/apt/sources.list.d/pve-enterprise.list || true
[ -f /etc/apt/sources.list.d/ceph.list ] && sed -i 's/^deb/#deb/' /etc/apt/sources.list.d/ceph.list || true

echo "==> Adding free/community repo..."
echo "deb http://download.proxmox.com/debian/pve bookworm pve-no-subscription" \
  > /etc/apt/sources.list.d/pve-no-subscription.list

echo "==> Updating packages..."
apt update && apt upgrade -y

echo "==> Installing useful tools..."
apt install -y curl wget git htop iotop ncdu lsof net-tools

echo "==> Disabling subscription nag popup..."
sed -i.bak "s/NotFound/Active/g" /usr/share/javascript/proxmox-widget-toolkit/proxmoxlib.js

echo "==> Setting up storage..."
# Check if 2TB HDD is present and create media storage
MEDIA_DISK=$(lsblk -dpno NAME,SIZE | grep -v sda | awk '{if($2~/[12]T/) print $1}' | head -1)
if [ -n "$MEDIA_DISK" ]; then
  echo "Found potential media disk: ${MEDIA_DISK}"
  echo "To format and mount: mkfs.ext4 ${MEDIA_DISK} && mkdir -p /mnt/media && mount ${MEDIA_DISK} /mnt/media"
  echo "Add to /etc/fstab: ${MEDIA_DISK} /mnt/media ext4 defaults 0 2"
else
  echo "No secondary disk detected — add media disk and run manually"
fi

echo "==> Creating media directories..."
mkdir -p /mnt/media/{movies,tv,music,books}
mkdir -p /mnt/downloads
mkdir -p /opt/appdata

echo "==> Downloading Alpine LXC template..."
pveam update
pveam download local alpine-3.19-default_20240207_amd64.tar.xz || \
  pveam download local $(pveam available | grep alpine | tail -1 | awk '{print $2}')

echo ""
echo "=== Proxmox Setup Complete ==="
echo ""
echo "Next steps:"
echo "  1. Mount your 2TB HDD to /mnt/media (see above)"
echo "  2. Run: scripts/deploy-media-stack.sh"
echo "  3. Access Proxmox UI: https://192.168.12.242:8006"
