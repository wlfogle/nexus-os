//! NexusOS Installer
//!
//! Runs as a kernel task at first boot when no FAT32 filesystem is present.
//! Writes a complete, bootable NexusOS installation to the VirtIO disk:
//!
//!   1. GPT + protective MBR (sectors 0..33)
//!   2. FAT32 EFI System Partition (sector 34..end)
//!   3. EFI/BOOT/BOOTX64.EFI  — Limine UEFI bootloader (embedded)
//!   4. boot/nexus-kernel      — this running kernel (copied from HHDM)
//!   5. boot/limine.conf       — boot configuration

pub mod crc32;
pub mod gpt;

use gpt::write_gpt;
use alloc::format;
use crate::{kprintln, fs};
use core::sync::atomic::Ordering;

// Disk capacity: arch-conditional
#[cfg(target_arch = "x86_64")]
use crate::drivers::blockdev::capacity;
#[cfg(target_arch = "aarch64")]
use crate::virtio_mmio::capacity;

// ─── Assets embedded at compile time ─────────────────────────────────────────

#[cfg(target_arch = "x86_64")]
static BOOT_EFI: &[u8] = include_bytes!("../../../limine/bin/BOOTX64.EFI");
#[cfg(target_arch = "aarch64")]
static BOOT_EFI: &[u8] = include_bytes!("../../../limine/bin/BOOTAA64.EFI");

#[cfg(target_arch = "x86_64")]
const BOOT_EFI_NAME: &str = "EFI/BOOT/BOOTX64.EFI";
#[cfg(target_arch = "aarch64")]
const BOOT_EFI_NAME: &str = "EFI/BOOT/BOOTAA64.EFI";

// Keep old name for compat — points to arch-correct binary above
#[allow(dead_code)]
static BOOTX64_EFI: &[u8] = BOOT_EFI;


/// Reference user program (static ELF64), written to the ESP root so the
/// ring-3 shell can `run HELLO.ELF` on an installed system.  Assembled and
/// linked by build.rs (see kernel/src/userspace/hello.asm).
static HELLO_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/hello.elf"));
static LINUX_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/linux_hello.elf"));
fn target_name() -> &'static str {
    if cfg!(feature = "tiamat") {
        "tiamat"
    } else if cfg!(feature = "bahamut") {
        "bahamut"
    } else {
        "laptop"
    }
}

fn target_title() -> &'static str {
    if cfg!(feature = "tiamat") {
        "NexusOS Tiamat"
    } else if cfg!(feature = "bahamut") {
        "NexusOS Bahamut"
    } else {
        "NexusOS Laptop"
    }
}

fn target_role() -> &'static str {
    if cfg!(feature = "tiamat") {
        "media-center"
    } else if cfg!(feature = "bahamut") {
        "network-edge"
    } else {
        "control-center"
    }
}

fn target_description() -> &'static str {
    if cfg!(feature = "tiamat") {
        "Media/file-share host profile for nexus-mediastack storage, services, and observability."
    } else if cfg!(feature = "bahamut") {
        "Lightweight edge/network profile for DNS, ingress, VPN, secrets, and routing."
    } else {
        "Control-center profile for NexusTerminal, Cockpit, Ollama, Stella, Max Jr., and orchestration."
    }
}

fn limine_conf() -> alloc::string::String {
    format!(
        "# NexusOS Boot Configuration\n\
timeout: 5\n\
default_entry: 1\n\
\n\
/{title}\n\
    protocol: limine\n\
    path: boot():/boot/nexus-kernel\n\
    cmdline: target={target} role={role} loglevel=info\n",
        title = target_title(),
        target = target_name(),
        role = target_role(),
    )
}

fn root_readme() -> alloc::string::String {
    format!(
        "NexusOS installed skeleton\n\
\n\
Target: {target}\n\
Role: {role}\n\
Purpose: {description}\n\
\n\
This disk was installed by the from-scratch NexusOS kernel installer.\n\
The skeleton layout reserves roots for personalities, packages, services,\n\
media workloads, user data, logs, and role-specific config.\n\
\n\
Try from the nexus> shell:\n\
  ls /\n\
  ls /bin\n\
  ls /personalities\n\
  cat /etc/os-release\n\
  run HELLO.ELF\n\
  run LINUX.ELF\n",
        target = target_name(),
        role = target_role(),
        description = target_description(),
    )
}

fn os_release() -> alloc::string::String {
    format!(
        "NAME=NexusOS\n\
PRETTY_NAME=\"NexusOS from-scratch microkernel\"\n\
ID=nexusos\n\
VERSION_ID=0.6.3\n\
VERSION=\"0.6.3 skeleton installer\"\n\
BUILD_TARGET={target}\n\
NEXUS_ROLE={role}\n\
AI_NATIVE=1\n",
        target = target_name(),
        role = target_role(),
    )
}

fn nexus_toml() -> alloc::string::String {
    format!(
        "[system]\n\
name = \"NexusOS\"\n\
version = \"0.6.3\"\n\
target = \"{target}\"\n\
role = \"{role}\"\n\
description = \"{description}\"\n\
\n\
[paths]\n\
boot = \"/boot\"\n\
bin = \"/bin\"\n\
home = \"/home\"\n\
packages = \"/pkg\"\n\
personalities = \"/personalities\"\n\
services = \"/services\"\n\
media = \"/media\"\n\
var = \"/var\"\n\
\n\
[personalities]\n\
nexus = \"native NexusOS ABI\"\n\
linux = \"Linux ELF/syscall personality\"\n\
bsd = \"BSD/POSIX personality\"\n\
windows = \"PE/Win32 personality\"\n\
macos = \"Mach-O/Darwin personality\"\n\
\n\
[roles]\n\
laptop = \"control-center\"\n\
tiamat = \"media-center\"\n\
bahamut = \"network-edge\"\n",
        target = target_name(),
        role = target_role(),
        description = target_description(),
    )
}

static PKG_README: &[u8] = b"NexusOS package root\n\
\n\
This tree is reserved for the Universal Package Substrate.\n\
Packages from Linux, BSD, Windows, macOS, language ecosystems, and portable\n\
formats will be installed through personality servers and exposed into /bin or\n\
per-profile namespaces.\n";

static PERSONALITIES_README: &[u8] = b"NexusOS personality roots\n\
\n\
Reserved personalities:\n\
  nexus.linux    Linux ELF + Linux syscall ABI\n\
  nexus.bsd      BSD/POSIX ELF ABI\n\
  nexus.win      PE/COFF + Win32 ABI\n\
  nexus.macos    Mach-O + Darwin ABI\n\
  nexus.native   NexusOS native ABI\n";

static SERVICES_README: &[u8] = b"NexusOS service roots\n\
\n\
This tree is reserved for AI services, package services, media workloads,\n\
network services, and future personality daemons.\n";

static LAPTOP_ROLE_README: &[u8] = b"NexusOS Laptop role: control center\n\
\n\
Reserved for NexusTerminal, Cockpit, Ollama, Stella, Max Jr., orchestration,\n\
model management, and administrative dashboards.\n";

static TIAMAT_ROLE_README: &[u8] = b"NexusOS Tiamat role: media center\n\
\n\
Reserved for nexus-mediastack service profiles, file-share roots, media indexes,\n\
transcode/cache paths, and health/observability data.\n";

static BAHAMUT_ROLE_README: &[u8] = b"NexusOS Bahamut role: network edge\n\
\n\
Reserved for DNS, ingress, VPN, secrets, lightweight routing, and edge service\n\
metadata.  Keep this profile small and RAM-conscious.\n";

static HOME_README: &[u8] = b"NexusOS user home skeleton\n\
\n\
Persistent user data begins here.  The current shell can list and read files;\n\
write-path shell commands land in the next VFS milestone.\n";

/// EFI shell startup script: auto-launches Limine on first power-on.
#[cfg(target_arch = "x86_64")]
static STARTUP_NSH: &[u8] = b"\\EFI\\BOOT\\BOOTX64.EFI\r\n";
#[cfg(target_arch = "aarch64")]
static STARTUP_NSH: &[u8] = b"\\EFI\\BOOT\\BOOTAA64.EFI\r\n";

// ─── Kernel ELF globals (set by _start from KernelFileRequest) ───────────────
// These hold the virtual address and byte-length of the original ELF file that
// Limine loaded from disk.  Writing these bytes to the installed disk produces
// a proper ELF that Limine can load on the next boot.

pub static KERNEL_ELF_BASE: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
pub static KERNEL_ELF_SIZE: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

// ─── Installer task ───────────────────────────────────────────────────────────

/// Synchronous installer entry for AArch64 (no scheduler — called directly from boot).
pub fn task_installer_run() {
    installer_body();
}

/// Scheduler task entry for x86_64.
#[cfg(target_arch = "x86_64")]
pub extern "C" fn task_installer() -> ! {
    for _ in 0..100 {
        unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
    }
    installer_body();
    loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack)); } }
}

fn installer_body() {
    if capacity() == 0 {
        kprintln!("[install] No disk — skipping.");
        return;
    }

    if fs::fat::is_mounted() {
        kprintln!("[install] Already installed.");
        return;
    }

    kprintln!();
    kprintln!("╔══════════════════════════════════════════╗");
    kprintln!("║        NexusOS Installer v0.6            ║");
    kprintln!("║  World's First AI-Native OS              ║");
    kprintln!("║  target={:<8} role={:<15} ║", target_name(), target_role());
    kprintln!("╚══════════════════════════════════════════╝");
    kprintln!();

    match run_install() {
        Ok(()) => {
            kprintln!();
            kprintln!("╔══════════════════════════════════════════╗");
            kprintln!("║   ✓  Installation complete!              ║");
            kprintln!("║   Remove ISO and reboot.                 ║");
            kprintln!("╚══════════════════════════════════════════╝");
        }
        Err(e) => kprintln!("[install] FAILED: {}", e),
    }
}

fn run_install() -> Result<(), &'static str> {
    kprintln!("[install] Disk: {} GiB", capacity() / (2 * 1024 * 1024));

    kprintln!("[install] Writing GPT...");
    let esp_lba = write_gpt()?;

    kprintln!("[install] Formatting ESP as FAT32 (LBA {})...", esp_lba);
    format_esp(esp_lba)?;

    kprintln!("[install] Creating skeleton directories...");
    for dir in [
        "EFI", "EFI/BOOT",
        "boot", "bin", "etc", "home", "pkg", "personalities", "services",
        "media", "var", "var/log", "var/cache", "tmp", "roles",
        "roles/laptop", "roles/tiamat", "roles/bahamut",
        "personalities/linux", "personalities/bsd", "personalities/windows",
        "personalities/macos", "personalities/native",
    ] {
        fs::fat::mkdir(dir)?;
    }

    kprintln!("[install] Writing {} ({} KB)...", BOOT_EFI_NAME, BOOT_EFI.len() / 1024);
    write_file_to_esp(BOOT_EFI_NAME, BOOT_EFI)?;

    kprintln!("[install] Writing nexus-kernel...");
    write_kernel()?;

    kprintln!("[install] Writing limine.conf...");
    let limine = limine_conf();
    write_file_to_esp("EFI/BOOT/limine.conf", limine.as_bytes())?;

    kprintln!("[install] Writing startup.nsh...");
    write_file_to_esp("startup.nsh", STARTUP_NSH)?;

    kprintln!("[install] Writing HELLO.ELF ({} bytes)...", HELLO_ELF.len());
    write_file_to_esp("HELLO.ELF", HELLO_ELF)?;
    write_file_to_esp("bin/HELLO.ELF", HELLO_ELF)?;

    kprintln!("[install] Writing LINUX.ELF ({} bytes)...", LINUX_ELF.len());
    write_file_to_esp("LINUX.ELF", LINUX_ELF)?;
    write_file_to_esp("bin/LINUX.ELF", LINUX_ELF)?;

    kprintln!("[install] Writing skeleton metadata...");
    let readme = root_readme();
    let os_release = os_release();
    let nexus_toml = nexus_toml();
    write_file_to_esp("README.TXT", readme.as_bytes())?;
    write_file_to_esp("etc/os-release", os_release.as_bytes())?;
    write_file_to_esp("etc/nexus.toml", nexus_toml.as_bytes())?;
    write_file_to_esp("pkg/README.TXT", PKG_README)?;
    write_file_to_esp("personalities/README.TXT", PERSONALITIES_README)?;
    write_file_to_esp("services/README.TXT", SERVICES_README)?;
    write_file_to_esp("home/README.TXT", HOME_README)?;
    write_file_to_esp("roles/laptop/README.TXT", LAPTOP_ROLE_README)?;
    write_file_to_esp("roles/tiamat/README.TXT", TIAMAT_ROLE_README)?;
    write_file_to_esp("roles/bahamut/README.TXT", BAHAMUT_ROLE_README)?;

    Ok(())
}

// ─── Format ESP ───────────────────────────────────────────────────────────────

fn format_esp(esp_lba: u64) -> Result<(), &'static str> {
    use crate::fs::fat::DiskIo;
    use fatfs::{FileSystem, FsOptions, FormatVolumeOptions, FatType, NullTimeProvider};

    {
        let mut disk = DiskIo::at_partition(esp_lba);
        fatfs::format_volume(
            &mut disk,
            FormatVolumeOptions::new()
                .fat_type(FatType::Fat32)
                .volume_label(*b"NEXUSOS    "),
        ).map_err(|_| "installer: format failed")?;
    }

    let opts = FsOptions::new().time_provider(NullTimeProvider::new());
    let fs = FileSystem::new(DiskIo::at_partition(esp_lba), opts)
        .map_err(|_| "installer: mount failed")?;

    *fs::fat::FS.lock() = Some(fs);
    Ok(())
}

// ─── File writing ─────────────────────────────────────────────────────────────

fn write_file_to_esp(path: &str, data: &[u8]) -> Result<(), &'static str> {
    use fatfs::Write;
    let guard = fs::fat::FS.lock();
    let fatfs = guard.as_ref().ok_or("installer: not mounted")?;
    let result = {
        let root = fatfs.root_dir();
        let mut file = root.create_file(path).map_err(|_| "installer: create failed")?;
        file.truncate().map_err(|_| "installer: truncate failed")?;
        let mut written = 0usize;
        while written < data.len() {
            match file.write(&data[written..]) {
                Ok(0) => return Err("installer: disk full"),
                Ok(n) => written += n,
                Err(_) => return Err("installer: write error"),
            }
        }
        file.flush().map_err(|_| "installer: flush failed")?;
        Ok(())
    };
    result
}

fn write_kernel() -> Result<(), &'static str> {
    let base = KERNEL_ELF_BASE.load(Ordering::Relaxed);
    let size = KERNEL_ELF_SIZE.load(Ordering::Relaxed) as usize;
    if base == 0 || size == 0 {
        return Err("installer: kernel ELF not available (KernelFileRequest failed)");
    }
    // Safety: Limine holds this memory live for the entire boot session.
    let kernel_bytes = unsafe { core::slice::from_raw_parts(base as *const u8, size) };
    kprintln!("[install] kernel ELF at virt={:#x} size={} KB", base, size / 1024);
    write_file_to_esp("boot/nexus-kernel", kernel_bytes)
}
