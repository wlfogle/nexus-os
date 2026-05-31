#!/usr/bin/env bash
# NexusOS VFIO Bind — RTX 4080 (IOMMU Group 16)
# Detaches the GPU from the nvidia driver and binds it to vfio-pci.
# The Intel iGPU continues to drive the laptop display — desktop is unaffected.
#
# Run BEFORE launching the NexusOS VM with GPU passthrough.
# Run scripts/vm/vfio-unbind.sh to restore the GPU to the host.
#
# PCI IDs:
#   02:00.0  NVIDIA RTX 4080 Mobile  [10de:27e0]
#   02:00.1  NVIDIA Audio            [10de:22bc]

set -euo pipefail

GPU="0000:02:00.0"
AUD="0000:02:00.1"

echo "==> NexusOS VFIO Bind"

# ── 1. Verify IOMMU is active ─────────────────────────────────────────────────
if [ ! -d /sys/kernel/iommu_groups/16/devices ]; then
    echo "ERROR: IOMMU group 16 not found. Is intel_iommu=on in kernel cmdline?"
    exit 1
fi
echo "[ok] IOMMU group 16 present"

# ── 2. Load vfio-pci module ───────────────────────────────────────────────────
modprobe vfio_pci
echo "[ok] vfio-pci module loaded"

# ── 3. Unbind GPU from nvidia driver ─────────────────────────────────────────
if [ -e "/sys/bus/pci/devices/$GPU/driver" ]; then
    CURRENT=$(basename $(readlink /sys/bus/pci/devices/$GPU/driver))
    echo "==> Unbinding GPU from: $CURRENT"
    echo "$GPU" > /sys/bus/pci/devices/$GPU/driver/unbind
fi

# ── 4. Unbind audio from snd_hda_intel ───────────────────────────────────────
if [ -e "/sys/bus/pci/devices/$AUD/driver" ]; then
    CURRENT=$(basename $(readlink /sys/bus/pci/devices/$AUD/driver))
    echo "==> Unbinding audio from: $CURRENT"
    echo "$AUD" > /sys/bus/pci/devices/$AUD/driver/unbind
fi

# ── 5. Register IDs with vfio-pci and bind ───────────────────────────────────
echo "10de 27e0" > /sys/bus/pci/drivers/vfio-pci/new_id 2>/dev/null || true
echo "10de 22bc" > /sys/bus/pci/drivers/vfio-pci/new_id 2>/dev/null || true

echo "$GPU" > /sys/bus/pci/drivers/vfio-pci/bind 2>/dev/null || true
echo "$AUD" > /sys/bus/pci/drivers/vfio-pci/bind 2>/dev/null || true

# ── 6. Verify ─────────────────────────────────────────────────────────────────
GPU_DRV=$(basename $(readlink /sys/bus/pci/devices/$GPU/driver) 2>/dev/null || echo "NONE")
AUD_DRV=$(basename $(readlink /sys/bus/pci/devices/$AUD/driver) 2>/dev/null || echo "NONE")

echo ""
echo "GPU  $GPU  driver: $GPU_DRV"
echo "AUD  $AUD  driver: $AUD_DRV"

if [ "$GPU_DRV" = "vfio-pci" ] && [ "$AUD_DRV" = "vfio-pci" ]; then
    echo ""
    echo "==> RTX 4080 bound to vfio-pci. Ready for VM passthrough."
else
    echo ""
    echo "WARNING: Not all devices bound to vfio-pci."
fi
