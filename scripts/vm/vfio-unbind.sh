#!/usr/bin/env bash
# NexusOS VFIO Unbind — restores RTX 4080 to nvidia driver
# Run AFTER shutting down the NexusOS VM to give the GPU back to the host.

set -euo pipefail

GPU="0000:02:00.0"
AUD="0000:02:00.1"

echo "==> NexusOS VFIO Unbind — restoring GPU to host"

# Unbind from vfio-pci
if [ -e "/sys/bus/pci/devices/$GPU/driver" ]; then
    echo "$GPU" > /sys/bus/pci/devices/$GPU/driver/unbind 2>/dev/null || true
fi
if [ -e "/sys/bus/pci/devices/$AUD/driver" ]; then
    echo "$AUD" > /sys/bus/pci/devices/$AUD/driver/unbind 2>/dev/null || true
fi

# Remove IDs from vfio-pci so it won't re-claim them
echo "10de 27e0" > /sys/bus/pci/drivers/vfio-pci/remove_id 2>/dev/null || true
echo "10de 22bc" > /sys/bus/pci/drivers/vfio-pci/remove_id 2>/dev/null || true

# Trigger re-probe so nvidia reclaims the GPU
echo "$GPU" > /sys/bus/pci/drivers_probe 2>/dev/null || true
echo "$AUD" > /sys/bus/pci/drivers_probe 2>/dev/null || true

GPU_DRV=$(basename $(readlink /sys/bus/pci/devices/$GPU/driver) 2>/dev/null || echo "NONE")
AUD_DRV=$(basename $(readlink /sys/bus/pci/devices/$AUD/driver) 2>/dev/null || echo "NONE")

echo "GPU  $GPU  driver: $GPU_DRV"
echo "AUD  $AUD  driver: $AUD_DRV"
echo "==> Done. You may need to run 'modprobe nvidia' if the driver didn't reload."
