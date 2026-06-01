//! NexusOS PS/2 Keyboard Driver
//!
//! Reads scan codes from the 8042 PS/2 controller (port 0x60) on IRQ1.
//! Translates scan-code set 1 to ASCII and stores in a ring buffer.
//! Processes blocked on SYS_READ_CHAR are woken when a key arrives.

use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};
use x86_64::instructions::port::Port;

// ─── PS/2 I/O ports ──────────────────────────────────────────────────────────

const PS2_DATA:   u16 = 0x60;
const PS2_STATUS: u16 = 0x64;
const OBF_BIT:    u8  = 0x01; // Output Buffer Full — data ready to read

// ─── Modifier state ───────────────────────────────────────────────────────────

static SHIFT:     AtomicBool = AtomicBool::new(false);
static CTRL:      AtomicBool = AtomicBool::new(false);
static ALT:       AtomicBool = AtomicBool::new(false);
static CAPS_LOCK: AtomicBool = AtomicBool::new(false);

// ─── Key ring buffer ──────────────────────────────────────────────────────────

const BUF_SIZE: usize = 256;

struct KeyBuf {
    buf:  [u8; BUF_SIZE],
    head: usize,
    tail: usize,
    len:  usize,
}

impl KeyBuf {
    const fn empty() -> Self {
        Self { buf: [0u8; BUF_SIZE], head: 0, tail: 0, len: 0 }
    }

    fn push(&mut self, ch: u8) {
        if self.len == BUF_SIZE { return; } // drop on overflow
        self.buf[self.tail] = ch;
        self.tail = (self.tail + 1) % BUF_SIZE;
        self.len += 1;
    }

    fn pop(&mut self) -> Option<u8> {
        if self.len == 0 { return None; }
        let ch = self.buf[self.head];
        self.head = (self.head + 1) % BUF_SIZE;
        self.len -= 1;
        Some(ch)
    }

    fn is_empty(&self) -> bool { self.len == 0 }
}

static KEY_BUF: Mutex<KeyBuf> = Mutex::new(KeyBuf::empty());

// ─── Scancode set 1 → ASCII tables ───────────────────────────────────────────
//
// Index = scancode (0x00–0x58).
// 0x00 = unmapped.  Special values: \x08 = Backspace, \x09 = Tab,
//   \x0A = Enter, \x1B = Esc.

#[rustfmt::skip]
static NORMAL: [u8; 84] = [
//  0     1     2     3     4     5     6     7     8     9     A     B     C     D     E     F
    0,    27,   b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', 8,    9,    // 0x00
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', 10,   0,    b'a', b's', // 0x10
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'',b'`', 0,    b'\\',b'z', b'x', b'c', b'v', // 0x20
    b'b', b'n', b'm', b',', b'.', b'/', 0,    b'*', 0,    b' ', 0,    0,    0,    0,    0,    0,    // 0x30
    0,    0,    0,    0,    0,    0,    0,    b'7', b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1', // 0x40
    b'2', b'3', b'0', b'.',                                                                         // 0x50-0x53 (numpad)
];

#[rustfmt::skip]
static SHIFTED: [u8; 84] = [
    0,    27,   b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'_', b'+', 8,    9,
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P', b'{', b'}', 10,   0,    b'A', b'S',
    b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':', b'"', b'~', 0,    b'|', b'Z', b'X', b'C', b'V',
    b'B', b'N', b'M', b'<', b'>', b'?', 0,    b'*', 0,    b' ', 0,    0,    0,    0,    0,    0,
    0,    0,    0,    0,    0,    0,    0,    b'7', b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1',
    b'2', b'3', b'0', b'.',
];

// ─── Public interface ─────────────────────────────────────────────────────────

/// Called from the IRQ1 handler.  Reads the scancode, translates, buffers.
pub fn handle_irq() {
    let scancode: u8 = unsafe { Port::<u8>::new(PS2_DATA).read() };
    process_scancode(scancode);
}

/// Process a raw scancode (also usable for testing / software injection).
pub fn process_scancode(sc: u8) {
    match sc {
        // ── Key releases (bit 7 set) ─────────────────────────────────────────
        0xAA | 0xB6 => SHIFT.store(false, Ordering::Relaxed), // L/R Shift up
        0x9D         => CTRL.store(false,  Ordering::Relaxed), // Ctrl up
        0xB8         => ALT.store(false,   Ordering::Relaxed), // Alt up
        0x80..=0xFF  => { /* other releases — ignore */ }

        // ── Modifier key presses ─────────────────────────────────────────────
        0x2A | 0x36  => SHIFT.store(true,  Ordering::Relaxed), // L/R Shift
        0x1D          => CTRL.store(true,   Ordering::Relaxed), // L Ctrl
        0x38          => ALT.store(true,    Ordering::Relaxed), // L Alt
        0x3A          => {                                        // Caps Lock
            let caps = CAPS_LOCK.load(Ordering::Relaxed);
            CAPS_LOCK.store(!caps, Ordering::Relaxed);
        }

        // ── Printable / control keys ─────────────────────────────────────────
        sc => {
            let idx = sc as usize;
            if idx >= NORMAL.len() { return; }

            let shift   = SHIFT.load(Ordering::Relaxed);
            let caps    = CAPS_LOCK.load(Ordering::Relaxed);
            let ctrl    = CTRL.load(Ordering::Relaxed);

            let mut ch = if shift { SHIFTED[idx] } else { NORMAL[idx] };
            if ch == 0 { return; } // unmapped

            // Caps Lock flips the case of letters
            if caps && ch.is_ascii_alphabetic() {
                ch = if shift { ch.to_ascii_lowercase() }
                     else     { ch.to_ascii_uppercase() };
            }

            // Ctrl+key → control character (Ctrl+C = 3, Ctrl+D = 4, etc.)
            if ctrl && ch.is_ascii_alphabetic() {
                ch = ch.to_ascii_uppercase() - b'@'; // 'A'-'@'=1 … 'Z'-'@'=26
            }

            // Push to ring buffer
            KEY_BUF.lock().push(ch);

            // Wake any process blocked waiting for keyboard input
            wake_blocked_on_key();
        }
    }
}

/// Non-blocking read.  Returns `None` if no key is queued.
pub fn try_read() -> Option<u8> {
    KEY_BUF.lock().pop()
}

/// Returns `true` if at least one key is waiting.
pub fn has_key() -> bool {
    !KEY_BUF.lock().is_empty()
}

// ─── Wake processes blocked on keyboard ──────────────────────────────────────

fn wake_blocked_on_key() {
    use crate::process::{self, ProcessState};

    // Scan for any process waiting on keyboard input and make it Ready.
    let mut ids = [0u64; 64];
    let n = {
        // Borrow the table to find all BlockedOnKey processes.
        let table = process::blocked_on_key_ids(&mut ids);
        table
    };
    for &pid in &ids[..n] {
        process::set_state(pid, ProcessState::Ready);
    }
}
