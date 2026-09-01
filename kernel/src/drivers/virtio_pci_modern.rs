//! NexusOS Generic VirtIO 1.0+ ("modern") PCI Transport — Phase K6
//!
//! Every existing VirtIO driver in this codebase (`drivers::virtio::{blk,net}`)
//! speaks the *legacy* (pre-1.0) I/O-port transport: a single fixed register
//! block at PCI BAR0, one physical-page-number write per queue. That transport
//! only exists for "transitional" device IDs (net/blk/etc, defined before
//! virtio 1.0). VirtIO-GPU was introduced *after* the 1.0 spec and has no
//! legacy/transitional PCI ID at all — QEMU's `virtio-gpu-pci` only speaks the
//! modern, PCI-capability-based transport, so a from-scratch layer is needed
//! here rather than reusing `drivers::virtio`'s I/O-port helpers.
//!
//! # Discovery
//! A modern VirtIO device advertises up to five vendor-specific (0x09) PCI
//! capabilities in its capability list, each a `struct virtio_pci_cap`
//! locating one functional region inside a BAR:
//!   - `cfg_type == 1` (COMMON_CFG)  — the main feature/status/queue registers
//!   - `cfg_type == 2` (NOTIFY_CFG)  — per-queue doorbell area (+ a multiplier)
//!   - `cfg_type == 3` (ISR_CFG)     — 1-byte interrupt status (unused; polled)
//!   - `cfg_type == 4` (DEVICE_CFG)  — device-type-specific config (optional)
//!
//! # Common configuration register layout (VirtIO 1.0 spec, byte offsets from
//! the mapped COMMON_CFG region — verified against the upstream
//! `include/uapi/linux/virtio_pci.h` header, not guessed):
//!   0x00 device_feature_select (u32 RW)   0x1E queue_notify_off (u16 RO)
//!   0x04 device_feature        (u32 RO)   0x20 queue_desc_lo    (u32 RW)
//!   0x08 driver_feature_select (u32 RW)   0x24 queue_desc_hi    (u32 RW)
//!   0x0C driver_feature        (u32 RW)   0x28 queue_avail_lo   (u32 RW)
//!   0x10 msix_config           (u16 RW)   0x2C queue_avail_hi   (u32 RW)
//!   0x12 num_queues            (u16 RO)   0x30 queue_used_lo    (u32 RW)
//!   0x14 device_status         (u8  RW)   0x34 queue_used_hi    (u32 RW)
//!   0x15 config_generation     (u8  RO)
//!   0x16 queue_select          (u16 RW)
//!   0x18 queue_size            (u16 RW)
//!   0x1A queue_msix_vector     (u16 RW)
//!   0x1C queue_enable          (u16 RW)
//!
//! Unlike the legacy transport (one physical *page number*, ring layout
//! derived by a fixed formula), modern virtio lets the driver give the
//! descriptor table, available ring, and used ring three fully independent
//! addresses — simpler to allocate (each fits in its own single page at the
//! small queue depths used here), at the cost of three register writes
//! instead of one.

use core::sync::atomic::{fence, Ordering};
use crate::drivers::{self, pci};
use crate::drivers::pci::PciDevice;
use crate::memory::{physical, paging};

// ─── PCI capability plumbing ──────────────────────────────────────────────────

const PCI_CAP_ID_VNDR: u8 = 0x09;
const PCI_STATUS_CAP_LIST: u16 = 1 << 4;

const CFG_TYPE_COMMON: u8 = 1;
const CFG_TYPE_NOTIFY: u8 = 2;
const CFG_TYPE_ISR:    u8 = 3;
const CFG_TYPE_DEVICE: u8 = 4;

// ─── Common configuration structure offsets ──────────────────────────────────

const COMMON_DFSELECT: u64 = 0x00;
const COMMON_DF:       u64 = 0x04;
const COMMON_GFSELECT: u64 = 0x08;
const COMMON_GF:       u64 = 0x0C;
const COMMON_STATUS:   u64 = 0x14;
const COMMON_Q_SELECT: u64 = 0x16;
const COMMON_Q_SIZE:   u64 = 0x18;
const COMMON_Q_ENABLE: u64 = 0x1C;
const COMMON_Q_NOFF:   u64 = 0x1E;
const COMMON_Q_DESCLO: u64 = 0x20;
const COMMON_Q_DESCHI: u64 = 0x24;
const COMMON_Q_AVAILLO:u64 = 0x28;
const COMMON_Q_AVAILHI:u64 = 0x2C;
const COMMON_Q_USEDLO: u64 = 0x30;
const COMMON_Q_USEDHI: u64 = 0x34;

// ─── Device status field (spec-wide, same bit values as the legacy transport)─

pub const STATUS_ACKNOWLEDGE: u8 = 1;
pub const STATUS_DRIVER:      u8 = 2;
pub const STATUS_DRIVER_OK:   u8 = 4;
pub const STATUS_FEATURES_OK: u8 = 8;

/// VIRTIO_F_VERSION_1 (feature bit 32): mandatory for every modern-only
/// device — without acking it, a spec-compliant device must refuse
/// FEATURES_OK.
pub const F_VERSION_1: u64 = 1u64 << 32;

/// The three (BAR-mapped) functional regions every modern VirtIO device
/// exposes, plus the device-specific config region when present.
pub struct VirtioModernCaps {
    common_virt: u64,
    notify_virt: u64,
    notify_off_multiplier: u32,
    #[allow(dead_code)] // ISR polling isn't used yet (no interrupts, matches the rest of this codebase's driver style)
    isr_virt: u64,
    pub device_virt: Option<u64>,
}

/// Walk `dev`'s PCI capability list and locate/map the COMMON_CFG,
/// NOTIFY_CFG, ISR_CFG, and (if present) DEVICE_CFG structures.
pub fn discover(dev: &PciDevice) -> Result<VirtioModernCaps, &'static str> {
    let pci_status = pci::read16(dev.bus, dev.dev, dev.func, 0x06);
    if pci_status & PCI_STATUS_CAP_LIST == 0 {
        return Err("virtio-modern: device has no PCI capability list");
    }

    let mut common: Option<(u8, u32, u32)> = None; // (bar, offset, length)
    let mut notify: Option<(u8, u32, u32, u32)> = None; // (bar, offset, length, off_multiplier)
    let mut isr:    Option<(u8, u32, u32)> = None;
    let mut device: Option<(u8, u32, u32)> = None;

    let mut ptr = (pci::read16(dev.bus, dev.dev, dev.func, 0x34) & 0xFF) as u8 & !0x3;
    let mut guard = 0u32;
    while ptr != 0 && guard < 64 {
        guard += 1;
        let dword0 = pci::read32(dev.bus, dev.dev, dev.func, ptr);
        let cap_id   = (dword0 & 0xFF) as u8;
        let cap_next = ((dword0 >> 8) & 0xFF) as u8;
        let cap_len  = ((dword0 >> 16) & 0xFF) as u8;
        let cfg_type = ((dword0 >> 24) & 0xFF) as u8;

        if cap_id == PCI_CAP_ID_VNDR && cap_len >= 16 {
            let dword1 = pci::read32(dev.bus, dev.dev, dev.func, ptr.wrapping_add(4));
            let bar    = (dword1 & 0xFF) as u8;
            let offset = pci::read32(dev.bus, dev.dev, dev.func, ptr.wrapping_add(8));
            let length = pci::read32(dev.bus, dev.dev, dev.func, ptr.wrapping_add(12));

            match cfg_type {
                CFG_TYPE_COMMON => common = Some((bar, offset, length)),
                CFG_TYPE_NOTIFY if cap_len >= 20 => {
                    let mult = pci::read32(dev.bus, dev.dev, dev.func, ptr.wrapping_add(16));
                    notify = Some((bar, offset, length, mult));
                }
                CFG_TYPE_ISR    => isr    = Some((bar, offset, length)),
                CFG_TYPE_DEVICE => device = Some((bar, offset, length)),
                _ => {}
            }
        }
        ptr = cap_next & !0x3;
    }

    let (common_bar, common_off, common_len) = common.ok_or("virtio-modern: no COMMON_CFG capability")?;
    let (notify_bar, notify_off, _notify_len, notify_mult) =
        notify.ok_or("virtio-modern: no NOTIFY_CFG capability")?;
    let (isr_bar, isr_off, isr_len) = isr.ok_or("virtio-modern: no ISR_CFG capability")?;

    let common_phys = drivers::read_bar_addr(dev, common_bar) + common_off as u64;
    let common_virt = drivers::map_mmio(common_phys, (common_len as usize).max(0x38));

    let notify_phys = drivers::read_bar_addr(dev, notify_bar) + notify_off as u64;
    // The notify region's true extent depends on num_queues * notify_off_multiplier,
    // which isn't known until after COMMON_CFG is mapped; one page comfortably
    // covers every queue count this driver will ever use (a handful of queues).
    let notify_virt = drivers::map_mmio(notify_phys, 4096);

    let isr_phys = drivers::read_bar_addr(dev, isr_bar) + isr_off as u64;
    let isr_virt = drivers::map_mmio(isr_phys, (isr_len as usize).max(1));

    let device_virt = device.map(|(bar, off, len)| {
        let phys = drivers::read_bar_addr(dev, bar) + off as u64;
        drivers::map_mmio(phys, (len as usize).max(4))
    });

    Ok(VirtioModernCaps {
        common_virt,
        notify_virt,
        notify_off_multiplier: notify_mult,
        isr_virt,
        device_virt,
    })
}

// ─── Common-config register access ───────────────────────────────────────────

#[inline] fn common_read8(c: &VirtioModernCaps, off: u64) -> u8 {
    unsafe { core::ptr::read_volatile((c.common_virt + off) as *const u8) }
}
#[inline] fn common_write8(c: &VirtioModernCaps, off: u64, v: u8) {
    unsafe { core::ptr::write_volatile((c.common_virt + off) as *mut u8, v) }
}
#[inline] fn common_read16(c: &VirtioModernCaps, off: u64) -> u16 {
    unsafe { core::ptr::read_volatile((c.common_virt + off) as *const u16) }
}
#[inline] fn common_write16(c: &VirtioModernCaps, off: u64, v: u16) {
    unsafe { core::ptr::write_volatile((c.common_virt + off) as *mut u16, v) }
}
#[inline] fn common_read32(c: &VirtioModernCaps, off: u64) -> u32 {
    unsafe { core::ptr::read_volatile((c.common_virt + off) as *const u32) }
}
#[inline] fn common_write32(c: &VirtioModernCaps, off: u64, v: u32) {
    unsafe { core::ptr::write_volatile((c.common_virt + off) as *mut u32, v) }
}

/// Reset the device (write 0 to device_status, then wait for it to read back
/// as 0 — the spec-mandated way to know a prior driver's state is gone).
pub fn reset(c: &VirtioModernCaps) {
    common_write8(c, COMMON_STATUS, 0);
    while common_read8(c, COMMON_STATUS) != 0 {
        core::hint::spin_loop();
    }
}

/// ACKNOWLEDGE → DRIVER → negotiate `wanted | VIRTIO_F_VERSION_1` → FEATURES_OK,
/// verifying the device actually latched FEATURES_OK (a spec-compliant device
/// clears its own status back down if it disliked the negotiated set — the
/// mandatory way to detect a bad negotiation before touching any queue).
pub fn negotiate(c: &VirtioModernCaps, wanted: u64) -> Result<(), &'static str> {
    common_write8(c, COMMON_STATUS, STATUS_ACKNOWLEDGE);
    common_write8(c, COMMON_STATUS, STATUS_ACKNOWLEDGE | STATUS_DRIVER);

    common_write32(c, COMMON_DFSELECT, 0);
    let dev_lo = common_read32(c, COMMON_DF) as u64;
    common_write32(c, COMMON_DFSELECT, 1);
    let dev_hi = common_read32(c, COMMON_DF) as u64;
    let device_features = dev_lo | (dev_hi << 32);

    let drv_features = device_features & (wanted | F_VERSION_1);
    if drv_features & F_VERSION_1 == 0 {
        return Err("virtio-modern: device does not offer VIRTIO_F_VERSION_1");
    }

    common_write32(c, COMMON_GFSELECT, 0);
    common_write32(c, COMMON_GF, drv_features as u32);
    common_write32(c, COMMON_GFSELECT, 1);
    common_write32(c, COMMON_GF, (drv_features >> 32) as u32);

    common_write8(c, COMMON_STATUS, STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK);
    if common_read8(c, COMMON_STATUS) & STATUS_FEATURES_OK == 0 {
        return Err("virtio-modern: device rejected FEATURES_OK");
    }
    Ok(())
}

/// Set DRIVER_OK: the device is now live and may process virtqueue traffic.
pub fn set_driver_ok(c: &VirtioModernCaps) {
    let status = common_read8(c, COMMON_STATUS);
    common_write8(c, COMMON_STATUS, status | STATUS_DRIVER_OK);
}

// ─── Virtqueue (modern layout: three independent addresses, no PFN formula) ──

#[repr(C)]
struct VirtqDesc {
    addr:  u64,
    len:   u32,
    flags: u16,
    next:  u16,
}

const VRING_DESC_F_NEXT:  u16 = 1;
const VRING_DESC_F_WRITE: u16 = 2;
const VRING_AVAIL_F_NO_INTERRUPT: u16 = 1;

/// A single modern-layout virtqueue. At the small queue depths used by this
/// driver (a handful of descriptors), the descriptor table, available ring,
/// and used ring each comfortably fit in one 4 KiB page on their own — no
/// multi-page contiguous-allocation formula (like the legacy transport
/// needs) is required here.
pub struct ModernQueue {
    index:         u16,
    size:          u16,
    desc_virt:     u64,
    avail_virt:    u64,
    used_virt:     u64,
    notify_addr:   u64,
    avail_idx:     u16,
    last_used_idx: u16,
}

impl ModernQueue {
    /// Select queue `index`, shrink it to at most `max_size` descriptors,
    /// allocate and register its three rings, and enable it.
    pub fn setup(c: &VirtioModernCaps, index: u16, max_size: u16) -> Result<Self, &'static str> {
        common_write16(c, COMMON_Q_SELECT, index);
        let dev_size = common_read16(c, COMMON_Q_SIZE);
        if dev_size == 0 {
            return Err("virtio-modern: selected queue is not available");
        }
        let size = dev_size.min(max_size).max(2);
        common_write16(c, COMMON_Q_SIZE, size);

        let desc_bytes  = 16usize * size as usize;
        let avail_bytes = 4 + 2 * size as usize;
        let used_bytes  = 4 + 8 * size as usize;
        assert!(desc_bytes <= 4096 && avail_bytes <= 4096 && used_bytes <= 4096,
            "virtio-modern: queue size too large for single-page rings");

        let desc_phys  = physical::alloc_frame();
        let avail_phys = physical::alloc_frame();
        let used_phys  = physical::alloc_frame();
        unsafe {
            core::ptr::write_bytes(paging::phys_to_virt(desc_phys)  as *mut u8, 0, 4096);
            core::ptr::write_bytes(paging::phys_to_virt(avail_phys) as *mut u8, 0, 4096);
            core::ptr::write_bytes(paging::phys_to_virt(used_phys)  as *mut u8, 0, 4096);
        }

        common_write32(c, COMMON_Q_DESCLO,  desc_phys as u32);
        common_write32(c, COMMON_Q_DESCHI, (desc_phys >> 32) as u32);
        common_write32(c, COMMON_Q_AVAILLO, avail_phys as u32);
        common_write32(c, COMMON_Q_AVAILHI,(avail_phys >> 32) as u32);
        common_write32(c, COMMON_Q_USEDLO,  used_phys as u32);
        common_write32(c, COMMON_Q_USEDHI, (used_phys >> 32) as u32);

        let notify_off = common_read16(c, COMMON_Q_NOFF);
        let notify_addr = c.notify_virt + (notify_off as u64) * (c.notify_off_multiplier as u64);

        common_write16(c, COMMON_Q_ENABLE, 1);

        let q = ModernQueue {
            index,
            size,
            desc_virt:  paging::phys_to_virt(desc_phys),
            avail_virt: paging::phys_to_virt(avail_phys),
            used_virt:  paging::phys_to_virt(used_phys),
            notify_addr,
            avail_idx: 0,
            last_used_idx: 0,
        };

        // Poll-only, matching every other driver in this codebase (xHCI, net,
        // blk): no interrupts are configured, so tell the device not to
        // bother raising one.
        unsafe { core::ptr::write_volatile(q.avail_virt as *mut u16, VRING_AVAIL_F_NO_INTERRUPT); }

        Ok(q)
    }

    unsafe fn write_desc(&self, id: usize, addr: u64, len: u32, flags: u16, next: u16) {
        let d = (self.desc_virt as *mut VirtqDesc).add(id);
        (*d).addr = addr;
        (*d).len = len;
        (*d).flags = flags;
        (*d).next = next;
    }

    fn publish(&mut self, head: u16) {
        let slot = (self.avail_idx as usize) % self.size as usize;
        let ring_entry = (self.avail_virt + 4 + slot as u64 * 2) as *mut u16;
        let avail_idx_p = (self.avail_virt + 2) as *mut u16;
        unsafe {
            core::ptr::write_volatile(ring_entry, head);
            fence(Ordering::SeqCst);
            core::ptr::write_volatile(avail_idx_p, self.avail_idx.wrapping_add(1));
        }
        self.avail_idx = self.avail_idx.wrapping_add(1);
    }

    fn kick(&self) {
        fence(Ordering::SeqCst);
        unsafe { core::ptr::write_volatile(self.notify_addr as *mut u16, self.index); }
    }

    fn used_idx(&self) -> u16 {
        unsafe { core::ptr::read_volatile((self.used_virt + 2) as *const u16) }
    }

    fn used_len(&self, slot: usize) -> u32 {
        let base = self.used_virt + 4 + (slot * 8) as u64;
        unsafe { core::ptr::read_volatile((base + 4) as *const u32) }
    }

    /// Submit a 2-descriptor chain (device-readable request, then
    /// device-writable response) and spin-wait for its completion. Returns
    /// the number of bytes the device actually wrote into the response
    /// buffer. Bounded spin, matching `virtio::net`'s synchronous TX pattern.
    pub fn send_sync(&mut self, req_phys: u64, req_len: u32, resp_phys: u64, resp_cap: u32) -> Result<u32, &'static str> {
        unsafe {
            self.write_desc(0, req_phys, req_len, VRING_DESC_F_NEXT, 1);
            self.write_desc(1, resp_phys, resp_cap, VRING_DESC_F_WRITE, 0);
        }
        self.publish(0);
        self.kick();

        let target = self.last_used_idx.wrapping_add(1);
        let mut spins: u64 = 0;
        const SPIN_LIMIT: u64 = 50_000_000;
        loop {
            fence(Ordering::SeqCst);
            if self.used_idx() == target { break; }
            spins += 1;
            if spins > SPIN_LIMIT {
                return Err("virtio-modern: command timed out waiting for completion");
            }
            core::hint::spin_loop();
        }
        let slot = (self.last_used_idx as usize) % self.size as usize;
        let written = self.used_len(slot);
        self.last_used_idx = target;
        Ok(written)
    }
}
