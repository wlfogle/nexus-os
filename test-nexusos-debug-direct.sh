#!/bin/bash

echo "🔍 NexusOS Direct Debug Test (Bypassing GRUB)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "This will boot NexusOS with maximum debugging directly"

# Configuration
ISO_FILE="nexusos-v4-debug-1.0.0-alpha-x86_64.iso"
DISK_FILE="nexusos-test-disk.qcow2"

# Create debug log directory
DEBUG_DIR="nexusos-direct-debug-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$DEBUG_DIR"

echo "🔍 Debug logs will be saved to: $DEBUG_DIR/"
echo "📊 Monitoring: full kernel boot, SystemD startup, service debugging"
echo "🔧 Serial console captures everything"
echo ""
echo "🚀 Starting VM with direct kernel debugging..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Direct boot with comprehensive kernel debugging
qemu-system-x86_64 \
    -name "NexusOS-DirectDebug" \
    -machine type=q35,accel=kvm \
    -cpu host \
    -smp 4 \
    -m 4G \
    -bios /usr/share/edk2/x64/OVMF.4m.fd \
    -drive file="$DISK_FILE",format=qcow2,if=virtio \
    -cdrom "$ISO_FILE" \
    -boot order=d \
    -netdev user,id=net0 \
    -device virtio-net-pci,netdev=net0 \
    -device virtio-vga-gl \
    -display gtk,gl=on \
    -device intel-hda \
    -device hda-duplex \
    -usb \
    -device usb-tablet \
    -rtc base=localtime \
    -serial file:"$DEBUG_DIR/serial.log" \
    -serial file:"$DEBUG_DIR/serial2.log" \
    -monitor stdio \
    -d guest_errors,unimp \
    -D "$DEBUG_DIR/qemu.log" \
    -no-reboot \
    2>&1 | tee "$DEBUG_DIR/qemu_output.log"

echo ""
echo "🔍 Debug Information Available:"
echo "   📄 Serial Console: $DEBUG_DIR/serial.log" 
echo "   📄 Secondary Serial: $DEBUG_DIR/serial2.log"
echo "   📄 QEMU Debug Log: $DEBUG_DIR/qemu.log"
echo "   📄 Full Output: $DEBUG_DIR/qemu_output.log"
