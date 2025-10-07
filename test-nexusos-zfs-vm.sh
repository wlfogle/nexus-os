#!/bin/bash

# NexusOS ZFS Edition VM Test Script
# Tests the newly built NexusOS with ZFS support

set -e

echo "🚀 Testing NexusOS v5 (ZFS Edition) in VM"

# Configuration
ISO_FILE="garuda-nexusos-zfs-linux-zen-251007.iso"
VM_NAME="nexusos-zfs-test"
VM_DISK="nexusos-zfs-test.qcow2"
VM_MEMORY="4G"
VM_CORES="4"

# Check if ISO exists
if [ ! -f "$ISO_FILE" ]; then
    echo "❌ Error: ISO file $ISO_FILE not found!"
    echo "Available ISO files:"
    ls -lh *.iso
    exit 1
fi

echo "✅ Found ISO: $ISO_FILE ($(du -h "$ISO_FILE" | cut -f1))"

# Create VM disk if it doesn't exist
if [ ! -f "$VM_DISK" ]; then
    echo "💾 Creating VM disk: $VM_DISK"
    qemu-img create -f qcow2 "$VM_DISK" 20G
else
    echo "💾 Using existing VM disk: $VM_DISK"
fi

# Copy UEFI VARS to writable location
VARS_FILE="${VM_NAME}-VARS.fd"
if [ ! -f "$VARS_FILE" ]; then
    echo "🔧 Creating UEFI variables file: $VARS_FILE"
    cp /usr/share/edk2/x64/OVMF_VARS.4m.fd "$VARS_FILE"
fi

# Verify ISO integrity
echo "🔍 Verifying ISO integrity..."
if [ -f "$ISO_FILE.sha256" ]; then
    sha256sum -c "$ISO_FILE.sha256" || { echo "❌ ISO integrity check failed!"; exit 1; }
    echo "✅ ISO integrity verified"
else
    echo "⚠️  No checksum file found, skipping integrity check"
fi

echo ""
echo "🖥️  Starting NexusOS ZFS Edition VM..."
echo "   ISO: $ISO_FILE"
echo "   Memory: $VM_MEMORY"
echo "   CPU Cores: $VM_CORES"
echo "   Disk: $VM_DISK"
echo ""
echo "Expected Features to Test:"
echo "   🗄️  ZFS support (zfs-dkms, zfs-utils)"
echo "   🖥️  KDE Plasma desktop with SDDM"
echo "   🎮 Gaming: Steam, Lutris, Wine"
echo "   🤖 AI/ML: Python, TensorFlow, PyTorch"
echo "   ⚡ Linux Zen kernel performance"
echo "   💾 SystemD-boot UEFI bootloader"
echo "   🔧 Development tools (gcc, make, etc.)"
echo ""
echo "🎮 VM Controls:"
echo "   Ctrl+Alt+G: Release mouse"
echo "   Ctrl+Alt+F: Toggle fullscreen"
echo "   Ctrl+Alt+1: Switch to QEMU monitor"
echo "   Ctrl+Alt+2: Switch back to VM"
echo ""
echo "Login Credentials:"
echo "   Username: nexus"
echo "   Password: nexus"
echo ""

# Launch QEMU with optimized settings for testing
exec qemu-system-x86_64 \
    -name "$VM_NAME" \
    -m "$VM_MEMORY" \
    -smp "$VM_CORES" \
    -cpu host \
    -enable-kvm \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2/x64/OVMF_CODE.4m.fd \
    -drive if=pflash,format=raw,file="$VARS_FILE" \
    -drive file="$VM_DISK",if=virtio,cache=writeback,discard=unmap \
    -cdrom "$ISO_FILE" \
    -boot order=d \
    -netdev user,id=net0,hostfwd=tcp::2222-:22 \
    -device virtio-net,netdev=net0 \
    -vga qxl \
    -device AC97 \
    -usb \
    -device usb-tablet \
    -monitor stdio \
    -rtc base=localtime,clock=host \
    -global kvm-pit.lost_tick_policy=delay