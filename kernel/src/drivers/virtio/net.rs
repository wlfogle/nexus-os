//! NexusOS VirtIO Network Device Driver (legacy / pre-1.0 I/O-port transport)
//!
//! Mirrors the legacy virtqueue layout proven by the VirtIO-blk driver
//! (`blk.rs`) but drives the two virtio-net virtqueues:
//!
//!   queue 0 — receiveq  (device → driver)
//!   queue 1 — transmitq (driver → device)
//!
//! Each queue's contiguous allocation holds the descriptor table, the available
//! ring, the (page-aligned) used ring, and finally a block of fixed-size DMA
//! buffers — one per descriptor we intend to use.  We deliberately negotiate
//! only VIRTIO_NET_F_MAC (+ STATUS): no mergeable RX buffers (so the packet
//! header is exactly 10 bytes) and no EVENT_IDX (so the ring layout matches the
//! blk formula).  No checksum/GSO offload is negotiated, so smoltcp produces
//! fully-formed frames with valid checksums.
//!
//! Per-buffer layout (single descriptor, no chaining):
//!   [ virtio_net_hdr (10 B) ][ ethernet frame (≤ 1514 B) ]
//!
//! RX is buffered (RX_BUFFERS posted up front, re-posted after consumption).
//! TX is synchronous: one buffer, kick, poll the used ring to completion.

use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{fence, Ordering};
use crate::memory::{physical, paging};
use super::{
    REG_QUEUE_ADDRESS, REG_QUEUE_SIZE, REG_QUEUE_SELECT, REG_QUEUE_NOTIFY,
    REG_DRIVER_FEATURES, REG_DEVICE_FEATURES,
    STATUS_ACKNOWLEDGE, STATUS_DRIVER, STATUS_DRIVER_OK,
    write16, write32, read16, read32,
    reset_and_ack, set_status, net_mac,
    VIRTIO_NET_F_MAC, VIRTIO_NET_F_STATUS,
};

// ─── Constants ────────────────────────────────────────────────────────────────

/// Legacy virtio_net_hdr size (no mergeable RX buffers).
const NET_HDR_LEN: usize = 10;

/// Bytes reserved per DMA buffer: header + max ethernet frame + slack.
const BUF_STRIDE: usize = 2048;

/// Largest ethernet frame we can place in one buffer.
const FRAME_MAX: usize = BUF_STRIDE - NET_HDR_LEN;

/// Number of receive buffers posted to the device.
const RX_BUFFERS: usize = 16;

/// Transmit buffers (TX is synchronous, so a single buffer suffices).
const TX_BUFFERS: usize = 1;

/// Virtqueue indices.
const RX_QUEUE: u16 = 0;
const TX_QUEUE: u16 = 1;

const VRING_DESC_F_WRITE: u16 = 2;

/// Available-ring flag: suppress device→driver interrupts (we poll).
const VRING_AVAIL_F_NO_INTERRUPT: u16 = 1;

/// Bounded spin budget for a synchronous TX completion before giving up.
const TX_SPIN_LIMIT: u64 = 50_000_000;

// ─── Layout helpers (identical formula to blk.rs) ─────────────────────────────

#[inline]
const fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

fn used_ring_offset(qsz: usize) -> usize {
    let desc  = 16 * qsz;
    let avail = 4 + 2 * qsz; // flags + idx + ring[qsz]
    align_up(desc + avail, 4096)
}

fn buf_offset(qsz: usize) -> usize {
    let used_off   = used_ring_offset(qsz);
    let used_bytes = 4 + 8 * qsz; // flags + idx + used_elem[qsz]
    align_up(used_off + used_bytes, 4096)
}

fn frames_needed(qsz: usize, nbuf: usize) -> usize {
    let total = buf_offset(qsz) + nbuf * BUF_STRIDE;
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

/// virtio_net_hdr (legacy, 10 bytes, no num_buffers).
#[repr(C)]
struct VirtioNetHdr {
    flags:      u8,
    gso_type:   u8,
    hdr_len:    u16,
    gso_size:   u16,
    csum_start: u16,
    csum_offset: u16,
}

/// A single legacy virtqueue plus its dedicated DMA buffer pool.
struct Virtqueue {
    queue_idx:     u16,
    qsz:           usize,
    desc_virt:     u64,
    avail_virt:    u64,
    used_virt:     u64,
    buf_phys_base: u64,
    buf_virt_base: u64,
    buf_count:     usize,
    avail_idx:     u16,
    last_used_idx: u16,
}

impl Virtqueue {
    /// Select `queue_idx`, allocate contiguous frames for the ring + `nbuf`
    /// buffers, register the queue's physical page number with the device, and
    /// mark the available ring as no-interrupt (polled).
    fn setup(io_base: u16, queue_idx: u16, nbuf: usize) -> Result<Virtqueue, &'static str> {
        write16(io_base, REG_QUEUE_SELECT, queue_idx);
        let qsz = read16(io_base, REG_QUEUE_SIZE) as usize;
        if qsz == 0 {
            return Err("VirtIO-net: queue size 0");
        }
        if nbuf > qsz {
            return Err("VirtIO-net: more buffers than queue slots");
        }

        let n = frames_needed(qsz, nbuf);
        let f0 = physical::alloc_frame();
        for i in 1..n {
            let fi = physical::alloc_frame();
            if fi != f0 + i as u64 * 4096 {
                return Err("VirtIO-net: queue frames not contiguous");
            }
        }

        // Zero the whole allocation (rings + buffers).
        for i in 0..n {
            unsafe {
                core::ptr::write_bytes(
                    paging::phys_to_virt(f0 + i as u64 * 4096) as *mut u8,
                    0, 4096);
            }
        }

        let avail_phys = f0 + (16 * qsz) as u64;
        let used_phys  = f0 + used_ring_offset(qsz) as u64;
        let buf_phys   = f0 + buf_offset(qsz) as u64;

        // Register the queue (device derives ring offsets from this PFN + qsz).
        write32(io_base, REG_QUEUE_ADDRESS, (f0 / 4096) as u32);

        let q = Virtqueue {
            queue_idx,
            qsz,
            desc_virt:     paging::phys_to_virt(f0),
            avail_virt:    paging::phys_to_virt(avail_phys),
            used_virt:     paging::phys_to_virt(used_phys),
            buf_phys_base: buf_phys,
            buf_virt_base: paging::phys_to_virt(buf_phys),
            buf_count:     nbuf,
            avail_idx:     0,
            last_used_idx: 0,
        };

        // Poll-only: tell the device not to interrupt us on used-ring updates.
        unsafe {
            core::ptr::write_volatile(q.avail_virt as *mut u16, VRING_AVAIL_F_NO_INTERRUPT);
        }

        Ok(q)
    }

    #[inline]
    fn buf_phys(&self, id: usize) -> u64 { self.buf_phys_base + (id * BUF_STRIDE) as u64 }

    #[inline]
    fn buf_virt(&self, id: usize) -> u64 { self.buf_virt_base + (id * BUF_STRIDE) as u64 }

    /// Write descriptor `id` in the descriptor table.
    unsafe fn write_desc(&self, id: usize, addr: u64, len: u32, flags: u16) {
        let d = (self.desc_virt as *mut VirtqDesc).add(id);
        (*d).addr  = addr;
        (*d).len   = len;
        (*d).flags = flags;
        (*d).next  = 0;
    }

    /// Publish descriptor `id` as the head of a new available-ring entry.
    fn publish(&mut self, id: u16) {
        let slot = (self.avail_idx as usize) % self.qsz;
        let ring_entry  = (self.avail_virt + 4 + slot as u64 * 2) as *mut u16;
        let avail_idx_p = (self.avail_virt + 2) as *mut u16;
        unsafe {
            core::ptr::write_volatile(ring_entry, id);
            fence(Ordering::SeqCst);
            core::ptr::write_volatile(avail_idx_p, self.avail_idx.wrapping_add(1));
        }
        self.avail_idx = self.avail_idx.wrapping_add(1);
    }

    /// Notify the device that the available ring changed.
    fn kick(&self, io_base: u16) {
        fence(Ordering::SeqCst);
        write16(io_base, REG_QUEUE_NOTIFY, self.queue_idx);
    }

    /// Current device-published used-ring index.
    #[inline]
    fn used_idx(&self) -> u16 {
        unsafe { core::ptr::read_volatile((self.used_virt + 2) as *const u16) }
    }

    /// Read the {descriptor id, written length} of used-ring entry `slot`.
    fn used_entry(&self, slot: usize) -> (u32, u32) {
        let base = self.used_virt + 4 + (slot * 8) as u64;
        unsafe {
            let id  = core::ptr::read_volatile(base as *const u32);
            let len = core::ptr::read_volatile((base + 4) as *const u32);
            (id, len)
        }
    }
}

// ─── Driver state ─────────────────────────────────────────────────────────────

pub struct VirtioNet {
    io_base:  u16,
    rx:       Virtqueue,
    tx:       Virtqueue,
    mac:      [u8; 6],
    rx_count: u64,
    tx_count: u64,
}

static NIC: Mutex<Option<VirtioNet>> = Mutex::new(None);

// ─── Initialisation ───────────────────────────────────────────────────────────

/// Initialise a legacy VirtIO-net device at the given I/O base.
/// Returns the device MAC address on success.
pub fn init(io_base: u16) -> Result<[u8; 6], &'static str> {
    reset_and_ack(io_base);

    // Negotiate only MAC + STATUS: no MRG_RXBUF (10-byte header), no EVENT_IDX
    // (ring layout matches the blk formula), no offloads (smoltcp builds full
    // frames with valid checksums).
    let dev_features = read32(io_base, REG_DEVICE_FEATURES);
    let drv_features = dev_features & (VIRTIO_NET_F_MAC | VIRTIO_NET_F_STATUS);
    write32(io_base, REG_DRIVER_FEATURES, drv_features);

    let mac = net_mac(io_base);

    let mut rx = Virtqueue::setup(io_base, RX_QUEUE, RX_BUFFERS)?;
    let tx = Virtqueue::setup(io_base, TX_QUEUE, TX_BUFFERS)?;

    // Device is live once DRIVER_OK is set; then post receive buffers.
    set_status(io_base, STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_DRIVER_OK);

    // Post all RX buffers: one device-writable descriptor each.
    for id in 0..RX_BUFFERS {
        unsafe {
            rx.write_desc(id, rx.buf_phys(id), BUF_STRIDE as u32, VRING_DESC_F_WRITE);
        }
        rx.publish(id as u16);
    }
    rx.kick(io_base);

    *NIC.lock() = Some(VirtioNet {
        io_base,
        rx,
        tx,
        mac,
        rx_count: 0,
        tx_count: 0,
    });

    Ok(mac)
}

/// Whether a NIC has been initialised.
pub fn present() -> bool {
    NIC.lock().is_some()
}

/// The initialised NIC's MAC address, if any.
pub fn mac() -> Option<[u8; 6]> {
    NIC.lock().as_ref().map(|n| n.mac)
}

/// (rx_frames, tx_frames) processed so far.
pub fn stats() -> (u64, u64) {
    NIC.lock().as_ref().map(|n| (n.rx_count, n.tx_count)).unwrap_or((0, 0))
}

// ─── RX / TX ──────────────────────────────────────────────────────────────────

/// Pull one received ethernet frame (header stripped), or `None` if the RX
/// ring is currently empty.  The consumed buffer is immediately re-posted.
pub fn receive_frame() -> Option<Vec<u8>> {
    let mut guard = NIC.lock();
    let n = guard.as_mut()?;

    let used_idx = n.rx.used_idx();
    if n.rx.last_used_idx == used_idx {
        return None;
    }

    let slot = (n.rx.last_used_idx as usize) % n.rx.qsz;
    let (id, len) = n.rx.used_entry(slot);
    let id = id as usize;

    // Strip the virtio_net_hdr; remaining bytes are the ethernet frame.
    let total = len as usize;
    let frame_len = total.saturating_sub(NET_HDR_LEN).min(FRAME_MAX);
    let mut frame = Vec::with_capacity(frame_len);
    if frame_len > 0 && id < n.rx.buf_count {
        let src = (n.rx.buf_virt(id) + NET_HDR_LEN as u64) as *const u8;
        unsafe {
            frame.set_len(frame_len);
            core::ptr::copy_nonoverlapping(src, frame.as_mut_ptr(), frame_len);
        }
    }

    n.rx.last_used_idx = n.rx.last_used_idx.wrapping_add(1);

    // Re-post the same buffer (descriptor unchanged) so the device can reuse it.
    n.rx.publish(id as u16);
    let io_base = n.io_base;
    n.rx.kick(io_base);

    n.rx_count = n.rx_count.wrapping_add(1);
    Some(frame)
}

/// Transmit a single ethernet frame synchronously.  Returns `false` if the
/// frame is too large, no NIC is present, or the device did not complete in
/// time.
pub fn transmit_frame(frame: &[u8]) -> bool {
    if frame.is_empty() || frame.len() > FRAME_MAX {
        return false;
    }

    let mut guard = NIC.lock();
    let n = match guard.as_mut() {
        Some(n) => n,
        None => return false,
    };

    // Build [virtio_net_hdr | frame] in TX buffer 0.
    let buf = n.tx.buf_virt(0);
    unsafe {
        let hdr = buf as *mut VirtioNetHdr;
        (*hdr).flags       = 0;
        (*hdr).gso_type    = 0; // VIRTIO_NET_HDR_GSO_NONE
        (*hdr).hdr_len     = 0;
        (*hdr).gso_size    = 0;
        (*hdr).csum_start  = 0;
        (*hdr).csum_offset = 0;
        core::ptr::copy_nonoverlapping(
            frame.as_ptr(),
            (buf + NET_HDR_LEN as u64) as *mut u8,
            frame.len());
    }

    let total = (NET_HDR_LEN + frame.len()) as u32;
    unsafe {
        n.tx.write_desc(0, n.tx.buf_phys(0), total, 0); // device-readable
    }
    n.tx.publish(0);
    let io_base = n.io_base;
    n.tx.kick(io_base);

    // Wait for completion (bounded).
    let target = n.tx.last_used_idx.wrapping_add(1);
    let mut spins = 0u64;
    loop {
        fence(Ordering::SeqCst);
        if n.tx.used_idx() == target {
            break;
        }
        spins += 1;
        if spins > TX_SPIN_LIMIT {
            return false;
        }
        core::hint::spin_loop();
    }
    n.tx.last_used_idx = target;
    n.tx_count = n.tx_count.wrapping_add(1);
    true
}
