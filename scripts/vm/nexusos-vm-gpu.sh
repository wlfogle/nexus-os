#!/usr/bin/env bash
# NexusOS VM — RTX 4080 GPU Passthrough
# Passes the physical RTX 4080 (IOMMU Group 16) directly to NexusOS.
# Use this when the ISO is ready for real hardware-level GPU testing.
#
# Prerequisites:
#   1. sudo ./scripts/vm/vfio-bind.sh    (detach GPU from nvidia)
#   2. Connect external display to laptop HDMI/DP (GPU output goes there)
#   3. Run this script
#   4. sudo ./scripts/vm/vfio-unbind.sh  (restore GPU after VM exits)
#
# Usage:
#   ./scripts/vm/nexusos-vm-gpu.sh              # boot installed disk
#   ./scripts/vm/nexusos-vm-gpu.sh --cdrom      # boot from ISO (install mode)

set -euo pipefail

VM_DIR="/media/loufogle/Data/vms/nexusos"
DISK="$VM_DIR/nexusos.qcow2"
ISO="/home/loufogle/nexus-os/build/nexusos-laptop.iso"
OVMF_CODE="/usr/share/OVMF/OVMF_CODE_4M.fd"
OVMF_VARS="$VM_DIR/OVMF_VARS.fd"
RAM="16G"
CPUS="8"

GPU_PCI="02:00.0"
AUD_PCI="02:00.1"

# ── Verify GPU is bound to vfio-pci ─────────────────────────────────────────
GPU_DRV=$(basename $(readlink /sys/bus/pci/devices/0000:$GPU_PCI/driver) 2>/dev/null || echo "NONE")
if [ "$GPU_DRV" != "vfio-pci" ]; then
    echo "ERROR: RTX 4080 is not bound to vfio-pci (current: $GPU_DRV)"
    echo "Run: sudo ./scripts/vm/vfio-bind.sh"
    exit 1
fi
echo "[ok] RTX 4080 bound to vfio-pci"

# ── Boot mode ────────────────────────────────────────────────────────────────
BOOT_EXTRA=""
if [[ "${1:-}" == "--cdrom" ]]; then
    BOOT_EXTRA="-cdrom $ISO -boot menu=on"
    echo "==> Boot mode: ISO install"
else
    echo "==> Boot mode: installed disk"
fi

echo "==> NexusOS VM (RTX 4080 passthrough)"
echo "    RAM: $RAM  CPUs: $CPUS"
echo "    GPU output: connect external display to laptop HDMI/DP"
echo ""

exec qemu-system-x86_64 \
    -enable-kvm \
    -cpu host,kvm=on,+invtsc,+topoext \
    -smp "$CPUS,sockets=1,cores=$CPUS,threads=1" \
    -m "$RAM" \
    -machine q35,accel=kvm \
    \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS" \
    \
    -drive file="$DISK",if=virtio,cache=writeback,discard=unmap \
    $BOOT_EXTRA \
    \
    -device vfio-pci,host=$GPU_PCI,multifunction=on,x-vga=true \
    -device vfio-pci,host=$AUD_PCI \
    -vga none \
    -nographic \
    -serial stdio \
    \
    -netdev user,id=net0,hostfwd=tcp::2222-:22 \
    -device virtio-net-pci,netdev=net0 \
    \
    -device virtio-rng-pci \
    -usb -device usb-tablet \
    \
    -name "NexusOS-GPU" \
    -no-reboot
