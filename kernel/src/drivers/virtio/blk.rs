//! NexusOS VirtIO Block Device Driver
//!
//! Implements synchronous sector read/write via the VirtIO legacy interface.
//!
//! Virtqueue layout for Q = 8 entries (allocated from physical frame allocator):
//!   Page 0 (4 KB):  descriptor table (128 B) + available ring (22 B)
//!   Page 1 (4 KB):  used ring (74 B)
//!
//! Each I/O reuses descriptors 0-2:
//!   0 → request header  (blk_req: type + reserved + sector, device-readable)
//!   1 → data buffer     (512 B, device-readable on write / device-writable on read)
//!   2 → status byte     (1 B,   device-writable)
//!
//! The driver polls the used ring index until the device signals completion.

use spin::Mutex;
use core::sync::atomic::{fence, Ordering};
use crate::memory::{physical, paging};
use super::{
    REG_QUEUE_ADDRESS, REG_QUEUE_SIZE, REG_QUEUE_SELECT, REG_QUEUE_NOTIFY,
    REG_DRIVER_FEATURES, REG_DEVICE_FEATURES,
    STATUS_ACKNOWLEDGE, STATUS_DRIVER, STATUS_DRIVER_OK,
    write16, write32, read16, read32, blk_capacity,
    reset_and_ack, set_status,
};

// ─── Constants ────────────────────────────────────────────────────────────────

pub const SECTOR_SIZE: usize = 512;
const QUEUE_SIZE: usize = 8;

/// VirtIO-blk request types
const BLK_T_IN:  u32 = 0; // read  (device → guest)
const BLK_T_OUT: u32 = 1; // write (guest  → device)

/// Virtqueue descriptor flags
const VRING_DESC_F_NEXT:  u16 = 1;
const VRING_DESC_F_WRITE: u16 = 2; // device may write to this buffer

/// VirtIO-blk status values (written by device into status byte)
const VIRTIO_BLK_S_OK: u8 = 0;

// ─── Virtqueue memory layout ──────────────────────────────────────────────────
//
// We use two contiguous physical frames:
//
// Frame 0:
//   offset 0x000 : VirtqDesc[8]   (16 × 8 = 128 bytes)
//   offset 0x080 : VirtqAvail     (2+2 + 2×8 = 20 bytes; ring[QUEUE_SIZE] of u16)
//
// Frame 1 (4096-aligned per spec):
//   offset 0x000 : VirtqUsed      (2+2 + 8×8 = 68 bytes; ring[QUEUE_SIZE] of (u32,u32))
//
// Request scratch buffer occupies the tail of Frame 0:
//   offset 0x100 : BlkReqHdr      (16 bytes: type u32, reserved u32, sector u64)
//   offset 0x110 : data[512]      (512 bytes for one sector)
//   offset 0x310 : status u8      (1 byte)

// Offsets within frame 0 (used only to compute physical addresses)
const OFF_DESC:    u64 = 0x000;
const OFF_AVAIL:   u64 = 0x080;
const OFF_REQ_HDR: u64 = 0x100;
const OFF_DATA:    u64 = 0x110;
const OFF_STATUS:  u64 = 0x310;

// ─── Virtqueue repr structs (repr(C), naturally aligned) ─────────────────────

#[repr(C)]
struct VirtqDesc {
    addr:  u64,
    len:   u32,
    flags: u16,
    next:  u16,
}

#[repr(C)]
struct VirtqAvail {
    flags: u16,
    idx:   u16,
    ring:  [u16; QUEUE_SIZE],
    // (avail_event omitted — not used in polling mode)
}

#[repr(C)]
struct VirtqUsedElem {
    id:  u32,
    len: u32,
}

#[repr(C)]
struct VirtqUsed {
    flags: u16,
    idx:   u16,
    ring:  [VirtqUsedElem; QUEUE_SIZE],
}

#[repr(C)]
struct BlkReqHdr {
    req_type: u32,
    reserved: u32,
    sector:   u64,
}

// ─── Driver state ─────────────────────────────────────────────────────────────

pub struct VirtioBlk {
    io_base:       u16,
    // Physical addresses for queue memory
    frame0_phys:   u64,
    frame1_phys:   u64,
    frame0_virt:   u64,
    // Rolling counters
    avail_idx:     u16,  // how many descriptors we've put in avail ring
    last_used_idx: u16,  // last used.idx we saw
    /// Disk capacity in 512-byte sectors.
    pub capacity:  u64,
}

// ─── Global singleton ─────────────────────────────────────────────────────────

static DISK: Mutex<Option<VirtioBlk>> = Mutex::new(None);

// ─── Initialisation ───────────────────────────────────────────────────────────

/// Initialise the VirtIO-blk driver given the PCI I/O base (from BAR0).
/// Returns `Ok(capacity_in_sectors)` or `Err(&str)`.
pub fn init(io_base: u16) -> Result<u64, &'static str> {
    // 1. Reset + acknowledge
    reset_and_ack(io_base);

    // 2. Negotiate features (accept everything the device offers, minus RO)
    let dev_features = read32(io_base, REG_DEVICE_FEATURES);
    let drv_features = dev_features & !super::VIRTIO_BLK_F_RO;
    write32(io_base, REG_DRIVER_FEATURES, drv_features);

    // 3. Allocate two contiguous physical frames for the virtqueue.
    //    At kernel init time the frame allocator returns frames in order,
    //    so consecutive alloc_frame() calls are always physically contiguous.
    let f0 = physical::alloc_frame();
    let f1 = physical::alloc_frame();
    if f1 != f0 + 4096 {
        return Err("VirtIO-blk: virtqueue frames are not contiguous");
    }
    init_with_frames(io_base, f0, f1)
}

fn init_with_frames(io_base: u16, f0: u64, f1: u64)
    -> Result<u64, &'static str>
{
    let v0 = paging::phys_to_virt(f0);

    // Zero both frames (use phys_to_virt for frame 1 directly)
    unsafe {
        core::ptr::write_bytes(v0 as *mut u8, 0, 4096);
        core::ptr::write_bytes(paging::phys_to_virt(f1) as *mut u8, 0, 4096);
    }

    // 4. Select queue 0 and verify size
    write16(io_base, REG_QUEUE_SELECT, 0);
    let qsz = read16(io_base, REG_QUEUE_SIZE) as usize;
    if qsz == 0 {
        return Err("VirtIO-blk: queue size is 0 (device not present?)");
    }

    // 5. Tell device the queue's physical page number (frame 0 / 4096)
    write32(io_base, REG_QUEUE_ADDRESS, (f0 / 4096) as u32);

    // 6. Set DRIVER_OK
    set_status(io_base,
        STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_DRIVER_OK);

    // 7. Read capacity
    let capacity = blk_capacity(io_base);

    let mut disk = DISK.lock();
    *disk = Some(VirtioBlk {
        io_base,
        frame0_phys: f0,
        frame1_phys: f1,
        frame0_virt: v0,
        avail_idx:     0,
        last_used_idx: 0,
        capacity,
    });

    Ok(capacity)
}

// ─── I/O operations ───────────────────────────────────────────────────────────

/// Read `count` 512-byte sectors starting at `lba` into `buf`.
/// `buf` must be at least `count * 512` bytes.
pub fn read_sectors(lba: u64, buf: &mut [u8]) -> Result<(), &'static str> {
    let count = buf.len() / SECTOR_SIZE;
    if count == 0 { return Ok(()); }
    let mut disk = DISK.lock();
    let d = disk.as_mut().ok_or("VirtIO-blk: not initialised")?;
    for i in 0..count {
        d.do_io(BLK_T_IN, lba + i as u64,
                &mut buf[i * SECTOR_SIZE .. (i+1) * SECTOR_SIZE])?;
    }
    Ok(())
}

/// Write `count` 512-byte sectors starting at `lba` from `buf`.
pub fn write_sectors(lba: u64, buf: &[u8]) -> Result<(), &'static str> {
    let count = buf.len() / SECTOR_SIZE;
    if count == 0 { return Ok(()); }
    let mut disk = DISK.lock();
    let d = disk.as_mut().ok_or("VirtIO-blk: not initialised")?;
    let mut tmp = [0u8; SECTOR_SIZE];
    for i in 0..count {
        tmp.copy_from_slice(&buf[i * SECTOR_SIZE .. (i+1) * SECTOR_SIZE]);
        d.do_io(BLK_T_OUT, lba + i as u64, &mut tmp)?;
    }
    Ok(())
}

/// Return the disk capacity in sectors (or 0 if not initialised).
pub fn capacity() -> u64 {
    DISK.lock().as_ref().map(|d| d.capacity).unwrap_or(0)
}

// ─── Internal: single-sector I/O ─────────────────────────────────────────────

impl VirtioBlk {
    /// Perform a single-sector read (BLK_T_IN) or write (BLK_T_OUT).
    /// `data` must be exactly 512 bytes.
    fn do_io(&mut self, req_type: u32, sector: u64, data: &mut [u8])
        -> Result<(), &'static str>
    {
        assert_eq!(data.len(), SECTOR_SIZE);

        let v0     = self.frame0_virt;
        let f0     = self.frame0_phys;
        let f1     = self.frame1_phys;

        // — Build request header in frame 0 scratch area —
        let hdr_virt = (v0 + OFF_REQ_HDR) as *mut BlkReqHdr;
        let dat_virt = (v0 + OFF_DATA)    as *mut u8;
        let sts_virt = (v0 + OFF_STATUS)  as *mut u8;

        unsafe {
            (*hdr_virt).req_type = req_type;
            (*hdr_virt).reserved = 0;
            (*hdr_virt).sector   = sector;

            // Copy write data into scratch; for read, device overwrites this
            core::ptr::copy_nonoverlapping(data.as_ptr(), dat_virt, SECTOR_SIZE);

            // Status byte: set to 0xFF so we can detect device writes
            *sts_virt = 0xFF;
        }

        // — Fill descriptor table (descriptors 0, 1, 2) —
        let desc_base = v0 + OFF_DESC;

        // Descriptor 0: request header (device-readable)
        let hdr_phys = f0 + OFF_REQ_HDR;
        let dat_phys = f0 + OFF_DATA;
        let sts_phys = f0 + OFF_STATUS;

        let data_flags: u16 = if req_type == BLK_T_IN {
            VRING_DESC_F_WRITE | VRING_DESC_F_NEXT  // device writes to data buf
        } else {
            VRING_DESC_F_NEXT  // device reads from data buf
        };

        unsafe {
            let desc = desc_base as *mut VirtqDesc;
            // Desc 0: header
            (*desc.add(0)).addr  = hdr_phys;
            (*desc.add(0)).len   = 16;
            (*desc.add(0)).flags = VRING_DESC_F_NEXT;
            (*desc.add(0)).next  = 1;
            // Desc 1: data
            (*desc.add(1)).addr  = dat_phys;
            (*desc.add(1)).len   = SECTOR_SIZE as u32;
            (*desc.add(1)).flags = data_flags;
            (*desc.add(1)).next  = 2;
            // Desc 2: status (device always writes)
            (*desc.add(2)).addr  = sts_phys;
            (*desc.add(2)).len   = 1;
            (*desc.add(2)).flags = VRING_DESC_F_WRITE;
            (*desc.add(2)).next  = 0;
        }

        // — Add to available ring —
        let avail = (v0 + OFF_AVAIL) as *mut VirtqAvail;
        let avail_slot = (self.avail_idx as usize) % QUEUE_SIZE;
        unsafe {
            (*avail).ring[avail_slot] = 0; // descriptor chain starts at index 0
            fence(Ordering::SeqCst);
            (*avail).idx = self.avail_idx.wrapping_add(1);
        }
        self.avail_idx = self.avail_idx.wrapping_add(1);

        // — Notify device: queue 0 —
        fence(Ordering::SeqCst);
        write16(self.io_base, REG_QUEUE_NOTIFY, 0);

        // — Poll used ring until device marks this request done —
        let used = (paging::phys_to_virt(f1)) as *const VirtqUsed;
        let target = self.last_used_idx.wrapping_add(1);
        loop {
            fence(Ordering::SeqCst);
            let used_idx = unsafe { core::ptr::read_volatile(&(*used).idx) };
            if used_idx == target { break; }
            // Busy-wait (Phase 5.0; Phase 5.1 will use IRQ9)
            core::hint::spin_loop();
        }
        self.last_used_idx = target;

        // — Check status byte —
        let status = unsafe { core::ptr::read_volatile(sts_virt) };
        if status != VIRTIO_BLK_S_OK {
            return Err("VirtIO-blk: I/O error (device returned non-zero status)");
        }

        // — Copy data out (for reads) —
        if req_type == BLK_T_IN {
            unsafe {
                core::ptr::copy_nonoverlapping(dat_virt, data.as_mut_ptr(), SECTOR_SIZE);
            }
        }

        Ok(())
    }
}
