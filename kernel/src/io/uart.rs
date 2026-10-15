//! AArch64 PL011 UART Driver — Bahamut
//!
//! The MMIO base is **not** fixed: QEMU `virt` puts PL011 at 0x09000000,
//! the BCM2711 (Pi 4) puts UART0 at 0xFE201000, and the baud divisors differ
//! too because the reference clocks differ (24 MHz vs 48 MHz). Increment B2
//! therefore configures both at runtime from
//! `arch::aarch64::platform::{uart_base, uart_divisors}` before `init()`.
//!
//! Either address is reachable during early boot: the Limine path
//! identity-maps low physical memory, and the custom boot stub's TTBR0
//! identity-maps the low 4 GiB as Device-nGnRnE. After HHDM is established,
//! `remap()` can move the base to a higher-half address.

use core::fmt;
use spin::Mutex;

// Default: QEMU virt machine PL011. Overridden by `configure()` once the
// platform is known -- kept as the default so a boot path that never calls
// `configure()` behaves exactly as it did before Increment B2.
const UART_BASE: usize = 0x0900_0000;
const DEFAULT_IBRD: u32 = 13;
const DEFAULT_FBRD: u32 = 1;

// PL011 register offsets (in bytes; each register is 32-bit)
const DR:     usize = 0x000;   // Data Register
const FR:     usize = 0x018;   // Flag Register
const IBRD:   usize = 0x024;   // Integer Baud Rate Divisor
const FBRD:   usize = 0x028;   // Fractional Baud Rate Divisor
const LCR_H:  usize = 0x02c;   // Line Control Register
const CR:     usize = 0x030;   // Control Register
const IMSC:   usize = 0x038;   // Interrupt Mask Set/Clear

// FR bits
const FR_TXFF: u32 = 1 << 5;   // Transmit FIFO full
const FR_RXFE: u32 = 1 << 4;   // Receive FIFO empty
const FR_BUSY: u32 = 1 << 3;   // UART busy

// CR bits
const CR_UARTEN: u32 = 1 << 0; // UART enable
const CR_TXE:    u32 = 1 << 8; // Transmit enable
const CR_RXE:    u32 = 1 << 9; // Receive enable

// LCR_H bits
const LCR_H_WLEN_8: u32 = 0b11 << 5;   // 8-bit word
const LCR_H_FEN:    u32 = 1 << 4;      // FIFO enable

struct Pl011Uart {
    base: usize,
    ibrd: u32,
    fbrd: u32,
}

impl Pl011Uart {
    const fn new(base: usize) -> Self {
        Self { base, ibrd: DEFAULT_IBRD, fbrd: DEFAULT_FBRD }
    }

    #[inline]
    unsafe fn read_reg(&self, offset: usize) -> u32 {
        core::ptr::read_volatile((self.base + offset) as *const u32)
    }

    #[inline]
    unsafe fn write_reg(&self, offset: usize, val: u32) {
        core::ptr::write_volatile((self.base + offset) as *mut u32, val);
    }

    /// Initialise at 115200 baud using the divisors set by `configure()`
    /// (defaults suit QEMU virt's 24 MHz clock; the BCM2711's 48 MHz clock
    /// needs different ones -- see `platform::uart_divisors`).
    unsafe fn init(&self) {
        // Disable UART
        self.write_reg(CR, 0);
        // Mask all interrupts
        self.write_reg(IMSC, 0);
        // Wait for any in-progress transmission to finish
        while self.read_reg(FR) & FR_BUSY != 0 {}
        // Baud rate divisors for this board's UART reference clock
        self.write_reg(IBRD, self.ibrd);
        self.write_reg(FBRD, self.fbrd);
        // 8-bit, FIFO enabled, 1 stop bit, no parity
        self.write_reg(LCR_H, LCR_H_WLEN_8 | LCR_H_FEN);
        // Re-enable UART with TX + RX
        self.write_reg(CR, CR_UARTEN | CR_TXE | CR_RXE);
    }

    fn write_byte(&self, byte: u8) {
        unsafe {
            while self.read_reg(FR) & FR_TXFF != 0 {}
            self.write_reg(DR, byte as u32);
        }
    }

    /// Block until a byte is received, return it.
    fn read_byte(&self) -> u8 {
        unsafe {
            while self.read_reg(FR) & FR_RXFE != 0 {
                core::hint::spin_loop();
            }
            (self.read_reg(DR) & 0xFF) as u8
        }
    }

    /// Non-blocking: return Some(byte) if available, None otherwise.
    fn try_read_byte(&self) -> Option<u8> {
        unsafe {
            if self.read_reg(FR) & FR_RXFE != 0 {
                None
            } else {
                Some((self.read_reg(DR) & 0xFF) as u8)
            }
        }
    }
}

impl fmt::Write for Pl011Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}

// ─── Singleton ───────────────────────────────────────────────────────────────

static UART: Mutex<Pl011Uart> = Mutex::new(Pl011Uart::new(UART_BASE));

/// Point the driver at this board's UART before `init()`.
///
/// Must run before any output is attempted: on a Pi 4 the default QEMU
/// `virt` base is simply the wrong device, and writes to it are silently
/// discarded -- which is exactly why Bahamut produced no serial output at
/// all prior to Increment B2.
pub fn configure(base: usize, ibrd: u32, fbrd: u32) {
    let mut uart = UART.lock();
    uart.base = base;
    uart.ibrd = ibrd;
    uart.fbrd = fbrd;
}

/// Called once during early boot.
pub fn init() {
    unsafe { UART.lock().init() };
}

/// Update UART base address after HHDM remapping (call after paging init).
/// Pass `hhdm_offset + UART_BASE_PHYS` if the physical address changes.
pub fn remap(virtual_base: usize) {
    UART.lock().base = virtual_base;
}

/// Internal: write formatted output.
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    UART.lock().write_fmt(args).expect("UART write failed");
}

/// Block until a character is received from UART RX.
pub fn read_char() -> u8 {
    UART.lock().read_byte()
}

/// Non-blocking read — returns None if RX FIFO empty.
pub fn try_read_char() -> Option<u8> {
    UART.lock().try_read_byte()
}
