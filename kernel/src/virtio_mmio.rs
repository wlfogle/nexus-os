//! VirtIO-MMIO Block Device Driver (AArch64 / QEMU virt)
//!
//! QEMU `virt` machine exposes VirtIO-MMIO devices at:
//!   Base: 0x0A00_0000, stride 0x200, up to 32 devices.
//!
//! Transport: VirtIO MMIO legacy (version 1).
//! Register layout uses volatile 32-bit MMIO reads/writes.
//!
//! Virtqueue layout identical to the x86_64 I/O-port driver:
//!   desc table | avail ring | [pad to page] | used ring | [pad] | req buf

use spin::Mutex;
use core::sync::atomic::{fence, Ordering};
use crate::memory::{physical, paging};

// ─── VirtIO-MMIO register offsets (32-bit each) ───────────────────────────────

const MMIO_MAGIC:           usize = 0x000; // R   should be 0x7472_6976 ("virt")
const MMIO_VERSION:         usize = 0x004; // R   1 = legacy, 2 = modern
const MMIO_DEVICE_ID:       usize = 0x008; // R   2 = block
const MMIO_VENDOR_ID:       usize = 0x00C;
const MMIO_DEVICE_FEATURES: usize = 0x010; // R
const MMIO_DRIVER_FEATURES: usize = 0x020; // W
const MMIO_GUEST_PAGE_SIZE: usize = 0x028; // W   legacy: page size = 4096
const MMIO_QUEUE_SEL:       usize = 0x030; // W
const MMIO_QUEUE_NUM_MAX:   usize = 0x034; // R
const MMIO_QUEUE_NUM:       usize = 0x038; // W
const MMIO_QUEUE_ALIGN:     usize = 0x03C; // W   legacy: must be 4096
const MMIO_QUEUE_PFN:       usize = 0x040; // W   legacy: physical page number
const MMIO_QUEUE_NOTIFY:    usize = 0x050; // W
const MMIO_INTERRUPT_STATUS:usize = 0x060; // R
const MMIO_INTERRUPT_ACK:   usize = 0x064; // W
const MMIO_STATUS:          usize = 0x070; // RW
const MMIO_CONFIG:          usize = 0x100; // R   device-specific config

// Block device config: u64 capacity at offset 0x100
const MMIO_BLK_CAPACITY_LO: usize = MMIO_CONFIG;
const MMIO_BLK_CAPACITY_HI: usize = MMIO_CONFIG + 4;

// ─── QEMU virt AArch64 VirtIO-MMIO device probe range ────────────────────────

const MMIO_BASE:    usize = 0x0A00_0000;
const MMIO_STRIDE:  usize = 0x200;
const MMIO_COUNT:   usize = 32;
const VIRTIO_MAGIC: u32   = 0x7472_6976; // "virt"
const DEVICE_BLK:   u32   = 2;

// ─── VirtIO status bits ───────────────────────────────────────────────────────

const STATUS_ACKNOWLEDGE: u32 = 1;
const STATUS_DRIVER:      u32 = 2;
const STATUS_DRIVER_OK:   u32 = 4;
const STATUS_FAILED:      u32 = 128;

// ─── Virtqueue constants ─────────────────────────────────────────────────────

pub const SECTOR_SIZE: usize = 512;
const QUEUE_SIZE: usize = 64; // request one smaller queue for simplicity

const BLK_T_IN:  u32 = 0;
const BLK_T_OUT: u32 = 1;
const VRING_DESC_F_NEXT:  u16 = 1;
const VRING_DESC_F_WRITE: u16 = 2;
const VIRTIO_BLK_S_OK: u8 = 0;

// ─── MMIO helpers ─────────────────────────────────────────────────────────────

#[inline]
fn mmio_read(base: usize, reg: usize) -> u32 {
    unsafe { core::ptr::read_volatile((base + reg) as *const u32) }
}

#[inline]
fn mmio_write(base: usize, reg: usize, val: u32) {
    unsafe { core::ptr::write_volatile((base + reg) as *mut u32, val) }
}

// ─── Layout helpers ───────────────────────────────────────────────────────────

#[inline]
const fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

fn used_ring_offset(qsz: usize) -> usize {
    let desc  = 16 * qsz;
    let avail = 4 + 2 * qsz;
    align_up(desc + avail, 4096)
}

fn req_buf_offset(qsz: usize) -> usize {
    let used_off   = used_ring_offset(qsz);
    let used_bytes = 4 + 8 * qsz;
    align_up(used_off + used_bytes, 4096)
}

fn frames_needed(qsz: usize) -> usize {
    let total = req_buf_offset(qsz) + 16 + SECTOR_SIZE + 1;
    (total + 4095) / 4096
}

// ─── Virtqueue structs ────────────────────────────────────────────────────────

#[repr(C)]
struct VirtqDesc {
    addr:  u64,
    len:   u32,
    flags: u16,
    next:  u16,
}

#[repr(C)]
struct BlkReqHdr {
    req_type: u32,
    reserved: u32,
    sector:   u64,
}

// ─── Driver state ─────────────────────────────────────────────────────────────

struct VirtioMmioBlk {
    mmio_base:     usize,
    queue_size:    usize,
    desc_virt:     u64,
    avail_virt:    u64,
    used_virt:     u64,
    req_hdr_phys:  u64,
    req_dat_phys:  u64,
    req_sts_phys:  u64,
    req_hdr_virt:  u64,
    req_dat_virt:  u64,
    req_sts_virt:  u64,
    avail_idx:     u16,
    last_used_idx: u16,
    pub capacity:  u64,
}

static DISK: Mutex<Option<VirtioMmioBlk>> = Mutex::new(None);

// ─── Probe + init ─────────────────────────────────────────────────────────────

/// Scan VirtIO-MMIO slots for a block device and initialise it.
/// Returns disk capacity in sectors, or 0 if no device found.
pub fn init() -> u64 {
    for i in 0..MMIO_COUNT {
        let base = MMIO_BASE + i * MMIO_STRIDE;
        let magic = mmio_read(base, MMIO_MAGIC);
        if magic != VIRTIO_MAGIC {
            continue;
        }
        let device_id = mmio_read(base, MMIO_DEVICE_ID);
        if device_id != DEVICE_BLK {
            continue;
        }
        let version = mmio_read(base, MMIO_VERSION);
        crate::kprintln!("[disk] VirtIO-MMIO blk at {:#010x} version={}", base, version);
        match init_device(base) {
            Ok(cap) => {
                let gib = cap / (2 * 1024 * 1024);
                crate::kprintln!("[disk] VirtIO-MMIO: {} GiB ({} sectors)", gib, cap);
                return cap;
            }
            Err(e) => {
                crate::kprintln!("[disk] VirtIO-MMIO init failed: {}", e);
            }
        }
    }
    crate::kprintln!("[disk] no VirtIO-MMIO block device found");
    0
}

fn init_device(base: usize) -> Result<u64, &'static str> {
    // Reset
    mmio_write(base, MMIO_STATUS, 0);
    // Acknowledge + Driver
    mmio_write(base, MMIO_STATUS, STATUS_ACKNOWLEDGE);
    mmio_write(base, MMIO_STATUS, STATUS_ACKNOWLEDGE | STATUS_DRIVER);

    // Feature negotiation: accept device features minus RO/EVENT_IDX
    let dev_feat = mmio_read(base, MMIO_DEVICE_FEATURES);
    let drv_feat = dev_feat
        & !(1u32 << 5)  // VIRTIO_BLK_F_RO
        & !(1u32 << 29); // VIRTIO_RING_F_EVENT_IDX
    mmio_write(base, MMIO_DRIVER_FEATURES, drv_feat);

    // Legacy: set guest page size = 4096
    mmio_write(base, MMIO_GUEST_PAGE_SIZE, 4096);

    // Select queue 0 and set size
    mmio_write(base, MMIO_QUEUE_SEL, 0);
    let max_q = mmio_read(base, MMIO_QUEUE_NUM_MAX) as usize;
    if max_q == 0 {
        return Err("queue size 0");
    }
    let qsz = QUEUE_SIZE.min(max_q);
    mmio_write(base, MMIO_QUEUE_NUM, qsz as u32);
    mmio_write(base, MMIO_QUEUE_ALIGN, 4096);

    // Allocate queue memory
    let n = frames_needed(qsz);
    let f0 = physical::alloc_frame();
    for i in 1..n {
        let fi = physical::alloc_frame();
        if fi != f0 + i as u64 * 4096 {
            return Err("queue frames not contiguous");
        }
    }
    for i in 0..n {
        unsafe {
            core::ptr::write_bytes(
                paging::phys_to_virt(f0 + i as u64 * 4096) as *mut u8,
                0, 4096,
            );
        }
    }

    // Register queue PFN (legacy: physical page number)
    mmio_write(base, MMIO_QUEUE_PFN, (f0 / 4096) as u32);

    // Driver OK
    mmio_write(base, MMIO_STATUS,
               STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_DRIVER_OK);

    // Read capacity from device config
    let cap_lo = mmio_read(base, MMIO_BLK_CAPACITY_LO) as u64;
    let cap_hi = mmio_read(base, MMIO_BLK_CAPACITY_HI) as u64;
    let capacity = cap_lo | (cap_hi << 32);

    let avail_off = (16 * qsz) as u64;
    let used_off  = used_ring_offset(qsz) as u64;
    let req_off   = req_buf_offset(qsz) as u64;

    *DISK.lock() = Some(VirtioMmioBlk {
        mmio_base:    base,
        queue_size:   qsz,
        desc_virt:    paging::phys_to_virt(f0),
        avail_virt:   paging::phys_to_virt(f0 + avail_off),
        used_virt:    paging::phys_to_virt(f0 + used_off),
        req_hdr_phys: f0 + req_off,
        req_dat_phys: f0 + req_off + 16,
        req_sts_phys: f0 + req_off + 16 + SECTOR_SIZE as u64,
        req_hdr_virt: paging::phys_to_virt(f0 + req_off),
        req_dat_virt: paging::phys_to_virt(f0 + req_off + 16),
        req_sts_virt: paging::phys_to_virt(f0 + req_off + 16 + SECTOR_SIZE as u64),
        avail_idx:    0,
        last_used_idx:0,
        capacity,
    });

    Ok(capacity)
}

// ─── Public I/O ───────────────────────────────────────────────────────────────

pub fn capacity() -> u64 {
    DISK.lock().as_ref().map(|d| d.capacity).unwrap_or(0)
}

pub fn read_sectors(lba: u64, buf: &mut [u8]) -> Result<(), &'static str> {
    let count = buf.len() / SECTOR_SIZE;
    if count == 0 { return Ok(()); }
    let mut disk = DISK.lock();
    let d = disk.as_mut().ok_or("VirtIO-MMIO: not initialised")?;
    for i in 0..count {
        d.do_io(BLK_T_IN, lba + i as u64,
                &mut buf[i * SECTOR_SIZE .. (i+1) * SECTOR_SIZE])?;
    }
    Ok(())
}

pub fn write_sectors(lba: u64, buf: &[u8]) -> Result<(), &'static str> {
    let count = buf.len() / SECTOR_SIZE;
    if count == 0 { return Ok(()); }
    let mut disk = DISK.lock();
    let d = disk.as_mut().ok_or("VirtIO-MMIO: not initialised")?;
    let mut tmp = [0u8; SECTOR_SIZE];
    for i in 0..count {
        tmp.copy_from_slice(&buf[i * SECTOR_SIZE .. (i+1) * SECTOR_SIZE]);
        d.do_io(BLK_T_OUT, lba + i as u64, &mut tmp)?;
    }
    Ok(())
}

// ─── Internal: single-sector I/O ─────────────────────────────────────────────

impl VirtioMmioBlk {
    fn do_io(&mut self, req_type: u32, sector: u64, data: &mut [u8])
        -> Result<(), &'static str>
    {
        assert_eq!(data.len(), SECTOR_SIZE);
        let qsz = self.queue_size;

        unsafe {
            let hdr = self.req_hdr_virt as *mut BlkReqHdr;
            (*hdr).req_type = req_type;
            (*hdr).reserved = 0;
            (*hdr).sector   = sector;
            if req_type == BLK_T_OUT {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr(), self.req_dat_virt as *mut u8, SECTOR_SIZE);
            }
            *(self.req_sts_virt as *mut u8) = 0xFF;
        }

        let data_flags = if req_type == BLK_T_IN {
            VRING_DESC_F_WRITE | VRING_DESC_F_NEXT
        } else {
            VRING_DESC_F_NEXT
        };

        unsafe {
            let d = self.desc_virt as *mut VirtqDesc;
            (*d.add(0)).addr  = self.req_hdr_phys;
            (*d.add(0)).len   = 16;
            (*d.add(0)).flags = VRING_DESC_F_NEXT;
            (*d.add(0)).next  = 1;
            (*d.add(1)).addr  = self.req_dat_phys;
            (*d.add(1)).len   = SECTOR_SIZE as u32;
            (*d.add(1)).flags = data_flags;
            (*d.add(1)).next  = 2;
            (*d.add(2)).addr  = self.req_sts_phys;
            (*d.add(2)).len   = 1;
            (*d.add(2)).flags = VRING_DESC_F_WRITE;
            (*d.add(2)).next  = 0;
        }

        // Post to available ring
        let avail_slot  = (self.avail_idx as usize) % qsz;
        let ring_entry  = (self.avail_virt + 4 + avail_slot as u64 * 2) as *mut u16;
        let avail_idx_p = (self.avail_virt + 2) as *mut u16;
        unsafe {
            core::ptr::write_volatile(ring_entry, 0);
            fence(Ordering::SeqCst);
            core::ptr::write_volatile(avail_idx_p, self.avail_idx.wrapping_add(1));
        }
        self.avail_idx = self.avail_idx.wrapping_add(1);

        // Notify device (queue 0)
        fence(Ordering::SeqCst);
        mmio_write(self.mmio_base, MMIO_QUEUE_NOTIFY, 0);

        // Poll used ring
        let used_idx_ptr = (self.used_virt + 2) as *const u16;
        let target = self.last_used_idx.wrapping_add(1);
        let mut spins = 0u32;
        loop {
            fence(Ordering::SeqCst);
            if unsafe { core::ptr::read_volatile(used_idx_ptr) } == target { break; }
            core::hint::spin_loop();
            spins += 1;
            if spins > 10_000_000 {
                return Err("VirtIO-MMIO: I/O timeout");
            }
        }
        self.last_used_idx = target;

        let st = unsafe { core::ptr::read_volatile(self.req_sts_virt as *const u8) };
        if st != VIRTIO_BLK_S_OK {
            return Err("VirtIO-MMIO: I/O error");
        }

        if req_type == BLK_T_IN {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.req_dat_virt as *const u8,
                    data.as_mut_ptr(), SECTOR_SIZE);
            }
        }

        Ok(())
    }
}
