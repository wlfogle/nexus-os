//! NexusOS AHCI (SATA) Block Device Driver
//!
//! Drives a PCI-class AHCI controller (class 0x01 / subclass 0x06 /
//! prog-if 0x01) through its memory-mapped HBA register set (ABAR = BAR5).
//!
//! Bring-up sequence (AHCI 1.3 spec):
//!   1. Enable AHCI mode (GHC.AE = 1).
//!   2. Scan the ports-implemented mask for an attached SATA disk
//!      (SSTS.DET == 3 and PxSIG == 0x0000_0101).
//!   3. Stop the port command engine, rebase its command list / received-FIS
//!      to freshly allocated DMA frames, and restart it.
//!   4. IDENTIFY DEVICE (ATA 0xEC) → model string + LBA48 sector count.
//!   5. READ/WRITE DMA EXT (ATA 0x25 / 0x35) for block transfers.
//!
//! Commands are issued in slot 0 and polled to completion, mirroring the
//! synchronous style of the VirtIO-blk driver.

use spin::Mutex;
use core::sync::atomic::{fence, Ordering};
use crate::memory::{physical, paging};
use crate::drivers;

/// Logical sector size exposed by this driver (matches VirtIO-blk).
pub const SECTOR_SIZE: usize = 512;

// ─── HBA global registers (from ABAR base) ───────────────────────────────────

const HBA_GHC: u64 = 0x04; // Global Host Control
const HBA_PI:  u64 = 0x0C; // Ports Implemented (bitmask)

const GHC_AE: u32 = 1 << 31; // AHCI Enable

// ─── Per-port register offsets (port base = 0x100 + port * 0x80) ─────────────

const PORT_BASE:   u64 = 0x100;
const PORT_STRIDE: u64 = 0x80;

const PX_CLB:  u64 = 0x00; // Command List Base (low)
const PX_CLBU: u64 = 0x04; // Command List Base (high)
const PX_FB:   u64 = 0x08; // FIS Base (low)
const PX_FBU:  u64 = 0x0C; // FIS Base (high)
const PX_IS:   u64 = 0x10; // Interrupt Status
const PX_CMD:  u64 = 0x18; // Command and Status
const PX_TFD:  u64 = 0x20; // Task File Data
const PX_SIG:  u64 = 0x24; // Signature
const PX_SSTS: u64 = 0x28; // SATA Status
const PX_SERR: u64 = 0x30; // SATA Error
const PX_CI:   u64 = 0x38; // Command Issue

// PxCMD bits
const CMD_ST:  u32 = 1 << 0;  // Start
const CMD_FRE: u32 = 1 << 4;  // FIS Receive Enable
const CMD_FR:  u32 = 1 << 14; // FIS Receive Running
const CMD_CR:  u32 = 1 << 15; // Command List Running

// PxTFD status bits
const TFD_BSY: u32 = 1 << 7;
const TFD_DRQ: u32 = 1 << 3;
const TFD_ERR: u32 = 1 << 0;

// PxIS error bit
const IS_TFES: u32 = 1 << 30; // Task File Error Status

const SIG_SATA: u32 = 0x0000_0101;

// ATA commands
const ATA_IDENTIFY:     u8 = 0xEC;
const ATA_READ_DMA_EXT: u8 = 0x25;
const ATA_WRITE_DMA_EXT: u8 = 0x35;

// ─── MMIO helpers ─────────────────────────────────────────────────────────────

#[inline]
unsafe fn r32(addr: u64) -> u32 { core::ptr::read_volatile(addr as *const u32) }
#[inline]
unsafe fn w32(addr: u64, v: u32) { core::ptr::write_volatile(addr as *mut u32, v); }

// ─── Driver state ─────────────────────────────────────────────────────────────

pub struct Ahci {
    port_base:     u64, // MMIO virt base of the active port's register block
    cmd_list_virt: u64, // 32 command headers (1 KiB)
    cmd_tbl_virt:  u64, // single command table (CFIS + PRDT)
    cmd_tbl_phys:  u64,
    data_virt:     u64, // DMA buffer for one sector
    data_phys:     u64,
    pub capacity:  u64, // capacity in 512-byte sectors
    model:         [u8; 40],
}

static AHCI: Mutex<Option<Ahci>> = Mutex::new(None);

// ─── Initialisation ───────────────────────────────────────────────────────────

/// Detect and bring up the first attached SATA disk behind an AHCI controller.
/// Returns `true` on success, logging model + capacity + sector-0 preview.
pub fn init() -> bool {
    // AHCI = base class 0x01, subclass 0x06, prog-if 0x01.
    let dev = match drivers::find_by_class(0x01, 0x06, 0x01) {
        Some(d) => d,
        None => return false,
    };

    drivers::enable_mem_and_busmaster(&dev);

    let abar_phys = drivers::read_bar_addr(&dev, 5);
    if abar_phys == 0 {
        crate::kprintln!("[ahci] controller found but ABAR (BAR5) is unmapped");
        return false;
    }
    let abar = drivers::map_mmio(abar_phys, 0x2000);

    crate::kprintln!(
        "[ahci] PCI {:02x}:{:02x}.{} vendor={:#06x} device={:#06x} ABAR={:#012x}",
        dev.bus, dev.dev, dev.func, dev.vendor_id, dev.device_id, abar_phys
    );

    match bringup(abar) {
        Ok(()) => {
            let guard = AHCI.lock();
            let a = guard.as_ref().unwrap();
            let model = core::str::from_utf8(&a.model).unwrap_or("<non-ascii>").trim_end();
            let mib = a.capacity / 2048;
            crate::kprintln!(
                "[ahci] model='{}' capacity={} MiB ({} sectors)",
                model, mib, a.capacity
            );
            true
        }
        Err(e) => {
            crate::kprintln!("[ahci] init failed: {}", e);
            false
        }
    }
}

fn bringup(abar: u64) -> Result<(), &'static str> {
    // 1. Ensure AHCI mode is enabled.
    unsafe {
        let ghc = r32(abar + HBA_GHC);
        w32(abar + HBA_GHC, ghc | GHC_AE);
    }

    // 2. Find the first implemented port with an attached SATA disk.
    let pi = unsafe { r32(abar + HBA_PI) };
    let mut found: Option<u64> = None;
    for port in 0u64..32 {
        if pi & (1 << port) == 0 {
            continue;
        }
        let pbase = abar + PORT_BASE + port * PORT_STRIDE;
        let ssts = unsafe { r32(pbase + PX_SSTS) };
        let det = ssts & 0x0F;        // device detection
        let ipm = (ssts >> 8) & 0x0F; // interface power management
        if det != 3 || ipm != 1 {
            continue; // no device / not in active state
        }
        let sig = unsafe { r32(pbase + PX_SIG) };
        if sig == SIG_SATA {
            found = Some(pbase);
            break;
        }
    }
    let port_base = found.ok_or("no attached SATA disk found")?;

    // 3. Stop the command engine before reprogramming the port.
    stop_cmd(port_base)?;

    // Allocate DMA structures (each in its own zeroed frame).
    let cmd_list_phys = alloc_zeroed_frame(); // 32 headers, 1 KiB used
    let fis_phys      = alloc_zeroed_frame(); // received FIS, 256 B used
    let cmd_tbl_phys  = alloc_zeroed_frame(); // command table
    let data_phys     = alloc_zeroed_frame(); // sector DMA buffer

    unsafe {
        w32(port_base + PX_CLB,  (cmd_list_phys & 0xFFFF_FFFF) as u32);
        w32(port_base + PX_CLBU, (cmd_list_phys >> 32) as u32);
        w32(port_base + PX_FB,   (fis_phys & 0xFFFF_FFFF) as u32);
        w32(port_base + PX_FBU,  (fis_phys >> 32) as u32);
        // Clear any latched errors / interrupts.
        w32(port_base + PX_SERR, 0xFFFF_FFFF);
        w32(port_base + PX_IS,   0xFFFF_FFFF);
    }

    // Restart the command engine.
    start_cmd(port_base)?;

    let mut a = Ahci {
        port_base,
        cmd_list_virt: paging::phys_to_virt(cmd_list_phys),
        cmd_tbl_virt:  paging::phys_to_virt(cmd_tbl_phys),
        cmd_tbl_phys,
        data_virt:     paging::phys_to_virt(data_phys),
        data_phys,
        capacity:      0,
        model:         [0u8; 40],
    };

    // 4. IDENTIFY DEVICE → model + LBA48 sector count.
    a.issue(false, ATA_IDENTIFY, 0, 0, SECTOR_SIZE)?;
    let id = a.data_virt;
    let mut model = [0u8; 40];
    unsafe {
        // Model number: words 27..=46, big-endian byte order within each word.
        for i in 0..20 {
            let word = core::ptr::read_volatile((id + (27 + i) as u64 * 2) as *const u16);
            model[i * 2]     = (word >> 8) as u8;
            model[i * 2 + 1] = (word & 0xFF) as u8;
        }
        // LBA48 total sectors: words 100..=103, little-endian word order.
        let w100 = core::ptr::read_volatile((id + 100 * 2) as *const u16) as u64;
        let w101 = core::ptr::read_volatile((id + 101 * 2) as *const u16) as u64;
        let w102 = core::ptr::read_volatile((id + 102 * 2) as *const u16) as u64;
        let w103 = core::ptr::read_volatile((id + 103 * 2) as *const u16) as u64;
        a.capacity = w100 | (w101 << 16) | (w102 << 32) | (w103 << 48);
    }
    a.model = model;

    *AHCI.lock() = Some(a);

    // Read sector 0 as a smoke test and log the first bytes.
    let mut sector0 = [0u8; SECTOR_SIZE];
    if read_sectors(0, &mut sector0).is_ok() {
        crate::kprintln!(
            "[ahci] sector 0: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
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
    let mut guard = AHCI.lock();
    let a = guard.as_mut().ok_or("AHCI: not initialised")?;
    for i in 0..count {
        a.issue(false, ATA_READ_DMA_EXT, lba + i as u64, 1, SECTOR_SIZE)?;
        unsafe {
            core::ptr::copy_nonoverlapping(
                a.data_virt as *const u8,
                buf[i * SECTOR_SIZE ..].as_mut_ptr(),
                SECTOR_SIZE);
        }
    }
    Ok(())
}

pub fn write_sectors(lba: u64, buf: &[u8]) -> Result<(), &'static str> {
    let count = buf.len() / SECTOR_SIZE;
    if count == 0 { return Ok(()); }
    let mut guard = AHCI.lock();
    let a = guard.as_mut().ok_or("AHCI: not initialised")?;
    for i in 0..count {
        unsafe {
            core::ptr::copy_nonoverlapping(
                buf[i * SECTOR_SIZE ..].as_ptr(),
                a.data_virt as *mut u8,
                SECTOR_SIZE);
        }
        a.issue(true, ATA_WRITE_DMA_EXT, lba + i as u64, 1, SECTOR_SIZE)?;
    }
    Ok(())
}

pub fn capacity() -> u64 {
    AHCI.lock().as_ref().map(|a| a.capacity).unwrap_or(0)
}

pub fn is_present() -> bool {
    AHCI.lock().is_some()
}

// ─── Command issue ────────────────────────────────────────────────────────────

impl Ahci {
    /// Issue a single ATA command in slot 0 and poll it to completion.
    /// `write` selects DMA direction; `byte_count` is the PRDT transfer size.
    fn issue(&mut self, write: bool, ata_cmd: u8, lba: u64, count: u16, byte_count: usize)
        -> Result<(), &'static str>
    {
        // Build command header slot 0.
        unsafe {
            let hdr = self.cmd_list_virt as *mut u32; // slot 0 at offset 0
            // DW0: CFL (FIS length in dwords) | W bit6 | PRDTL (1 entry) << 16
            let cfl = (core::mem::size_of::<[u32; 5]>() / 4) as u32; // 5 dwords
            let dw0 = cfl | (if write { 1 << 6 } else { 0 }) | (1u32 << 16);
            core::ptr::write_volatile(hdr.add(0), dw0);
            core::ptr::write_volatile(hdr.add(1), 0); // PRDBC
            core::ptr::write_volatile(hdr.add(2), (self.cmd_tbl_phys & 0xFFFF_FFFF) as u32);
            core::ptr::write_volatile(hdr.add(3), (self.cmd_tbl_phys >> 32) as u32);
        }

        // Zero the command table, then build the Command FIS + PRDT.
        unsafe {
            core::ptr::write_bytes(self.cmd_tbl_virt as *mut u8, 0, 256);

            let cfis = self.cmd_tbl_virt as *mut u8;
            core::ptr::write_volatile(cfis.add(0), 0x27);              // FIS type H2D
            core::ptr::write_volatile(cfis.add(1), 0x80);              // C = 1 (command)
            core::ptr::write_volatile(cfis.add(2), ata_cmd);           // command
            core::ptr::write_volatile(cfis.add(3), 0);                 // features (low)
            core::ptr::write_volatile(cfis.add(4), (lba & 0xFF) as u8);        // LBA 0
            core::ptr::write_volatile(cfis.add(5), ((lba >> 8) & 0xFF) as u8); // LBA 1
            core::ptr::write_volatile(cfis.add(6), ((lba >> 16) & 0xFF) as u8);// LBA 2
            core::ptr::write_volatile(cfis.add(7), 0x40);              // device: LBA mode
            core::ptr::write_volatile(cfis.add(8),  ((lba >> 24) & 0xFF) as u8); // LBA 3
            core::ptr::write_volatile(cfis.add(9),  ((lba >> 32) & 0xFF) as u8); // LBA 4
            core::ptr::write_volatile(cfis.add(10), ((lba >> 40) & 0xFF) as u8); // LBA 5
            core::ptr::write_volatile(cfis.add(11), 0);               // features (high)
            core::ptr::write_volatile(cfis.add(12), (count & 0xFF) as u8);       // count low
            core::ptr::write_volatile(cfis.add(13), ((count >> 8) & 0xFF) as u8);// count high

            // PRDT entry 0 begins at command-table offset 0x80.
            let prdt = (self.cmd_tbl_virt + 0x80) as *mut u32;
            core::ptr::write_volatile(prdt.add(0), (self.data_phys & 0xFFFF_FFFF) as u32);
            core::ptr::write_volatile(prdt.add(1), (self.data_phys >> 32) as u32);
            core::ptr::write_volatile(prdt.add(2), 0);
            // DW3: byte count - 1 in bits 21:0 (interrupt-on-completion not used).
            core::ptr::write_volatile(prdt.add(3), (byte_count as u32 - 1) & 0x003F_FFFF);
        }

        // Wait for the port to be idle (not BSY/DRQ).
        wait_not_busy(self.port_base)?;

        // Clear any stale interrupt status, then issue command in slot 0.
        unsafe {
            w32(self.port_base + PX_IS, 0xFFFF_FFFF);
            fence(Ordering::SeqCst);
            w32(self.port_base + PX_CI, 1);
        }

        // Poll until the slot-0 bit clears, watching for task-file errors.
        for _ in 0..100_000_000u64 {
            fence(Ordering::SeqCst);
            let ci = unsafe { r32(self.port_base + PX_CI) };
            if ci & 1 == 0 {
                let is = unsafe { r32(self.port_base + PX_IS) };
                if is & IS_TFES != 0 {
                    return Err("AHCI: task-file error");
                }
                let tfd = unsafe { r32(self.port_base + PX_TFD) };
                if tfd & TFD_ERR != 0 {
                    return Err("AHCI: device reported error");
                }
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err("AHCI: command timeout")
    }
}

// ─── Port command-engine control ─────────────────────────────────────────────

/// Stop the port's command list + FIS-receive engines and wait for them to halt.
fn stop_cmd(port_base: u64) -> Result<(), &'static str> {
    unsafe {
        let cmd = r32(port_base + PX_CMD);
        w32(port_base + PX_CMD, cmd & !(CMD_ST | CMD_FRE));
    }
    for _ in 0..10_000_000u64 {
        let cmd = unsafe { r32(port_base + PX_CMD) };
        if cmd & (CMD_CR | CMD_FR) == 0 {
            return Ok(());
        }
        core::hint::spin_loop();
    }
    Err("port command engine failed to stop")
}

/// Start the port's FIS-receive then command-list engines.
fn start_cmd(port_base: u64) -> Result<(), &'static str> {
    // Wait for the command list to be not running before starting.
    for _ in 0..10_000_000u64 {
        if unsafe { r32(port_base + PX_CMD) } & CMD_CR == 0 {
            break;
        }
        core::hint::spin_loop();
    }
    unsafe {
        let cmd = r32(port_base + PX_CMD);
        w32(port_base + PX_CMD, cmd | CMD_FRE);
        let cmd = r32(port_base + PX_CMD);
        w32(port_base + PX_CMD, cmd | CMD_ST);
    }
    Ok(())
}

/// Spin until the port reports neither BSY nor DRQ in its task-file register.
fn wait_not_busy(port_base: u64) -> Result<(), &'static str> {
    for _ in 0..100_000_000u64 {
        let tfd = unsafe { r32(port_base + PX_TFD) };
        if tfd & (TFD_BSY | TFD_DRQ) == 0 {
            return Ok(());
        }
        core::hint::spin_loop();
    }
    Err("port stuck busy")
}

/// Allocate a single physical frame and zero it through the HHDM.
fn alloc_zeroed_frame() -> u64 {
    let phys = physical::alloc_frame();
    unsafe { core::ptr::write_bytes(paging::phys_to_virt(phys) as *mut u8, 0, 4096); }
    phys
}
