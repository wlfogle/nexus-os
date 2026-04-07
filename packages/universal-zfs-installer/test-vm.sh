#!/bin/bash

# ═══════════════════════════════════════════════════════════════════════════
# 🧪 VM Testing Environment for Universal ZFS Installer
# Uses MobaLiveCD for easy ISO/VM testing
# ═══════════════════════════════════════════════════════════════════════════

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="${SCRIPT_DIR}/test-env"
MOBALIVECD_DIR="${SCRIPT_DIR}/../mobalivecd-linux"
DISK_SIZE="${DISK_SIZE:-50G}"
MEMORY="${MEMORY:-8G}"
CPUS="${CPUS:-4}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log() { echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

check_dependencies() {
    log "Checking dependencies..."
    
    local missing=()
    
    command -v qemu-system-x86_64 &>/dev/null || missing+=("qemu-system-x86")
    command -v qemu-img &>/dev/null || missing+=("qemu-utils")
    command -v python3 &>/dev/null || missing+=("python3")
    
    if [ ${#missing[@]} -gt 0 ]; then
        warn "Missing dependencies: ${missing[*]}"
        read -p "Install them now? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sudo apt-get update -qq
            sudo apt-get install -y qemu-system-x86 qemu-utils ovmf python3 python3-gi gir1.2-gtk-4.0 gir1.2-adw-1
        else
            error "Cannot proceed without dependencies"
        fi
    fi
    
    log "✓ All dependencies installed"
}

setup_mobalivecd() {
    if [ ! -d "$MOBALIVECD_DIR" ]; then
        log "Cloning MobaLiveCD..."
        git clone -q https://github.com/wlfogle/mobalivecd-linux "$MOBALIVECD_DIR" || \
            error "Failed to clone MobaLiveCD"
    fi
    
    log "✓ MobaLiveCD ready"
}

download_iso() {
    log "Checking for Kubuntu ISO..."
    
    local iso_url="https://cdimage.ubuntu.com/kubuntu/releases/24.04/release/kubuntu-24.04.1-desktop-amd64.iso"
    local iso_file="${TEST_DIR}/kubuntu-24.04.1-desktop-amd64.iso"
    
    if [ -f "$iso_file" ]; then
        log "✓ ISO already downloaded"
        echo "$iso_file"
        return
    fi
    
    mkdir -p "$TEST_DIR"
    
    log "Downloading Kubuntu 24.04 ISO (this may take a while)..."
    wget -q --show-progress -O "$iso_file" "$iso_url" || {
        warn "Direct download failed, trying alternative..."
        # Try torrent or mirror
        error "Download failed. Please download manually: $iso_url"
    }
    
    log "✓ ISO downloaded"
    echo "$iso_file"
}

create_test_disk() {
    log "Creating virtual test disk (${DISK_SIZE})..."
    
    local disk_file="${TEST_DIR}/test-disk.qcow2"
    
    if [ -f "$disk_file" ]; then
        warn "Test disk already exists"
        read -p "Recreate it? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -f "$disk_file"
        else
            echo "$disk_file"
            return
        fi
    fi
    
    qemu-img create -f qcow2 "$disk_file" "$DISK_SIZE" || error "Failed to create disk"
    
    log "✓ Test disk created"
    echo "$disk_file"
}

create_installer_iso() {
    log "Creating installer data disk..."
    
    local data_dir="${TEST_DIR}/installer-data"
    local iso_file="${TEST_DIR}/installer.iso"
    
    mkdir -p "$data_dir"
    
    # Copy installer script
    cp "${SCRIPT_DIR}/install.sh" "$data_dir/"
    
    # Create auto-run script
    cat > "$data_dir/auto-install.sh" <<'AUTOINSTALL'
#!/bin/bash
# Auto-install script for VM testing

export TARGET_DISK=/dev/vda
export USERNAME=testuser
export USER_PASSWORD=testpass
export HOSTNAME=test-vm

cd /media/installer
sudo ./install.sh
AUTOINSTALL
    
    chmod +x "$data_dir/auto-install.sh"
    
    # Create README
    cat > "$data_dir/README.txt" <<'README'
VM Testing Environment
======================

The installer is available at: /media/installer/install.sh

To run the installer:
1. Open Konsole
2. cd /media/installer
3. sudo ./install.sh

Or use the auto-installer:
sudo /media/installer/auto-install.sh

Target disk: /dev/vda (50GB virtual disk)
README
    
    # Create ISO
    if command -v genisoimage &>/dev/null; then
        genisoimage -o "$iso_file" -R -J -V "INSTALLER" "$data_dir" &>/dev/null
    elif command -v mkisofs &>/dev/null; then
        mkisofs -o "$iso_file" -R -J -V "INSTALLER" "$data_dir" &>/dev/null
    else
        error "Need genisoimage or mkisofs. Install: sudo apt install genisoimage"
    fi
    
    log "✓ Installer ISO created"
    echo "$iso_file"
}

start_vm() {
    local kubuntu_iso=$1
    local test_disk=$2
    local installer_iso=$3
    
    log "Starting VM..."
    log "Memory: $MEMORY, CPUs: $CPUS, Disk: $DISK_SIZE"
    
    # Find OVMF firmware
    local ovmf_code="/usr/share/OVMF/OVMF_CODE.fd"
    local ovmf_vars="${TEST_DIR}/OVMF_VARS.fd"
    
    if [ ! -f "$ovmf_code" ]; then
        ovmf_code="/usr/share/qemu/OVMF_CODE.fd"
    fi
    
    if [ ! -f "$ovmf_code" ]; then
        error "OVMF firmware not found. Install: sudo apt install ovmf"
    fi
    
    # Create OVMF vars if needed
    if [ ! -f "$ovmf_vars" ]; then
        cp /usr/share/OVMF/OVMF_VARS.fd "$ovmf_vars" 2>/dev/null || \
        cp /usr/share/qemu/OVMF_VARS.fd "$ovmf_vars" 2>/dev/null || \
        error "Failed to copy OVMF vars"
    fi
    
    log "Launching QEMU..."
    log "Instructions:"
    log "  1. VM will boot from Kubuntu live ISO"
    log "  2. Once booted, installer is at: /media/installer/"
    log "  3. Run: cd /media/installer && sudo ./install.sh"
    log "  4. Or use: sudo /media/installer/auto-install.sh"
    log ""
    log "Press Enter to start VM..."
    read
    
    qemu-system-x86_64 \
        -enable-kvm \
        -cpu host \
        -smp $CPUS \
        -m $MEMORY \
        -drive if=pflash,format=raw,readonly=on,file="$ovmf_code" \
        -drive if=pflash,format=raw,file="$ovmf_vars" \
        -drive file="$test_disk",format=qcow2,if=virtio \
        -cdrom "$kubuntu_iso" \
        -drive file="$installer_iso",media=cdrom,readonly=on \
        -boot d \
        -vga virtio \
        -display gtk,gl=on \
        -netdev user,id=net0 \
        -device virtio-net-pci,netdev=net0 \
        -name "ZFS Installer Test VM" \
        -monitor stdio
}

start_installed_system() {
    local test_disk="${TEST_DIR}/test-disk.qcow2"
    
    if [ ! -f "$test_disk" ]; then
        error "No installed system found. Run test first."
    fi
    
    log "Booting installed system..."
    
    local ovmf_code="/usr/share/OVMF/OVMF_CODE.fd"
    local ovmf_vars="${TEST_DIR}/OVMF_VARS.fd"
    
    [ ! -f "$ovmf_code" ] && ovmf_code="/usr/share/qemu/OVMF_CODE.fd"
    
    qemu-system-x86_64 \
        -enable-kvm \
        -cpu host \
        -smp $CPUS \
        -m $MEMORY \
        -drive if=pflash,format=raw,readonly=on,file="$ovmf_code" \
        -drive if=pflash,format=raw,file="$ovmf_vars" \
        -drive file="$test_disk",format=qcow2,if=virtio \
        -vga virtio \
        -display gtk,gl=on \
        -netdev user,id=net0 \
        -device virtio-net-pci,netdev=net0 \
        -name "ZFS Installed System" \
        -monitor stdio
}

cleanup() {
    log "Cleaning up test environment..."
    
    if [ -d "$TEST_DIR" ]; then
        du -sh "$TEST_DIR"
        read -p "Delete test directory? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$TEST_DIR"
            log "✓ Cleaned up"
        fi
    fi
}

show_help() {
    cat <<EOF
🧪 VM Testing Environment for Universal ZFS Installer

Usage: $0 [command] [options]

Commands:
    test        Test with MobaLiveCD GUI (recommended)
    quick       Quick test - just launch ISO in VM
    qemu        Raw QEMU test mode (advanced)
    boot        Boot previously installed system
    clean       Clean up test environment
    help        Show this help

Options:
    DISK_SIZE   Virtual disk size (default: 50G)
    MEMORY      VM memory (default: 8G)
    CPUS        Number of CPUs (default: 4)

Examples:
    # Easy GUI test (recommended)
    ./test-vm.sh test
    
    # Quick launch
    ./test-vm.sh quick
    
    # Advanced QEMU mode
    MEMORY=16G CPUS=8 ./test-vm.sh qemu
    
    # Boot installed system
    ./test-vm.sh boot
    
    # Clean up
    ./test-vm.sh clean

Notes:
    - Uses MobaLiveCD for easy VM testing
    - Downloads ~4GB Kubuntu ISO on first run
    - Installer will be at /media/installer/ in VM
    - Run: sudo /media/installer/auto-install.sh
    - Test disk: /dev/vda (50GB virtual)
EOF
}

test_with_mobalivecd() {
    log "Setting up MobaLiveCD testing environment..."
    
    check_dependencies
    setup_mobalivecd
    
    local kubuntu_iso=$(download_iso)
    local test_disk=$(create_test_disk)
    local installer_iso=$(create_installer_iso)
    
    log "Launching MobaLiveCD GUI..."
    log ""
    log "Instructions:"
    log "  1. MobaLiveCD will open"
    log "  2. Select the Kubuntu ISO: $kubuntu_iso"
    log "  3. VM will boot from live ISO"
    log "  4. In VM, installer is at /media/installer/"
    log "  5. Run: sudo /media/installer/auto-install.sh"
    log ""
    log "Press Enter to launch MobaLiveCD..."
    read
    
    cd "$MOBALIVECD_DIR"
    python3 mobalivecd.py "$kubuntu_iso" || error "MobaLiveCD failed"
}

quick_test() {
    log "Quick test mode - GUI only..."
    
    check_dependencies
    setup_mobalivecd
    
    local kubuntu_iso=$(download_iso)
    
    log "Opening Kubuntu ISO in MobaLiveCD..."
    cd "$MOBALIVECD_DIR"
    python3 mobalivecd.py "$kubuntu_iso" &
    
    log "✓ VM launched! Installer will be at /media/installer/ in the VM"
}

main() {
    local command="${1:-test}"
    
    case "$command" in
        test)
            test_with_mobalivecd
            ;;
        quick)
            quick_test
            ;;
        qemu)
            # Original QEMU method
            check_dependencies
            local kubuntu_iso=$(download_iso)
            local test_disk=$(create_test_disk)
            local installer_iso=$(create_installer_iso)
            start_vm "$kubuntu_iso" "$test_disk" "$installer_iso"
            ;;
        boot)
            start_installed_system
            ;;
        clean)
            cleanup
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            error "Unknown command: $command (use 'help' for usage)"
            ;;
    esac
}

main "$@"
