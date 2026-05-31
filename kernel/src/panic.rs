//! NexusOS Kernel Panic Handler
//!
//! On panic: disable interrupts, print location + message, halt forever.

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Disable interrupts immediately — we don't want anything pre-empting
    // the panic output.
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("msr daifset, #0xf", options(nomem, nostack));
    }

    crate::kprintln!();
    crate::kprintln!("!!! KERNEL PANIC !!!");
    crate::kprintln!("───────────────────────────────────────────");

    if let Some(location) = info.location() {
        crate::kprintln!("  Location : {}:{}:{}",
            location.file(),
            location.line(),
            location.column());
    }

    // Display the panic message.  PanicInfo implements Display.
    crate::kprintln!("  Message  : {}", info);

    crate::kprintln!("───────────────────────────────────────────");
    crate::kprintln!("System halted.");

    // Halt forever — no recovery.
    loop {
        crate::arch::halt();
    }
}
