#!/bin/bash
#
# NexusOS Complete Build Script
# Builds the entire NexusOS system with ZFS as default filesystem
#
# Copyright 2024 NexusOS Project
# SPDX-License-Identifier: GPL-3.0+

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="${SCRIPT_DIR}/build"
INSTALL_DIR="${BUILD_DIR}/install"
ISO_DIR="${BUILD_DIR}/iso"
LOG_FILE="${BUILD_DIR}/build.log"

# Build options (can be overridden by environment variables)
BUILD_KERNEL="${BUILD_KERNEL:-true}"
BUILD_BOOTLOADER="${BUILD_BOOTLOADER:-true}"
BUILD_USERSPACE="${BUILD_USERSPACE:-true}"
BUILD_INSTALLER="${BUILD_INSTALLER:-true}"
BUILD_ISO="${BUILD_ISO:-true}"
WITH_ZFS_DEFAULT="${WITH_ZFS_DEFAULT:-true}"
WITH_AI_FEATURES="${WITH_AI_FEATURES:-true}"
PARALLEL_JOBS="${PARALLEL_JOBS:-$(nproc)}"
CMAKE_BUILD_TYPE="${CMAKE_BUILD_TYPE:-Release}"

# Print functions
print_header() {
    echo -e "${WHITE}================================${NC}"
    echo -e "${WHITE} $1 ${NC}"
    echo -e "${WHITE}================================${NC}"
}

print_info() {
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

# Logging function
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" >> "${LOG_FILE}"
}

# Error handling
handle_error() {
    local line_no=$1
    print_error "Build failed at line ${line_no}. Check ${LOG_FILE} for details."
    log "ERROR: Build failed at line ${line_no}"
    exit 1
}

trap 'handle_error ${LINENO}' ERR

# Check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"
    
    local missing_deps=()
    
    # Essential build tools
    command -v cmake >/dev/null || missing_deps+=("cmake")
    command -v make >/dev/null || missing_deps+=("make")
    command -v gcc >/dev/null || missing_deps+=("gcc")
    command -v g++ >/dev/null || missing_deps+=("g++")
    command -v nasm >/dev/null || missing_deps+=("nasm")
    command -v ld >/dev/null || missing_deps+=("binutils")
    
    # ZFS requirements
    if [[ "${WITH_ZFS_DEFAULT}" == "true" ]]; then
        command -v zfs >/dev/null || missing_deps+=("zfs-utils")
        command -v zpool >/dev/null || missing_deps+=("zfs-utils")
        [[ -f /proc/modules ]] && grep -q zfs /proc/modules || print_warning "ZFS module not loaded"
    fi
    
    # ISO creation tools
    if [[ "${BUILD_ISO}" == "true" ]]; then
        command -v grub-mkrescue >/dev/null || missing_deps+=("grub-tools")
        command -v xorriso >/dev/null || missing_deps+=("xorriso")
    fi
    
    # Installer requirements
    if [[ "${BUILD_INSTALLER}" == "true" ]]; then
        command -v qmake-qt6 >/dev/null || command -v qmake >/dev/null || missing_deps+=("qt6-base-dev")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        print_error "Missing dependencies: ${missing_deps[*]}"
        print_info "On Garuda Linux/Arch, install with: pacman -S ${missing_deps[*]}"
        log "ERROR: Missing dependencies: ${missing_deps[*]}"
        exit 1
    fi
    
    print_success "All prerequisites satisfied"
    log "INFO: Prerequisites check passed"
}

# Setup build environment
setup_build_env() {
    print_header "Setting up Build Environment"
    
    # Create build directories
    mkdir -p "${BUILD_DIR}"
    mkdir -p "${INSTALL_DIR}"
    mkdir -p "${ISO_DIR}"
    mkdir -p "${BUILD_DIR}/logs"
    
    # Initialize log file
    echo "NexusOS Build Log - $(date)" > "${LOG_FILE}"
    
    # Check available disk space
    local available_space
    available_space=$(df "${BUILD_DIR}" | awk 'NR==2 {print $4}')
    if [[ ${available_space} -lt 5242880 ]]; then  # 5GB in KB
        print_warning "Less than 5GB available in build directory"
        print_info "Available space: $((available_space / 1024))MB"
    fi
    
    print_success "Build environment ready"
    log "INFO: Build environment setup completed"
}

# Configure CMake build
configure_build() {
    print_header "Configuring CMake Build"
    
    cd "${BUILD_DIR}"
    
    local cmake_args=(
        -DCMAKE_BUILD_TYPE="${CMAKE_BUILD_TYPE}"
        -DBUILD_KERNEL="${BUILD_KERNEL}"
        -DBUILD_BOOTLOADER="${BUILD_BOOTLOADER}"
        -DBUILD_USERSPACE="${BUILD_USERSPACE}"
        -DBUILD_INSTALLER="${BUILD_INSTALLER}"
        -DBUILD_ISO="${BUILD_ISO}"
        -DWITH_ZFS_DEFAULT="${WITH_ZFS_DEFAULT}"
        -DWITH_AI_FEATURES="${WITH_AI_FEATURES}"
        -DENABLE_TESTING=ON
        -DCMAKE_INSTALL_PREFIX="${INSTALL_DIR}"
    )
    
    print_info "CMake arguments: ${cmake_args[*]}"
    log "INFO: Configuring with: ${cmake_args[*]}"
    
    cmake "${SCRIPT_DIR}" "${cmake_args[@]}" 2>&1 | tee -a "${LOG_FILE}"
    
    print_success "Build configured successfully"
}

# Build kernel
build_kernel() {
    if [[ "${BUILD_KERNEL}" != "true" ]]; then
        return 0
    fi
    
    print_header "Building NexusOS Kernel"
    
    cd "${BUILD_DIR}"
    
    print_info "Compiling kernel with ${PARALLEL_JOBS} parallel jobs"
    log "INFO: Starting kernel build"
    
    make nexus-kernel -j"${PARALLEL_JOBS}" 2>&1 | tee -a "${LOG_FILE}"
    
    # Check if kernel was built successfully
    if [[ -f "${BUILD_DIR}/kernel/nexus-kernel.bin" ]]; then
        local kernel_size
        kernel_size=$(stat -c%s "${BUILD_DIR}/kernel/nexus-kernel.bin")
        print_success "Kernel built successfully (${kernel_size} bytes)"
        log "INFO: Kernel build completed - size: ${kernel_size} bytes"
    else
        print_error "Kernel build failed - binary not found"
        log "ERROR: Kernel binary not found after build"
        exit 1
    fi
}

# Build bootloader
build_bootloader() {
    if [[ "${BUILD_BOOTLOADER}" != "true" ]]; then
        return 0
    fi
    
    print_header "Building UEFI Bootloader"
    
    cd "${BUILD_DIR}"
    
    print_info "Building UEFI bootloader"
    log "INFO: Starting bootloader build"
    
    make nexus-bootloader -j"${PARALLEL_JOBS}" 2>&1 | tee -a "${LOG_FILE}"
    
    print_success "Bootloader built successfully"
    log "INFO: Bootloader build completed"
}

# Build userspace
build_userspace() {
    if [[ "${BUILD_USERSPACE}" != "true" ]]; then
        return 0
    fi
    
    print_header "Building NexusOS Userspace"
    
    cd "${BUILD_DIR}"
    
    print_info "Building userspace components"
    log "INFO: Starting userspace build"
    
    make nexus-userspace -j"${PARALLEL_JOBS}" 2>&1 | tee -a "${LOG_FILE}"
    
    print_success "Userspace built successfully"
    log "INFO: Userspace build completed"
}

# Build installer
build_installer() {
    if [[ "${BUILD_INSTALLER}" != "true" ]]; then
        return 0
    fi
    
    print_header "Building Calamares Installer with ZFS Support"
    
    cd "${BUILD_DIR}"
    
    print_info "Building Calamares-based installer"
    log "INFO: Starting installer build"
    
    # Build enhanced Calamares with ZFS support
    make calamares-zfs -j"${PARALLEL_JOBS}" 2>&1 | tee -a "${LOG_FILE}"
    
    print_success "Installer built successfully"
    log "INFO: Installer build completed"
}

# Create ISO image
build_iso() {
    if [[ "${BUILD_ISO}" != "true" ]]; then
        return 0
    fi
    
    print_header "Creating NexusOS Bootable ISO"
    
    cd "${BUILD_DIR}"
    
    print_info "Creating ISO image with ZFS support"
    log "INFO: Starting ISO creation"
    
    make nexus-iso 2>&1 | tee -a "${LOG_FILE}"
    
    if [[ -f "${BUILD_DIR}/nexusos.iso" ]]; then
        local iso_size
        iso_size=$(stat -c%s "${BUILD_DIR}/nexusos.iso")
        print_success "ISO created successfully ($(( iso_size / 1024 / 1024 ))MB)"
        log "INFO: ISO created - size: ${iso_size} bytes"
        
        # Create checksum
        cd "${BUILD_DIR}"
        sha256sum nexusos.iso > nexusos.iso.sha256
        print_info "SHA256 checksum created"
    else
        print_error "ISO creation failed"
        log "ERROR: ISO file not found after build"
        exit 1
    fi
}

# Run tests
run_tests() {
    print_header "Running NexusOS Tests"
    
    cd "${BUILD_DIR}"
    
    print_info "Running test suite"
    log "INFO: Starting test suite"
    
    if make test 2>&1 | tee -a "${LOG_FILE}"; then
        print_success "All tests passed"
        log "INFO: Test suite completed successfully"
    else
        print_warning "Some tests failed - check ${LOG_FILE}"
        log "WARNING: Test failures detected"
    fi
}

# Setup ZFS for development/testing
setup_zfs_dev() {
    if [[ "${WITH_ZFS_DEFAULT}" != "true" ]]; then
        return 0
    fi
    
    print_header "Setting up ZFS Development Environment"
    
    if [[ $EUID -ne 0 ]]; then
        print_warning "ZFS setup requires root privileges"
        print_info "Run 'sudo make zfs-setup' from build directory after build completes"
        return 0
    fi
    
    cd "${BUILD_DIR}"
    
    print_info "Setting up ZFS pools for development"
    log "INFO: Starting ZFS development setup"
    
    make zfs-setup 2>&1 | tee -a "${LOG_FILE}"
    
    print_success "ZFS development environment ready"
    log "INFO: ZFS setup completed"
}

# Print build summary
print_summary() {
    print_header "Build Summary"
    
    local build_time_end
    build_time_end=$(date +%s)
    local total_time=$((build_time_end - build_time_start))
    
    echo -e "${WHITE}NexusOS Build Complete!${NC}"
    echo ""
    echo -e "${GREEN}✓ Build Type:${NC} ${CMAKE_BUILD_TYPE}"
    echo -e "${GREEN}✓ Build Time:${NC} ${total_time}s"
    echo -e "${GREEN}✓ Log File:${NC} ${LOG_FILE}"
    echo ""
    
    if [[ "${BUILD_KERNEL}" == "true" ]]; then
        echo -e "${GREEN}✓ Kernel:${NC} Built successfully"
    fi
    
    if [[ "${BUILD_BOOTLOADER}" == "true" ]]; then
        echo -e "${GREEN}✓ Bootloader:${NC} Built successfully"
    fi
    
    if [[ "${BUILD_USERSPACE}" == "true" ]]; then
        echo -e "${GREEN}✓ Userspace:${NC} Built successfully"
    fi
    
    if [[ "${BUILD_INSTALLER}" == "true" ]]; then
        echo -e "${GREEN}✓ Installer:${NC} Built with ZFS support"
    fi
    
    if [[ "${BUILD_ISO}" == "true" ]] && [[ -f "${BUILD_DIR}/nexusos.iso" ]]; then
        local iso_size
        iso_size=$(stat -c%s "${BUILD_DIR}/nexusos.iso")
        echo -e "${GREEN}✓ ISO Image:${NC} ${BUILD_DIR}/nexusos.iso ($(( iso_size / 1024 / 1024 ))MB)"
    fi
    
    if [[ "${WITH_ZFS_DEFAULT}" == "true" ]]; then
        echo -e "${GREEN}✓ ZFS Support:${NC} Enabled as default filesystem"
    fi
    
    echo ""
    echo -e "${CYAN}Next Steps:${NC}"
    if [[ -f "${BUILD_DIR}/nexusos.iso" ]]; then
        echo -e "  • Test ISO: ${WHITE}qemu-system-x86_64 -m 2G -cdrom ${BUILD_DIR}/nexusos.iso${NC}"
    fi
    echo -e "  • Install: ${WHITE}sudo make install${NC} (from build directory)"
    if [[ "${WITH_ZFS_DEFAULT}" == "true" ]] && [[ $EUID -ne 0 ]]; then
        echo -e "  • Setup ZFS: ${WHITE}sudo make zfs-setup${NC} (from build directory)"
    fi
    echo ""
    
    log "INFO: Build completed successfully in ${total_time}s"
}

# Main build process
main() {
    local build_time_start
    build_time_start=$(date +%s)
    
    print_header "NexusOS Complete Build System"
    
    echo -e "${CYAN}Building NexusOS v1.0.0${NC}"
    echo -e "${CYAN}AI-Native Desktop OS with ZFS Default Filesystem${NC}"
    echo ""
    
    # Show build configuration
    print_info "Build Configuration:"
    echo "  • Kernel: ${BUILD_KERNEL}"
    echo "  • Bootloader: ${BUILD_BOOTLOADER}"
    echo "  • Userspace: ${BUILD_USERSPACE}"
    echo "  • Installer: ${BUILD_INSTALLER}"
    echo "  • ISO Image: ${BUILD_ISO}"
    echo "  • ZFS Default: ${WITH_ZFS_DEFAULT}"
    echo "  • AI Features: ${WITH_AI_FEATURES}"
    echo "  • Parallel Jobs: ${PARALLEL_JOBS}"
    echo "  • Build Type: ${CMAKE_BUILD_TYPE}"
    echo ""
    
    # Execute build steps
    check_prerequisites
    setup_build_env
    configure_build
    build_kernel
    build_bootloader
    build_userspace
    build_installer
    build_iso
    run_tests
    setup_zfs_dev
    
    print_summary
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            echo "NexusOS Build Script"
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --help, -h          Show this help"
            echo "  --kernel-only       Build only the kernel"
            echo "  --userspace-only    Build only userspace"
            echo "  --no-iso           Skip ISO creation"
            echo "  --no-zfs           Disable ZFS support"
            echo "  --debug            Debug build"
            echo "  --jobs N           Parallel build jobs (default: $(nproc))"
            echo ""
            echo "Environment Variables:"
            echo "  BUILD_KERNEL        Build kernel (default: true)"
            echo "  BUILD_BOOTLOADER    Build bootloader (default: true)"
            echo "  BUILD_USERSPACE     Build userspace (default: true)"
            echo "  BUILD_INSTALLER     Build installer (default: true)"
            echo "  BUILD_ISO           Build ISO image (default: true)"
            echo "  WITH_ZFS_DEFAULT    Use ZFS as default (default: true)"
            echo "  WITH_AI_FEATURES    Enable AI features (default: true)"
            echo "  PARALLEL_JOBS       Parallel jobs (default: $(nproc))"
            echo "  CMAKE_BUILD_TYPE    Build type (default: Release)"
            exit 0
            ;;
        --kernel-only)
            BUILD_KERNEL="true"
            BUILD_BOOTLOADER="false"
            BUILD_USERSPACE="false"
            BUILD_INSTALLER="false"
            BUILD_ISO="false"
            ;;
        --userspace-only)
            BUILD_KERNEL="false"
            BUILD_BOOTLOADER="false"
            BUILD_USERSPACE="true"
            BUILD_INSTALLER="false"
            BUILD_ISO="false"
            ;;
        --no-iso)
            BUILD_ISO="false"
            ;;
        --no-zfs)
            WITH_ZFS_DEFAULT="false"
            ;;
        --debug)
            CMAKE_BUILD_TYPE="Debug"
            ;;
        --jobs)
            PARALLEL_JOBS="$2"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
    shift
done

# Run main build process
main