//! NexusOS Installer — GPT Partition Table Writer
//!
//! Writes a minimal but spec-correct GUID Partition Table to the VirtIO disk:
//!
//!   LBA 0        Protective MBR
//!   LBA 1        Primary GPT header
//!   LBA 2..33    Partition entries (128 × 128 B = 32 sectors)
//!   LBA 34..end  Single EFI System Partition (ESP)
//!   LBA last-32  Backup partition entries
//!   LBA last     Backup GPT header
//!
//! The ESP is then formatted as FAT32 by the caller.

use super::crc32::crc32;
use crate::drivers::virtio::blk::{write_sectors, capacity, SECTOR_SIZE};

// ─── GUIDs (little-endian mixed-endian format as stored in GPT) ──────────────

/// EFI System Partition type GUID:  C12A7328-F81F-11D2-BA4B-00A0C93EC93B
const ESP_TYPE_GUID: [u8; 16] = [
    0x28, 0x73, 0x2A, 0xC1,  // time_low  (LE)
    0x1F, 0xF8,              // time_mid  (LE)
    0xD2, 0x11,              // time_hi   (BE)
    0xBA, 0x4B,              // clock_seq (BE)
    0x00, 0xA0, 0xC9, 0x3E, 0xC9, 0x3B,  // node
];

/// Unique disk GUID (fixed, deterministic for NexusOS).
const DISK_GUID: [u8; 16] = [
    0x4E, 0x58, 0x55, 0x53,  // "NEXU"
    0x4F, 0x53,              // "OS"
    0x00, 0x01,
    0xAB, 0xCD,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
];

/// Unique partition GUID for the ESP.
const PART_GUID: [u8; 16] = [
    0x4E, 0x58, 0x55, 0x53,
    0x45, 0x53, 0x50, 0x00,
    0xAB, 0xCD,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
];

// ─── Layout constants ─────────────────────────────────────────────────────────

pub const ESP_START_LBA: u64 = 34;          // first usable LBA
const PART_ENTRY_SECTORS: u64 = 32;        // 128 × 128 B = 4 sectors needed; we reserve 32
const GPT_SIGNATURE: u64 = 0x5452_4150_2049_4645; // "EFI PART"
const GPT_REVISION:  u32 = 0x0001_0000;
const GPT_HEADER_SIZE: u32 = 92;
const PART_ENTRY_SIZE: u32 = 128;
const PART_ENTRY_COUNT: u32 = 128;

// ─── Sector buffers ───────────────────────────────────────────────────────────

fn write_le16(buf: &mut [u8], off: usize, v: u16) {
    buf[off..off+2].copy_from_slice(&v.to_le_bytes());
}
fn write_le32(buf: &mut [u8], off: usize, v: u32) {
    buf[off..off+4].copy_from_slice(&v.to_le_bytes());
}
fn write_le64(buf: &mut [u8], off: usize, v: u64) {
    buf[off..off+8].copy_from_slice(&v.to_le_bytes());
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Write GPT (protective MBR + headers + partition entries) to the disk.
/// Returns `Ok(esp_start_lba)` on success.
pub fn write_gpt() -> Result<u64, &'static str> {
    let disk_sectors = capacity();
    if disk_sectors < 100 {
        return Err("GPT: disk too small");
    }

    let last_lba         = disk_sectors - 1;
    let backup_parts_lba = last_lba - PART_ENTRY_SECTORS;  // backup partition entries
    let esp_end_lba      = backup_parts_lba - 1;            // last usable LBA

    // ── 1. Protective MBR ────────────────────────────────────────────────────
    write_mbr(disk_sectors)?;

    // ── 2. Primary partition entries (LBAs 2..33) ────────────────────────────
    let parts_crc = write_partition_entries(2, ESP_START_LBA, esp_end_lba)?;

    // ── 3. Primary GPT header (LBA 1) ────────────────────────────────────────
    write_gpt_header(
        1,            // my LBA
        last_lba,     // alternate LBA
        ESP_START_LBA, esp_end_lba,
        2,            // partition entries LBA
        parts_crc,
    )?;

    // ── 4. Backup partition entries (LBAs backup_parts_lba..last_lba-1) ──────
    let parts_crc2 = write_partition_entries(backup_parts_lba, ESP_START_LBA, esp_end_lba)?;

    // ── 5. Backup GPT header (last LBA) ──────────────────────────────────────
    write_gpt_header(
        last_lba,
        1,
        ESP_START_LBA, esp_end_lba,
        backup_parts_lba,
        parts_crc2,
    )?;

    Ok(ESP_START_LBA)
}

// ─── Protective MBR ──────────────────────────────────────────────────────────

fn write_mbr(disk_sectors: u64) -> Result<(), &'static str> {
    let mut buf = [0u8; SECTOR_SIZE];

    // Single protective partition covering the whole disk
    // Partition entry at offset 446 (first entry), type 0xEE = GPT protective
    let entry = &mut buf[446..462];
    entry[0] = 0x00;         // not bootable
    entry[1] = 0x00;         // CHS start (ignored)
    entry[2] = 0x02;
    entry[3] = 0x00;
    entry[4] = 0xEE;         // partition type: GPT protective
    entry[5] = 0xFF;         // CHS end (ignored)
    entry[6] = 0xFF;
    entry[7] = 0xFF;
    // LBA start (LE32) = 1
    entry[8..12].copy_from_slice(&1u32.to_le_bytes());
    // LBA size (LE32) — clamped to u32::MAX
    let size = (disk_sectors - 1).min(0xFFFF_FFFF) as u32;
    entry[12..16].copy_from_slice(&size.to_le_bytes());

    // MBR boot signature
    buf[510] = 0x55;
    buf[511] = 0xAA;

    write_sectors(0, &buf).map_err(|_| "GPT: MBR write failed")
}

// ─── Partition entries ────────────────────────────────────────────────────────

/// Write 32 sectors of partition entries starting at `start_lba`.
/// Only entry 0 is populated (the ESP); the rest are zeroed.
/// Returns the CRC32 of the 128×128-byte partition entry array.
fn write_partition_entries(
    start_lba: u64,
    esp_first: u64,
    esp_last:  u64,
) -> Result<u32, &'static str> {
    // Build the full 128 × 128 = 16 384-byte entry array in a heap vec.
    // Each sector write is 512 bytes; we flush 32 sectors.
    const ENTRY_ARRAY_SIZE: usize = 128 * 128; // 16 384 bytes
    let mut entries = alloc::vec![0u8; ENTRY_ARRAY_SIZE];

    // Fill entry 0: EFI System Partition
    let e = &mut entries[0..128];
    e[0..16].copy_from_slice(&ESP_TYPE_GUID);   // type GUID
    e[16..32].copy_from_slice(&PART_GUID);       // unique GUID
    write_le64(e, 32, esp_first);                // start LBA
    write_le64(e, 40, esp_last);                 // end LBA (inclusive)
    // attributes = 0 (none)
    // name = "EFI System\0" in UTF-16LE
    let name = b"EFI System";
    for (i, &c) in name.iter().enumerate() {
        e[56 + i * 2] = c;
        e[56 + i * 2 + 1] = 0;
    }

    let parts_crc = crc32(&entries);

    // Write 32 sectors of partition entries
    let sectors_per_write = ENTRY_ARRAY_SIZE / SECTOR_SIZE; // 32
    for i in 0..sectors_per_write {
        let lba = start_lba + i as u64;
        let off = i * SECTOR_SIZE;
        let mut sector = [0u8; SECTOR_SIZE];
        sector.copy_from_slice(&entries[off..off + SECTOR_SIZE]);
        write_sectors(lba, &sector).map_err(|_| "GPT: partition entry write failed")?;
    }

    Ok(parts_crc)
}

// ─── GPT header ───────────────────────────────────────────────────────────────

fn write_gpt_header(
    my_lba:        u64,
    alternate_lba: u64,
    first_usable:  u64,
    last_usable:   u64,
    parts_lba:     u64,
    parts_crc:     u32,
) -> Result<(), &'static str> {
    let mut buf = [0u8; SECTOR_SIZE];

    write_le64(&mut buf,  0, GPT_SIGNATURE);
    write_le32(&mut buf,  8, GPT_REVISION);
    write_le32(&mut buf, 12, GPT_HEADER_SIZE);
    // CRC at offset 16 — filled after computing
    write_le32(&mut buf, 20, 0);                         // reserved
    write_le64(&mut buf, 24, my_lba);
    write_le64(&mut buf, 32, alternate_lba);
    write_le64(&mut buf, 40, first_usable);
    write_le64(&mut buf, 48, last_usable);
    buf[56..72].copy_from_slice(&DISK_GUID);
    write_le64(&mut buf, 72, parts_lba);
    write_le32(&mut buf, 80, PART_ENTRY_COUNT);
    write_le32(&mut buf, 84, PART_ENTRY_SIZE);
    write_le32(&mut buf, 88, parts_crc);

    // Compute header CRC over the first 92 bytes (with CRC field zeroed)
    let header_crc = crc32(&buf[..GPT_HEADER_SIZE as usize]);
    write_le32(&mut buf, 16, header_crc);

    write_sectors(my_lba, &buf).map_err(|_| "GPT: header write failed")
}
