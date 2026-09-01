//! NexusOS VirtIO-GPU 2D Driver — Phase K6
//!
//! VirtIO-GPU was introduced after the VirtIO 1.0 spec and has no legacy
//! transitional PCI ID at all, so — unlike `virtio::{blk,net}` — this driver
//! is built entirely on `drivers::virtio_pci_modern`'s capability-based
//! transport rather than the legacy I/O-port one. Scope is deliberately
//! narrow, matching every other driver in this codebase: 2D only (no VIRGL
//! 3D), a single scanout, one control virtqueue (the cursor virtqueue is
//! unused), and fully synchronous polling (no interrupts).
//!
//! This exists to give the QEMU/dev-loop test path a real GPU *driver* to
//! exercise (resource lifecycle, scanout binding, host transfer/flush)
//! independent of whatever fixed mode Limine's firmware-level framebuffer
//! request already set up pre-boot — see the "NexusOS Kernel Completion
//! Roadmap" plan's Phase K6 for why this is the QEMU-side counterpart to
//! real-hardware framebuffer validation on the Intel iGPU.
//!
//! Command structures below mirror the upstream
//! `include/uapi/linux/virtio_gpu.h` byte-for-byte (verified against it, not
//! guessed) — x86_64 is little-endian so plain `u32`/`u64` fields need no
//! explicit LE wrapper, the same convention `virtio::net`'s `VirtioNetHdr`
//! already uses in this codebase.

use crate::drivers::{self, pci, virtio_pci_modern};
use crate::memory::{physical, paging};
use alloc::vec::Vec;

const VIRTIO_VENDOR: u16 = 0x1AF4;
/// Modern-only PCI device ID: `0x1040 + virtio_device_id`, virtio_device_id
/// 16 = GPU (same `0x1040 + N` pattern this codebase's own `net::VIRTIO_NET_MODERN
/// = 0x1041` already confirms for device_id 1).
const VIRTIO_GPU_MODERN: u16 = 0x1050;

// ─── virtio-gpu control-queue command/response types ─────────────────────────

const CMD_GET_DISPLAY_INFO:        u32 = 0x0100;
const CMD_RESOURCE_CREATE_2D:      u32 = 0x0101;
const CMD_SET_SCANOUT:             u32 = 0x0103;
const CMD_RESOURCE_FLUSH:          u32 = 0x0104;
const CMD_TRANSFER_TO_HOST_2D:     u32 = 0x0105;
const CMD_RESOURCE_ATTACH_BACKING: u32 = 0x0106;

const RESP_OK_NODATA:       u32 = 0x1100;
const RESP_OK_DISPLAY_INFO: u32 = 0x1101;

const FORMAT_B8G8R8A8_UNORM: u32 = 1;

// ─── Wire structures (repr(C), byte-exact to the spec) ───────────────────────

#[repr(C)]
#[derive(Clone, Copy)]
struct CtrlHdr {
    type_:    u32,
    flags:    u32,
    fence_id: u64,
    ctx_id:   u32,
    ring_idx: u8,
    padding:  [u8; 3],
}

impl CtrlHdr {
    fn new(type_: u32) -> Self {
        Self { type_, flags: 0, fence_id: 0, ctx_id: 0, ring_idx: 0, padding: [0; 3] }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Rect { x: u32, y: u32, width: u32, height: u32 }

#[repr(C)]
struct ResourceCreate2d { hdr: CtrlHdr, resource_id: u32, format: u32, width: u32, height: u32 }

#[repr(C)]
struct SetScanout { hdr: CtrlHdr, r: Rect, scanout_id: u32, resource_id: u32 }

#[repr(C)]
struct ResourceFlush { hdr: CtrlHdr, r: Rect, resource_id: u32, padding: u32 }

#[repr(C)]
struct TransferToHost2d { hdr: CtrlHdr, r: Rect, offset: u64, resource_id: u32, padding: u32 }

#[repr(C)]
struct MemEntry { addr: u64, length: u32, padding: u32 }

#[repr(C)]
struct ResourceAttachBackingHdr { hdr: CtrlHdr, resource_id: u32, nr_entries: u32 }

// ─── Smoke-test resource: deliberately small and independent of whatever ────
// resolution GET_DISPLAY_INFO reports — this driver proves the 2D command
// protocol round-trips correctly end to end, not that it takes over the
// full display (there is nothing to visually confirm under `-display none`
// anyway, matching this session's established precedent for USB HID's
// interrupt-IN report parsing: prove the protocol layer, since interactive
// confirmation isn't available).
const TEST_RESOURCE_ID: u32 = 1;
const TEST_W: u32 = 256;
const TEST_H: u32 = 256;

/// Fixed-size scratch buffer for building a request/reading a response.
/// One page each is ample: the largest request here (RESOURCE_ATTACH_BACKING)
/// is a 32-byte header plus at most `TEST_H*4/4096`-ish coalesced mem
/// entries (16 bytes each) — nowhere close to a page — and the largest
/// response (RESP_OK_DISPLAY_INFO) is a fixed 24 + 16*24 = 408 bytes.
struct Scratch {
    req_phys:  u64,
    req_virt:  u64,
    resp_phys: u64,
    resp_virt: u64,
}

impl Scratch {
    fn new() -> Self {
        let req_phys = physical::alloc_frame();
        let resp_phys = physical::alloc_frame();
        let req_virt = paging::phys_to_virt(req_phys);
        let resp_virt = paging::phys_to_virt(resp_phys);
        unsafe {
            core::ptr::write_bytes(req_virt as *mut u8, 0, 4096);
            core::ptr::write_bytes(resp_virt as *mut u8, 0, 4096);
        }
        Self { req_phys, req_virt, resp_phys, resp_virt }
    }
}

/// Discover, bring up, and smoke-test a VirtIO-GPU device. Non-fatal to the
/// rest of boot if anything is missing or fails — matches every other
/// optional device driver in this codebase (no NIC / no USB controller /
/// no disk all just log and continue).
pub fn init() {
    let dev = match pci::find(&[(VIRTIO_VENDOR, VIRTIO_GPU_MODERN)]) {
        Some(d) => d,
        None => {
            crate::kprintln!("[gpu]  no VirtIO-GPU device found");
            return;
        }
    };
    drivers::enable_mem_and_busmaster(&dev);
    crate::kprintln!(
        "[gpu]  PCI {:02x}:{:02x}.{} vendor={:#06x} device={:#06x}",
        dev.bus, dev.dev, dev.func, dev.vendor_id, dev.device_id
    );

    let caps = match virtio_pci_modern::discover(&dev) {
        Ok(c) => c,
        Err(e) => { crate::kprintln!("[gpu]  capability discovery failed: {}", e); return; }
    };

    virtio_pci_modern::reset(&caps);
    if let Err(e) = virtio_pci_modern::negotiate(&caps, 0) {
        crate::kprintln!("[gpu]  feature negotiation failed: {}", e);
        return;
    }

    // Queue 0 is always the controlq per the virtio-gpu spec (queue 1 would
    // be the cursorq, unused here). 8 descriptors is ample for a single
    // in-flight, fully synchronous request/response pair at a time.
    let mut controlq = match virtio_pci_modern::ModernQueue::setup(&caps, 0, 8) {
        Ok(q) => q,
        Err(e) => { crate::kprintln!("[gpu]  controlq setup failed: {}", e); return; }
    };
    virtio_pci_modern::set_driver_ok(&caps);

    let scratch = Scratch::new();

    // ── GET_DISPLAY_INFO — informational only; the smoke-test resource
    // below deliberately uses a fixed small size instead of whatever this
    // reports, so a disabled/zero-size scanout doesn't block verification.
    unsafe {
        *(scratch.req_virt as *mut CtrlHdr) = CtrlHdr::new(CMD_GET_DISPLAY_INFO);
    }
    match controlq.send_sync(scratch.req_phys, core::mem::size_of::<CtrlHdr>() as u32, scratch.resp_phys, 4096) {
        Ok(_) => {
            let resp_type = unsafe { core::ptr::read_volatile(scratch.resp_virt as *const u32) };
            if resp_type == RESP_OK_DISPLAY_INFO {
                let r = unsafe { core::ptr::read_volatile((scratch.resp_virt + 24) as *const Rect) };
                let enabled = unsafe { core::ptr::read_volatile((scratch.resp_virt + 24 + 16) as *const u32) };
                crate::kprintln!("[gpu]  display 0: {}x{} enabled={}", r.width, r.height, enabled);
            } else {
                crate::kprintln!("[gpu]  GET_DISPLAY_INFO unexpected response type={:#06x}", resp_type);
            }
        }
        Err(e) => crate::kprintln!("[gpu]  GET_DISPLAY_INFO failed: {}", e),
    }

    if !run_2d_smoke_test(&mut controlq, &scratch) {
        crate::kprintln!("[gpu]  2D smoke test FAILED — see preceding log lines");
        return;
    }

    crate::kprintln!(
        "[gpu]  VirtIO-GPU 2D smoke test passed: resource {}x{} created, backed, scanned out, transferred, flushed",
        TEST_W, TEST_H
    );
}

/// RESOURCE_CREATE_2D → RESOURCE_ATTACH_BACKING → SET_SCANOUT →
/// TRANSFER_TO_HOST_2D → RESOURCE_FLUSH, checking every response is the
/// expected `RESP_OK_NODATA` (not an error type) before proceeding to the
/// next step. Returns `false` on the first unexpected response or timeout.
fn run_2d_smoke_test(controlq: &mut virtio_pci_modern::ModernQueue, scratch: &Scratch) -> bool {
    let full_rect = Rect { x: 0, y: 0, width: TEST_W, height: TEST_H };

    // ── RESOURCE_CREATE_2D ────────────────────────────────────────────────
    unsafe {
        *(scratch.req_virt as *mut ResourceCreate2d) = ResourceCreate2d {
            hdr: CtrlHdr::new(CMD_RESOURCE_CREATE_2D),
            resource_id: TEST_RESOURCE_ID,
            format: FORMAT_B8G8R8A8_UNORM,
            width: TEST_W,
            height: TEST_H,
        };
    }
    if !expect_ok(controlq, scratch, core::mem::size_of::<ResourceCreate2d>(), "RESOURCE_CREATE_2D") {
        return false;
    }

    // ── Backing store: allocate ceil(W*H*4 / 4096) frames (not required to
    // be contiguous — mem entries are coalesced into runs where the
    // allocator happens to hand out adjacent frames, and passed as however
    // many independent runs remain otherwise; the spec allows any number of
    // mem entries, unlike the legacy transport's single-PFN queue setup). ──
    let bytes = (TEST_W as usize) * (TEST_H as usize) * 4;
    let pages = (bytes + 4095) / 4096;
    let mut frames = Vec::with_capacity(pages);
    for _ in 0..pages {
        let f = physical::alloc_frame();
        unsafe { core::ptr::write_bytes(paging::phys_to_virt(f) as *mut u8, 0, 4096); }
        frames.push(f);
    }
    // Paint a recognizable solid pattern (opaque mid-blue in B8G8R8A8) into
    // the backing store. Nothing can visually confirm this under
    // `-display none`, so the correctness signal here is entirely protocol-
    // level (every command below getting RESP_OK_NODATA back), matching
    // this session's established precedent for USB HID's report parsing.
    for &f in &frames {
        let base = paging::phys_to_virt(f) as *mut u32;
        for i in 0..(4096 / 4) {
            unsafe { core::ptr::write_volatile(base.add(i), 0xFF3060C0u32); }
        }
    }
    let mut runs: Vec<(u64, u32)> = Vec::new();
    for &f in &frames {
        if let Some(last) = runs.last_mut() {
            if last.0 + last.1 as u64 == f {
                last.1 += 4096;
                continue;
            }
        }
        runs.push((f, 4096));
    }

    // ── RESOURCE_ATTACH_BACKING ───────────────────────────────────────────
    let hdr_size = core::mem::size_of::<ResourceAttachBackingHdr>();
    let entries_size = runs.len() * core::mem::size_of::<MemEntry>();
    if hdr_size + entries_size > 4096 {
        crate::kprintln!("[gpu]  RESOURCE_ATTACH_BACKING: {} mem entries too large for one request page", runs.len());
        return false;
    }
    unsafe {
        *(scratch.req_virt as *mut ResourceAttachBackingHdr) = ResourceAttachBackingHdr {
            hdr: CtrlHdr::new(CMD_RESOURCE_ATTACH_BACKING),
            resource_id: TEST_RESOURCE_ID,
            nr_entries: runs.len() as u32,
        };
        let entries_ptr = (scratch.req_virt as usize + hdr_size) as *mut MemEntry;
        for (i, &(addr, length)) in runs.iter().enumerate() {
            *entries_ptr.add(i) = MemEntry { addr, length, padding: 0 };
        }
    }
    if !expect_ok(controlq, scratch, hdr_size + entries_size, "RESOURCE_ATTACH_BACKING") {
        return false;
    }

    // ── SET_SCANOUT ───────────────────────────────────────────────────────
    unsafe {
        *(scratch.req_virt as *mut SetScanout) = SetScanout {
            hdr: CtrlHdr::new(CMD_SET_SCANOUT),
            r: full_rect,
            scanout_id: 0,
            resource_id: TEST_RESOURCE_ID,
        };
    }
    if !expect_ok(controlq, scratch, core::mem::size_of::<SetScanout>(), "SET_SCANOUT") {
        return false;
    }

    // ── TRANSFER_TO_HOST_2D ───────────────────────────────────────────────
    unsafe {
        *(scratch.req_virt as *mut TransferToHost2d) = TransferToHost2d {
            hdr: CtrlHdr::new(CMD_TRANSFER_TO_HOST_2D),
            r: full_rect,
            offset: 0,
            resource_id: TEST_RESOURCE_ID,
            padding: 0,
        };
    }
    if !expect_ok(controlq, scratch, core::mem::size_of::<TransferToHost2d>(), "TRANSFER_TO_HOST_2D") {
        return false;
    }

    // ── RESOURCE_FLUSH ────────────────────────────────────────────────────
    unsafe {
        *(scratch.req_virt as *mut ResourceFlush) = ResourceFlush {
            hdr: CtrlHdr::new(CMD_RESOURCE_FLUSH),
            r: full_rect,
            resource_id: TEST_RESOURCE_ID,
            padding: 0,
        };
    }
    expect_ok(controlq, scratch, core::mem::size_of::<ResourceFlush>(), "RESOURCE_FLUSH")
}

/// Submit the request already built in `scratch.req_virt` and confirm the
/// device replied `RESP_OK_NODATA` (logging the actual type/error otherwise).
fn expect_ok(controlq: &mut virtio_pci_modern::ModernQueue, scratch: &Scratch, req_len: usize, label: &str) -> bool {
    match controlq.send_sync(scratch.req_phys, req_len as u32, scratch.resp_phys, 4096) {
        Ok(_) => {
            let resp_type = unsafe { core::ptr::read_volatile(scratch.resp_virt as *const u32) };
            if resp_type == RESP_OK_NODATA {
                true
            } else {
                crate::kprintln!("[gpu]  {}: unexpected response type={:#06x}", label, resp_type);
                false
            }
        }
        Err(e) => {
            crate::kprintln!("[gpu]  {}: {}", label, e);
            false
        }
    }
}
