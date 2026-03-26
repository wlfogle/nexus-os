#!/bin/bash

echo "🎯 GARUDA LINUX GITLAB ISSUE CREATION HELPER"
echo ""
echo "1. Open this URL in your browser:"
echo "   https://gitlab.com/garuda-linux/pkgbuilds/-/issues/new"
echo ""
echo "2. Copy and paste this TITLE:"
echo "═══════════════════════════════════════════════"
echo "feat: Enhanced ZFS Support for Calamares Installer"
echo "═══════════════════════════════════════════════"
echo ""
echo "3. Copy and paste this DESCRIPTION:"
echo "═══════════════════════════════════════════════════════════════════"

cat << 'ISSUE_CONTENT'
## 🚀 Feature Request: Enhanced ZFS Support for Calamares

### Summary
I've developed comprehensive ZFS integration for Calamares 3.3.14, specifically optimized for Garuda Linux. This enhances the existing basic ZFS support with post-installation configuration, service management, and system optimization.

### 🔗 Repository
https://github.com/wlfogle/calamares-zfs-integration

### ✨ Key Features
- **Post-Installation Module**: Automatic ZFS service enablement and system configuration
- **Enhanced Dataset Layout**: Optimized for Garuda Linux with `rpool/ROOT/garuda` structure
- **Service Management**: Automatic enablement of zfs.target, zfs-import-cache, etc.
- **Initramfs Integration**: Proper mkinitcpio configuration with ZFS hooks
- **System Optimization**: ZFS module parameters and cache configuration

### 🏗️ Technical Details
- **Works with**: Calamares 3.3.14+ with existing ZFS module
- **Language**: Python 3.6+ (zfspostcfg module)
- **Tested on**: Garuda Linux with Qt6 build
- **Integration**: Enhances existing C++ ZFS module, doesn't replace it

### 📋 Installation Sequence Enhancement
The integration adds a new `zfspostcfg` module to the installation sequence:
1. partition → zfs → mount → unpackfs → fstab → zfshostid → **zfspostcfg** → initramfs → bootloader

### 🎯 Benefits for Garuda Users
- **Seamless ZFS Installation**: No manual post-installation configuration needed
- **Optimized Performance**: zstd compression, autotrim for SSDs
- **Proper Boot Support**: Automatic initramfs configuration with ZFS modules
- **Service Management**: All ZFS services properly enabled
- **Better Dataset Layout**: Organized structure for system management

### 📖 Documentation
- Complete README with installation instructions
- Detailed installation guide with troubleshooting
- Comprehensive changelog and version tracking
- Schema validation for all configurations

### 🤝 How to Integrate
**Option 1: Package Integration**
- Add as a garuda-calamares-zfs package in your PKGBUILDs
- Include in default Calamares configuration for ZFS installations

**Option 2: Direct Integration**
- Include modules directly in your Calamares build
- Add to garuda-tools for ISO generation

**Option 3: Collaboration**
- Fork/clone the repository to garuda-linux organization
- Collaborate on further enhancements and maintenance

### 🧪 Testing Status
- ✅ Python syntax validation passed
- ✅ YAML configuration validation passed  
- ✅ Calamares 3.3.14 build integration successful
- ✅ Module detection and configuration verified
- 🔄 Full installation testing in progress

### 🆘 Questions/Support
- Would you be interested in integrating this into Garuda's installer?
- Any specific Garuda requirements or modifications needed?
- Would you prefer this as a separate package or integrated into existing builds?
- Any testing or validation you'd like me to perform?

### 📞 Contact
- GitHub: @wlfogle  
- Repository: https://github.com/wlfogle/calamares-zfs-integration
- Issues/Discussions welcome in the repository

Looking forward to potentially contributing to Garuda Linux's ZFS support! 🐧
ISSUE_CONTENT

echo "═══════════════════════════════════════════════════════════════════"
echo ""
echo "4. Add these LABELS (if available):"
echo "   - enhancement"
echo "   - calamares"
echo "   - zfs"
echo ""
echo "5. Click 'Create issue'"
echo ""
echo "✅ READY TO CREATE THE ISSUE!"
echo "🔗 GitLab URL: https://gitlab.com/garuda-linux/pkgbuilds/-/issues/new"
echo "📦 Our Repo: https://github.com/wlfogle/calamares-zfs-integration"