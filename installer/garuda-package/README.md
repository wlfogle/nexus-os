# Garuda Linux ZFS Calamares Integration Package

This directory contains the complete package for integrating enhanced ZFS support into Garuda Linux's Calamares installer.

## 🎯 Package Overview

**Package Name**: `garuda-calamares-zfs`  
**Version**: 1.0.0  
**Description**: Enhanced ZFS support for Calamares installer - optimized for Garuda Linux  

## 📦 Package Contents

### Core Files
- `PKGBUILD` - Arch/Garuda package build script
- `.SRCINFO` - Package metadata for repository integration
- `.CI/config` - Garuda CI/build system configuration

### Installation
- `install-package.sh` - Interactive installation script
- `README.md` - This documentation

## 🚀 Quick Installation

### Option 1: Interactive Install (Recommended)
```bash
cd garuda-package
./install-package.sh
```
Choose option 3 for complete setup with ZFS configuration.

### Option 2: Manual Build and Install
```bash
cd garuda-package/garuda-calamares-zfs
makepkg -si
```

### Option 3: Just Build Package
```bash
cd garuda-package/garuda-calamares-zfs
makepkg -s
```

## 📋 What Gets Installed

### Module Files
- `/usr/lib/calamares/modules/zfspostcfg/` - Post-installation ZFS configuration module
- `/usr/share/calamares/modules/zfspostcfg/` - Module configuration files
- `/usr/share/calamares/modules/zfs/zfs-garuda.conf` - Enhanced ZFS configuration

### Settings
- `/usr/share/calamares/settings-zfs.conf` - ZFS-enabled Calamares settings

### Documentation
- `/usr/share/doc/garuda-calamares-zfs/` - Complete documentation package

## 🔧 Configuration

After installation, you need to activate ZFS support:

1. **Enable ZFS Settings**:
   ```bash
   sudo cp /usr/share/calamares/settings-zfs.conf /etc/calamares/settings.conf
   ```

2. **Load ZFS Module**:
   ```bash
   sudo modprobe zfs
   ```

3. **Optional: Use Enhanced ZFS Config**:
   ```bash
   sudo cp /usr/share/calamares/modules/zfs/zfs-garuda.conf /usr/share/calamares/modules/zfs/zfs.conf
   ```

## 🛠️ Dependencies

### Required
- `calamares >= 3.3.14` - Base installer framework
- `zfs-utils` - ZFS userspace utilities  
- `python >= 3.6` - Python runtime
- `python-yaml` - YAML parsing support

### Optional
- `zfs-dkms` - ZFS kernel modules (DKMS)
- `zfs-linux` - ZFS kernel modules (linux kernel)

## 🎯 Features

### Enhanced ZFS Integration
- **Post-Installation Configuration**: Automatic service enablement and system setup
- **Optimized Dataset Layout**: `rpool/ROOT/garuda` structure for Garuda Linux
- **Performance Optimizations**: zstd compression, autotrim for SSDs
- **Boot Support**: Proper initramfs configuration with ZFS support

### Installation Sequence
1. `partition` - Create ZFS partitions
2. `zfs` - Create ZFS pools and datasets  
3. `mount` - Mount filesystems
4. `unpackfs` - Unpack root filesystem
5. `fstab` - Generate fstab
6. `zfshostid` - Copy ZFS hostid
7. `zfspostcfg` - **NEW** - Configure ZFS system services
8. `initramfs` - Generate initramfs with ZFS support
9. `bootloader` - Install bootloader

## 🏗️ For Garuda Developers

### Integration into PKGBUILDs Repository

1. **Copy Package Directory**:
   ```bash
   cp -r garuda-calamares-zfs /path/to/garuda/pkgbuilds/
   ```

2. **Update CI Configuration**:
   - Verify `.CI/config` matches your build system requirements
   - Update `CI_PKGBUILD_SOURCE` if needed

3. **Add to Build Pipeline**:
   - The package will be automatically detected by Garuda's CI
   - Tags on the source repository will trigger builds

### Customization Options

- **Pool Name**: Modify `poolName` in configurations
- **Dataset Layout**: Adjust `datasets` array in ZFS config  
- **Services**: Customize `services` list in zfspostcfg config
- **Dependencies**: Update PKGBUILD dependencies as needed

## 🧪 Testing

### Pre-Installation Testing
```bash
# Check ZFS availability
sudo modprobe zfs
lsmod | grep zfs

# Validate package files
cd garuda-calamares-zfs
makepkg --printsrcinfo
```

### Post-Installation Testing
```bash
# Verify module installation
ls -la /usr/lib/calamares/modules/zfspostcfg/

# Test Python module syntax
python3 -c "import sys; sys.path.append('/usr/lib/calamares/modules/zfspostcfg'); import main"

# Verify configurations
python3 -c "import yaml; print(yaml.safe_load(open('/usr/share/calamares/modules/zfspostcfg/zfspostcfg.conf')))"
```

## 📞 Support & Issues

- **Repository**: https://github.com/wlfogle/calamares-zfs-integration
- **Issues**: https://github.com/wlfogle/calamares-zfs-integration/issues
- **Documentation**: `/usr/share/doc/garuda-calamares-zfs/` (after installation)

## 🔄 Maintenance

### Updating Package
1. Update version in `PKGBUILD`
2. Regenerate `.SRCINFO`: `makepkg --printsrcinfo > .SRCINFO`
3. Update checksums: `makepkg --nobuild && updpkgsums`
4. Test build: `makepkg -s`

### Release Process
1. Tag new version in source repository
2. Update `pkgver` in PKGBUILD
3. Update checksums
4. Rebuild and test
5. Commit to PKGBUILDs repository

## 📄 License

This package maintains the same licensing as the source project:
- **Code**: GPL-3.0-or-later
- **Configuration**: CC0-1.0

---

**Built for Garuda Linux** 🐧  
**Enhancing ZFS Support in Calamares** 🚀