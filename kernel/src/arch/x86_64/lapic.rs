//! Local APIC driver — Phase K5 increment 3.
//!
//! Replaces the 8259 PIC as the BSP's interrupt-acknowledgement mechanism
//! once an I/O APIC is also available to do the actual IRQ routing (see
//! `super::ioapic`): every hardware interrupt still lands on exactly the
//! same IDT vectors the legacy PIC path already used (0x20 timer, 0x21
//! keyboard, 0x2C mouse), but each handler must send its End-Of-Interrupt
//! to the Local APIC's MMIO register instead of the 8259's I/O ports once
//! this is active — see `is_active()`/`eoi()`.

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use x86_64::registers::model_specific::Msr;

use super::MAX_CPUS;

const IA32_APIC_BASE: u32 = 0x1B;
const APIC_BASE_ENABLE: u64 = 1 << 11;

// Register offsets into the 4 KiB MMIO page (Intel SDM Vol. 3A, Table 10-1).
const REG_ID:            u64 = 0x020;
const REG_EOI:           u64 = 0x0B0;
const REG_SVR:           u64 = 0x0F0;
const REG_LVT_LINT0:     u64 = 0x350;
const REG_LVT_LINT1:     u64 = 0x360;
const REG_LVT_TIMER:     u64 = 0x320;
const REG_INITIAL_COUNT: u64 = 0x380;
const REG_CURRENT_COUNT: u64 = 0x390;
const REG_DIVIDE_CONFIG: u64 = 0x3E0;

const LVT_MASKED: u32 = 1 << 16;
const LVT_TIMER_PERIODIC: u32 = 1 << 17;

/// Divide the LAPIC timer's input clock by 16 before counting down — an
/// arbitrary but common choice (matches what most minimal kernels use)
/// that keeps the raw count comfortably away from both the 32-bit overflow
/// ceiling during calibration and single-digit counts that would make the
/// calibration measurement imprecise.
const DIVIDE_BY_16: u32 = 0b0011;

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

// ─── core_id ↔ hardware LAPIC ID registry (Phase K5 increment 4) ─────────
//
// `syscall::current_core_id()` is the fast, GS-relative, allocation-free
// dense array index used to pick per-core storage slots. This registry
// links that index back to the raw hardware identity (`id()` above) that
// IOAPIC redirection destinations and cross-checks actually need, so code
// can go either direction without re-deriving one from the other. Only the
// owning core ever writes its own slot (BSP writes index 0 here; each AP
// will write its own index in Phase K5 increment 5), so this needs no lock.
static mut CPU_LAPIC_IDS: [Option<u8>; MAX_CPUS] = [None; MAX_CPUS];

/// Record that dense core index `core_id` is running on the hardware LAPIC
/// ID this function reads right now. Must be called by that core itself,
/// after `init()`.
pub fn register_core(core_id: usize) {
    unsafe { CPU_LAPIC_IDS[core_id] = Some(id()); }
}

/// The hardware LAPIC ID registered for a given dense core index, if any
/// core has called `register_core(core_id)` yet.
pub fn lapic_id_for_core(core_id: usize) -> Option<u8> {
    unsafe { CPU_LAPIC_IDS[core_id] }
}

// ─── LAPIC timer (Phase K5 increment 6) ──────────────────────────────────
//
// Gives each AP a real, periodic interrupt source of its own so it can
// actually call `scheduler_tick` instead of parking forever — the I/O APIC
// only ever routes the PIT-driven timer IRQ to the BSP's LAPIC id (see
// main.rs), so an AP has no other way to receive a timer interrupt.

/// Calibrated LAPIC timer Initial Count for one Phase-2 scheduler tick
/// period (`timer::TIMER_HZ`, currently 10 ms). `0` means "not yet
/// calibrated" — every AP's `start_periodic_timer` waits for this to
/// become nonzero before touching its own timer registers.
///
/// A single BSP-side calibration (see `calibrate_timer`) is reused by
/// every AP rather than each core calibrating independently: the LAPIC
/// timer runs off each core's local bus/crystal clock, which is shared
/// across every core in the same package on all hardware this kernel
/// targets, so per-core calibration would measure the same value anyway
/// at the cost of a slower, more complex AP bring-up.
static TIMER_INITIAL_COUNT: AtomicU32 = AtomicU32::new(0);

/// Calibrate the LAPIC timer against the PIT (`crate::timer`), the only
/// trusted wall-clock source available this early in boot. Must be called
/// on the BSP, after `init()`, and after `arch::enable_interrupts()` has
/// actually started the PIT tick counter advancing — `timer::ticks()`
/// never moves while interrupts are disabled, so calling this any earlier
/// would spin forever waiting for a clock that isn't running yet.
pub fn calibrate_timer() {
    let virt = LAPIC_VIRT.load(Ordering::Relaxed);
    if virt == 0 {
        return;
    }

    // 5 PIT ticks at TIMER_HZ=100 = 50 ms — long enough for a reasonably
    // precise measurement, short enough not to noticeably delay boot.
    const MEASURE_TICKS: u64 = 5;

    unsafe {
        write(virt, REG_DIVIDE_CONFIG, DIVIDE_BY_16);
        // Masked one-shot: we poll Current Count ourselves rather than
        // waiting for this measurement countdown to actually interrupt.
        write(virt, REG_LVT_TIMER, LVT_MASKED);
        write(virt, REG_INITIAL_COUNT, u32::MAX);
    }

    let start = crate::timer::ticks();
    while crate::timer::ticks().wrapping_sub(start) < MEASURE_TICKS {
        core::hint::spin_loop();
    }

    let remaining = unsafe { read(virt, REG_CURRENT_COUNT) };
    let elapsed = u32::MAX - remaining;
    let per_tick = (elapsed / MEASURE_TICKS as u32).max(1);

    // Stop the one-shot countdown; `start_periodic_timer` reprograms this
    // register (and puts the LVT entry into periodic mode) once each core
    // is ready to actually start ticking.
    unsafe { write(virt, REG_INITIAL_COUNT, 0); }

    TIMER_INITIAL_COUNT.store(per_tick, Ordering::SeqCst);
    crate::kprintln!(
        "[lapic] timer calibrated: {} counts per {}ms tick (divide-by-16)",
        per_tick, 1000 / crate::timer::TIMER_HZ
    );
}

/// Whether `calibrate_timer()` has finished. Every AP's own timer bring-up
/// waits on this rather than assuming any particular ordering between
/// "BSP calibrates" and "AP wants to start its own timer".
pub fn timer_calibrated() -> bool {
    TIMER_INITIAL_COUNT.load(Ordering::Relaxed) != 0
}

/// Start this core's own LAPIC timer in periodic mode, delivering `vector`
/// once per calibrated tick period forever. Must be called after this
/// core's own `init()` and after `timer_calibrated()` is true.
pub fn start_periodic_timer(vector: u8) {
    let virt = LAPIC_VIRT.load(Ordering::Relaxed);
    if virt == 0 {
        return;
    }
    let count = TIMER_INITIAL_COUNT.load(Ordering::SeqCst);
    unsafe {
        write(virt, REG_DIVIDE_CONFIG, DIVIDE_BY_16);
        write(virt, REG_LVT_TIMER, LVT_TIMER_PERIODIC | vector as u32);
        write(virt, REG_INITIAL_COUNT, count);
    }
}
