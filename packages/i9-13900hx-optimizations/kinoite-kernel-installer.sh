#!/bin/bash

# Kinoite Custom Kernel Installer
# Designed for Intel i9-13900HX optimized kernel on Fedora Kinoite
# Author: AI Assistant for wlfogle
# Date: 2025-09-08

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
KERNEL_VERSION="6.17.0-rc5-i9-13900hx-optimized-20250908"
KERNEL_RPM_DIR="${HOME}/kernel-rpms"
TEMP_EXTRACT_DIR="/tmp/kernel-install-$(date +%s)"
BACKUP_DIR="${HOME}/.kernel-backup-$(date +%Y%m%d-%H%M%S)"

echo -e "${BLUE}🚀 KINOITE CUSTOM KERNEL INSTALLER${NC}"
echo -e "${BLUE}===================================${NC}"
echo -e "${GREEN}Target Kernel: ${KERNEL_VERSION}${NC}"
echo -e "${GREEN}System: Fedora Kinoite (Immutable)${NC}"
echo

# Function to print status
print_status() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if running as root when needed
check_root() {
    if [[ $EUID -eq 0 ]]; then
        print_error "This script should NOT be run as root initially. It will use sudo when needed."
        exit 1
    fi
}

# Function to verify kernel RPMs exist
verify_kernel_rpms() {
    print_status "Verifying kernel RPM packages..."
    
    local kernel_rpm="${KERNEL_RPM_DIR}/kernel-6.17.0_rc5_i9_13900hx_optimized_20250908-4.x86_64.rpm"
    local devel_rpm="${KERNEL_RPM_DIR}/kernel-devel-6.17.0_rc5_i9_13900hx_optimized_20250908-4.x86_64.rpm"
    local headers_rpm="${KERNEL_RPM_DIR}/kernel-headers-6.17.0_rc5_i9_13900hx_optimized_20250908-4.x86_64.rpm"
    
    if [[ ! -f "$kernel_rpm" ]]; then
        print_error "Kernel RPM not found: $kernel_rpm"
        return 1
    fi
    
    if [[ ! -f "$devel_rpm" ]]; then
        print_error "Kernel devel RPM not found: $devel_rpm"
        return 1
    fi
    
    if [[ ! -f "$headers_rpm" ]]; then
        print_error "Kernel headers RPM not found: $headers_rpm"
        return 1
    fi
    
    print_success "All kernel RPMs found"
    return 0
}

# Function to create backup
create_backup() {
    print_status "Creating backup of current kernel files..."
    mkdir -p "$BACKUP_DIR"
    
    # Backup current kernel files if they exist
    if [[ -f "/boot/vmlinuz-$(uname -r)" ]]; then
        cp "/boot/vmlinuz-$(uname -r)" "$BACKUP_DIR/"
        cp "/boot/initramfs-$(uname -r).img" "$BACKUP_DIR/" 2>/dev/null || true
        cp "/boot/System.map-$(uname -r)" "$BACKUP_DIR/" 2>/dev/null || true
    fi
    
    print_success "Backup created at: $BACKUP_DIR"
}

# Function to extract kernel RPMs
extract_kernel_rpms() {
    print_status "Extracting kernel RPM packages..."
    
    mkdir -p "$TEMP_EXTRACT_DIR"
    cd "$TEMP_EXTRACT_DIR"
    
    # Extract main kernel package
    local kernel_rpm="${KERNEL_RPM_DIR}/kernel-6.17.0_rc5_i9_13900hx_optimized_20250908-4.x86_64.rpm"
    rpm2cpio "$kernel_rpm" | cpio -idm --quiet
    
    print_success "Kernel files extracted to: $TEMP_EXTRACT_DIR"
}

# Function to install kernel using bootupctl (if available) or traditional method
install_kernel_files() {
    print_status "Installing custom kernel files..."
    
    local extracted_kernel_dir="${TEMP_EXTRACT_DIR}/lib/modules/${KERNEL_VERSION}"
    
    if [[ ! -d "$extracted_kernel_dir" ]]; then
        print_error "Extracted kernel directory not found: $extracted_kernel_dir"
        return 1
    fi
    
    # Method 1: Try bootupctl (modern ostree approach)
    if command -v bootupctl &> /dev/null; then
        print_status "Using bootupctl for kernel installation..."
        
        # Copy kernel files to temporary location
        sudo mkdir -p "/tmp/kernel-staging"
        sudo cp "$extracted_kernel_dir/vmlinuz" "/tmp/kernel-staging/vmlinuz-$KERNEL_VERSION"
        sudo cp "$extracted_kernel_dir/System.map" "/tmp/kernel-staging/System.map-$KERNEL_VERSION"
        sudo cp "$extracted_kernel_dir/config" "/tmp/kernel-staging/config-$KERNEL_VERSION"
        
        # Install kernel
        sudo bootupctl install "/tmp/kernel-staging/vmlinuz-$KERNEL_VERSION"
        
        # Cleanup
        sudo rm -rf "/tmp/kernel-staging"
        
        print_success "Kernel installed using bootupctl"
        return 0
    fi
    
    # Method 2: Traditional manual installation (fallback)
    print_status "Using traditional manual installation..."
    
    # Create a script to handle the installation with proper permissions
    cat << 'EOF' > "/tmp/install_kernel.sh"
#!/bin/bash
set -e

KERNEL_VERSION="$1"
SOURCE_DIR="$2"

# Mount /boot as read-write if it's mounted read-only
if mount | grep -q "/boot.*ro,"; then
    mount -o remount,rw /boot
    REMOUNT_RO=1
else
    REMOUNT_RO=0
fi

# Install kernel files
cp "$SOURCE_DIR/vmlinuz" "/boot/vmlinuz-$KERNEL_VERSION"
cp "$SOURCE_DIR/System.map" "/boot/System.map-$KERNEL_VERSION"
cp "$SOURCE_DIR/config" "/boot/config-$KERNEL_VERSION"

# Set proper permissions
chmod 644 "/boot/vmlinuz-$KERNEL_VERSION"
chmod 644 "/boot/System.map-$KERNEL_VERSION"
chmod 644 "/boot/config-$KERNEL_VERSION"

# Generate initramfs
echo "Generating initramfs..."
dracut -f "/boot/initramfs-$KERNEL_VERSION.img" "$KERNEL_VERSION"

# Update bootloader entries
if command -v grub2-mkconfig &> /dev/null; then
    echo "Updating GRUB configuration..."
    grub2-mkconfig -o /boot/grub2/grub.cfg
elif command -v bootctl &> /dev/null; then
    echo "Updating systemd-boot configuration..."
    bootctl update
fi

# Remount /boot as read-only if it was originally
if [ "$REMOUNT_RO" -eq 1 ]; then
    mount -o remount,ro /boot
fi

echo "Kernel installation completed successfully!"
EOF
    
    chmod +x "/tmp/install_kernel.sh"
    
    # Execute the installation script with sudo
    sudo "/tmp/install_kernel.sh" "$KERNEL_VERSION" "$extracted_kernel_dir"
    
    # Cleanup
    rm "/tmp/install_kernel.sh"
    
    print_success "Kernel installed using traditional method"
}

# Function to create custom boot entry
create_boot_entry() {
    print_status "Creating custom boot entry..."
    
    local boot_entry_file="/boot/loader/entries/custom-${KERNEL_VERSION}.conf"
    
    # Check if systemd-boot is in use
    if [[ -d "/boot/loader/entries" ]]; then
        sudo tee "$boot_entry_file" > /dev/null << EOF
title    Fedora Kinoite Custom (${KERNEL_VERSION})
version  ${KERNEL_VERSION}
machine-id $(cat /etc/machine-id)
options  root=UUID=$(findmnt -no UUID /) rhgb quiet intel_iommu=on iommu=pt processor.max_cstate=1 intel_idle.max_cstate=0
linux    /vmlinuz-${KERNEL_VERSION}
initrd   /initramfs-${KERNEL_VERSION}.img
EOF
        print_success "Custom boot entry created: $boot_entry_file"
    else
        print_warning "systemd-boot entries directory not found. Boot entry creation skipped."
    fi
}

# Function to update rpm-ostree with kernel override
update_rpm_ostree() {
    print_status "Attempting to register kernel with rpm-ostree..."
    
    # Try to install kernel packages via rpm-ostree override
    if rpm-ostree override replace "${KERNEL_RPM_DIR}"/kernel-*.rpm &> /dev/null; then
        print_success "Kernel registered with rpm-ostree"
    else
        print_warning "Failed to register with rpm-ostree (this is expected and not critical)"
    fi
}

# Function to verify installation
verify_installation() {
    print_status "Verifying kernel installation..."
    
    local kernel_file="/boot/vmlinuz-${KERNEL_VERSION}"
    local initrd_file="/boot/initramfs-${KERNEL_VERSION}.img"
    local config_file="/boot/config-${KERNEL_VERSION}"
    
    if [[ -f "$kernel_file" ]]; then
        print_success "Kernel binary installed: $kernel_file"
        print_status "Kernel size: $(du -h "$kernel_file" | cut -f1)"
    else
        print_error "Kernel binary not found: $kernel_file"
        return 1
    fi
    
    if [[ -f "$initrd_file" ]]; then
        print_success "Initramfs installed: $initrd_file"
        print_status "Initramfs size: $(du -h "$initrd_file" | cut -f1)"
    else
        print_error "Initramfs not found: $initrd_file"
        return 1
    fi
    
    if [[ -f "$config_file" ]]; then
        print_success "Kernel config installed: $config_file"
    else
        print_warning "Kernel config not found: $config_file"
    fi
    
    return 0
}

# Function to cleanup
cleanup() {
    print_status "Cleaning up temporary files..."
    if [[ -d "$TEMP_EXTRACT_DIR" ]]; then
        rm -rf "$TEMP_EXTRACT_DIR"
    fi
    print_success "Cleanup completed"
}

# Function to show next steps
show_next_steps() {
    echo
    echo -e "${PURPLE}🎉 KERNEL INSTALLATION COMPLETED! 🎉${NC}"
    echo -e "${PURPLE}====================================${NC}"
    echo
    echo -e "${YELLOW}Next Steps:${NC}"
    echo -e "1. ${GREEN}Reboot your system${NC}: sudo systemctl reboot"
    echo -e "2. ${GREEN}Select custom kernel${NC} from boot menu if not default"
    echo -e "3. ${GREEN}Verify after reboot${NC}: uname -r"
    echo -e "4. ${GREEN}Expected version${NC}: ${KERNEL_VERSION}"
    echo
    echo -e "${YELLOW}Post-installation scripts:${NC}"
    echo -e "• ${CYAN}WireGuard setup${NC}: ./wireguard-dual-role-optimized.sh"
    echo -e "• ${CYAN}NVIDIA optimization${NC}: ./nvidia-wireguard-optimization.sh"
    echo -e "• ${CYAN}Performance testing${NC}: ./benchmark-custom-kernel.sh"
    echo
    echo -e "${YELLOW}Backup location${NC}: $BACKUP_DIR"
    echo -e "${YELLOW}Installation guide${NC}: ~/kernel-installation-guide.md"
    echo
    echo -e "${GREEN}🚀 Your Intel i9-13900HX optimized kernel is ready!${NC}"
}

# Main installation function
main() {
    print_status "Starting Kinoite custom kernel installation..."
    echo
    
    # Run pre-installation checks
    check_root
    
    # Verify prerequisites
    if ! verify_kernel_rpms; then
        print_error "Kernel RPM verification failed. Exiting."
        exit 1
    fi
    
    # Create backup
    create_backup
    
    # Extract kernel files
    extract_kernel_rpms
    
    # Install kernel
    if ! install_kernel_files; then
        print_error "Kernel installation failed. Check the logs above."
        cleanup
        exit 1
    fi
    
    # Create boot entry
    create_boot_entry
    
    # Verify installation
    if ! verify_installation; then
        print_error "Kernel installation verification failed."
        cleanup
        exit 1
    fi
    
    # Cleanup
    cleanup
    
    # Show next steps
    show_next_steps
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
