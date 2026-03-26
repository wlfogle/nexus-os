# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

This is a Calamares ZFS Integration project that enhances the default ZFS support in Calamares 3.3.14 for Garuda Linux installations. The project adds a comprehensive post-installation configuration module (`zfspostcfg`) that handles ZFS system service setup, initramfs configuration, and other ZFS-specific system optimizations.

## Quick Start

### Complete Automated Build
For the complete workflow from source download to package creation:

```bash
# Run the complete build script
./build-complete.sh
```

This script will:
1. Install all required dependencies for Garuda Linux
2. Download Calamares 3.3.14 source code
3. Integrate ZFS enhancements into the source
4. Build Calamares with ZFS support
5. Create the Garuda Linux package
6. Validate the entire build process

## Development Commands

### Complete Build Process

#### 1. Download and Prepare Calamares Source
```bash
# Download Calamares 3.3.14 source
wget https://github.com/calamares/calamares/releases/download/v3.3.14/calamares-3.3.14.tar.gz
tar -xzf calamares-3.3.14.tar.gz
cd calamares-3.3.14

# Install build dependencies for Garuda Linux
sudo pacman -S --needed cmake qt6-base qt6-tools extra-cmake-modules yaml-cpp boost kpmcore polkit-qt6 python python-yaml base-devel
```

#### 2. Integrate ZFS Enhancements
```bash
# Copy the new zfspostcfg module
cp -r ../modules/zfspostcfg src/modules/

# Replace the ZFS configuration with enhanced version
cp ../modules/zfs.conf src/modules/zfs/

# Verify integration
ls -la src/modules/zfs*
python3 -m py_compile src/modules/zfspostcfg/main.py
```

#### 3. Build Calamares with ZFS Integration
```bash
# Create build directory
mkdir build && cd build

# Configure build (Debug for development, Release for production)
cmake .. -DCMAKE_BUILD_TYPE=Release -DWITH_QT6=ON -DCMAKE_INSTALL_PREFIX=/usr

# Build (use all CPU cores)
make -j$(nproc)

# Install to system (optional, for testing)
sudo make install
```

#### 4. Create Garuda Package
```bash
# Return to project root
cd ../../

# Build the integration package
cd garuda-package/garuda-calamares-zfs
makepkg -si

# Generate package metadata
makepkg --printsrcinfo > .SRCINFO
```

### Automated Build Script

The repository includes a complete build script (`build-complete.sh`) that automates the entire process. Use it for quick builds:

```bash
# Make executable and run
chmod +x build-complete.sh
./build-complete.sh
```

The script includes error handling, logging, validation, and produces both the Calamares binary and Garuda package.

### Development Testing Commands

#### Module Validation
```bash
# Validate Python module syntax
python3 -m py_compile modules/zfspostcfg/main.py

# Test YAML configuration loading  
python3 -c "import yaml; print(yaml.safe_load(open('modules/zfspostcfg/zfspostcfg.conf')))"

# Test all Python modules in the project
find . -name "*.py" -exec python3 -m py_compile {} \;

# Validate all YAML configurations
find . -name "*.conf" -o -name "*.yaml" | grep -v ".git" | while read f; do echo "Testing $f:"; python3 -c "import yaml; yaml.safe_load(open('$f'))" && echo "✓ Valid" || echo "✗ Invalid"; done
```

#### ZFS System Testing
```bash
# Check ZFS kernel module availability
modinfo zfs

# Load ZFS module (requires root)
sudo modprobe zfs

# Check ZFS tools
which zpool zfs

# Test ZFS functionality (on test system only)
sudo zpool list
sudo zfs list
```

#### Installation Testing
```bash
# Test Calamares in debug mode (after build/install)
sudo calamares -d

# Check module installation
ls -la /usr/lib/calamares/modules/zfs*

# Verify configuration deployment
sudo cp settings/settings-zfs.conf /etc/calamares/settings.conf
```

## Code Architecture

### Core Components

**Existing Calamares Modules** (C++):
- `zfs` - Creates ZFS pools and datasets during installation
- `zfshostid` - Copies ZFS hostid file to installed system

**New Python Module** (`modules/zfspostcfg/`):
- `main.py` - Post-installation configuration logic
- `zfspostcfg.conf` - Module configuration
- `module.desc` - Calamares module descriptor
- `zfspostcfg.schema.yaml` - Configuration schema validation

### Installation Flow Architecture

The ZFS installation follows this critical sequence:
1. `partition` → 2. `zfs` → 3. `mount` → 4. `unpackfs` → 5. `fstab` → 6. `zfshostid` → 7. **`zfspostcfg`** → 8. `initramfs` → 9. `bootloader`

The `zfspostcfg` module is inserted after basic ZFS setup but before initramfs generation, ensuring proper system service configuration.

### ZFS Dataset Architecture

Default dataset structure optimized for Garuda Linux:
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

### Configuration System

**Global Storage Integration**: The `zfspostcfg` module reads ZFS configuration from Calamares global storage (`zfsPoolInfo`, `zfsDatasets`, `useZfs`) set by the C++ ZFS module, ensuring consistency across installation phases.

**Service Management**: Automatically enables critical ZFS systemd services:
- `zfs-import-cache.service` - Fast pool import using cache file
- `zfs-import.target` - Fallback pool import
- `zfs-mount.service` - Dataset mounting
- `zfs.target` - ZFS service coordination

### Key Python Functions

**`copy_zfs_hostid()`**: Ensures ZFS hostid consistency between live and installed systems
**`enable_zfs_services()`**: Chroot-based systemd service enablement  
**`configure_initramfs()`**: Modifies mkinitcpio.conf to add ZFS hooks and modules
**`reset_zfs_mountpoints()`**: Sets final mountpoints after installation
**`setup_zfs_cache()`**: Configures zpool.cache for faster boot times

## Configuration Files

### Primary Configurations
- `modules/zfs.conf` - Enhanced ZFS module config with zstd compression, autotrim
- `modules/zfspostcfg/zfspostcfg.conf` - Post-installation service list
- `settings/settings-zfs.conf` - Complete Calamares settings with ZFS sequence

### Key Configuration Concepts
- **Pool Options**: `-f -o ashift=12 -O mountpoint=none -O acltype=posixacl -O relatime=on -O compression=zstd -O xattr=sa -O autotrim=on`
- **Dataset Options**: `-o compression=zstd -o atime=off -o xattr=sa`
- **Pool Name**: `rpool` (configurable but should match across all configs)

## Development Patterns

### Error Handling Pattern
All functions in `main.py` return `None` on success or error string on failure. The main `run()` function catches exceptions and returns formatted error messages to Calamares.

### Configuration Override Pattern  
The module checks multiple sources for ZFS configuration in priority order:
1. Calamares global storage (from C++ modules)
2. Module configuration file
3. Hardcoded defaults

### Chroot Operations Pattern
System modifications use `["chroot", root_mount_point, "command"]` pattern for proper environment isolation during installation.

## Testing Strategy

### Unit Testing
- Python syntax validation with `py_compile`
- YAML configuration parsing tests
- Import testing for all modules

### Integration Testing  
- Build system integration (makepkg)
- Calamares module detection
- Service file installation verification

### Manual Testing Requirements
- Test on ZFS-capable system with spare disk
- Verify in Calamares debug mode (`calamares -d`)
- Check post-installation boot success

## Dependencies and Requirements

**Required**: calamares >= 3.3.14, zfs-utils, python >= 3.6, python-yaml
**Runtime**: ZFS kernel modules must be loaded during installation
**Architecture**: Works with Garuda Linux, Arch Linux, and compatible distributions

## Common Issues and Solutions

**ZFS modules not loading**: Ensure `zfs-dkms` or `zfs-linux` packages installed and `modprobe zfs` successful
**Service enablement failures**: Verify systemd services exist in chroot environment
**Initramfs generation issues**: Check mkinitcpio.conf modifications and ZFS hook availability
**Pool import failures**: Verify hostid consistency and zpool.cache file setup