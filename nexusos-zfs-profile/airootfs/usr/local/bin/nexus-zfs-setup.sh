#!/bin/bash

# NexusOS ZFS Installation and Setup Script
# This script sets up ZFS root installation with boot environments

set -e

echo "🚀 NexusOS ZFS Setup"
echo "━━━━━━━━━━━━━━━━━━━━━"

# Check if running in live environment
if [ ! -f /run/archiso/bootmnt/.arch/pkglist.x86_64.txt ]; then
    echo "❌ This script must be run from the NexusOS live environment"
    exit 1
fi

# Prompt for target disk
echo ""
echo "📋 Available disks:"
lsblk -d -o NAME,SIZE,MODEL | grep -E '^sd|^nvme|^vd'

echo ""
read -p "Enter target disk (e.g., /dev/nvme0n1): " TARGET_DISK

if [ ! -b "$TARGET_DISK" ]; then
    echo "❌ Invalid disk: $TARGET_DISK"
    exit 1
fi

echo ""
echo "⚠️  WARNING: This will DESTROY all data on $TARGET_DISK"
read -p "Continue? (yes/no): " CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    echo "Installation cancelled."
    exit 1
fi

# Partition the disk
echo ""
echo "🔧 Partitioning disk..."
parted "$TARGET_DISK" --script -- \
    mklabel gpt \
    mkpart ESP fat32 1MiB 513MiB \
    set 1 esp on \
    mkpart primary 513MiB 100%

# Get partition names
if [[ "$TARGET_DISK" =~ nvme ]]; then
    ESP_PARTITION="${TARGET_DISK}p1"
    ZFS_PARTITION="${TARGET_DISK}p2"
else
    ESP_PARTITION="${TARGET_DISK}1"
    ZFS_PARTITION="${TARGET_DISK}2"
fi

echo "✅ Created partitions: $ESP_PARTITION (ESP), $ZFS_PARTITION (ZFS)"

# Format ESP
echo ""
echo "🔧 Formatting ESP..."
mkfs.fat -F 32 -n ESP "$ESP_PARTITION"

# Create ZFS pool
echo ""
echo "🔧 Creating ZFS pool 'rpool'..."
zpool create -f -o ashift=12 \
    -O mountpoint=none \
    -O canmount=off \
    -O compression=zstd \
    -O acltype=posixacl \
    -O xattr=sa \
    -O relatime=on \
    rpool "$ZFS_PARTITION"

# Create root filesystem datasets
echo ""
echo "🔧 Creating ZFS datasets..."

# Create ROOT container
zfs create -o mountpoint=none -o canmount=off rpool/ROOT

# Create first boot environment
CURRENT_BE="nexusos-$(date +%Y%m%d)"
zfs create -o mountpoint=/ -o canmount=noauto rpool/ROOT/"$CURRENT_BE"

# Create other datasets
zfs create -o mountpoint=/home rpool/home
zfs create -o mountpoint=/var -o canmount=off rpool/var
zfs create -o mountpoint=/var/log rpool/var/log
zfs create -o mountpoint=/var/cache rpool/var/cache
zfs create -o mountpoint=/opt rpool/opt

# Mount the root filesystem
echo ""
echo "🔧 Mounting filesystems..."
zfs set canmount=on rpool/ROOT/"$CURRENT_BE"
zpool export rpool
zpool import -d /dev/disk/by-id -R /mnt rpool
zfs mount rpool/ROOT/"$CURRENT_BE"

# Create mount points and mount ESP
mkdir -p /mnt/boot
mount "$ESP_PARTITION" /mnt/boot

# Mount other ZFS datasets
zfs mount -a

echo ""
echo "🔧 Installing NexusOS base system..."

# Copy the live system as base
rsync -aHAXx --exclude={/dev/*,/proc/*,/sys/*,/tmp/*,/run/*,/mnt/*,/media/*,/lost+found} / /mnt/

# Generate fstab for ESP only (ZFS handles its own mounting)
echo "# ESP mount point" > /mnt/etc/fstab
echo "$(blkid -s UUID -o value "$ESP_PARTITION") /boot vfat defaults,umask=0077 0 1" >> /mnt/etc/fstab

# Configure ZFS for boot
echo ""
echo "🔧 Configuring ZFS boot environment..."

# Set boot filesystem
zpool set bootfs=rpool/ROOT/"$CURRENT_BE" rpool

# Enable ZFS services
arch-chroot /mnt systemctl enable zfs-import-cache.service
arch-chroot /mnt systemctl enable zfs-mount.service
arch-chroot /mnt systemctl enable zfs.target

# Configure ZFSBootMenu
echo ""
echo "🔧 Configuring ZFSBootMenu..."

# Create ZFSBootMenu config
mkdir -p /mnt/etc/zfsbootmenu
cat > /mnt/etc/zfsbootmenu/config.yaml << ZBMEOF
Global:
  ManageImages: true
  BootMountPoint: /boot
  DracutConfDir: /etc/zfsbootmenu/dracut.conf.d
Components:
  ImageDir: /boot/EFI/zbm
  Versions: false
  Enabled: false
EFI:
  ImageDir: /boot/EFI/zbm
  Versions: false
  Enabled: true
Kernel:
  CommandLine: ro quiet loglevel=0
ZBMEOF

# Generate ZFSBootMenu images
arch-chroot /mnt generate-zbm --config /etc/zfsbootmenu/config.yaml

echo ""
echo "🎉 NexusOS ZFS installation completed!"
echo ""
echo "📋 Summary:"
echo "   💾 ZFS Pool: rpool on $ZFS_PARTITION"
echo "   🏠 Boot Environment: $CURRENT_BE"
echo "   💽 ESP: $ESP_PARTITION mounted at /boot"
echo "   🚀 Bootloader: ZFSBootMenu"
echo ""
echo "🔄 You can now reboot and boot directly from ZFS!"
echo "📸 Take snapshots with: zfs snapshot rpool/ROOT/$CURRENT_BE@snapshot-name"
echo "📋 Create new boot environments with: zfs clone rpool/ROOT/$CURRENT_BE@snapshot rpool/ROOT/new-be"

