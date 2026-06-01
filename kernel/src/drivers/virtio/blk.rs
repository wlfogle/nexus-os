//! NexusOS VirtIO Block Device Driver
//!
//! Uses the device-reported queue size to compute the correct virtqueue layout.
//! Root cause of previous hang: QEMU ignores REG_QUEUE_SIZE writes and uses
//! qsz=256 by default.  For qsz=256 the used ring sits at f0+8192, not f0+4096.
//!
//! Memory layout (for device-reported queue size Q):
//!   [f0 + 0          ] Descriptor table  (16 * Q bytes)
//!   [f0 + 16*Q       ] Available ring    (4 + 2*Q bytes)
//!   [f0 + used_off   ] Used ring         (4 + 8*Q bytes)  [page-aligned]
//!   [f0 + req_off    ] Request buffers   (header 16 B + data 512 B + status 1 B)
//!
//! We always use descriptors 0-2 with one outstanding request at a time.

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

const BLK_T_IN:  u32 = 0;
const BLK_T_OUT: u32 = 1;

const VRING_DESC_F_NEXT:  u16 = 1;
const VRING_DESC_F_WRITE: u16 = 2;

const VIRTIO_BLK_S_OK: u8 = 0;

// ─── Layout helpers ───────────────────────────────────────────────────────────

#[inline]
const fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

fn used_ring_offset(qsz: usize) -> usize {
    // desc + avail (no avail_event; we clear EVENT_IDX in feature negotiation)
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

pub struct VirtioBlk {
    io_base:        u16,
    queue_size:     u16,
    desc_virt:      u64,
    avail_virt:     u64,
    used_virt:      u64,
    req_hdr_phys:   u64,
    req_dat_phys:   u64,
    req_sts_phys:   u64,
    req_hdr_virt:   u64,
    req_dat_virt:   u64,
    req_sts_virt:   u64,
    avail_idx:      u16,
    last_used_idx:  u16,
    pub capacity:   u64,
}

static DISK: Mutex<Option<VirtioBlk>> = Mutex::new(None);

// ─── Initialisation ───────────────────────────────────────────────────────────

pub fn init(io_base: u16) -> Result<u64, &'static str> {
    reset_and_ack(io_base);

    // Feature negotiation: accept all except RO and EVENT_IDX.
    // Clearing EVENT_IDX ensures the avail ring has no avail_event field,
    // keeping our layout formula correct.
    let dev_features = read32(io_base, REG_DEVICE_FEATURES);
    let drv_features = dev_features
        & !super::VIRTIO_BLK_F_RO
        & !(1u32 << 29); // VIRTIO_RING_F_EVENT_IDX
    write32(io_base, REG_DRIVER_FEATURES, drv_features);

    // Read device-reported queue size (typically 256 in QEMU)
    write16(io_base, REG_QUEUE_SELECT, 0);
    let qsz = read16(io_base, REG_QUEUE_SIZE) as usize;
    if qsz == 0 {
        return Err("VirtIO-blk: queue size 0");
    }

    // Allocate contiguous frames for desc+avail+used+req_buf
    let n = frames_needed(qsz);
    let f0 = physical::alloc_frame();
    for i in 1..n {
        let fi = physical::alloc_frame();
        if fi != f0 + i as u64 * 4096 {
            return Err("VirtIO-blk: queue frames not contiguous");
        }
    }

    // Zero all frames
    for i in 0..n {
        unsafe {
            core::ptr::write_bytes(
                paging::phys_to_virt(f0 + i as u64 * 4096) as *mut u8,
                0, 4096);
        }
    }

    // Compute layout
    let avail_off = (16 * qsz) as u64;
    let used_off  = used_ring_offset(qsz) as u64;
    let req_off   = req_buf_offset(qsz)   as u64;

    let avail_phys = f0 + avail_off;
    let used_phys  = f0 + used_off;
    let req_phys   = f0 + req_off;

    // Register queue with device
    write32(io_base, REG_QUEUE_ADDRESS, (f0 / 4096) as u32);

    // Signal DRIVER_OK
    set_status(io_base, STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_DRIVER_OK);

    let capacity = blk_capacity(io_base);

    *DISK.lock() = Some(VirtioBlk {
        io_base,
        queue_size:   qsz as u16,
        desc_virt:    paging::phys_to_virt(f0),
        avail_virt:   paging::phys_to_virt(avail_phys),
        used_virt:    paging::phys_to_virt(used_phys),
        req_hdr_phys: req_phys,
        req_dat_phys: req_phys + 16,
        req_sts_phys: req_phys + 16 + SECTOR_SIZE as u64,
        req_hdr_virt: paging::phys_to_virt(req_phys),
        req_dat_virt: paging::phys_to_virt(req_phys + 16),
        req_sts_virt: paging::phys_to_virt(req_phys + 16 + SECTOR_SIZE as u64),
        avail_idx:     0,
        last_used_idx: 0,
        capacity,
    });

    Ok(capacity)
}

// ─── Public I/O ───────────────────────────────────────────────────────────────

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

pub fn capacity() -> u64 {
    DISK.lock().as_ref().map(|d| d.capacity).unwrap_or(0)
}

// ─── Internal: single-sector I/O ─────────────────────────────────────────────

impl VirtioBlk {
    fn do_io(&mut self, req_type: u32, sector: u64, data: &mut [u8])
        -> Result<(), &'static str>
    {
        assert_eq!(data.len(), SECTOR_SIZE);

        let qsz = self.queue_size as usize;

        // Build request header and data in the dedicated buffer page
        unsafe {
            let hdr = self.req_hdr_virt as *mut BlkReqHdr;
            (*hdr).req_type = req_type;
            (*hdr).reserved = 0;
            (*hdr).sector   = sector;
            core::ptr::copy_nonoverlapping(
                data.as_ptr(), self.req_dat_virt as *mut u8, SECTOR_SIZE);
            *(self.req_sts_virt as *mut u8) = 0xFF;
        }

        // Fill descriptors 0..2
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

        // Publish in available ring
        // avail ring layout: flags(u16) idx(u16) ring[qsz](u16)
        let avail_slot  = (self.avail_idx as usize) % qsz;
        let ring_entry  = (self.avail_virt + 4 + avail_slot as u64 * 2) as *mut u16;
        let avail_idx_p = (self.avail_virt + 2) as *mut u16;
        unsafe {
            core::ptr::write_volatile(ring_entry, 0); // head = descriptor 0
            fence(Ordering::SeqCst);
            core::ptr::write_volatile(avail_idx_p, self.avail_idx.wrapping_add(1));
        }
        self.avail_idx = self.avail_idx.wrapping_add(1);

        // Kick device
        fence(Ordering::SeqCst);
        write16(self.io_base, REG_QUEUE_NOTIFY, 0);

        // Poll used ring: used.idx is at offset 2 within the used ring
        let used_idx_ptr = (self.used_virt + 2) as *const u16;
        let target = self.last_used_idx.wrapping_add(1);
        loop {
            fence(Ordering::SeqCst);
            if unsafe { core::ptr::read_volatile(used_idx_ptr) } == target { break; }
            core::hint::spin_loop();
        }
        self.last_used_idx = target;

        // Check status
        let st = unsafe { core::ptr::read_volatile(self.req_sts_virt as *const u8) };
        if st != VIRTIO_BLK_S_OK {
            return Err("VirtIO-blk: I/O error");
        }

        // Copy data out on read
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
