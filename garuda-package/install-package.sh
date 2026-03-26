#!/bin/bash

# Garuda ZFS Calamares Integration Package Installer
# This script builds and installs the package locally

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PACKAGE_DIR="${SCRIPT_DIR}/garuda-calamares-zfs"

echo "🎯 Garuda ZFS Calamares Integration Package Installer"
echo "════════════════════════════════════════════════════"

# Check if we're on Garuda Linux
if ! grep -q "Garuda" /etc/os-release 2>/dev/null; then
    echo "⚠️  Warning: This package is optimized for Garuda Linux"
    echo "   It should work on other Arch-based systems but may need adjustments"
    echo ""
fi

# Check dependencies
echo "🔍 Checking dependencies..."
missing_deps=()

if ! command -v makepkg >/dev/null 2>&1; then
    missing_deps+=("base-devel")
fi

if ! pacman -Qi calamares >/dev/null 2>&1; then
    echo "⚠️  Warning: Calamares is not installed"
    echo "   This package extends Calamares functionality"
fi

if ! pacman -Qi zfs-utils >/dev/null 2>&1; then
    missing_deps+=("zfs-utils")
fi

if [[ ${#missing_deps[@]} -gt 0 ]]; then
    echo "❌ Missing dependencies: ${missing_deps[*]}"
    echo "   Install with: sudo pacman -S ${missing_deps[*]}"
    exit 1
fi

echo "✅ Dependencies check passed"

# Enter package directory
cd "$PACKAGE_DIR"

# Update checksums
echo "🔄 Updating checksums..."
if [[ -f "PKGBUILD" ]]; then
    # Download source to get actual checksum
    makepkg --nobuild --noextract
    makepkg --printsrcinfo > .SRCINFO
    
    # Extract actual checksum
    if [[ -f "calamares-zfs-integration-1.0.0.tar.gz" ]]; then
        ACTUAL_CHECKSUM=$(sha256sum "calamares-zfs-integration-1.0.0.tar.gz" | cut -d' ' -f1)
        echo "📋 Calculated checksum: $ACTUAL_CHECKSUM"
        
        # Update PKGBUILD with actual checksum
        sed -i "s/sha256sums=('SKIP')/sha256sums=('$ACTUAL_CHECKSUM')/g" PKGBUILD
        makepkg --printsrcinfo > .SRCINFO
        echo "✅ Checksums updated"
    fi
fi

# Ask user what they want to do
echo ""
echo "🚀 What would you like to do?"
echo "   1) Build package only"
echo "   2) Build and install package"  
echo "   3) Build, install, and configure Calamares for ZFS"
echo "   q) Quit"
echo ""
read -p "Choose option [1-3]: " choice

case $choice in
    1)
        echo "🔨 Building package..."
        makepkg -s --clean
        echo "✅ Package built successfully!"
        echo "   Package file: $(ls -1 *.pkg.tar.* | head -1)"
        ;;
    2)
        echo "🔨 Building and installing package..."
        makepkg -si --clean
        echo "✅ Package installed successfully!"
        ;;
    3)
        echo "🔨 Building, installing, and configuring..."
        makepkg -si --clean
        
        echo ""
        echo "🔧 Configuring Calamares for ZFS..."
        
        # Backup existing settings
        if [[ -f /etc/calamares/settings.conf ]]; then
            sudo cp /etc/calamares/settings.conf /etc/calamares/settings.conf.backup
            echo "✅ Backed up existing Calamares settings"
        fi
        
        # Enable ZFS settings
        sudo cp /usr/share/calamares/settings-zfs.conf /etc/calamares/settings.conf
        echo "✅ Enabled ZFS Calamares configuration"
        
        # Load ZFS module
        if ! lsmod | grep -q zfs; then
            echo "🔄 Loading ZFS kernel module..."
            sudo modprobe zfs
            echo "✅ ZFS module loaded"
        else
            echo "✅ ZFS module already loaded"
        fi
        
        # Enable ZFS module loading at boot
        if [[ ! -f /etc/modules-load.d/zfs.conf ]]; then
            echo "zfs" | sudo tee /etc/modules-load.d/zfs.conf
            echo "✅ ZFS module will load automatically at boot"
        fi
        
        echo ""
        echo "🎉 Complete ZFS integration setup finished!"
        echo ""
        echo "💡 You can now:"
        echo "   - Run Calamares with ZFS support: sudo calamares"
        echo "   - Create ZFS pools during installation"
        echo "   - Benefit from automatic ZFS system configuration"
        echo ""
        echo "📖 Documentation: /usr/share/doc/garuda-calamares-zfs/"
        ;;
    q|Q)
        echo "👋 Goodbye!"
        exit 0
        ;;
    *)
        echo "❌ Invalid option"
        exit 1
        ;;
esac

echo ""
echo "📚 For more information:"
echo "   - Documentation: /usr/share/doc/garuda-calamares-zfs/"  
echo "   - Repository: https://github.com/wlfogle/calamares-zfs-integration"
echo "   - Issues: https://github.com/wlfogle/calamares-zfs-integration/issues"
echo ""
echo "✨ Thank you for using Garuda ZFS Calamares Integration!"