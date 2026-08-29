//! I/O APIC driver — Phase K5 increment 3.
//!
//! Programs redirection table entries so hardware IRQs (timer, keyboard,
//! mouse) are delivered to the BSP's Local APIC instead of through the
//! legacy 8259 PIC. Each redirection entry targets the *same* IDT vector
//! the PIC path already used (see `timer::pic::PIC1_OFFSET`/`PIC2_OFFSET`),
//! so the IDT itself (installed once, early in boot) never needs to change
//! — only which interrupt controller is responsible for getting the
//! interrupt to the CPU in the first place.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

// Indirect register access: write the register index to IOREGSEL, then
// read/write the value through IOWIN (Intel/AMD I/O APIC spec).
const IOREGSEL: u64 = 0x00;
const IOWIN:    u64 = 0x10;

const REG_VER:      u32 = 0x01;
const REG_REDTBL0:  u32 = 0x10; // low dword of GSI 0's entry; GSI n = REG_REDTBL0 + 2*n

const RED_MASKED:      u32 = 1 << 16;
const RED_ACTIVE_LOW:  u32 = 1 << 13;
const RED_LEVEL_TRIG:  u32 = 1 << 15;

static IOAPIC_VIRT: AtomicU64 = AtomicU64::new(0);
static GSI_BASE: AtomicU32 = AtomicU32::new(0);

#[inline]
unsafe fn reg_write(virt: u64, reg: u32, val: u32) {
    core::ptr::write_volatile((virt + IOREGSEL) as *mut u32, reg);
    core::ptr::write_volatile((virt + IOWIN) as *mut u32, val);
}

#[inline]
unsafe fn reg_read(virt: u64, reg: u32) -> u32 {
    core::ptr::write_volatile((virt + IOREGSEL) as *mut u32, reg);
    core::ptr::read_volatile((virt + IOWIN) as *const u32)
}

/// Map the I/O APIC's MMIO region (address + GSI base from the MADT).
pub fn init(phys_addr: u32, gsi_base: u32) {
    let virt = crate::drivers::map_mmio(phys_addr as u64, 0x20);
    IOAPIC_VIRT.store(virt, Ordering::SeqCst);
    GSI_BASE.store(gsi_base, Ordering::SeqCst);

    let ver = unsafe { reg_read(virt, REG_VER) };
    let max_entries = ((ver >> 16) & 0xFF) + 1;
    crate::kprintln!(
        "[ioapic] enabled at {:#010x} (virt {:#018x}), gsi_base={}, {} redirection entries",
        phys_addr, virt, gsi_base, max_entries
    );
}

/// Program a redirection table entry for a Global System Interrupt.
///
/// `vector` is the IDT vector to deliver (the same vector the legacy PIC
/// path already installed a handler at), `dest_apic_id` the target CPU's
/// Local APIC ID (physical destination mode — the only mode needed with a
/// single active CPU), and `active_low`/`level_triggered` the polarity and
/// trigger mode to use — callers must pass what the MADT's Interrupt
/// Source Override actually specified (see
/// `acpi::Madt::isa_irq_polarity_trigger`) rather than assuming the ISA
/// default, since a real override changing these would silently break
/// interrupt delivery.
pub fn set_redirection(
    gsi: u32,
    vector: u8,
    dest_apic_id: u8,
    masked: bool,
    active_low: bool,
    level_triggered: bool,
) {
    let virt = IOAPIC_VIRT.load(Ordering::Relaxed);
    if virt == 0 {
        return;
    }
    let base = GSI_BASE.load(Ordering::Relaxed);
    if gsi < base {
        return;
    }
    let index = (gsi - base) * 2;
    let low_reg = REG_REDTBL0 + index;
    let high_reg = low_reg + 1;

    // High dword: bits 24-31 = destination APIC ID (physical mode).
    let high = (dest_apic_id as u32) << 24;

    // Low dword: bits 0-7 = vector; delivery mode = fixed (bits 8-10 = 0);
    // destination mode = physical (bit 11 = 0); polarity/trigger as given.
    let mut low = vector as u32;
    if masked { low |= RED_MASKED; }
    if active_low { low |= RED_ACTIVE_LOW; }
    if level_triggered { low |= RED_LEVEL_TRIG; }

    unsafe {
        // Destination first, then vector/mode/mask — if an interrupt could
        // somehow arrive mid-update it should see a masked-or-consistent
        // entry, never a valid vector paired with a stale destination.
        reg_write(virt, high_reg, high);
        reg_write(virt, low_reg, low);
    }
}
