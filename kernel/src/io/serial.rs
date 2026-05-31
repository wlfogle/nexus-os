//! x86_64 COM1 Serial Driver (0x3F8)
//!
//! Uses direct port I/O — works immediately after power-on with no memory
//! mapping required.  Baud: 38400, 8N1.

use core::fmt;
use spin::Mutex;
use x86_64::instructions::port::Port;

const COM1_BASE: u16 = 0x3F8;

struct SerialPort {
    data:           Port<u8>,   // +0 Data register / DLAB divisor low
    int_enable:     Port<u8>,   // +1 Interrupt enable / DLAB divisor high
    fifo_ctrl:      Port<u8>,   // +2 FIFO control
    line_ctrl:      Port<u8>,   // +3 Line control (DLAB bit)
    modem_ctrl:     Port<u8>,   // +4 Modem control
    line_status:    Port<u8>,   // +5 Line status (read-only)
}

impl SerialPort {
    const fn new(base: u16) -> Self {
        Self {
            data:           Port::new(base),
            int_enable:     Port::new(base + 1),
            fifo_ctrl:      Port::new(base + 2),
            line_ctrl:      Port::new(base + 3),
            modem_ctrl:     Port::new(base + 4),
            line_status:    Port::new(base + 5),
        }
    }

    /// Initialise the UART: 38400 baud, 8 data bits, no parity, 1 stop bit.
    unsafe fn init(&mut self) {
        // Disable interrupts
        self.int_enable.write(0x00);
        // Set DLAB to configure baud rate
        self.line_ctrl.write(0x80);
        // Divisor = 3 → 115200/3 = 38400 baud
        self.data.write(0x03);        // low byte
        self.int_enable.write(0x00);  // high byte
        // 8 bits, no parity, 1 stop bit (DLAB cleared)
        self.line_ctrl.write(0x03);
        // Enable FIFO, clear TX/RX, 14-byte threshold
        self.fifo_ctrl.write(0xC7);
        // RTS + DTR asserted
        self.modem_ctrl.write(0x0B);
    }

    /// Block until transmit holding register is empty, then write one byte.
    fn write_byte(&mut self, byte: u8) {
        // Wait for Transmit Holding Register Empty (bit 5 of line status)
        loop {
            let status = unsafe { self.line_status.read() };
            if status & 0x20 != 0 {
                break;
            }
        }
        unsafe { self.data.write(byte) };
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            // Convert bare LF → CRLF for terminal compatibility
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}

// ─── Singleton ───────────────────────────────────────────────────────────────

static SERIAL: Mutex<SerialPort> = Mutex::new(SerialPort::new(COM1_BASE));

/// Called once during early boot.
pub fn init() {
    unsafe { SERIAL.lock().init() };
}

/// Internal: write formatted output to COM1.
/// Called only via `kprint!` → `io::_kprint`.
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    SERIAL.lock().write_fmt(args).expect("serial write failed");
}
