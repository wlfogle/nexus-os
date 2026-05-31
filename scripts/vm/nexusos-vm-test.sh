#!/usr/bin/env bash
# NexusOS VM — Quick ISO test (virtual display, no GPU passthrough)
# Use this for rapid dev iteration: build ISO → boot → check serial output.
# No VFIO needed. Runs on the laptop without any GPU rebinding.
#
# Usage:
#   ./scripts/vm/nexusos-vm-test.sh              # boot latest ISO
#   ./scripts/vm/nexusos-vm-test.sh path/to.iso  # boot specific ISO

set -euo pipefail

ISO="${1:-/home/loufogle/nexus-os/build/nexusos-laptop.iso}"
VM_DIR="/media/loufogle/Data/vms/nexusos"
DISK="$VM_DIR/nexusos.qcow2"
OVMF_CODE="/usr/share/OVMF/OVMF_CODE_4M.fd"
OVMF_VARS="$VM_DIR/OVMF_VARS.fd"
RAM="8G"
CPUS="4"

if [ ! -f "$ISO" ]; then
    echo "ERROR: ISO not found: $ISO"
    echo "Run 'make iso-laptop' first."
    exit 1
fi

echo "==> NexusOS VM (test mode, virtual display)"
echo "    ISO  : $ISO"
echo "    Disk : $DISK"
echo "    RAM  : $RAM  CPUs: $CPUS"
echo ""

exec qemu-system-x86_64 \
    -enable-kvm \
    -cpu host,+invtsc \
    -smp "$CPUS" \
    -m "$RAM" \
    -machine q35,accel=kvm \
    \
    -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE" \
    -drive if=pflash,format=raw,file="$OVMF_VARS" \
    \
    -drive file="$DISK",if=virtio,cache=writeback,discard=unmap \
    -cdrom "$ISO" \
    -boot menu=on \
    \
    -vga virtio \
    -display gtk,zoom-to-fit=on \
    -serial stdio \
    \
    -netdev user,id=net0,hostfwd=tcp::2222-:22 \
    -device virtio-net-pci,netdev=net0 \
    \
    -device virtio-rng-pci \
    -usb -device usb-tablet \
    \
    -name "NexusOS-test" \
    "$@"
