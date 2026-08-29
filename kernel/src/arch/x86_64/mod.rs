//! x86_64 CPU Initialisation

pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod ioapic;
pub mod keyboard_irq;
pub mod lapic;
pub mod mouse_irq;
pub mod timer_isr;

/// Maximum number of logical CPUs this kernel's per-core arrays (GDT/TSS in
/// `gdt`, `PerCpu` in `syscall`) can hold. Sized for the real i9-13900HX
/// laptop target's logical processor count; QEMU test boots use far fewer
/// (`-smp N`). Purely a static-array bound — unused slots cost BSS, not
/// binary size or runtime work.
pub const MAX_CPUS: usize = 32;

/// Load this core's GDT/TSS → load the (shared, core-independent) IDT →
/// done. `core_id` identifies which per-core slot this call owns (0 = BSP,
/// the only caller until Phase K5 increment 5 brings up APs).
/// Interrupts remain disabled until `arch::enable_interrupts()` is called.
pub fn init(core_id: usize) {
    gdt::init(core_id);
    idt::init();
}
