#ifndef PKG_COMPAT_H
#define PKG_COMPAT_H

#include <stdint.h>
#include <stddef.h>

// Universal Package Compatibility Layer
// Provides kernel-level support for all Linux package formats

// Package format identifiers
typedef enum {
    PKG_FORMAT_NATIVE = 0,      // NexusOS native
    PKG_FORMAT_DEB = 1,         // Debian/Ubuntu
    PKG_FORMAT_RPM = 2,         // RedHat/SUSE/Fedora
    PKG_FORMAT_ZST = 3,         // Arch Linux
    PKG_FORMAT_XBPS = 4,        // Void Linux
    PKG_FORMAT_APK = 5,         // Alpine Linux
    PKG_FORMAT_FLATPAK = 6,     // Universal sandboxed
    PKG_FORMAT_SNAP = 7,        // Ubuntu Snap
    PKG_FORMAT_APPIMAGE = 8,    // Portable applications
    PKG_FORMAT_NIX = 9,         // NixOS packages
    PKG_FORMAT_GUIX = 10,       // GNU Guix
    PKG_FORMAT_GENTOO = 11,     // Gentoo ebuilds
    PKG_FORMAT_SLACKWARE = 12,  // Slackware txz
    PKG_FORMAT_CRUX = 13,       // CRUX ports
    PKG_FORMAT_LFS = 14,        // Linux From Scratch
    PKG_FORMAT_KISS = 15,       // KISS Linux
    PKG_FORMAT_CONDA = 16,      // Conda packages
    PKG_FORMAT_BREW = 17,       // Homebrew (Linux)
    PKG_FORMAT_DOCKER = 18,     // Docker containers
    PKG_FORMAT_OCI = 19,        // OCI containers
    PKG_FORMAT_MAX
} pkg_format_t;

// Distribution identifiers
typedef enum {
    DISTRO_UNKNOWN = 0,
    DISTRO_DEBIAN,
    DISTRO_UBUNTU,
    DISTRO_FEDORA,
    DISTRO_RHEL,
    DISTRO_CENTOS,
    DISTRO_SUSE,
    DISTRO_OPENSUSE,
    DISTRO_ARCH,
    DISTRO_MANJARO,
    DISTRO_VOID,
    DISTRO_ALPINE,
    DISTRO_GENTOO,
    DISTRO_SLACKWARE,
    DISTRO_NIXOS,
    DISTRO_GUIX,
    DISTRO_CRUX,
    DISTRO_LFS,
    DISTRO_KISS,
    DISTRO_CLEAR_LINUX,
    DISTRO_SOLUS,
    DISTRO_ELEMENTARY,
    DISTRO_MINT,
    DISTRO_KALI,
    DISTRO_PARROT,
    DISTRO_TAILS,
    DISTRO_MAX
} distro_id_t;

// Architecture compatibility
typedef enum {
    ARCH_X86_64 = 0,
    ARCH_I386,
    ARCH_ARM64,
    ARCH_ARMHF,
    ARCH_RISCV64,
    ARCH_PPC64LE,
    ARCH_S390X,
    ARCH_MIPS64,
    ARCH_UNIVERSAL,
    ARCH_MAX
} arch_t;

// Package compatibility information
struct pkg_compat_info {
    pkg_format_t format;
    distro_id_t distro;
    arch_t arch;
    uint32_t abi_version;
    uint32_t kernel_version;
    char signature[64];
    uint32_t flags;
};

// ABI compatibility flags
#define PKG_COMPAT_GLIBC        (1 << 0)
#define PKG_COMPAT_MUSL         (1 << 1)
#define PKG_COMPAT_SYSTEMD      (1 << 2)
#define PKG_COMPAT_OPENRC       (1 << 3)
#define PKG_COMPAT_RUNIT        (1 << 4)
#define PKG_COMPAT_SYSV         (1 << 5)
#define PKG_COMPAT_DBUS         (1 << 6)
#define PKG_COMPAT_PULSEAUDIO   (1 << 7)
#define PKG_COMPAT_ALSA         (1 << 8)
#define PKG_COMPAT_X11          (1 << 9)
#define PKG_COMPAT_WAYLAND      (1 << 10)
#define PKG_COMPAT_GTK          (1 << 11)
#define PKG_COMPAT_QT           (1 << 12)
#define PKG_COMPAT_SELINUX      (1 << 13)
#define PKG_COMPAT_APPARMOR     (1 << 14)

// Package metadata structure
struct pkg_metadata {
    char name[256];
    char version[64];
    char description[512];
    char maintainer[128];
    char homepage[256];
    char license[128];
    uint64_t size_installed;
    uint64_t size_download;
    char dependencies[1024];
    char conflicts[512];
    char provides[512];
    char replaces[512];
    struct pkg_compat_info compat;
    uint32_t checksum[8]; // SHA-256
};

// Kernel syscall numbers for package operations
#define SYS_PKG_INSTALL         400
#define SYS_PKG_REMOVE          401
#define SYS_PKG_QUERY           402
#define SYS_PKG_LIST            403
#define SYS_PKG_VERIFY          404
#define SYS_PKG_CONVERT         405
#define SYS_PKG_COMPAT_CHECK    406

// Function prototypes for kernel package support
int pkg_compat_init(void);
int pkg_detect_format(const void* data, size_t size);
int pkg_check_compatibility(const struct pkg_metadata* meta);
int pkg_convert_format(pkg_format_t from, pkg_format_t to, const void* src, void** dst);
int pkg_install_kernel(const struct pkg_metadata* meta, const void* data);
int pkg_remove_kernel(const char* name);
int pkg_query_kernel(const char* name, struct pkg_metadata* meta);

// Compatibility layer initialization
void init_debian_compat(void);
void init_rpm_compat(void);
void init_arch_compat(void);
void init_alpine_compat(void);
void init_void_compat(void);
void init_gentoo_compat(void);
void init_nix_compat(void);
void init_flatpak_compat(void);
void init_snap_compat(void);
void init_appimage_compat(void);

// ABI compatibility checks
int check_glibc_compat(const char* required_version);
int check_kernel_compat(uint32_t required_version);
int check_init_system_compat(uint32_t flags);
int check_desktop_compat(uint32_t flags);

// Format detection magic numbers
#define DEB_MAGIC       0x21726164  // "ar!\n"
#define RPM_MAGIC       0xEDABEEDB
#define ZST_MAGIC       0x28B52FFD
#define XZ_MAGIC        0xFD377A58
#define APK_MAGIC       0x504B0304  // ZIP-like
#define FLATPAK_MAGIC   0x4F535452  // OSTree
#define SNAP_MAGIC      0x73717368  // "sqsh"
#define APPIMAGE_MAGIC  0x7F454C46  // ELF
#define TAR_MAGIC       0x75737461  // "usta"

// Error codes
#define PKG_SUCCESS             0
#define PKG_ERROR_INVALID       -1
#define PKG_ERROR_INCOMPATIBLE  -2
#define PKG_ERROR_DEPENDENCY    -3
#define PKG_ERROR_CONFLICT      -4
#define PKG_ERROR_PERMISSION    -5
#define PKG_ERROR_DISK_SPACE    -6
#define PKG_ERROR_NETWORK       -7
#define PKG_ERROR_CORRUPTION    -8
#define PKG_ERROR_UNSUPPORTED   -9

// Logging and debug
void pkg_log(int level, const char* format, ...);
void pkg_debug(const char* format, ...);

#define PKG_LOG_ERROR   0
#define PKG_LOG_WARN    1
#define PKG_LOG_INFO    2
#define PKG_LOG_DEBUG   3

#endif // PKG_COMPAT_H