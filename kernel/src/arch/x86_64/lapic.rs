//! Local APIC driver — Phase K5 increment 3.
//!
//! Replaces the 8259 PIC as the BSP's interrupt-acknowledgement mechanism
//! once an I/O APIC is also available to do the actual IRQ routing (see
//! `super::ioapic`): every hardware interrupt still lands on exactly the
//! same IDT vectors the legacy PIC path already used (0x20 timer, 0x21
//! keyboard, 0x2C mouse), but each handler must send its End-Of-Interrupt
//! to the Local APIC's MMIO register instead of the 8259's I/O ports once
//! this is active — see `is_active()`/`eoi()`.

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use x86_64::registers::model_specific::Msr;

const IA32_APIC_BASE: u32 = 0x1B;
const APIC_BASE_ENABLE: u64 = 1 << 11;

// Register offsets into the 4 KiB MMIO page (Intel SDM Vol. 3A, Table 10-1).
const REG_ID:        u64 = 0x020;
const REG_EOI:       u64 = 0x0B0;
const REG_SVR:       u64 = 0x0F0;
const REG_LVT_LINT0: u64 = 0x350;
const REG_LVT_LINT1: u64 = 0x360;

const LVT_MASKED: u32 = 1 << 16;

static LAPIC_VIRT: AtomicU64 = AtomicU64::new(0);
static ACTIVE: AtomicBool = AtomicBool::new(false);

#[inline]
unsafe fn read(virt: u64, reg: u64) -> u32 {
    core::ptr::read_volatile((virt + reg) as *const u32)
}

#[inline]
unsafe fn write(virt: u64, reg: u64, val: u32) {
    core::ptr::write_volatile((virt + reg) as *mut u32, val);
}

/// Map the Local APIC's MMIO region (address from the MADT) and enable it.
///
/// Also ensures `IA32_APIC_BASE`'s global enable bit is set (firmware
/// normally already does this, but this is cheap insurance rather than an
/// assumption), and masks the LINT0/LINT1 local interrupt pins: on real
/// hardware LINT0 is very commonly left configured by firmware as ExtINT
/// (the wire that used to carry the legacy 8259's interrupts straight into
/// the CPU before an I/O APIC existed to do the routing properly) and
/// LINT1 as NMI. Now that the I/O APIC is doing that routing instead (see
/// `super::ioapic`) and the 8259 is fully masked, leaving LINT0 configured
/// as ExtINT is a latent spurious-interrupt hazard for no benefit — masking
/// it is standard practice for exactly this reason.
pub fn init(phys_addr: u32) {
    unsafe {
        let mut base_msr = Msr::new(IA32_APIC_BASE);
        let base = base_msr.read();
        if base & APIC_BASE_ENABLE == 0 {
            base_msr.write(base | APIC_BASE_ENABLE);
        }
    }

    let virt = crate::drivers::map_mmio(phys_addr as u64, 0x1000);

    unsafe {
        // Spurious Interrupt Vector Register: bit 8 = APIC Software Enable,
        // bits 0-7 = spurious vector. Use 0xFF (an otherwise-unused vector,
        // matching every existing PIC1/PIC2_OFFSET choice's spirit of
        // staying clear of both CPU exception vectors and real IRQ
        // vectors) so a spurious interrupt can never be mistaken for one.
        write(virt, REG_SVR, 0x100 | 0xFF);
        write(virt, REG_LVT_LINT0, LVT_MASKED);
        write(virt, REG_LVT_LINT1, LVT_MASKED);
    }

    LAPIC_VIRT.store(virt, Ordering::SeqCst);
    ACTIVE.store(true, Ordering::SeqCst);

    crate::kprintln!(
        "[lapic] enabled at {:#010x} (virt {:#018x}), id={}",
        phys_addr, virt, id()
    );
}

/// Whether the Local APIC has been initialised and should be used for EOI
/// instead of the legacy 8259 PIC.
pub fn is_active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

/// This CPU's Local APIC ID (bits 24-31 of the ID register) — the
/// destination field every I/O APIC redirection entry needs in physical
/// destination mode.
pub fn id() -> u8 {
    let virt = LAPIC_VIRT.load(Ordering::Relaxed);
    if virt == 0 {
        return 0;
    }
    unsafe { (read(virt, REG_ID) >> 24) as u8 }
}

/// Signal End-Of-Interrupt to the Local APIC. Must be called instead of
/// `timer::pic::send_eoi` for any IRQ delivered while `is_active()`.
pub fn eoi() {
    let virt = LAPIC_VIRT.load(Ordering::Relaxed);
    if virt != 0 {
        unsafe { write(virt, REG_EOI, 0) };
    }
}
