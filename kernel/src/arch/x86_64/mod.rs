//! x86_64 CPU Initialisation

pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod keyboard_irq;
pub mod timer_isr;

/// Load GDT → load IDT → done.
/// Interrupts remain disabled until `arch::enable_interrupts()` is called.
pub fn init() {
    gdt::init();
    idt::init();
}
