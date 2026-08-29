//! NexusOS Unified Block Device Dispatch
//!
//! `fs::fat`, the installer, and `SYS_DISK_READ`/`SYS_DISK_WRITE` all need one
//! disk backend to issue sector I/O against, but three independent drivers
//! can attach the boot disk depending on the machine:
//!   - VirtIO-blk  — QEMU/KVM testing (`make run-*`)
//!   - NVMe        — real laptops/desktops with an NVMe SSD
//!   - AHCI (SATA) — real machines with a SATA disk/controller
//!
//! Each driver populates its own private static the moment its `init()`
//! succeeds (see `main.rs`'s boot probe order: VirtIO-blk, then NVMe, then
//! AHCI). This module picks whichever one actually attached and forwards all
//! sector I/O to it, so every caller works unmodified on bare metal — no
//! caller needs to know or care which controller is really backing the disk.
//!
//! At most one backend will ever report present in practice (a real machine
//! has one boot disk); the priority order below only matters in the
//! theoretical case of more than one being available simultaneously.

use super::{ahci, nvme, virtio};

/// Sector size shared by all three backends (they only ever expose 512-byte
/// logical sectors regardless of the underlying disk's physical block size).
pub const SECTOR_SIZE: usize = 512;

/// Read `buf.len() / SECTOR_SIZE` sectors starting at `lba` from whichever
/// disk backend is attached.
pub fn read_sectors(lba: u64, buf: &mut [u8]) -> Result<(), &'static str> {
    if virtio::blk::is_present() {
        virtio::blk::read_sectors(lba, buf)
    } else if nvme::is_present() {
        nvme::read_sectors(lba, buf)
    } else if ahci::is_present() {
        ahci::read_sectors(lba, buf)
    } else {
        Err("blockdev: no disk backend initialised")
    }
}

/// Write `buf.len() / SECTOR_SIZE` sectors starting at `lba` to whichever
/// disk backend is attached.
pub fn write_sectors(lba: u64, buf: &[u8]) -> Result<(), &'static str> {
    if virtio::blk::is_present() {
        virtio::blk::write_sectors(lba, buf)
    } else if nvme::is_present() {
        nvme::write_sectors(lba, buf)
    } else if ahci::is_present() {
        ahci::write_sectors(lba, buf)
    } else {
        Err("blockdev: no disk backend initialised")
    }
}

/// Capacity, in 512-byte sectors, of whichever disk backend is attached.
/// Returns 0 if none initialised (matches each backend's own `capacity()`).
pub fn capacity() -> u64 {
    let c = virtio::blk::capacity();
    if c > 0 {
        return c;
    }
    let c = nvme::capacity();
    if c > 0 {
        return c;
    }
    ahci::capacity()
}
