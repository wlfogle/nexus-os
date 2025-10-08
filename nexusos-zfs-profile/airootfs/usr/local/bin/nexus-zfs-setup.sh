#!/bin/bash
set -euo pipefail

# NexusOS ZFS Installation Script with ZFSBootMenu
# This script sets up a proper ZFS root system with boot environments

echo "=== NexusOS ZFS Installation with ZFSBootMenu ==="

# Check if we're running as root
if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root" 
   exit 1
fi

# Function to prompt for user input with default
prompt_with_default() {
    local prompt="$1"
    local default="$2"
    local result
    
    read -p "$prompt [$default]: " result
    echo "${result:-$default}"
}

# Get disk information
echo "Available disks:"
lsblk -d -o NAME,SIZE,MODEL
echo

DISK=$(prompt_with_default "Enter disk to install to (e.g. /dev/sda)" "/dev/sda")
HOSTNAME=$(prompt_with_default "Enter hostname" "nexusos")
USERNAME=$(prompt_with_default "Enter username" "user")

# Confirm destructive operation
echo
echo "WARNING: This will DESTROY all data on $DISK"
read -p "Are you sure? (type 'yes' to continue): " confirm
if [[ "$confirm" != "yes" ]]; then
    echo "Installation cancelled."
    exit 1
fi

echo "Starting ZFS installation..."

# Load ZFS module
modprobe zfs

# Partition the disk
echo "Partitioning disk..."
sgdisk --zap-all "$DISK"
sgdisk -n1:1M:+512M -t1:EF00 "$DISK"  # EFI boot
sgdisk -n2:0:0 -t2:BF00 "$DISK"       # ZFS root

# Get partition names
if [[ "$DISK" =~ nvme|loop|mmcblk ]]; then
    EFI_PARTITION="${DISK}p1"
    ZFS_PARTITION="${DISK}p2"
else
    EFI_PARTITION="${DISK}1"
    ZFS_PARTITION="${DISK}2"
fi

# Format EFI partition
echo "Formatting EFI partition..."
mkfs.fat -F32 "$EFI_PARTITION"

# Create ZFS pool with proper settings for root filesystem
echo "Creating ZFS pool..."
zpool create -o ashift=12 \
    -o autotrim=on \
    -O acltype=posixacl \
    -O canmount=off \
    -O compression=zstd \
    -O dnodesize=auto \
    -O normalization=formD \
    -O relatime=on \
    -O xattr=sa \
    -O mountpoint=none \
    zroot "$ZFS_PARTITION"

# Create boot environment structure
echo "Creating boot environments..."

# Root dataset (container)
zfs create -o canmount=off -o mountpoint=none zroot/ROOT

# Default boot environment
zfs create -o canmount=noauto -o mountpoint=/ zroot/ROOT/default
zfs mount zroot/ROOT/default

# Set bootfs property for ZFSBootMenu
zpool set bootfs=zroot/ROOT/default zroot

# Create other datasets
echo "Creating additional datasets..."
zfs create -o mountpoint=/home zroot/home
zfs create -o mountpoint=/var zroot/var
zfs create -o mountpoint=/var/log zroot/var/log
zfs create -o mountpoint=/var/cache zroot/var/cache
zfs create -o mountpoint=/tmp zroot/tmp
chmod 1777 /mnt/tmp

# Create swap volume
echo "Creating swap..."
zfs create -V 4G -b $(getconf PAGESIZE) \
    -o compression=zle \
    -o logbias=throughput \
    -o sync=always \
    -o primarycache=metadata \
    -o secondarycache=none \
    -o com.sun:auto-snapshot=false \
    zroot/swap
mkswap -f /dev/zvol/zroot/swap

# Mount EFI partition
mkdir -p /mnt/boot/efi
mount "$EFI_PARTITION" /mnt/boot/efi

# Install base system
echo "Installing base system..."
pacstrap /mnt base linux-zen linux-zen-headers linux-firmware \
    base-devel git vim nano networkmanager openssh sudo \
    zfs-dkms zfs-utils

# Generate fstab (without ZFS entries, they're handled by ZFS)
genfstab -U /mnt | grep -v zroot > /mnt/etc/fstab

# Add EFI and swap entries
echo "UUID=$(blkid -s UUID -o value $EFI_PARTITION) /boot/efi vfat defaults 0 2" >> /mnt/etc/fstab
echo "/dev/zvol/zroot/swap none swap discard 0 0" >> /mnt/etc/fstab

# Configure system in chroot
arch-chroot /mnt /bin/bash -c "
    # Set hostname
    echo '$HOSTNAME' > /etc/hostname
    
    # Configure hosts
    cat > /etc/hosts << EOF
127.0.0.1   localhost
::1         localhost
127.0.1.1   $HOSTNAME
EOF

    # Set timezone
    ln -sf /usr/share/zoneinfo/UTC /etc/localtime
    hwclock --systohc
    
    # Configure locale
    echo 'en_US.UTF-8 UTF-8' >> /etc/locale.gen
    locale-gen
    echo 'LANG=en_US.UTF-8' > /etc/locale.conf
    
    # Enable services
    systemctl enable NetworkManager
    systemctl enable sshd
    systemctl enable zfs.target
    systemctl enable zfs-import-cache
    systemctl enable zfs-mount
    systemctl enable zfs-import.target
    
    # Create user
    useradd -m -G wheel '$USERNAME'
    echo '$USERNAME ALL=(ALL) ALL' >> /etc/sudoers
    
    # Set passwords
    echo 'Set root password:'
    passwd
    echo 'Set user password for $USERNAME:'
    passwd '$USERNAME'
"

# Install and configure ZFSBootMenu
echo "Installing ZFSBootMenu..."

# Install ZFSBootMenu from AUR in chroot
arch-chroot /mnt /bin/bash -c "
    # Create temporary user for AUR builds
    useradd -m -G wheel -s /bin/bash aurbuilder
    echo 'aurbuilder ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers
    
    # Install ZFSBootMenu
    sudo -u aurbuilder bash -c '
        cd /tmp
        git clone https://aur.archlinux.org/zfsbootmenu.git
        cd zfsbootmenu
        makepkg -si --noconfirm
    '
    
    # Clean up
    userdel -r aurbuilder
    sed -i '/aurbuilder/d' /etc/sudoers
"

# Configure ZFSBootMenu
arch-chroot /mnt /bin/bash -c "
    # Create ZFSBootMenu config
    mkdir -p /etc/zfsbootmenu
    cat > /etc/zfsbootmenu/config.yaml << EOF
Global:
  ManageImages: true
  BootMountPoint: /boot/efi
  ImageDir: /boot/efi/EFI/zbm
  
Components:
  Enabled: true
  Versions: false
  
EFI:
  ImageDir: /boot/efi/EFI/zbm
  Versions: false
  Enabled: true
  
Kernel:
  CommandLine: ro quiet loglevel=4
EOF

    # Set ZFS properties for boot environment
    zfs set org.zfsbootmenu:commandline='ro quiet loglevel=4' zroot/ROOT/default
    
    # Generate ZFSBootMenu
    generate-zbm
    
    # Create EFI boot entries
    efibootmgr --create --disk $DISK --part 1 --loader '\EFI\zbm\zfsbootmenu.efi' --label 'ZFSBootMenu'
"

# Configure mkinitcpio for ZFS
arch-chroot /mnt /bin/bash -c "
    # Update mkinitcpio.conf for ZFS
    sed -i 's/^HOOKS=.*/HOOKS=(base udev autodetect keyboard keymap modconf block filesystems fsck)/' /etc/mkinitcpio.conf
    sed -i 's/^MODULES=.*/MODULES=(zfs)/' /etc/mkinitcpio.conf
    
    # Rebuild initramfs
    mkinitcpio -P
"

# Enable ZFS services and import cache
arch-chroot /mnt /bin/bash -c "
    # Generate hostid
    zgenhostid
    
    # Create zpool cache
    zpool set cachefile=/etc/zfs/zpool.cache zroot
"

echo "Installation complete!"
echo
echo "Boot environments created:"
echo "- zroot/ROOT/default (current)"
echo
echo "To create a new boot environment:"
echo "  zfs snapshot zroot/ROOT/default@backup"
echo "  zfs clone zroot/ROOT/default@backup zroot/ROOT/new-version"
echo "  zpool set bootfs=zroot/ROOT/new-version zroot"
echo
echo "System will boot using ZFSBootMenu."
echo "Reboot to start using your new ZFS system!"

# Unmount everything
umount /mnt/boot/efi
zfs umount -a
zpool export zroot