//! AArch64 Exception Vector Table
//!
//! The ARM architecture requires a 2048-byte aligned vector table with 16 entries
//! of 128 bytes each.  Each entry contains a branch to the real handler.
//!
//! Table layout (each slot = 128 bytes, one `b` instruction + 31 NOPs):
//!   0x000  EL1t Sync       0x080  EL1t IRQ       0x100  EL1t FIQ       0x180  EL1t SError
//!   0x200  EL1h Sync       0x280  EL1h IRQ       0x300  EL1h FIQ       0x380  EL1h SError
//!   0x400  EL0(64) Sync    0x480  EL0(64) IRQ    0x500  EL0(64) FIQ    0x580  EL0(64) SError
//!   0x600  EL0(32) Sync    0x680  EL0(32) IRQ    0x700  EL0(32) FIQ    0x780  EL0(32) SError

use core::arch::global_asm;

// Each vector slot is 128 bytes.  We place a `b` (branch) to the actual Rust
// handler and then fill the remaining 124 bytes with `nop` instructions.
// The `.space N` GNU assembler directive fills N bytes with zeros which on
// AArch64 decodes as `udf #0` (undefined) — we use explicit NOPs instead.
macro_rules! exc_slot {
    ($label:literal) => {
        concat!("b ", $label, "\n", ".rept 31\nnop\n.endr\n")
    };
}

global_asm!(
    // Place in .exception_vectors section — linker script puts this first in
    // .text so the VBAR_EL1 address is at the start of the text segment.
    ".section .exception_vectors, \"ax\", @progbits",
    ".balign 2048",
    ".global _nexus_exception_vectors",
    "_nexus_exception_vectors:",

    // ── EL1t (uses SP_EL0) ───────────────────────────────────────────────────
    exc_slot!("_exc_el1t_sync"),
    exc_slot!("_exc_el1t_irq"),
    exc_slot!("_exc_unhandled"),    // FIQ — not used
    exc_slot!("_exc_el1_serror"),

    // ── EL1h (uses SP_EL1) — normal kernel exceptions ────────────────────────
    exc_slot!("_exc_el1h_sync"),
    exc_slot!("_exc_el1h_irq"),
    exc_slot!("_exc_unhandled"),    // FIQ — not used
    exc_slot!("_exc_el1_serror"),

    // ── EL0 AArch64 (user-space 64-bit) — Phase 2+ ──────────────────────────
    exc_slot!("_exc_el0_sync"),
    exc_slot!("_exc_el0_irq"),
    exc_slot!("_exc_unhandled"),    // FIQ
    exc_slot!("_exc_el0_serror"),

    // ── EL0 AArch32 — not supported ─────────────────────────────────────────
    exc_slot!("_exc_unhandled"),
    exc_slot!("_exc_unhandled"),
    exc_slot!("_exc_unhandled"),
    exc_slot!("_exc_unhandled"),

    // ── Handler stubs ────────────────────────────────────────────────────────
    // Each stub saves the return address (LR) on stack and calls a Rust fn.
    ".text",

    "_exc_el1t_sync:",
    "str  lr, [sp, #-16]!",
    "bl   nexus_exc_el1_sync",
    "ldr  lr, [sp], #16",
    "eret",

    "_exc_el1t_irq:",
    "_exc_el1h_irq:",
    "_exc_el0_irq:",
    "bl   nexus_exc_irq",
    "b    .",                    // IRQ should not return in Phase 1

    "_exc_el1h_sync:",
    "str  lr, [sp, #-16]!",
    "bl   nexus_exc_el1_sync",
    "ldr  lr, [sp], #16",
    "eret",

    "_exc_el1_serror:",
    "bl   nexus_exc_serror",
    "b    .",

    "_exc_el0_sync:",
    "bl   nexus_exc_el0_sync",
    "b    .",

    "_exc_el0_serror:",
    "bl   nexus_exc_serror",
    "b    .",

    "_exc_unhandled:",
    "bl   nexus_exc_unhandled",
    "b    .",
);

// ─── Rust exception handlers ──────────────────────────────────────────────────

/// Read EL1 system registers for diagnostics.
fn read_exc_info() -> (u64, u64, u64) {
    let (esr, far, elr): (u64, u64, u64);
    unsafe {
        core::arch::asm!(
            "mrs {0}, esr_el1",
            "mrs {1}, far_el1",
            "mrs {2}, elr_el1",
            out(reg) esr,
            out(reg) far,
            out(reg) elr,
            options(nomem, nostack),
        );
    }
    (esr, far, elr)
}

#[no_mangle]
pub extern "C" fn nexus_exc_el1_sync() {
    let (esr, far, elr) = read_exc_info();
    let ec = (esr >> 26) & 0x3f;  // Exception Class
    panic!(
        "[AARCH64] EL1 Synchronous Exception\n  EC={:#04x} ESR={:#010x} FAR={:#018x} ELR={:#018x}",
        ec, esr, far, elr
    );
}

#[no_mangle]
pub extern "C" fn nexus_exc_irq() {
    panic!("[AARCH64] Unexpected IRQ — GICC not yet configured (Phase 2)");
}

#[no_mangle]
pub extern "C" fn nexus_exc_serror() {
    let (esr, far, elr) = read_exc_info();
    panic!(
        "[AARCH64] SError Abort\n  ESR={:#010x} FAR={:#018x} ELR={:#018x}",
        esr, far, elr
    );
}

#[no_mangle]
pub extern "C" fn nexus_exc_el0_sync() {
    let (esr, far, elr) = read_exc_info();
    let ec = (esr >> 26) & 0x3f;
    panic!(
        "[AARCH64] EL0 Synchronous Exception (no user space yet)\n  EC={:#04x} ELR={:#018x} FAR={:#018x}",
        ec, elr, far
    );
}

#[no_mangle]
pub extern "C" fn nexus_exc_unhandled() {
    panic!("[AARCH64] Unhandled exception (FIQ or AArch32)");
}

// ─── Public interface ─────────────────────────────────────────────────────────

extern "C" {
    fn _nexus_exception_vectors();
}

/// Install the exception vector table into VBAR_EL1.
pub fn init() {
    let vbar = _nexus_exception_vectors as usize as u64;
    // Verify 2048-byte alignment
    assert_eq!(vbar & 0x7ff, 0, "Exception vector table not 2048-byte aligned");
    unsafe {
        core::arch::asm!(
            "msr vbar_el1, {0}",
            "isb",
            in(reg) vbar,
            options(nomem, nostack),
        );
    }
}
