//! NexusOS Architecture Abstraction
//!
//! Provides a uniform interface over x86_64 and AArch64 CPU specifics.

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

/// Initialise all CPU-specific structures: GDT/IDT (x86_64) or VBAR (AArch64).
pub fn init() {
    #[cfg(target_arch = "x86_64")]
    x86_64::init();

    #[cfg(target_arch = "aarch64")]
    aarch64::init();
}

/// Enable hardware interrupts.
#[inline]
pub fn enable_interrupts() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        // Clear DAIF.I (IRQ mask) and DAIF.F (FIQ mask)
        core::arch::asm!("msr daifclr, #3", options(nomem, nostack));
    }
}

/// Halt the CPU until the next interrupt (idle loop building block).
#[inline]
pub fn halt() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        // Wait For Interrupt — low-power idle
        core::arch::asm!("wfi", options(nomem, nostack));
    }
}
