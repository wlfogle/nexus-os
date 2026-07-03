//! NexusOS NVMe Block Device Driver
//!
//! Drives a PCI-class NVMe controller (class 0x01 / subclass 0x08 /
//! prog-if 0x02) through its memory-mapped register set (BAR0/BAR1).
//!
//! Bring-up sequence (NVMe 1.x base spec):
//!   1. Disable the controller (CC.EN = 0) and wait for CSTS.RDY = 0.
//!   2. Allocate the admin submission/completion queues and program AQA/ASQ/ACQ.
//!   3. Configure CC (queue entry sizes, NVM command set) and set CC.EN = 1.
//!   4. Wait for CSTS.RDY = 1.
//!   5. IDENTIFY controller (CNS = 1) → model string.
//!   6. IDENTIFY namespace 1 (CNS = 0) → size in LBAs + LBA data size.
//!   7. Create one I/O completion queue and one I/O submission queue.
//!
//! All commands are issued one-at-a-time and polled to completion via the
//! completion-queue phase tag, mirroring the synchronous style of the
//! VirtIO-blk driver.  A single dedicated 4 KiB DMA page backs each I/O.

use spin::Mutex;
use core::sync::atomic::{fence, Ordering};
use crate::memory::{physical, paging};
use crate::drivers;

/// Logical sector size exposed by this driver (matches VirtIO-blk).
pub const SECTOR_SIZE: usize = 512;

// ─── Controller register offsets (from MMIO BAR base) ────────────────────────

const REG_CAP:   u64 = 0x00; // u64  Controller Capabilities
const REG_CC:    u64 = 0x14; // u32  Controller Configuration
const REG_CSTS:  u64 = 0x1C; // u32  Controller Status
const REG_AQA:   u64 = 0x24; // u32  Admin Queue Attributes
const REG_ASQ:   u64 = 0x28; // u64  Admin Submission Queue base
const REG_ACQ:   u64 = 0x30; // u64  Admin Completion Queue base
const DOORBELL_BASE: u64 = 0x1000;

// CC field shifts / values
const CC_EN:      u32 = 1 << 0;
const CC_CSS_NVM: u32 = 0 << 4;   // NVM command set
const CC_MPS_4K:  u32 = 0 << 7;   // memory page size = 4 KiB (2^(12+0))
const CC_IOSQES:  u32 = 6 << 16;  // I/O SQ entry size = 2^6 = 64 bytes
const CC_IOCQES:  u32 = 4 << 20;  // I/O CQ entry size = 2^4 = 16 bytes

// CSTS bits
const CSTS_RDY: u32 = 1 << 0;

// Admin opcodes
const ADMIN_CREATE_IO_SQ: u32 = 0x01;
const ADMIN_CREATE_IO_CQ: u32 = 0x05;
const ADMIN_IDENTIFY:     u32 = 0x06;

// NVM I/O opcodes
const NVM_WRITE: u32 = 0x01;
const NVM_READ:  u32 = 0x02;

// Queue depths (entries).  Small fixed queues are sufficient for synchronous,
// single-outstanding-command I/O.
const ADMIN_Q_DEPTH: usize = 8;
const IO_Q_DEPTH:    usize = 8;
const IO_QID:        u16   = 1;

// ─── MMIO helpers ─────────────────────────────────────────────────────────────

#[inline]
unsafe fn mmio_r32(base: u64, off: u64) -> u32 {
    core::ptr::read_volatile((base + off) as *const u32)
}
#[inline]
unsafe fn mmio_w32(base: u64, off: u64, val: u32) {
    core::ptr::write_volatile((base + off) as *mut u32, val);
}
#[inline]
unsafe fn mmio_r64(base: u64, off: u64) -> u64 {
    core::ptr::read_volatile((base + off) as *const u64)
}
#[inline]
unsafe fn mmio_w64(base: u64, off: u64, val: u64) {
    core::ptr::write_volatile((base + off) as *mut u64, val);
}

// ─── A submission/completion queue pair view ─────────────────────────────────

struct Queue {
    sq_virt:   u64,    // submission queue entries (64 B each)
    cq_virt:   u64,    // completion queue entries (16 B each)
    depth:     usize,
    sq_tail:   usize,
    cq_head:   usize,
    cq_phase:  u32,    // expected phase tag (starts at 1)
    sq_db:     u64,    // submission-queue tail doorbell MMIO virt addr
    cq_db:     u64,    // completion-queue head doorbell MMIO virt addr
}

impl Queue {
    /// Submit a 64-byte (16-dword) command and poll for its completion.
    /// Returns the 15-bit status field (0 = success).
    fn submit(&mut self, cmd: &[u32; 16]) -> u16 {
        unsafe {
            // Write the command into SQ[tail].
            let entry = (self.sq_virt + (self.sq_tail as u64) * 64) as *mut u32;
            for (i, &dw) in cmd.iter().enumerate() {
                core::ptr::write_volatile(entry.add(i), dw);
            }
        }

        self.sq_tail = (self.sq_tail + 1) % self.depth;

        // Ring the submission doorbell with the new tail.
        fence(Ordering::SeqCst);
        unsafe { core::ptr::write_volatile(self.sq_db as *mut u32, self.sq_tail as u32); }

        // Poll the completion queue entry's phase tag.
        let cqe = (self.cq_virt + (self.cq_head as u64) * 16) as *const u32;
        let status;
        loop {
            fence(Ordering::SeqCst);
            let dw3 = unsafe { core::ptr::read_volatile(cqe.add(3)) };
            let phase = (dw3 >> 16) & 1;
            if phase == self.cq_phase {
                status = ((dw3 >> 17) & 0x7FFF) as u16;
                break;
            }
            core::hint::spin_loop();
        }

        // Advance the completion-queue head, flipping phase on wrap.
        self.cq_head = (self.cq_head + 1) % self.depth;
        if self.cq_head == 0 {
            self.cq_phase ^= 1;
        }
        fence(Ordering::SeqCst);
        unsafe { core::ptr::write_volatile(self.cq_db as *mut u32, self.cq_head as u32); }

        status
    }
}

// ─── Driver state ─────────────────────────────────────────────────────────────

pub struct Nvme {
    io:           Queue,
    data_phys:    u64,           // DMA buffer for a single sector transfer
    data_virt:    u64,
    nsid:         u32,
    lba_size:     usize,         // bytes per LBA reported by the namespace
    pub capacity: u64,           // capacity in 512-byte sectors
    model:        [u8; 40],
}

static NVME: Mutex<Option<Nvme>> = Mutex::new(None);

// ─── Initialisation ───────────────────────────────────────────────────────────

/// Detect and bring up the first NVMe controller.  Returns `true` if a
/// controller was initialised, logging model + capacity + sector-0 preview.
pub fn init() -> bool {
    // NVMe = base class 0x01, subclass 0x08, prog-if 0x02.
    let dev = match drivers::find_by_class(0x01, 0x08, 0x02) {
        Some(d) => d,
        None => return false,
    };

    drivers::enable_mem_and_busmaster(&dev);

    let bar_phys = drivers::read_bar_addr(&dev, 0);
    if bar_phys == 0 {
        crate::kprintln!("[nvme] controller found but BAR0 is unmapped");
        return false;
    }
    // NVMe register set fits well within a single 8 KiB MMIO window
    // (registers + admin/IO doorbells).
    let base = drivers::map_mmio(bar_phys, 0x2000);

    crate::kprintln!(
        "[nvme] PCI {:02x}:{:02x}.{} vendor={:#06x} device={:#06x} MMIO={:#012x}",
        dev.bus, dev.dev, dev.func, dev.vendor_id, dev.device_id, bar_phys
    );

    match bringup(base) {
        Ok(()) => {
            let guard = NVME.lock();
            let n = guard.as_ref().unwrap();
            let model = core::str::from_utf8(&n.model).unwrap_or("<non-ascii>").trim_end();
            let mib = n.capacity / 2048; // 512-byte sectors → MiB
            crate::kprintln!(
                "[nvme] model='{}' lba={}B capacity={} MiB ({} sectors)",
                model, n.lba_size, mib, n.capacity
            );
            true
        }
        Err(e) => {
            crate::kprintln!("[nvme] init failed: {}", e);
            false
        }
    }
}

fn bringup(base: u64) -> Result<(), &'static str> {
    let cap = unsafe { mmio_r64(base, REG_CAP) };
    let dstrd = ((cap >> 32) & 0xF) as u64;
    let db_stride = 4u64 << dstrd; // bytes between consecutive doorbells

    // 1. Disable the controller and wait for it to quiesce.
    unsafe {
        let cc = mmio_r32(base, REG_CC);
        mmio_w32(base, REG_CC, cc & !CC_EN);
    }
    wait_ready(base, false)?;

    // 2. Allocate admin queues (one zeroed frame each).
    let asq_phys = alloc_zeroed_frame();
    let acq_phys = alloc_zeroed_frame();

    // Program AQA (0-based sizes) and the admin queue base addresses.
    let aqa = (((ADMIN_Q_DEPTH - 1) as u32) << 16) | ((ADMIN_Q_DEPTH - 1) as u32);
    unsafe {
        mmio_w32(base, REG_AQA, aqa);
        mmio_w64(base, REG_ASQ, asq_phys);
        mmio_w64(base, REG_ACQ, acq_phys);
    }

    let mut admin = Queue {
        sq_virt:  paging::phys_to_virt(asq_phys),
        cq_virt:  paging::phys_to_virt(acq_phys),
        depth:    ADMIN_Q_DEPTH,
        sq_tail:  0,
        cq_head:  0,
        cq_phase: 1,
        sq_db:    base + DOORBELL_BASE,                 // SQ0TDBL
        cq_db:    base + DOORBELL_BASE + db_stride,     // CQ0HDBL
    };

    // 3. Configure + enable the controller.
    let cc = CC_EN | CC_CSS_NVM | CC_MPS_4K | CC_IOSQES | CC_IOCQES;
    unsafe { mmio_w32(base, REG_CC, cc); }
    wait_ready(base, true)?;

    // 5. IDENTIFY controller (CNS = 1) → model string.
    let id_phys = alloc_zeroed_frame();
    let id_virt = paging::phys_to_virt(id_phys);
    let mut cmd = identify_cmd(0, 1, id_phys); // nsid ignored for CNS=1
    if admin.submit(&cmd) != 0 {
        return Err("IDENTIFY controller failed");
    }
    let mut model = [0u8; 40];
    unsafe {
        core::ptr::copy_nonoverlapping((id_virt + 24) as *const u8, model.as_mut_ptr(), 40);
    }

    // 6. IDENTIFY namespace 1 (CNS = 0) → size + LBA format.
    let nsid = 1u32;
    unsafe { core::ptr::write_bytes(id_virt as *mut u8, 0, 4096); }
    cmd = identify_cmd(nsid, 0, id_phys);
    if admin.submit(&cmd) != 0 {
        return Err("IDENTIFY namespace failed");
    }
    let (nsze, lba_size) = unsafe {
        let nsze = core::ptr::read_volatile(id_virt as *const u64);
        let flbas = core::ptr::read_volatile((id_virt + 26) as *const u8) & 0x0F;
        // LBA format list begins at byte 128; each entry is 4 bytes,
        // LBADS (log2 of data size) is bits 23:16.
        let lbaf = core::ptr::read_volatile((id_virt + 128 + (flbas as u64) * 4) as *const u32);
        let lbads = ((lbaf >> 16) & 0xFF) as u32;
        (nsze, 1usize << lbads)
    };
    if lba_size == 0 {
        return Err("namespace reports zero LBA size");
    }
    let capacity = nsze * (lba_size as u64) / (SECTOR_SIZE as u64);

    // 7. Create the I/O completion queue, then the I/O submission queue.
    let iocq_phys = alloc_zeroed_frame();
    let iosq_phys = alloc_zeroed_frame();

    // Create I/O CQ: CDW10 = (size-1)<<16 | QID ; CDW11 = PC(bit0)
    let mut c = blank_cmd(ADMIN_CREATE_IO_CQ, 0);
    c[6] = (iocq_phys & 0xFFFF_FFFF) as u32;     // PRP1 low
    c[7] = (iocq_phys >> 32) as u32;             // PRP1 high
    c[10] = (((IO_Q_DEPTH - 1) as u32) << 16) | IO_QID as u32;
    c[11] = 1; // PC = 1 (physically contiguous), interrupts disabled
    if admin.submit(&c) != 0 {
        return Err("create I/O CQ failed");
    }

    // Create I/O SQ: CDW10 = (size-1)<<16 | QID ; CDW11 = CQID<<16 | PC(bit0)
    let mut c = blank_cmd(ADMIN_CREATE_IO_SQ, 0);
    c[6] = (iosq_phys & 0xFFFF_FFFF) as u32;
    c[7] = (iosq_phys >> 32) as u32;
    c[10] = (((IO_Q_DEPTH - 1) as u32) << 16) | IO_QID as u32;
    c[11] = ((IO_QID as u32) << 16) | 1; // CQID + PC
    if admin.submit(&c) != 0 {
        return Err("create I/O SQ failed");
    }

    let io = Queue {
        sq_virt:  paging::phys_to_virt(iosq_phys),
        cq_virt:  paging::phys_to_virt(iocq_phys),
        depth:    IO_Q_DEPTH,
        sq_tail:  0,
        cq_head:  0,
        cq_phase: 1,
        // Doorbell index for queue y: SQyTDBL at 2y, CQyHDBL at 2y+1.
        sq_db:    base + DOORBELL_BASE + (2 * IO_QID as u64) * db_stride,
        cq_db:    base + DOORBELL_BASE + (2 * IO_QID as u64 + 1) * db_stride,
    };

    let data_phys = alloc_zeroed_frame();

    // The admin queue is only needed during bring-up; once the I/O queues
    // exist all block transfers use `io`, so it is intentionally not retained.
    *NVME.lock() = Some(Nvme {
        io,
        data_phys,
        data_virt: paging::phys_to_virt(data_phys),
        nsid,
        lba_size,
        capacity,
        model,
    });

    // Read logical block 0 as a smoke test and log the first bytes.
    let mut sector0 = [0u8; SECTOR_SIZE];
    if read_sectors(0, &mut sector0).is_ok() {
        crate::kprintln!(
            "[nvme] sector 0: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            sector0[0], sector0[1], sector0[2], sector0[3],
            sector0[4], sector0[5], sector0[6], sector0[7]
        );
    }

    Ok(())
}

// ─── Public block I/O ────────────────────────────────────────────────────────

pub fn read_sectors(lba: u64, buf: &mut [u8]) -> Result<(), &'static str> {
    let count = buf.len() / SECTOR_SIZE;
    if count == 0 { return Ok(()); }
    let mut guard = NVME.lock();
    let n = guard.as_mut().ok_or("NVMe: not initialised")?;
    for i in 0..count {
        n.do_io(NVM_READ, lba + i as u64,
                &mut buf[i * SECTOR_SIZE .. (i + 1) * SECTOR_SIZE])?;
    }
    Ok(())
}

pub fn write_sectors(lba: u64, buf: &[u8]) -> Result<(), &'static str> {
    let count = buf.len() / SECTOR_SIZE;
    if count == 0 { return Ok(()); }
    let mut guard = NVME.lock();
    let n = guard.as_mut().ok_or("NVMe: not initialised")?;
    let mut tmp = [0u8; SECTOR_SIZE];
    for i in 0..count {
        tmp.copy_from_slice(&buf[i * SECTOR_SIZE .. (i + 1) * SECTOR_SIZE]);
        n.do_io(NVM_WRITE, lba + i as u64, &mut tmp)?;
    }
    Ok(())
}

pub fn capacity() -> u64 {
    NVME.lock().as_ref().map(|n| n.capacity).unwrap_or(0)
}

pub fn is_present() -> bool {
    NVME.lock().is_some()
}

// ─── Internal single-sector I/O ──────────────────────────────────────────────

impl Nvme {
    fn do_io(&mut self, opcode: u32, lba: u64, data: &mut [u8])
        -> Result<(), &'static str>
    {
        debug_assert_eq!(data.len(), SECTOR_SIZE);
        if self.lba_size != SECTOR_SIZE {
            return Err("NVMe: namespace LBA size != 512");
        }

        // On write, stage the caller's data into the DMA buffer first.
        if opcode == NVM_WRITE {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr(), self.data_virt as *mut u8, SECTOR_SIZE);
            }
        }

        let mut c = blank_cmd(opcode, self.nsid);
        c[6] = (self.data_phys & 0xFFFF_FFFF) as u32; // PRP1 low
        c[7] = (self.data_phys >> 32) as u32;         // PRP1 high
        c[10] = (lba & 0xFFFF_FFFF) as u32;           // starting LBA low
        c[11] = (lba >> 32) as u32;                   // starting LBA high
        c[12] = 0;                                    // NLB = 0 → one block

        fence(Ordering::SeqCst);
        let status = self.io.submit(&c);
        if status != 0 {
            return Err("NVMe: I/O command error");
        }

        if opcode == NVM_READ {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.data_virt as *const u8, data.as_mut_ptr(), SECTOR_SIZE);
            }
        }
        Ok(())
    }
}

// ─── Command builders / helpers ──────────────────────────────────────────────

/// Build an empty 16-dword command with the given opcode and namespace ID.
/// A fixed command identifier of 0 is fine for single-outstanding I/O.
fn blank_cmd(opcode: u32, nsid: u32) -> [u32; 16] {
    let mut c = [0u32; 16];
    c[0] = opcode & 0xFF;  // CID = 0 in bits 31:16
    c[1] = nsid;
    c
}

/// Build an IDENTIFY command: `cns` selects controller (1) vs namespace (0).
fn identify_cmd(nsid: u32, cns: u32, prp1: u64) -> [u32; 16] {
    let mut c = blank_cmd(ADMIN_IDENTIFY, nsid);
    c[6] = (prp1 & 0xFFFF_FFFF) as u32;
    c[7] = (prp1 >> 32) as u32;
    c[10] = cns & 0xFF; // CDW10: CNS in bits 7:0
    c
}

/// Allocate a single physical frame and zero it through the HHDM.
fn alloc_zeroed_frame() -> u64 {
    let phys = physical::alloc_frame();
    unsafe { core::ptr::write_bytes(paging::phys_to_virt(phys) as *mut u8, 0, 4096); }
    phys
}

/// Spin until CSTS.RDY matches `want` (true = ready, false = not ready).
fn wait_ready(base: u64, want: bool) -> Result<(), &'static str> {
    // Bounded spin so a wedged controller can't hang boot forever.
    for _ in 0..100_000_000u64 {
        let rdy = unsafe { mmio_r32(base, REG_CSTS) } & CSTS_RDY != 0;
        if rdy == want {
            return Ok(());
        }
        core::hint::spin_loop();
    }
    Err("controller readiness timeout")
}
