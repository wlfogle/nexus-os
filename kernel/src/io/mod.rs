//! NexusOS I/O subsystem
//!
//! All kernel print output routes through `_kprint`.
//! x86_64 → COM1 serial (port I/O, no mapping needed)
//! aarch64 → PL011 UART (MMIO at 0x09000000, QEMU virt machine)
//! laptop  → serial + framebuffer

#[cfg(target_arch = "x86_64")]
pub mod serial;

#[cfg(target_arch = "aarch64")]
pub mod uart;

#[cfg(feature = "framebuffer")]
pub mod framebuffer;

/// Initialise output as early as possible (before memory management).
pub fn init_early() {
    #[cfg(target_arch = "x86_64")]
    serial::init();

    #[cfg(target_arch = "aarch64")]
    uart::init();
}

/// Route a formatted string to all active outputs.
/// Called by `kprint!` macro.
pub fn _kprint(args: core::fmt::Arguments) {
    #[cfg(target_arch = "x86_64")]
    serial::_print(args);

    // On aarch64 the fmt::Arguments is Copy so it can be passed twice;
    // on x86_64 framebuffer gets its own copy via the same Copy guarantee.
    #[cfg(feature = "framebuffer")]
    framebuffer::_print(args);

    #[cfg(target_arch = "aarch64")]
    uart::_print(args);
}
