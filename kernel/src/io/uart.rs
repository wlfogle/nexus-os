//! AArch64 PL011 UART Driver — Bahamut
//!
//! MMIO base address 0x09000000 — matches QEMU's `virt` machine type.
//! Limine identity-maps low physical memory before calling _start, so
//! physical 0x09000000 == virtual 0x09000000 during early boot.
//! After HHDM is established, use phys_to_virt(0x09000000) if remapping.

use core::fmt;
use spin::Mutex;

// QEMU virt machine PL011 UART
const UART_BASE: usize = 0x0900_0000;

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
}

impl Pl011Uart {
    const fn new(base: usize) -> Self {
        Self { base }
    }

    #[inline]
    unsafe fn read_reg(&self, offset: usize) -> u32 {
        core::ptr::read_volatile((self.base + offset) as *const u32)
    }

    #[inline]
    unsafe fn write_reg(&self, offset: usize, val: u32) {
        core::ptr::write_volatile((self.base + offset) as *mut u32, val);
    }

    /// Initialise: 115200 baud assuming a 24 MHz UART clock (QEMU default).
    ///   IBRD = 13, FBRD = 1  →  24_000_000 / (16 × (13 + 1/64)) ≈ 115200
    unsafe fn init(&self) {
        // Disable UART
        self.write_reg(CR, 0);
        // Mask all interrupts
        self.write_reg(IMSC, 0);
        // Wait for any in-progress transmission to finish
        while self.read_reg(FR) & FR_BUSY != 0 {}
        // Baud rate: IBRD=13, FBRD=1
        self.write_reg(IBRD, 13);
        self.write_reg(FBRD, 1);
        // 8-bit, FIFO enabled, 1 stop bit, no parity
        self.write_reg(LCR_H, LCR_H_WLEN_8 | LCR_H_FEN);
        // Re-enable UART with TX + RX
        self.write_reg(CR, CR_UARTEN | CR_TXE | CR_RXE);
    }

    fn write_byte(&self, byte: u8) {
        unsafe {
            // Wait while TX FIFO is full
            while self.read_reg(FR) & FR_TXFF != 0 {}
            self.write_reg(DR, byte as u32);
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
