# Changelog

All notable changes to the Calamares ZFS Integration project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2024-09-17

### Added
- **New zfspostcfg module**: Comprehensive post-installation ZFS configuration
  - Automatic ZFS service enablement (zfs.target, zfs-import-cache, etc.)
  - Mkinitcpio configuration with ZFS hooks and modules  
  - ZFS module parameter configuration (/etc/modprobe.d/zfs.conf)
  - ZFS cache file setup for faster boot times
  - ZFS hostid management integration
  - Dataset mountpoint reset to proper final values

### Enhanced
- **Enhanced ZFS module configuration**: Optimized for Garuda Linux
  - Changed default pool name from "zpcala" to "rpool"
  - Updated compression from lz4 to zstd for better performance
  - Added autotrim=on for SSD optimization
  - Improved dataset layout with Garuda-specific naming
  - Added var/cache dataset for better package cache management

### Changed
- **Dataset Structure**: Reorganized for better Garuda Linux integration
  - ROOT/garuda/root instead of ROOT/distro/root
  - Separated var/log and var/cache datasets
  - Optimized mount point configuration

### Added Files
- `modules/zfspostcfg/main.py`: Post-installation ZFS configuration module
- `modules/zfspostcfg/module.desc`: Module descriptor for zfspostcfg
- `modules/zfspostcfg/zfspostcfg.conf`: Configuration file for post-installation setup
- `modules/zfspostcfg/zfspostcfg.schema.yaml`: Schema validation for configuration
- `modules/zfs.conf`: Enhanced ZFS module configuration for Garuda Linux
- `settings/settings-zfs.conf`: Complete Calamares settings with ZFS integration enabled
- `README.md`: Comprehensive documentation
- `INSTALL.md`: Detailed installation guide
- `CHANGELOG.md`: This changelog

### Technical Details
- **Language**: Python 3.6+ (zfspostcfg module)
- **Integration**: Works with existing C++ ZFS module in Calamares 3.3.14
- **Dependencies**: Standard Calamares dependencies plus ZFS utilities
- **Compatibility**: Tested with Calamares 3.3.14, Qt6, Garuda Linux

### Installation Sequence
The enhanced installation sequence now includes:
1. partition (creates ZFS partitions)
2. zfs (creates pools and datasets) 
3. mount (mounts filesystems)
4. unpackfs (unpacks root filesystem)
5. fstab (generates minimal fstab for ZFS)
6. zfshostid (copies ZFS hostid)
7. **zfspostcfg** (NEW - configures ZFS services and system settings)
8. initramfs (generates initramfs with ZFS support)
9. bootloader (installs bootloader)

### Configuration Options
- **Pool Name**: Configurable (default: rpool)
- **Compression**: zstd (high performance)
- **Services**: Configurable list of systemd services to enable
- **Dataset Layout**: Flexible dataset configuration
- **Module Parameters**: Configurable ZFS kernel module parameters

### Testing
- **Syntax Validation**: Python modules pass py_compile validation
- **Configuration Validation**: YAML configurations validate against schemas
- **Build Testing**: Successfully builds with Calamares 3.3.14
- **Integration Testing**: Modules properly detected and configured

### Documentation
- Comprehensive README with installation instructions
- Detailed installation guide with troubleshooting
- Code comments and inline documentation
- Schema validation for configuration files

### Compatibility Notes
- **Calamares**: 3.3.14+ (tested)
- **ZFS**: OpenZFS 2.0+ required
- **Garuda Linux**: All current versions
- **Arch Linux**: Compatible with ZFS packages
- **Other Distributions**: May require configuration adjustments

### Known Limitations
- Requires ZFS kernel modules to be available during installation
- GRUB bootloader configuration may need manual adjustment for ZFS boot
- Encryption passphrase handling integrated with existing Calamares encryption UI

### Future Enhancements
- Support for ZFS encryption with custom key formats
- Advanced pool layouts (mirror, raidz)
- ZFS snapshot management during installation
- Boot environment (BE) support similar to illumos systems
- Integration with systemd-boot for ZFS root support

## Development Notes

### Architecture Decisions
- **Modular Design**: Separate post-configuration from initial setup
- **Backward Compatibility**: Works with existing ZFS module  
- **Configuration Driven**: Most behavior configurable via YAML files
- **Error Handling**: Comprehensive error reporting and graceful degradation
- **Service Integration**: Proper systemd service management

### Code Quality
- **Python Standards**: PEP 8 compliant code
- **Error Handling**: Try-catch blocks with informative error messages
- **Logging**: Comprehensive debug logging via libcalamares
- **Documentation**: Docstrings and inline comments
- **Validation**: Schema validation for all configuration files

### Testing Strategy  
- **Unit Testing**: Python syntax and import validation
- **Integration Testing**: Build system integration
- **Configuration Testing**: YAML validation and parsing
- **Manual Testing**: Full installation workflow testing

---

## Contributing

When contributing to this project:

1. Update this CHANGELOG.md with your changes
2. Follow semantic versioning for releases
3. Add appropriate sections (Added, Changed, Fixed, Removed)
4. Include technical details and compatibility notes
5. Test thoroughly before submitting changes

## Version Format

- **Major.Minor.Patch** (e.g., 1.0.0)
- **Major**: Breaking changes or significant new features
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes, minor improvements