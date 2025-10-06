# Calamares ZFS Integration for Garuda Linux

This repository contains enhanced ZFS support for Calamares 3.3.14, specifically optimized for Garuda Linux. The integration provides comprehensive ZFS pool creation, dataset management, and post-installation system configuration.

## Overview

Calamares 3.3.14 already includes basic ZFS support, but this project enhances it with:

- **Enhanced ZFS Configuration**: Optimized for Garuda Linux with better dataset layouts
- **Post-Installation Module**: Comprehensive system configuration for ZFS
- **Service Management**: Automatic enablement of ZFS systemd services
- **Initramfs Integration**: Proper mkinitcpio configuration for ZFS support
- **System Optimization**: ZFS module parameters and cache configuration

## Architecture

### Existing Calamares ZFS Modules
- `zfs` (C++ module): Creates ZFS pools and datasets
- `zfshostid` (Python module): Copies ZFS hostid file

### New Enhanced Modules
- `zfspostcfg` (Python module): Comprehensive post-installation configuration

## Features

### ZFS Pool Configuration
- **Pool Name**: `rpool` (configurable)
- **Compression**: zstd compression by default
- **Properties**: ashift=12, autotrim=on, acltype=posixacl
- **Dataset Layout**: Optimized for Garuda Linux

### Dataset Structure
```
rpool/
├── ROOT/
│   └── garuda/
│       └── root (mountpoint: /)
├── home (mountpoint: /home)
├── var (mountpoint: /var)
│   ├── log (mountpoint: /var/log)
│   └── cache (mountpoint: /var/cache)
```

### Post-Installation Configuration
- **Systemd Services**: Enables zfs.target, zfs-import-cache, zfs-mount, zfs-import.target
- **Initramfs**: Configures mkinitcpio with ZFS hooks and modules
- **System Configuration**: Sets up ZFS module parameters and cache files
- **Hostid Management**: Ensures proper ZFS hostid configuration

## Installation

### Prerequisites
- Calamares 3.3.14 or later
- ZFS kernel modules
- Garuda Linux (or Arch-based system)
- CMake, Qt6, and development tools

### Building Calamares with ZFS Support

1. **Extract Calamares source**:
   ```bash
   tar -xzf calamares-3.3.14.tar.gz
   cd calamares-3.3.14
   ```

2. **Copy the enhanced ZFS modules**:
   ```bash
   # Copy the zfspostcfg module
   cp -r /path/to/this/repo/modules/zfspostcfg src/modules/
   
   # Replace the ZFS configuration
   cp /path/to/this/repo/modules/zfs.conf src/modules/zfs/
   ```

3. **Build Calamares**:
   ```bash
   mkdir build && cd build
   cmake .. -DCMAKE_BUILD_TYPE=Debug -DWITH_QT6=ON
   make -j$(nproc)
   ```

4. **Install**:
   ```bash
   sudo make install
   ```

### Configuration

Use the provided `settings-zfs.conf` as your Calamares settings file to enable ZFS support:

```bash
sudo cp /path/to/this/repo/settings/settings-zfs.conf /etc/calamares/settings.conf
```

## Module Configuration

### zfs.conf (Enhanced)
```yaml
poolName: rpool
poolOptions: "-f -o ashift=12 -O mountpoint=none -O acltype=posixacl -O relatime=on -O compression=zstd -O xattr=sa -O autotrim=on"
datasetOptions: "-o compression=zstd -o atime=off -o xattr=sa"
```

### zfspostcfg.conf
```yaml
poolName: "rpool"
services:
    - "zfs-import-cache.service"
    - "zfs-import.target"
    - "zfs-mount.service"
    - "zfs.target"
```

## Installation Sequence

The ZFS-enabled installation follows this sequence:

1. **partition**: Creates ZFS partitions
2. **zfs**: Creates ZFS pools and datasets
3. **mount**: Mounts filesystems
4. **unpackfs**: Unpacks the root filesystem
5. **fstab**: Generates fstab (minimal for ZFS)
6. **zfshostid**: Copies ZFS hostid
7. **zfspostcfg**: **NEW** - Configures ZFS system services and settings
8. **initramfs**: Generates initramfs with ZFS support
9. **bootloader**: Installs bootloader with ZFS awareness

## System Requirements

- **Memory**: At least 2GB RAM (ZFS requirement)
- **Storage**: ZFS-compatible storage device
- **Kernel**: Linux kernel with ZFS support
- **Bootloader**: GRUB with ZFS support

## Troubleshooting

### Common Issues

1. **ZFS modules not loading**:
   ```bash
   sudo modprobe zfs
   ```

2. **Pool import failures**:
   - Check ZFS hostid: `cat /etc/hostid`
   - Verify pool status: `zpool status`

3. **Boot issues**:
   - Ensure mkinitcpio includes ZFS hooks
   - Check bootloader configuration for ZFS support

### Debug Information

Enable debug mode in Calamares to see detailed ZFS operation logs:
```bash
calamares -d
```

## Development

### Testing

The modules include Python syntax validation and YAML configuration validation:

```bash
# Validate Python syntax
python3 -m py_compile modules/zfspostcfg/main.py

# Test configuration loading
python3 -c "import yaml; print(yaml.safe_load(open('modules/zfspostcfg/zfspostcfg.conf')))"
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

This project maintains the same licensing as Calamares:
- **Code**: GPL-3.0+
- **Configuration**: CC0-1.0

## Compatibility

- **Calamares**: 3.3.14+
- **ZFS**: OpenZFS 2.0+
- **Garuda Linux**: All versions
- **Arch Linux**: Compatible
- **Other distributions**: May require configuration adjustments

## Credits

- **Calamares Team**: For the excellent installer framework
- **OpenZFS Project**: For ZFS filesystem implementation  
- **Garuda Linux Team**: For the distribution and testing environment
- **Original ZFS Integration**: Evan James (@dalto8798)

## Support

For issues and questions:
- Check the troubleshooting section
- Review Calamares documentation
- Open an issue in this repository
- Consult the Garuda Linux community forums