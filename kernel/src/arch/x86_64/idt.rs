//! x86_64 Interrupt Descriptor Table
//!
//! Handles CPU exceptions.  Hardware IRQs are not yet wired (Phase 2 — PIC/APIC).

use spin::Lazy;
use x86_64::structures::idt::InterruptDescriptorTable;

use super::{gdt::DOUBLE_FAULT_IST, interrupts, timer_isr, keyboard_irq};
use crate::timer::pic::{PIC1_OFFSET, IRQ_KEYBOARD};

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // ── CPU exceptions ────────────────────────────────────────────────────────

    idt.divide_error
        .set_handler_fn(interrupts::divide_error_handler);

    idt.breakpoint
        .set_handler_fn(interrupts::breakpoint_handler);

    idt.invalid_opcode
        .set_handler_fn(interrupts::invalid_opcode_handler);

    idt.general_protection_fault
        .set_handler_fn(interrupts::general_protection_fault_handler);

    idt.page_fault
        .set_handler_fn(interrupts::page_fault_handler);

    idt.stack_segment_fault
        .set_handler_fn(interrupts::stack_segment_fault_handler);

    // Double fault uses IST so it has a guaranteed-valid stack even if the
    // normal kernel stack has overflowed or been corrupted.
    unsafe {
        idt.double_fault
            .set_handler_fn(interrupts::double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST);
    }

    idt.alignment_check
        .set_handler_fn(interrupts::alignment_check_handler);

    // ── Hardware IRQs (remapped by PIC to 0x20-0x2F) ─────────────────────────
    // IRQ0 = Timer (INT 0x20)
    idt[PIC1_OFFSET as usize]
        .set_handler_fn(timer_isr::timer_isr_naked);
    // IRQ1 = PS/2 Keyboard (INT 0x21)
    idt[(PIC1_OFFSET + IRQ_KEYBOARD) as usize]
        .set_handler_fn(keyboard_irq::keyboard_irq_handler);

    idt
});

/// Load the IDT into the IDTR register.
pub fn init() {
    IDT.load();
}
