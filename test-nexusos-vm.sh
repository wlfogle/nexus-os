#!/bin/bash

# NexusOS Virtual Machine Testing Script
# This script launches NexusOS in a VM for installation testing

set -e

echo "🚀 NexusOS Virtual Machine Test Environment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Configuration
ISO_FILE="nexusos-v4-debug-1.0.0-alpha-x86_64.iso"
DISK_FILE="nexusos-test-disk.qcow2"
VM_NAME="NexusOS-Test"
MEMORY="4G"
CPUS="4"

# Check if files exist
if [ ! -f "$ISO_FILE" ]; then
    echo "❌ Error: NexusOS ISO not found: $ISO_FILE"
    exit 1
fi

if [ ! -f "$DISK_FILE" ]; then
    echo "❌ Error: Virtual disk not found: $DISK_FILE"
    exit 1
fi

echo "📦 ISO File: $ISO_FILE ($(du -h "$ISO_FILE" | cut -f1))"
echo "💾 Virtual Disk: $DISK_FILE (50GB)"
echo "🧠 Memory: $MEMORY"
echo "⚡ CPUs: $CPUS"
echo ""

# Function to test NexusOS with QEMU/KVM and comprehensive debugging
launch_qemu() {
    echo "🖥️  Launching NexusOS with QEMU/KVM + Full Debug Mode..."
    echo "   - Full UEFI boot support"
    echo "   - Hardware acceleration (KVM)"
    echo "   - Network connectivity"
    echo "   - Audio support"
    echo "   - Serial console logging"
    echo "   - System call monitoring"
    echo ""
    
    # Create debug log directory with timestamp
    DEBUG_DIR="nexusos-debug-$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$DEBUG_DIR"
    
    echo "🔍 Debug logs will be saved to: $DEBUG_DIR/"
    echo "📊 Monitoring: boot process, kernel messages, service startup"
    echo "🔧 Press Ctrl+Alt+2 in QEMU to access monitor console"
    echo "📝 Serial console output will be in: $DEBUG_DIR/serial.log"
    echo ""
    
    echo "🚀 Starting VM with enhanced debugging..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Enhanced QEMU with comprehensive debugging
    qemu-system-x86_64 \
        -name "$VM_NAME-Debug" \
        -machine type=q35,accel=kvm \
        -cpu host \
        -smp $CPUS \
        -m $MEMORY \
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
        -monitor stdio \
        -d guest_errors,unimp \
        -D "$DEBUG_DIR/qemu.log" \
        -no-reboot \
        -chardev socket,id=monitor,path="$DEBUG_DIR/monitor.sock",server=on,wait=off \
        -mon chardev=monitor,mode=readline \
        2>&1 | tee "$DEBUG_DIR/qemu_output.log"
    
    echo ""
    echo "🔍 Debug Information Available:"
    echo "   📄 Serial Console: $DEBUG_DIR/serial.log"
    echo "   📄 QEMU Debug Log: $DEBUG_DIR/qemu.log"
    echo "   📄 Full Output: $DEBUG_DIR/qemu_output.log"
    echo "   🔌 Monitor Socket: $DEBUG_DIR/monitor.sock"
}

# Function to test NexusOS with VirtualBox
launch_virtualbox() {
    echo "📦 Setting up VirtualBox VM for NexusOS..."
    
    # Create VM if it doesn't exist
    if ! VBoxManage list vms | grep -q "\"$VM_NAME\""; then
        echo "   Creating new VirtualBox VM: $VM_NAME"
        VBoxManage createvm --name "$VM_NAME" --register --ostype "Linux_64"
        VBoxManage modifyvm "$VM_NAME" \
            --memory 4096 \
            --cpus $CPUS \
            --vram 128 \
            --graphicscontroller vmsvga \
            --accelerate3d on \
            --firmware efi \
            --chipset ich9 \
            --ioapic on \
            --boot1 dvd \
            --boot2 disk \
            --boot3 none \
            --boot4 none \
            --audio-driver pulse \
            --audio-controller hda \
            --nic1 nat
        
        # Create and attach storage
        VBoxManage storagectl "$VM_NAME" --name "SATA" --add sata --controller IntelAhci
        VBoxManage storageattach "$VM_NAME" --storagectl "SATA" --port 0 --device 0 \
            --type hdd --medium "$PWD/$DISK_FILE"
        VBoxManage storageattach "$VM_NAME" --storagectl "SATA" --port 1 --device 0 \
            --type dvddrive --medium "$PWD/$ISO_FILE"
    else
        echo "   VM already exists, updating settings..."
        VBoxManage storageattach "$VM_NAME" --storagectl "SATA" --port 1 --device 0 \
            --type dvddrive --medium "$PWD/$ISO_FILE"
    fi
    
    echo "🖥️  Starting VirtualBox VM..."
    VBoxManage startvm "$VM_NAME" --type gui
}

# Function to show manual instructions
show_manual_instructions() {
    echo "📋 Manual VM Setup Instructions"
    echo ""
    echo "For VMware Workstation/Player:"
    echo "  1. Create new VM with Linux/Other Linux 5.x kernel 64-bit"
    echo "  2. Set memory to 4GB, 4 CPU cores"
    echo "  3. Use existing virtual disk: $PWD/$DISK_FILE"
    echo "  4. Set CD/DVD to: $PWD/$ISO_FILE"
    echo "  5. Enable UEFI boot mode"
    echo "  6. Start VM and boot from CD/DVD"
    echo ""
    echo "For MobaLiveCD:"
    echo "  1. Download MobaLiveCD from mobatek.net"
    echo "  2. Select ISO: $PWD/$ISO_FILE"
    echo "  3. Click 'Run LiveCD'"
    echo "  4. For installation testing, you'll need to set up"
    echo "     a separate VM with the virtual disk attached"
    echo ""
    echo "VM Specifications:"
    echo "  💾 Disk: $PWD/$DISK_FILE (50GB, qcow2 format)"
    echo "  📀 ISO: $PWD/$ISO_FILE (4.0GB)"
    echo "  🧠 RAM: 4GB minimum (8GB recommended)"
    echo "  ⚡ CPU: 4 cores minimum"
    echo "  🔧 Boot: UEFI mode required for ZFS support"
}

# Main menu
echo "Choose testing method:"
echo "  1) QEMU/KVM (recommended)"
echo "  2) VirtualBox"
echo "  3) Show manual setup instructions"
echo ""
read -p "Enter choice (1-3): " choice

case $choice in
    1)
        if command -v qemu-system-x86_64 >/dev/null; then
            launch_qemu
        else
            echo "❌ QEMU not found. Install with: sudo pacman -S qemu-desktop"
            exit 1
        fi
        ;;
    2)
        if command -v VBoxManage >/dev/null; then
            launch_virtualbox
        else
            echo "❌ VirtualBox not found. Install with: sudo pacman -S virtualbox"
            exit 1
        fi
        ;;
    3)
        show_manual_instructions
        ;;
    *)
        echo "❌ Invalid choice"
        exit 1
        ;;
esac

echo ""
echo "✨ NexusOS VM Test Environment Ready!"
echo ""
echo "🧪 Installation Testing Steps:"
echo "  1. Boot from the NexusOS ISO"
echo "  2. Test the live environment"
echo "  3. Run the installer to test disk installation"
echo "  4. Test ZFS root filesystem setup"
echo "  5. Verify all features work post-install"
echo ""
echo "💡 Tips:"
echo "  - The virtual disk supports ZFS root filesystem"
echo "  - Test the universal package manager (nexuspkg)"
echo "  - Verify AI/ML libraries and gaming features"
echo "  - Check KDE Plasma desktop functionality"