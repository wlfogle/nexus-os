//! NexusOS FAT32 Filesystem Driver
//!
//! Uses the `fatfs` crate (0.4 git) over the VirtIO-blk storage backend.
//!
//! Storage layout: raw FAT32 at byte offset 0 (no partition table).
//! The NexusOS installer (Phase 5.3) formats the disk before writing files.
//!
//! # Sector-buffer design
//!
//! `fatfs` expects byte-level Read/Write/Seek; we translate those to 512-byte
//! VirtIO-blk sector operations via a one-sector write-back cache in `DiskIo`.
//! The cache is flushed on seek (if dirty) and via `Drop`.
//!
//! # Global state
//!
//! `FileSystem<DiskIo, NullTimeProvider, LossyOemCpConverter>` is `Send`
//! (because `DiskIo` is `Send`, and the 0.4 API stores owned TP/OCC instead of
//! `&'static dyn` references).  Wrapping it in `spin::Mutex` provides the `Sync`
//! needed for a `static`.

use spin::Mutex;
use fatfs::{
    FileSystem, FsOptions, FormatVolumeOptions, FatType,
    IoBase, Read, Write, Seek, SeekFrom,
    NullTimeProvider, LossyOemCpConverter,
};
use crate::drivers::virtio::blk::{read_sectors, write_sectors, capacity, SECTOR_SIZE};

// ─── I/O error type ──────────────────────────────────────────────────────────

/// Disk-level I/O error.  Implements `fatfs::IoError` as required by `IoBase`.
#[derive(Debug, Clone, Copy)]
pub struct DiskError;

impl fatfs::IoError for DiskError {
    fn is_interrupted(&self) -> bool { false }
    fn new_unexpected_eof_error() -> Self { DiskError }
    fn new_write_zero_error()     -> Self { DiskError }
}

// ─── Sector-buffered I/O adapter ─────────────────────────────────────────────

/// Wraps the global VirtIO-blk driver and presents byte-level `IoBase + Read +
/// Write + Seek` as required by `fatfs`.  Internally caches one 512-byte sector.
pub struct DiskIo {
    pos:        u64,
    buf:        [u8; SECTOR_SIZE],
    buf_lba:    u64,         // LBA currently cached (u64::MAX = empty)
    buf_dirty:  bool,
    lba_offset: u64,         // add to every LBA before hitting the driver
}

impl DiskIo {
    /// Whole-disk access (offset 0).
    pub fn new() -> Self {
        DiskIo { pos: 0, buf: [0u8; SECTOR_SIZE], buf_lba: u64::MAX,
                 buf_dirty: false, lba_offset: 0 }
    }

    /// Partition-relative access: all I/O is offset by `lba_offset` sectors.
    pub fn at_partition(lba_offset: u64) -> Self {
        DiskIo { pos: 0, buf: [0u8; SECTOR_SIZE], buf_lba: u64::MAX,
                 buf_dirty: false, lba_offset }
    }

    /// Write the cached sector back to disk if it has been modified.
    fn flush_cache(&mut self) -> Result<(), DiskError> {
        if self.buf_dirty && self.buf_lba != u64::MAX {
            write_sectors(self.buf_lba + self.lba_offset, &self.buf).map_err(|_| DiskError)?;
            self.buf_dirty = false;
        }
        Ok(())
    }

    /// Ensure `lba` is in the sector cache, flushing a dirty sector first if needed.
    fn load(&mut self, lba: u64) -> Result<(), DiskError> {
        if self.buf_lba == lba { return Ok(()); }
        self.flush_cache()?;
        read_sectors(lba + self.lba_offset, &mut self.buf).map_err(|_| DiskError)?;
        self.buf_lba   = lba;
        self.buf_dirty = false;
        Ok(())
    }
}

/// Flush any dirty cache on drop so data is not silently lost.
impl Drop for DiskIo {
    fn drop(&mut self) {
        let _ = self.flush_cache();
    }
}

// ─── fatfs trait implementations ─────────────────────────────────────────────

impl IoBase for DiskIo {
    type Error = DiskError;
}

impl Read for DiskIo {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DiskError> {
        let mut done = 0;
        while done < buf.len() {
            let lba   = self.pos / SECTOR_SIZE as u64;
            let off   = (self.pos % SECTOR_SIZE as u64) as usize;
            let chunk = (SECTOR_SIZE - off).min(buf.len() - done);
            self.load(lba)?;
            buf[done..done + chunk].copy_from_slice(&self.buf[off..off + chunk]);
            self.pos += chunk as u64;
            done     += chunk;
        }
        Ok(done)
    }
}

impl Write for DiskIo {
    fn write(&mut self, buf: &[u8]) -> Result<usize, DiskError> {
        let mut done = 0;
        while done < buf.len() {
            let lba   = self.pos / SECTOR_SIZE as u64;
            let off   = (self.pos % SECTOR_SIZE as u64) as usize;
            let chunk = (SECTOR_SIZE - off).min(buf.len() - done);

            if off != 0 || chunk < SECTOR_SIZE {
                // Partial sector write — read existing contents first
                self.load(lba)?;
            } else {
                // Full sector overwrite — no read required
                self.flush_cache()?;
                self.buf_lba = lba;
            }

            self.buf[off..off + chunk].copy_from_slice(&buf[done..done + chunk]);
            self.buf_dirty = true;
            self.pos      += chunk as u64;
            done          += chunk;
        }
        Ok(done)
    }

    fn flush(&mut self) -> Result<(), DiskError> {
        self.flush_cache()
    }
}

impl Seek for DiskIo {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, DiskError> {
        let disk_len = capacity() * SECTOR_SIZE as u64;
        let new_pos: i64 = match pos {
            SeekFrom::Start(n)   =>                   n as i64,
            SeekFrom::Current(n) => self.pos as i64 + n,
            SeekFrom::End(n)     => disk_len as i64  + n,
        };
        if new_pos < 0 { return Err(DiskError); }
        self.pos = new_pos as u64;
        Ok(self.pos)
    }
}

// ─── Global filesystem state ──────────────────────────────────────────────────

type FatDiskFs = FileSystem<DiskIo, NullTimeProvider, LossyOemCpConverter>;

// `FileSystem<DiskIo, NullTimeProvider, LossyOemCpConverter>` is `Send`:
//   • DiskIo holds only primitives → Send
//   • NullTimeProvider / LossyOemCpConverter are ZSTs → Send
//   • fatfs 0.4 stores owned TP/OCC (not &'static dyn), so auto-Send is sound
// Wrapping in Mutex<Option<_>> gives us Sync for the static.
pub(crate) static FS: Mutex<Option<FatDiskFs>> = Mutex::new(None);

// ─── GPT detection ───────────────────────────────────────────────────────────

/// Check for a GUID Partition Table at LBA 1.  If found, return the LBA where
/// the NexusOS EFI System Partition starts.  The NexusOS installer always
/// places the ESP at LBA 34 (GPT `first_usable_lba`).  A full partition-entry
/// scan for portability is deferred to Phase 5.5.
fn detect_gpt_esp() -> Option<u64> {
    const GPT_MAGIC: u64 = 0x5452_4150_2049_4645; // b"EFI PART" LE
    let mut buf = [0u8; SECTOR_SIZE];
    read_sectors(1, &mut buf).ok()?;
    let magic = u64::from_le_bytes(buf[0..8].try_into().ok()?);
    if magic != GPT_MAGIC {
        return None;
    }
    Some(34) // NexusOS installer ESP_START_LBA
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Probe the VirtIO-blk disk for a FAT filesystem and mount it.
///
/// Two attempts are made:
///   1. Raw FAT32 at LBA 0 — used when the disk is unpartitioned (first install
///      from ISO writes the partition table and re-mounts via `format()`).
///   2. GPT detected at LBA 1 → mount FAT32 at the ESP start LBA (34) — used
///      when booting from the fully-installed NexusOS disk.
///
/// Called at boot (after disk driver init and framebuffer).
/// Returns a string suitable for the `[fs]` boot log line.
pub fn init() -> &'static str {
    if capacity() == 0 {
        return "no disk — skipping FAT32";
    }

    // Attempt 1: raw FAT32 at LBA 0 (unpartitioned disk or first-boot ISO)
    let opts = FsOptions::new().time_provider(NullTimeProvider::new());
    match FileSystem::new(DiskIo::new(), opts) {
        Ok(fs) => {
            *FS.lock() = Some(fs);
            return "FAT32 mounted";
        }
        Err(_) => {}
    }

    // Attempt 2: GPT-partitioned disk — locate and mount the ESP
    if let Some(esp_lba) = detect_gpt_esp() {
        let opts = FsOptions::new().time_provider(NullTimeProvider::new());
        match FileSystem::new(DiskIo::at_partition(esp_lba), opts) {
            Ok(fs) => {
                *FS.lock() = Some(fs);
                return "FAT32 mounted (GPT ESP at LBA 34)";
            }
            Err(_) => {}
        }
    }

    "disk present, not formatted — run installer to format"
}

/// Format the VirtIO-blk disk as FAT32 and mount the new filesystem.
///
/// Called by the NexusOS installer (Phase 5.3).
/// WARNING: destroys all existing data on the disk.
pub fn format() -> Result<(), &'static str> {
    if capacity() == 0 {
        return Err("[fs] no disk to format");
    }

    // Drop any existing mount
    *FS.lock() = None;

    // format_volume borrows &mut storage; Drop impl flushes the cache when
    // disk goes out of scope at the end of this block.
    {
        let mut disk = DiskIo::new();
        fatfs::format_volume(
            &mut disk,
            FormatVolumeOptions::new()
                .fat_type(FatType::Fat32)
                .volume_label(*b"NEXUSOS    "),
        ).map_err(|_| "FAT32: format failed")?;
    } // disk dropped here — flush_cache() called via Drop

    // Mount the freshly formatted volume using a fresh DiskIo (clean cache).
    let opts = FsOptions::new().time_provider(NullTimeProvider::new());
    let fs = FileSystem::new(DiskIo::new(), opts)
        .map_err(|_| "FAT32: mount after format failed")?;
    *FS.lock() = Some(fs);
    Ok(())
}

/// Create a directory at `path` (relative to root; no leading `/`).
/// Silently succeeds if the directory already exists.
pub fn mkdir(path: &str) -> Result<(), &'static str> {
    let guard = FS.lock();
    let fs = guard.as_ref().ok_or("FAT32: not mounted")?;
    // Explicit block ensures root + created Dir are dropped before guard.
    let result = {
        let root = fs.root_dir();
        match root.create_dir(path) {
            Ok(_)                            => Ok(()),
            Err(fatfs::Error::AlreadyExists) => Ok(()),
            Err(_)                           => Err("FAT32: mkdir failed"),
        }
    };
    result
}

/// Read an entire file at `path` into `buf`.
/// Returns the number of bytes read.
pub fn read_file(path: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let guard = FS.lock();
    let fs    = guard.as_ref().ok_or("FAT32: not mounted")?;
    // Explicit block: root + file are dropped before guard is released.
    let result = {
        let root = fs.root_dir();
        let mut file = root.open_file(path).map_err(|_| "FAT32: file not found")?;
        let mut total = 0usize;
        loop {
            if total >= buf.len() { break; }
            match file.read(&mut buf[total..]) {
                Ok(0) => break,
                Ok(n) => total += n,
                Err(_) => { return Err("FAT32: read error"); }
            }
        }
        Ok(total)
    };
    result
}

/// Create or overwrite the file at `path` with `data`.
pub fn write_file(path: &str, data: &[u8]) -> Result<(), &'static str> {
    let guard = FS.lock();
    let fs    = guard.as_ref().ok_or("FAT32: not mounted")?;
    // Explicit block: root + file are dropped before guard is released.
    let result = {
        let root = fs.root_dir();
        let mut file = root.create_file(path).map_err(|_| "FAT32: create failed")?;
        file.truncate().map_err(|_| "FAT32: truncate failed")?;
        let mut written = 0usize;
        while written < data.len() {
            match file.write(&data[written..]) {
                Ok(0) => { return Err("FAT32: disk full"); }
                Ok(n) => written += n,
                Err(_) => { return Err("FAT32: write error"); }
            }
        }
        file.flush().map_err(|_| "FAT32: flush failed")?;
        Ok(())
    };
    result
}

/// List the root directory, writing newline-separated entry names into `buf`.
/// Returns the number of bytes written.  Stops early if `buf` fills up.
pub fn list_root(buf: &mut [u8]) -> Result<usize, &'static str> {
    let guard = FS.lock();
    let fs    = guard.as_ref().ok_or("FAT32: not mounted")?;
    let mut total = 0usize;
    // Explicit block: root + iterator are dropped before the guard is released.
    {
        let root = fs.root_dir();
        for entry in root.iter() {
            let entry = entry.map_err(|_| "FAT32: dir read error")?;
            let name  = entry.file_name();      // alloc::string::String (lfn feature)
            let nb    = name.as_bytes();
            // Need room for the name plus a trailing newline.
            if total + nb.len() + 1 > buf.len() { break; }
            buf[total..total + nb.len()].copy_from_slice(nb);
            total += nb.len();
            buf[total] = b'\n';
            total += 1;
        }
    }
    Ok(total)
}

/// Returns `true` if a FAT filesystem is currently mounted.
pub fn is_mounted() -> bool {
    FS.lock().is_some()
}
