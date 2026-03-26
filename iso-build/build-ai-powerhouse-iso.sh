#!/bin/bash
# AI Powerhouse ISO Builder
# Uses archiso method for reliable building in live environments

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as regular user (we'll use sudo when needed)
if [[ $EUID -eq 0 ]]; then
    print_error "Please run this script as a regular user (not root). We'll use sudo when needed."
    exit 1
fi

# Default paths - can be overridden with environment variables
WORK_DIR="${WORK_DIR:-/tmp/ai-powerhouse-workdir}"
OUTPUT_DIR="${OUTPUT_DIR:-/tmp/ai-powerhouse-output}"
BUILD_DIR="$(dirname "$(realpath "$0")")/archiso-method"

print_status "AI Powerhouse ISO Builder"
print_status "=========================="
echo ""

# Check for archiso
if ! command -v mkarchiso &> /dev/null; then
    print_error "archiso not found. Installing..."
    sudo pacman -S --noconfirm archiso
    print_success "archiso installed"
fi

# Check if build directory exists
if [[ ! -d "$BUILD_DIR" ]]; then
    print_error "Build directory not found: $BUILD_DIR"
    print_error "Please ensure you're running this from the iso-build directory"
    exit 1
fi

# Check disk space
print_status "Checking disk space..."
AVAILABLE_SPACE=$(df -BG "$HOME" | awk 'NR==2 {gsub(/G/, "", $4); print $4}')
if [[ $AVAILABLE_SPACE -lt 25 ]]; then
    print_warning "Low disk space detected (${AVAILABLE_SPACE}GB available)"
    print_warning "Recommend at least 25GB free space"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_error "Build cancelled"
        exit 1
    fi
fi

# Create directories
print_status "Creating build directories..."
mkdir -p "$OUTPUT_DIR"
sudo rm -rf "$WORK_DIR" 2>/dev/null || true

# Show build configuration
print_status "Build Configuration:"
echo "  Build Profile: $BUILD_DIR"
echo "  Work Directory: $WORK_DIR"
echo "  Output Directory: $OUTPUT_DIR"
echo ""

# Check packages file
PACKAGES_FILE="$BUILD_DIR/packages.x86_64"
if [[ -f "$PACKAGES_FILE" ]]; then
    PACKAGE_COUNT=$(grep -v "^#" "$PACKAGES_FILE" | grep -v "^$" | wc -l)
    print_status "Package List: $PACKAGE_COUNT packages found"
else
    print_error "Packages file not found: $PACKAGES_FILE"
    exit 1
fi

# Start the build
print_status "Starting ISO build..."
echo ""

BUILD_LOG="$OUTPUT_DIR/build.log"
cd "$BUILD_DIR"

if sudo mkarchiso -v -w "$WORK_DIR" -o "$OUTPUT_DIR" . 2>&1 | tee "$BUILD_LOG"; then
    print_success "ISO build completed successfully!"
    echo ""
    
    # Find the created ISO
    ISO_FILE=$(find "$OUTPUT_DIR" -name "*.iso" -type f | head -n 1)
    if [[ -n "$ISO_FILE" ]]; then
        ISO_SIZE=$(du -h "$ISO_FILE" | cut -f1)
        print_success "ISO created: $(basename "$ISO_FILE") (${ISO_SIZE})"
        print_status "Location: $ISO_FILE"
        echo ""
        
        # Test with MobaLiveCD suggestion
        print_status "Ready for testing with MobaLiveCD!"
        print_status "Copy the ISO to your MobaLiveCD directory and test."
        echo ""
        
        # Show verification commands
        print_status "Verification commands:"
        echo "  File info: file '$ISO_FILE'"
        echo "  ISO info: isoinfo -d -i '$ISO_FILE'"
        echo "  Mount test: sudo mkdir -p /mnt/iso && sudo mount -o loop '$ISO_FILE' /mnt/iso"
        
    else
        print_error "ISO file not found in output directory"
        exit 1
    fi
else
    print_error "ISO build failed!"
    print_error "Check the build log: $BUILD_LOG"
    exit 1
fi

# Cleanup option
echo ""
read -p "Clean up work directory? (Y/n): " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Nn]$ ]]; then
    print_status "Cleaning up work directory..."
    sudo rm -rf "$WORK_DIR"
    print_success "Cleanup completed"
fi

print_success "AI Powerhouse ISO build process completed!"