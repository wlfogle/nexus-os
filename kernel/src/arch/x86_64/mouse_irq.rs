//! NexusOS PS/2 Mouse IRQ12 Handler
//!
//! Installed at IDT vector PIC2_OFFSET + (IRQ_MOUSE - 8) = 0x2C.
//! Reads one raw packet byte, hands it to the mouse driver, sends EOI.

use x86_64::structures::idt::InterruptStackFrame;

/// IRQ12 handler — fires on every PS/2 mouse packet byte.
pub extern "x86-interrupt" fn mouse_irq_handler(_frame: InterruptStackFrame) {
    crate::io::mouse::handle_irq();

    // Send End-of-Interrupt to both PICs (IRQ >= 8 needs the slave EOI too).
    crate::timer::pic::send_eoi(crate::timer::pic::IRQ_MOUSE);
}
