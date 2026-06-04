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
use crate::{kprintln, fs};
use crate::drivers::virtio::blk::capacity;
use core::sync::atomic::Ordering;

// ─── Assets embedded at compile time ─────────────────────────────────────────

static BOOTX64_EFI: &[u8] = include_bytes!("../../../limine/bin/BOOTX64.EFI");

static LIMINE_CONF: &[u8] =
    b"# NexusOS Boot Configuration\ntimeout: 5\ndefault_entry: 1\n\n/NexusOS\n    protocol: limine\n    path: boot():/boot/nexus-kernel\n    cmdline: target=laptop\n";

/// EFI shell startup script: auto-launches Limine when OVMF has no saved
/// boot entry for this disk (first power-on after install). After Limine
/// runs successfully, OVMF writes a permanent NVRAM entry and subsequent
/// boots go directly to Limine without the shell.
static STARTUP_NSH: &[u8] = b"\\EFI\\BOOT\\BOOTX64.EFI\r\n";

// ─── Kernel ELF globals (set by _start from KernelFileRequest) ───────────────
// These hold the virtual address and byte-length of the original ELF file that
// Limine loaded from disk.  Writing these bytes to the installed disk produces
// a proper ELF that Limine can load on the next boot.

pub static KERNEL_ELF_BASE: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);
pub static KERNEL_ELF_SIZE: core::sync::atomic::AtomicU64 =
    core::sync::atomic::AtomicU64::new(0);

// ─── Installer task ───────────────────────────────────────────────────────────

pub extern "C" fn task_installer() -> ! {
    for _ in 0..100 {
        unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
    }

    if capacity() == 0 {
        kprintln!("[install] No disk — skipping.");
        loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack)); } }
    }

    if fs::fat::is_mounted() {
        kprintln!("[install] Already installed.");
        loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack)); } }
    }

    kprintln!();
    kprintln!("╔══════════════════════════════════════════╗");
    kprintln!("║        NexusOS Installer v0.5            ║");
    kprintln!("║  World's First AI-Native OS              ║");
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

    loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack)); } }
}

fn run_install() -> Result<(), &'static str> {
    kprintln!("[install] Disk: {} GiB", capacity() / (2 * 1024 * 1024));

    kprintln!("[install] Writing GPT...");
    let esp_lba = write_gpt()?;

    kprintln!("[install] Formatting ESP as FAT32 (LBA {})...", esp_lba);
    format_esp(esp_lba)?;

    kprintln!("[install] Creating directories...");
    fs::fat::mkdir("EFI")?;
    fs::fat::mkdir("EFI/BOOT")?;
    fs::fat::mkdir("boot")?;

    kprintln!("[install] Writing BOOTX64.EFI ({} KB)...", BOOTX64_EFI.len() / 1024);
    write_file_to_esp("EFI/BOOT/BOOTX64.EFI", BOOTX64_EFI)?;

    kprintln!("[install] Writing nexus-kernel...");
    write_kernel()?;

    kprintln!("[install] Writing limine.conf...");
    write_file_to_esp("EFI/BOOT/limine.conf", LIMINE_CONF)?;

    kprintln!("[install] Writing startup.nsh...");
    write_file_to_esp("startup.nsh", STARTUP_NSH)?;

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
