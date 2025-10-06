#!/bin/bash
# Complete Calamares ZFS Integration Build Script
# This script downloads Calamares source, integrates ZFS enhancements, 
# builds Calamares, and creates the Garuda Linux package

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 1. Ensure we're in project root
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)
log_info "Working in project root: $PROJECT_ROOT"

# 2. Install dependencies
log_info "Installing build dependencies..."
sudo pacman -S --needed cmake qt6-base qt6-tools extra-cmake-modules yaml-cpp boost kpmcore polkit-qt6 python python-yaml base-devel zfs-utils zfs-dkms || {
    log_error "Failed to install dependencies"
    exit 1
}
log_success "Dependencies installed"

# 3. Download Calamares if not present
CALAMARES_VERSION="3.3.14"
CALAMARES_DIR="calamares-${CALAMARES_VERSION}"
CALAMARES_TARBALL="${CALAMARES_DIR}.tar.gz"

if [ ! -d "$CALAMARES_DIR" ]; then
    log_info "Downloading Calamares $CALAMARES_VERSION..."
    wget "https://github.com/calamares/calamares/releases/download/v${CALAMARES_VERSION}/${CALAMARES_TARBALL}" || {
        log_error "Failed to download Calamares"
        exit 1
    }
    
    log_info "Extracting Calamares source..."
    tar -xzf "$CALAMARES_TARBALL" || {
        log_error "Failed to extract Calamares"
        exit 1
    }
    
    log_success "Calamares source downloaded and extracted"
else
    log_warning "Calamares directory already exists, using existing source"
fi

# 4. Validate our ZFS modules before integration
log_info "Validating ZFS modules..."
python3 -m py_compile modules/zfspostcfg/main.py || {
    log_error "ZFS post-config module syntax validation failed"
    exit 1
}

python3 -c "import yaml; yaml.safe_load(open('modules/zfspostcfg/zfspostcfg.conf'))" || {
    log_error "ZFS post-config configuration validation failed"
    exit 1
}

python3 -c "import yaml; yaml.safe_load(open('modules/zfs.conf'))" || {
    log_error "ZFS configuration validation failed"
    exit 1
}
log_success "ZFS modules validated"

# 5. Integrate ZFS modules
log_info "Integrating ZFS enhancements..."
cp -r modules/zfspostcfg "$CALAMARES_DIR/src/modules/" || {
    log_error "Failed to copy zfspostcfg module"
    exit 1
}

cp modules/zfs.conf "$CALAMARES_DIR/src/modules/zfs/" || {
    log_error "Failed to copy enhanced ZFS configuration"
    exit 1
}

# Verify integration
if [ -f "$CALAMARES_DIR/src/modules/zfspostcfg/main.py" ] && [ -f "$CALAMARES_DIR/src/modules/zfs/zfs.conf" ]; then
    log_success "ZFS enhancements integrated successfully"
else
    log_error "ZFS integration verification failed"
    exit 1
fi

# 6. Build Calamares
log_info "Building Calamares with ZFS integration..."
cd "$CALAMARES_DIR"

# Clean previous build if exists
if [ -d "build" ]; then
    log_warning "Removing previous build directory"
    rm -rf build
fi

mkdir build && cd build

log_info "Configuring build with CMake..."
cmake .. -DCMAKE_BUILD_TYPE=Release -DWITH_QT6=ON -DCMAKE_INSTALL_PREFIX=/usr || {
    log_error "CMake configuration failed"
    exit 1
}

log_info "Compiling Calamares (using $(nproc) cores)..."
make -j$(nproc) || {
    log_error "Calamares build failed"
    exit 1
}

log_success "Calamares built successfully"
cd "$PROJECT_ROOT"

# 7. Load ZFS kernel module if not loaded
if ! lsmod | grep -q "^zfs "; then
    log_info "Loading ZFS kernel module..."
    sudo modprobe zfs || {
        log_warning "Failed to load ZFS module - this may affect testing"
    }
else
    log_info "ZFS kernel module already loaded"
fi

# 8. Build Garuda package
log_info "Building Garuda Linux package..."
cd garuda-package/garuda-calamares-zfs

# Clean any previous packages
rm -f *.pkg.tar.zst

log_info "Running makepkg..."
makepkg -f || {
    log_error "Package build failed"
    exit 1
}

log_info "Generating package metadata..."
makepkg --printsrcinfo > .SRCINFO || {
    log_warning "Failed to generate .SRCINFO"
}

cd "$PROJECT_ROOT"

# 9. Final verification
log_info "Performing final verification..."

# Check if package was created
PACKAGE_FILE=$(find garuda-package/garuda-calamares-zfs/ -name "garuda-calamares-zfs-*.pkg.tar.zst" -type f | head -1)
if [ -n "$PACKAGE_FILE" ]; then
    log_success "Package created: $(basename "$PACKAGE_FILE")"
else
    log_error "Package file not found"
    exit 1
fi

# Check if Calamares binary was built
if [ -f "$CALAMARES_DIR/build/calamares" ]; then
    log_success "Calamares binary built successfully"
else
    log_error "Calamares binary not found"
    exit 1
fi

# 10. Display completion summary
echo ""
log_success "=== BUILD COMPLETE ==="
echo ""
echo "📦 Package: $PACKAGE_FILE"
echo "🏗️  Calamares: $CALAMARES_DIR/build/calamares"
echo ""
echo "Next steps:"
echo "  • Install package: sudo pacman -U '$PACKAGE_FILE'"
echo "  • Test Calamares: sudo $PROJECT_ROOT/$CALAMARES_DIR/build/calamares -d"
echo "  • Enable ZFS config: sudo cp settings/settings-zfs.conf /etc/calamares/settings.conf"
echo ""
log_success "Build script completed successfully!"