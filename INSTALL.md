# Installation Guide - Calamares ZFS Integration

## Quick Start

### 1. Download and Extract Calamares
```bash
# Download Calamares 3.3.14
wget https://github.com/calamares/calamares/releases/download/v3.3.14/calamares-3.3.14.tar.gz
tar -xzf calamares-3.3.14.tar.gz
cd calamares-3.3.14
```

### 2. Install Dependencies
```bash
# On Garuda Linux / Arch Linux
sudo pacman -S --needed cmake qt6-base qt6-tools extra-cmake-modules yaml-cpp boost kpmcore polkit-qt6 python python-yaml base-devel
```

### 3. Apply ZFS Integration
```bash
# Clone this repository
git clone https://github.com/yourusername/calamares-zfs-integration.git

# Copy the enhanced modules
cp -r calamares-zfs-integration/modules/zfspostcfg src/modules/
cp calamares-zfs-integration/modules/zfs.conf src/modules/zfs/
```

### 4. Build and Install
```bash
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DWITH_QT6=ON
make -j$(nproc)
sudo make install
```

### 5. Configure Calamares
```bash
# Copy the ZFS-enabled settings
sudo cp calamares-zfs-integration/settings/settings-zfs.conf /etc/calamares/settings.conf

# Ensure ZFS kernel module is available
sudo modprobe zfs
```

## Detailed Installation

### Prerequisites Check

Before installing, verify your system has the necessary components:

```bash
# Check ZFS availability
modinfo zfs

# Check required packages
pacman -Q cmake qt6-base kpmcore python yaml-cpp

# Check available disk space (minimum 10GB recommended)
df -h
```

### Build Configuration Options

The cmake configuration supports various options:

```bash
# Release build (recommended for production)
cmake .. -DCMAKE_BUILD_TYPE=Release -DWITH_QT6=ON

# Debug build (for development)
cmake .. -DCMAKE_BUILD_TYPE=Debug -DWITH_QT6=ON -DCMAKE_INSTALL_PREFIX=/usr/local

# With additional features
cmake .. -DCMAKE_BUILD_TYPE=Release -DWITH_QT6=ON -DWITH_PYTHONQT=OFF -DWITH_KF6=ON
```

### Module Installation Verification

After installation, verify the ZFS modules are properly installed:

```bash
# Check module files
ls -la /usr/lib/calamares/modules/zfs*
ls -la /usr/lib/calamares/modules/zfspostcfg/

# Verify Python module syntax
python3 -m py_compile /usr/lib/calamares/modules/zfspostcfg/main.py

# Check configuration files
ls -la /usr/share/calamares/modules/zfs*/
```

### System Integration

#### 1. Systemd Service Files
Ensure Calamares can be run as a service:
```bash
# Copy service file if needed
sudo systemctl enable calamares.service
```

#### 2. Polkit Rules
Verify polkit permissions for ZFS operations:
```bash
# Check polkit rules
ls -la /usr/share/polkit-1/actions/com.github.calamares.calamares.policy
```

#### 3. Desktop Integration
For GUI environments:
```bash
# Verify desktop file
ls -la /usr/share/applications/calamares.desktop
```

## Configuration

### ZFS Module Configuration

Edit `/usr/share/calamares/modules/zfs/zfs.conf`:

```yaml
# Pool name for your installation
poolName: rpool

# Pool creation options
poolOptions: "-f -o ashift=12 -O mountpoint=none -O acltype=posixacl -O relatime=on -O compression=zstd -O xattr=sa -O autotrim=on"

# Dataset creation options  
datasetOptions: "-o compression=zstd -o atime=off -o xattr=sa"

# Dataset layout
datasets:
    - dsName: ROOT
      mountpoint: none
      canMount: off
    - dsName: ROOT/garuda
      mountpoint: none  
      canMount: off
    - dsName: ROOT/garuda/root
      mountpoint: /
      canMount: noauto
    - dsName: home
      mountpoint: /home
      canMount: on
    - dsName: var
      mountpoint: /var
      canMount: on
    - dsName: var/log
      mountpoint: /var/log
      canMount: on
    - dsName: var/cache
      mountpoint: /var/cache
      canMount: on
```

### Post-Configuration Module

Edit `/usr/share/calamares/modules/zfspostcfg/zfspostcfg.conf`:

```yaml
# Default pool name (will be overridden by global storage values)
poolName: "rpool"

# ZFS services to enable
services:
    - "zfs-import-cache.service"
    - "zfs-import.target"
    - "zfs-mount.service"
    - "zfs.target"
```

### Main Settings Configuration

Update `/etc/calamares/settings.conf` to include ZFS modules:

```yaml
sequence:
- show:
  - welcome
  - locale  
  - keyboard
  - partition
  - users
  - summary
- exec:
  - partition
  - zfs           # ZFS pool and dataset creation
  - mount         # Mount filesystems
  - unpackfs      # Unpack the filesystem
  - machineid     # Generate machine ID
  - locale        # Set locale
  - keyboard      # Set keyboard
  - localecfg     # Configure locale
  - fstab         # Generate fstab
  - zfshostid     # Copy ZFS hostid
  - zfspostcfg    # ZFS post-installation configuration (NEW)
  - initcpiocfg   # Configure mkinitcpio
  - initcpio      # Generate initcpio
  - users         # Create users
  - displaymanager # Configure display manager
  - networkcfg    # Configure network
  - hwclock       # Set hardware clock
  - services-systemd # Enable systemd services
  - initramfs     # Generate initramfs
  - bootloader    # Install bootloader
  - umount        # Unmount filesystems
```

## Testing

### Pre-Installation Tests

Before using in production:

```bash
# Test ZFS module loading
sudo modprobe zfs
lsmod | grep zfs

# Test pool creation (on spare device)
sudo zpool create test /dev/sdX
sudo zpool destroy test

# Validate configuration files
python3 -c "import yaml; yaml.safe_load(open('/usr/share/calamares/modules/zfs/zfs.conf'))"
```

### Installation Testing

Test the installation process:

```bash
# Run Calamares in debug mode
sudo calamares -d

# Check log files
tail -f ~/.cache/calamares/session.log
```

### Post-Installation Verification

After a successful installation:

```bash
# Check ZFS status
zpool status
zfs list

# Verify services are enabled
systemctl is-enabled zfs.target
systemctl is-enabled zfs-import-cache.service

# Check boot configuration
cat /etc/mkinitcpio.conf | grep zfs
```

## Troubleshooting

### Build Issues

**CMake configuration fails**:
```bash
# Install missing dependencies
sudo pacman -S cmake extra-cmake-modules

# Clear build directory
rm -rf build && mkdir build && cd build
```

**Missing Qt6 components**:
```bash
sudo pacman -S qt6-base qt6-tools qt6-declarative
```

### Runtime Issues

**ZFS module not found**:
```bash
# Install ZFS
sudo pacman -S zfs-dkms

# Load module
sudo modprobe zfs

# Add to module loading
echo "zfs" | sudo tee /etc/modules-load.d/zfs.conf
```

**Permission errors**:
```bash
# Check polkit rules
sudo systemctl status polkit
```

### Installation Issues

**Pool creation fails**:
- Verify disk is not in use
- Check ZFS is loaded: `lsmod | grep zfs`
- Ensure sufficient permissions

**Service enablement fails**:
- Check if services exist: `systemctl list-unit-files | grep zfs`
- Verify chroot environment: `ls -la /mnt/etc/systemd/system/`

## Support

If you encounter issues:

1. Check the main README.md for common solutions
2. Review log files in `~/.cache/calamares/`
3. Test individual components manually
4. Open an issue with detailed logs and system information