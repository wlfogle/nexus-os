cmake_minimum_required(VERSION 3.22)

project(NexusOS 
    VERSION 1.0.0
    DESCRIPTION "AI-Native Desktop Operating System with ZFS Default Filesystem"
    HOMEPAGE_URL "https://nexusos.org"
    LANGUAGES C CXX ASM
)

# Set default build type
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Build options
option(BUILD_KERNEL "Build NexusOS kernel" ON)
option(BUILD_BOOTLOADER "Build UEFI bootloader" ON)
option(BUILD_USERSPACE "Build userspace components" ON)
option(BUILD_INSTALLER "Build Calamares-based installer with ZFS support" ON)
option(BUILD_ISO "Build bootable ISO image" ON)
option(WITH_ZFS_DEFAULT "Use ZFS as default filesystem" ON)
option(WITH_AI_FEATURES "Enable AI features and models" ON)
option(ENABLE_TESTING "Enable testing framework" ON)

# Compiler requirements
set(CMAKE_C_STANDARD 17)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_ASM_SYNTAX INTEL)

# Cross-compilation setup for x86_64 target
set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR x86_64)

# Compiler flags for kernel build
set(KERNEL_CFLAGS 
    "-ffreestanding"
    "-fno-stack-protector"
    "-fno-stack-check"
    "-mno-red-zone"
    "-mno-mmx"
    "-mno-sse"
    "-mno-sse2"
    "-mcmodel=kernel"
    "-Wall"
    "-Wextra"
    "-Werror"
)

# Linker flags for kernel
set(KERNEL_LDFLAGS
    "-T ${CMAKE_SOURCE_DIR}/kernel/arch/x86_64/linker.ld"
    "-nostdlib"
    "-zmax-page-size=0x1000"
)

# Find required packages
find_package(PkgConfig REQUIRED)

# ZFS support requirements
if(WITH_ZFS_DEFAULT)
    find_program(ZFS_CMD zfs REQUIRED)
    find_program(ZPOOL_CMD zpool REQUIRED)
    find_library(ZFS_LIB zfs)
    find_library(NVPAIR_LIB nvpair)
    find_library(UUTIL_LIB uutil)
    find_library(ZFS_CORE_LIB zfs_core)
    
    if(NOT ZFS_LIB)
        message(FATAL_ERROR "ZFS libraries not found. Install zfs-dkms and zfs-utils packages.")
    endif()
    
    add_definitions(-DWITH_ZFS_DEFAULT=1)
    message(STATUS "ZFS default filesystem support enabled")
endif()

# AI features support
if(WITH_AI_FEATURES)
    find_package(PkgConfig REQUIRED)
    add_definitions(-DWITH_AI_FEATURES=1)
    message(STATUS "AI features enabled")
endif()

# Global include directories
include_directories(
    ${CMAKE_SOURCE_DIR}/kernel/include
    ${CMAKE_SOURCE_DIR}/userspace/include
    ${CMAKE_SOURCE_DIR}/common/include
)

# Installation directories
set(CMAKE_INSTALL_PREFIX /usr/local/nexusos)
set(NEXUS_KERNEL_DIR ${CMAKE_INSTALL_PREFIX}/boot)
set(NEXUS_MODULES_DIR ${CMAKE_INSTALL_PREFIX}/lib/modules)
set(NEXUS_CONFIG_DIR ${CMAKE_INSTALL_PREFIX}/etc)
set(NEXUS_BIN_DIR ${CMAKE_INSTALL_PREFIX}/bin)
set(NEXUS_LIB_DIR ${CMAKE_INSTALL_PREFIX}/lib)

# Custom functions for NexusOS build
function(nexus_add_kernel_module target)
    add_library(${target} STATIC ${ARGN})
    target_compile_options(${target} PRIVATE ${KERNEL_CFLAGS})
    target_include_directories(${target} PRIVATE 
        ${CMAKE_SOURCE_DIR}/kernel/include
        ${CMAKE_SOURCE_DIR}/kernel/arch/x86_64/include
    )
endfunction()

function(nexus_add_userspace_binary target)
    add_executable(${target} ${ARGN})
    target_include_directories(${target} PRIVATE
        ${CMAKE_SOURCE_DIR}/userspace/include
        ${CMAKE_SOURCE_DIR}/common/include
    )
endfunction()

# Version information
configure_file(
    ${CMAKE_SOURCE_DIR}/version.h.in
    ${CMAKE_BINARY_DIR}/version.h
)

# Build subdirectories
if(BUILD_BOOTLOADER)
    add_subdirectory(bootloader)
endif()

if(BUILD_KERNEL)
    add_subdirectory(kernel)
endif()

if(BUILD_USERSPACE)
    add_subdirectory(userspace)
endif()

if(BUILD_INSTALLER)
    add_subdirectory(installer)
endif()

# Documentation
add_subdirectory(docs)

# Testing
if(ENABLE_TESTING)
    enable_testing()
    add_subdirectory(tests)
endif()

# Tools and utilities
add_subdirectory(tools)

# Custom targets for building complete system

# Kernel target
add_custom_target(nexus-kernel
    DEPENDS nexus_kernel_binary
    COMMENT "Building NexusOS kernel"
)

# Bootloader target
add_custom_target(nexus-bootloader
    DEPENDS uefi_bootloader
    COMMENT "Building UEFI bootloader"
)

# Userspace target
add_custom_target(nexus-userspace
    DEPENDS nexus_init nexus_file_manager nexus_desktop
    COMMENT "Building NexusOS userspace"
)

# ISO generation target
if(BUILD_ISO)
    find_program(GRUB_MKRESCUE grub-mkrescue REQUIRED)
    find_program(XORRISO xorriso REQUIRED)
    
    add_custom_target(nexus-iso
        COMMAND ${CMAKE_SOURCE_DIR}/tools/build-iso.sh 
            ${CMAKE_BINARY_DIR} 
            ${CMAKE_SOURCE_DIR}
            ${CMAKE_BINARY_DIR}/nexusos.iso
        DEPENDS nexus-kernel nexus-bootloader nexus-userspace
        COMMENT "Building NexusOS bootable ISO with ZFS support"
        VERBATIM
    )
endif()

# Installation target
add_custom_target(nexus-install
    COMMAND ${CMAKE_SOURCE_DIR}/tools/install-system.sh 
        ${CMAKE_BINARY_DIR}
        ${CMAKE_INSTALL_PREFIX}
    DEPENDS nexus-kernel nexus-bootloader nexus-userspace
    COMMENT "Installing NexusOS to system"
    VERBATIM
)

# Complete build target
add_custom_target(nexus-complete
    DEPENDS nexus-kernel nexus-bootloader nexus-userspace
    COMMENT "Building complete NexusOS system"
)

if(BUILD_ISO)
    add_dependencies(nexus-complete nexus-iso)
endif()

# ZFS pool creation helper
if(WITH_ZFS_DEFAULT)
    add_custom_target(zfs-setup
        COMMAND ${CMAKE_SOURCE_DIR}/tools/setup-zfs.sh
        COMMENT "Setting up ZFS pools and datasets"
        VERBATIM
    )
endif()

# Development helpers
add_custom_target(clean-all
    COMMAND ${CMAKE_BUILD_TOOL} clean
    COMMAND rm -rf ${CMAKE_BINARY_DIR}/iso
    COMMAND rm -f ${CMAKE_BINARY_DIR}/*.iso
    COMMENT "Clean all build artifacts"
)

# Print build configuration
message(STATUS "")
message(STATUS "NexusOS Build Configuration:")
message(STATUS "============================")
message(STATUS "Version: ${PROJECT_VERSION}")
message(STATUS "Build Type: ${CMAKE_BUILD_TYPE}")
message(STATUS "Target: ${CMAKE_SYSTEM_PROCESSOR}")
message(STATUS "")
message(STATUS "Components:")
message(STATUS "  Kernel:     ${BUILD_KERNEL}")
message(STATUS "  Bootloader: ${BUILD_BOOTLOADER}") 
message(STATUS "  Userspace:  ${BUILD_USERSPACE}")
message(STATUS "  Installer:  ${BUILD_INSTALLER}")
message(STATUS "  ISO Image:  ${BUILD_ISO}")
message(STATUS "")
message(STATUS "Features:")
message(STATUS "  ZFS Default:  ${WITH_ZFS_DEFAULT}")
message(STATUS "  AI Features:  ${WITH_AI_FEATURES}")
message(STATUS "  Testing:      ${ENABLE_TESTING}")
message(STATUS "")
message(STATUS "Install Prefix: ${CMAKE_INSTALL_PREFIX}")
message(STATUS "")

# Summary
message(STATUS "Ready to build NexusOS!")
message(STATUS "Run 'make nexus-complete' to build the complete system")
if(BUILD_ISO)
    message(STATUS "Run 'make nexus-iso' to create bootable ISO")
endif()
if(WITH_ZFS_DEFAULT)
    message(STATUS "Run 'make zfs-setup' to configure ZFS (requires root)")
endif()
message(STATUS "")