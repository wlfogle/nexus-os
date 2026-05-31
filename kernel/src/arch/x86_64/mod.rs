//! x86_64 CPU Initialisation

pub mod gdt;
pub mod idt;
pub mod interrupts;

/// Load GDT → load IDT → done.
/// Interrupts remain disabled until `arch::enable_interrupts()` is called.
pub fn init() {
    gdt::init();
    idt::init();
}
