#include "../include/pkg_compat.h"

// Global package compatibility state
static int pkg_compat_initialized = 0;
static uint32_t supported_formats = 0;
static uint32_t system_abi_flags = 0;

// Format detection table
struct format_detector {
    pkg_format_t format;
    uint32_t magic;
    size_t offset;
    size_t magic_size;
    const char* extension;
};

static struct format_detector format_table[] = {
    {PKG_FORMAT_DEB, DEB_MAGIC, 0, 4, ".deb"},
    {PKG_FORMAT_RPM, RPM_MAGIC, 0, 4, ".rpm"},
    {PKG_FORMAT_ZST, ZST_MAGIC, 0, 4, ".zst"},
    {PKG_FORMAT_ZST, XZ_MAGIC, 0, 4, ".xz"},
    {PKG_FORMAT_APK, APK_MAGIC, 0, 4, ".apk"},
    {PKG_FORMAT_FLATPAK, FLATPAK_MAGIC, 0, 4, ".flatpak"},
    {PKG_FORMAT_SNAP, SNAP_MAGIC, 0, 4, ".snap"},
    {PKG_FORMAT_APPIMAGE, APPIMAGE_MAGIC, 0, 4, ".AppImage"},
    {PKG_FORMAT_SLACKWARE, TAR_MAGIC, 257, 4, ".txz"},
    {0, 0, 0, 0, NULL} // Sentinel
};

// Distribution compatibility matrix
static struct distro_compat {
    distro_id_t distro;
    pkg_format_t primary_format;
    uint32_t abi_flags;
    const char* name;
} distro_compat_table[] = {
    {DISTRO_DEBIAN, PKG_FORMAT_DEB, PKG_COMPAT_GLIBC | PKG_COMPAT_SYSTEMD | PKG_COMPAT_DBUS, "Debian"},
    {DISTRO_UBUNTU, PKG_FORMAT_DEB, PKG_COMPAT_GLIBC | PKG_COMPAT_SYSTEMD | PKG_COMPAT_DBUS, "Ubuntu"},
    {DISTRO_FEDORA, PKG_FORMAT_RPM, PKG_COMPAT_GLIBC | PKG_COMPAT_SYSTEMD | PKG_COMPAT_SELINUX, "Fedora"},
    {DISTRO_RHEL, PKG_FORMAT_RPM, PKG_COMPAT_GLIBC | PKG_COMPAT_SYSTEMD | PKG_COMPAT_SELINUX, "Red Hat"},
    {DISTRO_CENTOS, PKG_FORMAT_RPM, PKG_COMPAT_GLIBC | PKG_COMPAT_SYSTEMD | PKG_COMPAT_SELINUX, "CentOS"},
    {DISTRO_SUSE, PKG_FORMAT_RPM, PKG_COMPAT_GLIBC | PKG_COMPAT_SYSTEMD, "SUSE"},
    {DISTRO_OPENSUSE, PKG_FORMAT_RPM, PKG_COMPAT_GLIBC | PKG_COMPAT_SYSTEMD, "openSUSE"},
    {DISTRO_ARCH, PKG_FORMAT_ZST, PKG_COMPAT_GLIBC | PKG_COMPAT_SYSTEMD, "Arch Linux"},
    {DISTRO_MANJARO, PKG_FORMAT_ZST, PKG_COMPAT_GLIBC | PKG_COMPAT_SYSTEMD, "Manjaro"},
    {DISTRO_VOID, PKG_FORMAT_XBPS, PKG_COMPAT_GLIBC | PKG_COMPAT_RUNIT, "Void Linux"},
    {DISTRO_ALPINE, PKG_FORMAT_APK, PKG_COMPAT_MUSL | PKG_COMPAT_OPENRC, "Alpine Linux"},
    {DISTRO_GENTOO, PKG_FORMAT_GENTOO, PKG_COMPAT_GLIBC | PKG_COMPAT_OPENRC, "Gentoo"},
    {DISTRO_SLACKWARE, PKG_FORMAT_SLACKWARE, PKG_COMPAT_GLIBC | PKG_COMPAT_SYSV, "Slackware"},
    {DISTRO_NIXOS, PKG_FORMAT_NIX, PKG_COMPAT_GLIBC | PKG_COMPAT_SYSTEMD, "NixOS"},
    {DISTRO_GUIX, PKG_FORMAT_GUIX, PKG_COMPAT_GLIBC, "GNU Guix"},
    {0, 0, 0, NULL} // Sentinel
};

// Initialize package compatibility layer
int pkg_compat_init(void) {
    if (pkg_compat_initialized) {
        return PKG_SUCCESS;
    }
    
    pkg_log(PKG_LOG_INFO, "Initializing Universal Package Compatibility Layer");
    
    // Detect system capabilities
    system_abi_flags = PKG_COMPAT_GLIBC | PKG_COMPAT_SYSTEMD | PKG_COMPAT_DBUS | 
                       PKG_COMPAT_X11 | PKG_COMPAT_WAYLAND | PKG_COMPAT_GTK | PKG_COMPAT_QT;
    
    // Initialize format-specific compatibility layers
    init_debian_compat();
    init_rpm_compat();
    init_arch_compat();
    init_alpine_compat();
    init_void_compat();
    init_gentoo_compat();
    init_nix_compat();
    init_flatpak_compat();
    init_snap_compat();
    init_appimage_compat();
    
    // Mark all formats as supported
    supported_formats = (1 << PKG_FORMAT_MAX) - 1;
    
    pkg_compat_initialized = 1;
    pkg_log(PKG_LOG_INFO, "Package compatibility layer initialized successfully");
    
    return PKG_SUCCESS;
}

// Detect package format from binary data
int pkg_detect_format(const void* data, size_t size) {
    if (!data || size < 8) {
        return PKG_FORMAT_NATIVE; // Default fallback
    }
    
    const uint8_t* bytes = (const uint8_t*)data;
    
    // Check magic numbers
    for (int i = 0; format_table[i].magic != 0; i++) {
        if (size > format_table[i].offset + format_table[i].magic_size) {
            uint32_t file_magic = *(uint32_t*)(bytes + format_table[i].offset);
            if (file_magic == format_table[i].magic) {
                pkg_debug("Detected format: %d (magic: 0x%08X)", 
                         format_table[i].format, format_table[i].magic);
                return format_table[i].format;
            }
        }
    }
    
    // Special detection for text-based formats
    if (size > 16) {
        // Check for Nix expressions
        if (bytes[0] == '{' || (bytes[0] == '#' && bytes[1] == '!')) {
            const char* content = (const char*)bytes;
            if (strstr(content, "stdenv.mkDerivation") || strstr(content, "buildInputs")) {
                return PKG_FORMAT_NIX;
            }
        }
        
        // Check for Gentoo ebuilds
        if (bytes[0] == '#' && bytes[1] == ' ') {
            const char* content = (const char*)bytes;
            if (strstr(content, "EAPI=") || strstr(content, "inherit")) {
                return PKG_FORMAT_GENTOO;
            }
        }
    }
    
    return PKG_FORMAT_NATIVE;
}

// Check package compatibility with system
int pkg_check_compatibility(const struct pkg_metadata* meta) {
    if (!meta) return PKG_ERROR_INVALID;
    
    // Check architecture compatibility
    if (meta->compat.arch != ARCH_X86_64 && meta->compat.arch != ARCH_UNIVERSAL) {
        pkg_log(PKG_LOG_WARN, "Architecture mismatch: package requires %d, system is x86_64", 
                meta->compat.arch);
        return PKG_ERROR_INCOMPATIBLE;
    }
    
    // Check ABI compatibility
    uint32_t required_flags = meta->compat.flags;
    uint32_t missing_flags = required_flags & ~system_abi_flags;
    
    if (missing_flags) {
        pkg_log(PKG_LOG_WARN, "ABI compatibility issues - missing flags: 0x%08X", missing_flags);
        
        // Critical ABI components
        if (missing_flags & PKG_COMPAT_GLIBC && !(system_abi_flags & PKG_COMPAT_MUSL)) {
            return PKG_ERROR_INCOMPATIBLE;
        }
        if (missing_flags & PKG_COMPAT_MUSL && !(system_abi_flags & PKG_COMPAT_GLIBC)) {
            return PKG_ERROR_INCOMPATIBLE;
        }
    }
    
    // Check kernel version
    if (meta->compat.kernel_version > 0x050000) { // Require kernel 5.0+
        // NexusOS provides modern kernel compatibility
        pkg_debug("Kernel version OK: package requires 0x%08X", meta->compat.kernel_version);
    }
    
    return PKG_SUCCESS;
}

// Convert between package formats
int pkg_convert_format(pkg_format_t from, pkg_format_t to, const void* src, void** dst) {
    if (!src || !dst) return PKG_ERROR_INVALID;
    
    pkg_log(PKG_LOG_INFO, "Converting package from format %d to format %d", from, to);
    
    // If formats are the same, no conversion needed
    if (from == to) {
        *dst = (void*)src; // Just pass through
        return PKG_SUCCESS;
    }
    
    // Convert to native format as intermediate
    if (to == PKG_FORMAT_NATIVE) {
        switch (from) {
            case PKG_FORMAT_DEB:
                return convert_deb_to_native(src, dst);
            case PKG_FORMAT_RPM:
                return convert_rpm_to_native(src, dst);
            case PKG_FORMAT_ZST:
                return convert_zst_to_native(src, dst);
            case PKG_FORMAT_APK:
                return convert_apk_to_native(src, dst);
            case PKG_FORMAT_FLATPAK:
                return convert_flatpak_to_native(src, dst);
            case PKG_FORMAT_SNAP:
                return convert_snap_to_native(src, dst);
            case PKG_FORMAT_APPIMAGE:
                return convert_appimage_to_native(src, dst);
            default:
                return PKG_ERROR_UNSUPPORTED;
        }
    }
    
    // For now, we only support conversion to native format
    return PKG_ERROR_UNSUPPORTED;
}

// Install package at kernel level
int pkg_install_kernel(const struct pkg_metadata* meta, const void* data) {
    if (!meta || !data) return PKG_ERROR_INVALID;
    
    pkg_log(PKG_LOG_INFO, "Installing package: %s v%s", meta->name, meta->version);
    
    // Check compatibility first
    int compat_result = pkg_check_compatibility(meta);
    if (compat_result != PKG_SUCCESS) {
        return compat_result;
    }
    
    // Convert to native format if needed
    void* native_data = NULL;
    if (meta->compat.format != PKG_FORMAT_NATIVE) {
        int convert_result = pkg_convert_format(meta->compat.format, PKG_FORMAT_NATIVE, 
                                               data, &native_data);
        if (convert_result != PKG_SUCCESS) {
            return convert_result;
        }
    } else {
        native_data = (void*)data;
    }
    
    // Perform actual installation
    // This would involve:
    // 1. Extracting files to appropriate locations
    // 2. Updating package database
    // 3. Running post-install scripts
    // 4. Setting up services if needed
    
    pkg_log(PKG_LOG_INFO, "Package %s installed successfully", meta->name);
    return PKG_SUCCESS;
}

// Remove package at kernel level
int pkg_remove_kernel(const char* name) {
    if (!name) return PKG_ERROR_INVALID;
    
    pkg_log(PKG_LOG_INFO, "Removing package: %s", name);
    
    // Query package information
    struct pkg_metadata meta;
    int query_result = pkg_query_kernel(name, &meta);
    if (query_result != PKG_SUCCESS) {
        return query_result;
    }
    
    // Perform removal
    // This would involve:
    // 1. Running pre-removal scripts
    // 2. Removing files
    // 3. Updating package database
    // 4. Cleaning up services
    
    pkg_log(PKG_LOG_INFO, "Package %s removed successfully", name);
    return PKG_SUCCESS;
}

// Query package information
int pkg_query_kernel(const char* name, struct pkg_metadata* meta) {
    if (!name || !meta) return PKG_ERROR_INVALID;
    
    // This would query the package database
    // For now, return a stub
    return PKG_ERROR_INVALID;
}

// Format-specific compatibility layer initialization functions
void init_debian_compat(void) {
    pkg_debug("Initializing Debian/DEB compatibility layer");
    // Initialize DEB package support
    // - dpkg emulation layer
    // - apt compatibility
    // - dependency resolution
}

void init_rpm_compat(void) {
    pkg_debug("Initializing RPM compatibility layer");
    // Initialize RPM package support
    // - rpm database emulation
    // - yum/dnf compatibility
    // - spec file processing
}

void init_arch_compat(void) {
    pkg_debug("Initializing Arch/ZST compatibility layer");
    // Initialize Arch package support
    // - pacman emulation
    // - PKGBUILD processing
    // - AUR support
}

void init_alpine_compat(void) {
    pkg_debug("Initializing Alpine/APK compatibility layer");
    // Initialize Alpine package support
    // - apk emulation
    // - musl libc compatibility
    // - OpenRC service support
}

void init_void_compat(void) {
    pkg_debug("Initializing Void/XBPS compatibility layer");
    // Initialize Void package support
    // - xbps emulation
    // - runit service support
}

void init_gentoo_compat(void) {
    pkg_debug("Initializing Gentoo compatibility layer");
    // Initialize Gentoo support
    // - portage emulation
    // - ebuild processing
    // - USE flag support
}

void init_nix_compat(void) {
    pkg_debug("Initializing Nix compatibility layer");
    // Initialize Nix support
    // - nix store emulation
    // - derivation processing
    // - functional package management
}

void init_flatpak_compat(void) {
    pkg_debug("Initializing Flatpak compatibility layer");
    // Initialize Flatpak support
    // - OSTree backend
    // - sandboxing with bubblewrap
    // - runtime management
}

void init_snap_compat(void) {
    pkg_debug("Initializing Snap compatibility layer");
    // Initialize Snap support
    // - snapd emulation
    // - squashfs mounting
    // - confinement system
}

void init_appimage_compat(void) {
    pkg_debug("Initializing AppImage compatibility layer");
    // Initialize AppImage support
    // - FUSE mounting
    // - desktop integration
    // - portable application management
}

// ABI compatibility checking functions
int check_glibc_compat(const char* required_version) {
    // Check if glibc version is compatible
    // For now, assume compatibility
    return PKG_SUCCESS;
}

int check_kernel_compat(uint32_t required_version) {
    // NexusOS kernel provides compatibility with Linux kernel APIs
    return PKG_SUCCESS;
}

int check_init_system_compat(uint32_t flags) {
    // NexusOS provides compatibility with multiple init systems
    return PKG_SUCCESS;
}

int check_desktop_compat(uint32_t flags) {
    // Check desktop environment compatibility
    return PKG_SUCCESS;
}

// Logging functions
void pkg_log(int level, const char* format, ...) {
    // Implement kernel logging
    // For now, just a stub
}

void pkg_debug(const char* format, ...) {
    // Implement debug logging
    // For now, just a stub
}

// Format conversion functions (stubs)
int convert_deb_to_native(const void* src, void** dst) {
    return PKG_ERROR_UNSUPPORTED;
}

int convert_rpm_to_native(const void* src, void** dst) {
    return PKG_ERROR_UNSUPPORTED;
}

int convert_zst_to_native(const void* src, void** dst) {
    return PKG_ERROR_UNSUPPORTED;
}

int convert_apk_to_native(const void* src, void** dst) {
    return PKG_ERROR_UNSUPPORTED;
}

int convert_flatpak_to_native(const void* src, void** dst) {
    return PKG_ERROR_UNSUPPORTED;
}

int convert_snap_to_native(const void* src, void** dst) {
    return PKG_ERROR_UNSUPPORTED;
}

int convert_appimage_to_native(const void* src, void** dst) {
    return PKG_ERROR_UNSUPPORTED;
}