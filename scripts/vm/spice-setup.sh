#!/usr/bin/env bash
set -euo pipefail

VM="${1:-win11}"

echo "[1/6] Installing tools (virt-manager, virt-viewer, virtinst)..."
sudo apt-get update -y
sudo apt-get install -y virt-manager virt-viewer virtinst

echo "[2/6] Ensuring VM is powered off..."
if virsh domstate "$VM" 2>/dev/null | grep -qi running; then
  virsh shutdown "$VM" || true
  for i in {1..30}; do
    sleep 2
    state="$(virsh domstate "$VM" 2>/dev/null || true)"
    [[ "$state" =~ [Ss]hut ]] && break
  done
  if virsh domstate "$VM" 2>/dev/null | grep -qi running; then
    echo "Force stopping VM..."
    virsh destroy "$VM"
  fi
fi

echo "[3/6] Backing up current XML..."
mkdir -p "$HOME/virt-backups"
ts="$(date +%Y%m%d-%H%M%S)"
virsh dumpxml "$VM" > "$HOME/virt-backups/${VM}-${ts}.xml"

echo "[4/6] Applying SPICE display, virtio-serial, SPICE agent channel, QXL video, USB tablet..."
# Use SPICE graphics (TCP on 127.0.0.1)
sudo virt-xml "$VM" --edit --graphics spice,listen=127.0.0.1 --check all,relax

# Ensure Virtio-serial controller
if ! virsh dumpxml "$VM" | grep -q "<controller type='virtio-serial'"; then
  sudo virt-xml "$VM" --add-device --controller type=virtio-serial --check all,relax
fi

# Ensure SPICE agent channel (clipboard path)
if ! virsh dumpxml "$VM" | grep -q "channel type='spicevmc'"; then
  sudo virt-xml "$VM" --add-device --channel spicevmc,target.type=virtio,target.name=com.redhat.spice.0 --check all,relax
fi

# Prefer QXL video (virtio-gpu also works)
sudo virt-xml "$VM" --edit --video model.type=qxl --check all,relax

# Ensure USB tablet for better pointer sync
if ! virsh dumpxml "$VM" | grep -q "<input type='tablet'"; then
  sudo virt-xml "$VM" --add-device --input type=tablet,bus=usb --check all,relax
fi

echo "[5/6] Starting VM..."
virsh start "$VM"

echo "[6/6] Opening SPICE viewer..."
URI="$(virsh domdisplay "$VM")"
echo "SPICE URI: $URI"
# Launch viewer in background
remote-viewer "$URI" >/dev/null 2>&1 &
echo "Remote Viewer launched. In the viewer, press Ctrl+Alt+Shift and ensure 'Disable clipboard' is NOT checked."

echo "Done. Test copy/paste host <-> guest."
