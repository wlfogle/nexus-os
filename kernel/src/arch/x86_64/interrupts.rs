//! x86_64 Exception Handlers
//!
//! All unrecoverable exceptions call panic! which halts the CPU.
//! The breakpoint exception is the only recoverable one (used by debuggers).

use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

// ─── Recoverable ─────────────────────────────────────────────────────────────

/// #BP — Breakpoint (INT3).  Recoverable: just log and return.
pub extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    crate::kprintln!(
        "[INT] Breakpoint exception at {:#x}",
        frame.instruction_pointer
    );
}

// ─── Fatal exceptions ─────────────────────────────────────────────────────────

/// #DE — Divide-by-zero.
pub extern "x86-interrupt" fn divide_error_handler(frame: InterruptStackFrame) {
    panic!(
        "[INT] #DE Divide Error\n  IP={:#x}  SP={:#x}  CS={:#x}  SS={:#x}",
        frame.instruction_pointer,
        frame.stack_pointer,
        frame.code_segment,
        frame.stack_segment,
    );
}

/// #UD — Invalid Opcode.
pub extern "x86-interrupt" fn invalid_opcode_handler(frame: InterruptStackFrame) {
    panic!(
        "[INT] #UD Invalid Opcode at {:#x}",
        frame.instruction_pointer
    );
}

/// #GP — General Protection Fault.
pub extern "x86-interrupt" fn general_protection_fault_handler(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "[INT] #GP General Protection Fault\n  error={:#x}  IP={:#x}  SP={:#x}",
        error_code,
        frame.instruction_pointer,
        frame.stack_pointer,
    );
}

/// #PF — Page Fault.
pub extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    let faulting_addr = Cr2::read();
    panic!(
        "[INT] #PF Page Fault\n  address={:#x}  error={:?}\n  IP={:#x}  SP={:#x}",
        faulting_addr,
        error_code,
        frame.instruction_pointer,
        frame.stack_pointer,
    );
}

/// #SS — Stack Segment Fault.
pub extern "x86-interrupt" fn stack_segment_fault_handler(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "[INT] #SS Stack Segment Fault\n  error={:#x}  IP={:#x}",
        error_code,
        frame.instruction_pointer,
    );
}

/// #AC — Alignment Check.
pub extern "x86-interrupt" fn alignment_check_handler(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "[INT] #AC Alignment Check\n  error={:#x}  IP={:#x}",
        error_code,
        frame.instruction_pointer,
    );
}

/// #DF — Double Fault.  This handler runs on the IST emergency stack.
/// Diverging (`-> !`) because double faults are never recoverable.
pub extern "x86-interrupt" fn double_fault_handler(
    frame: InterruptStackFrame,
    _error_code: u64,   // always 0 for double fault
) -> ! {
    panic!(
        "[INT] #DF DOUBLE FAULT (unrecoverable)\n  IP={:#x}  SP={:#x}",
        frame.instruction_pointer,
        frame.stack_pointer,
    );
}
