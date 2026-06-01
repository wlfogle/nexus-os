//! NexusOS PS/2 Keyboard IRQ1 Handler
//!
//! Installed at IDT vector 0x21 (PIC1_OFFSET + IRQ1).
//! Reads the scancode, hands it to the keyboard driver, sends EOI.

use x86_64::structures::idt::InterruptStackFrame;

/// IRQ1 handler — fires on every key press and release.
pub extern "x86-interrupt" fn keyboard_irq_handler(_frame: InterruptStackFrame) {
    // Read scancode and process it (the driver reads port 0x60 internally)
    crate::io::keyboard::handle_irq();

    // Send End-of-Interrupt to the master PIC
    crate::timer::pic::send_eoi(crate::timer::pic::IRQ_KEYBOARD);
}
