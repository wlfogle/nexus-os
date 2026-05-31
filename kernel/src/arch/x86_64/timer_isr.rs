//! NexusOS Timer ISR — naked function context switch
//!
//! This is the actual IRQ0 handler.  It must be `naked` because we need
//! precise control over the stack: we push all GP registers, hand RSP
//! to the scheduler, receive a (possibly different) RSP back, then pop
//! and IRETQ into whichever process the scheduler chose.
//!
//! Register save/restore order (matches `process::spawn` stack layout):
//!   push rbp, rax, rbx, rcx, rdx, rsi, rdi, r8, r9, r10, r11, r12, r13, r14, r15
//!   (15 registers × 8 bytes = 120 bytes below the CPU interrupt frame)

use core::arch::naked_asm;

/// Timer IRQ0 handler — installed at IDT vector 0x20.
#[unsafe(naked)]
pub extern "x86-interrupt" fn timer_isr_naked(
    _frame: x86_64::structures::idt::InterruptStackFrame
) {
    // SAFETY: naked function — no Rust prologue/epilogue.
    unsafe {
        naked_asm!(
            // ── Save all caller/callee-saved GP registers ─────────────────
            "push rbp",
            "push rax",
            "push rbx",
            "push rcx",
            "push rdx",
            "push rsi",
            "push rdi",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            "push r12",
            "push r13",
            "push r14",
            "push r15",

            // ── Call scheduler_tick(current_rsp) ──────────────────────────
            // The C calling convention passes the first arg in RDI.
            // RAX will hold the return value (next RSP).
            "mov rdi, rsp",         // arg1 = current RSP (after pushing regs)
            "call scheduler_tick",  // RAX = next process RSP

            // ── Switch to next process's stack ────────────────────────────
            "mov rsp, rax",

            // ── Send PIC EOI (master PIC at 0x20) ─────────────────────────
            "mov al, 0x20",
            "out 0x20, al",

            // ── Restore GP registers of the next process ──────────────────
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rdi",
            "pop rsi",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            "pop rax",
            "pop rbp",

            // ── Return to the next process (RIP/CS/RFLAGS/RSP/SS on stack)
            "iretq",
        );
    }
}
