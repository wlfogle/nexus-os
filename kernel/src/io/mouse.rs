//! NexusOS PS/2 Mouse Driver
//!
//! Reads standard 3-byte PS/2 mouse packets from the 8042 controller's
//! auxiliary (second) port on IRQ12. Decodes relative motion + button state
//! into a small ring buffer. Mirrors `io::keyboard`'s structure; this is
//! Phase 6.3's first piece of the native GUI stack (pointer input).

use spin::Mutex;
use x86_64::instructions::port::Port;

const PS2_DATA:   u16 = 0x60;
const PS2_STATUS: u16 = 0x64;
const OBF_BIT:    u8  = 0x01; // Output Buffer Full — data ready to read
const IBF_BIT:    u8  = 0x02; // Input Buffer Full — controller not ready for a write

#[inline]
fn wait_input_clear() {
    let mut status = Port::<u8>::new(PS2_STATUS);
    let mut spins = 0u32;
    while unsafe { status.read() } & IBF_BIT != 0 && spins < 100_000 {
        spins += 1;
    }
}

#[inline]
fn wait_output_full() -> bool {
    let mut status = Port::<u8>::new(PS2_STATUS);
    let mut spins = 0u32;
    while unsafe { status.read() } & OBF_BIT == 0 {
        spins += 1;
        if spins >= 100_000 {
            return false;
        }
    }
    true
}

/// Send a command byte to the mouse (second PS/2 port) via the controller's
/// 0xD4 "write to auxiliary device" command, then consume its ACK (0xFA).
fn mouse_write(byte: u8) {
    unsafe {
        let mut cmd = Port::<u8>::new(PS2_STATUS);
        wait_input_clear();
        cmd.write(0xD4);
        wait_input_clear();
        Port::<u8>::new(PS2_DATA).write(byte);
        if wait_output_full() {
            let _ = Port::<u8>::new(PS2_DATA).read(); // ACK, discarded
        }
    }
}

/// Enable the second PS/2 port (mouse), turn on its IRQ (bit 1 of the
/// controller config byte, routed to IRQ12), and tell the mouse to start
/// streaming standard 3-byte packets. Called once at boot, after
/// `keyboard::init()` and before interrupts are enabled.
pub fn init() {
    unsafe {
        let mut cmd    = Port::<u8>::new(PS2_STATUS);
        let mut data   = Port::<u8>::new(PS2_DATA);
        let mut status = Port::<u8>::new(PS2_STATUS);

        // 1. Enable the second PS/2 port.
        wait_input_clear();
        cmd.write(0xA8);

        // 2. Enable its IRQ (bit 1) in the controller config byte.
        wait_input_clear();
        cmd.write(0x20); // "read config byte"
        let mut spins = 0u32;
        while status.read() & OBF_BIT == 0 && spins < 100_000 {
            spins += 1;
        }
        let mut cfg = data.read();
        cfg |= 1 << 1; // enable second-port (mouse) interrupt -> IRQ12
        wait_input_clear();
        cmd.write(0x60); // "write config byte"
        wait_input_clear();
        data.write(cfg);
    }

    // 3. Reset to defaults, then enable data reporting.
    mouse_write(0xF6); // set defaults
    mouse_write(0xF4); // enable data reporting
}

// ─── Packet decoding ───────────────────────────────────────────────────────

/// One decoded relative-motion + button-state update.
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub dx: i16,
    pub dy: i16,
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

impl MouseEvent {
    const fn zero() -> Self {
        Self { dx: 0, dy: 0, left: false, right: false, middle: false }
    }
}

struct PacketAssembler {
    bytes: [u8; 3],
    idx: usize,
}

impl PacketAssembler {
    const fn new() -> Self {
        Self { bytes: [0; 3], idx: 0 }
    }

    /// Feed one raw byte. Returns a decoded event once a full packet lands.
    fn feed(&mut self, b: u8) -> Option<MouseEvent> {
        // Byte 0 of every packet always has bit 3 set; use it to resync if a
        // byte was ever dropped (e.g. IRQ jitter) so we don't get stuck
        // misaligned forever.
        if self.idx == 0 && b & 0x08 == 0 {
            return None;
        }
        self.bytes[self.idx] = b;
        self.idx += 1;
        if self.idx < 3 {
            return None;
        }
        self.idx = 0;

        let flags = self.bytes[0];
        let mut dx = self.bytes[1] as i16;
        let mut dy = self.bytes[2] as i16;
        if flags & 0x10 != 0 {
            dx -= 256; // X sign bit
        }
        if flags & 0x20 != 0 {
            dy -= 256; // Y sign bit
        }
        if flags & 0xC0 != 0 {
            // X or Y overflow — the packet's motion data is unreliable.
            dx = 0;
            dy = 0;
        }

        Some(MouseEvent {
            dx,
            dy: -dy, // PS/2 reports +Y as "up"; flip so +Y means "down" on screen
            left: flags & 0x01 != 0,
            right: flags & 0x02 != 0,
            middle: flags & 0x04 != 0,
        })
    }
}

const BUF_SIZE: usize = 64;

struct EventBuf {
    buf: [MouseEvent; BUF_SIZE],
    head: usize,
    tail: usize,
    len: usize,
}

impl EventBuf {
    const fn empty() -> Self {
        Self { buf: [MouseEvent::zero(); BUF_SIZE], head: 0, tail: 0, len: 0 }
    }

    fn push(&mut self, ev: MouseEvent) {
        if self.len == BUF_SIZE {
            return; // drop on overflow
        }
        self.buf[self.tail] = ev;
        self.tail = (self.tail + 1) % BUF_SIZE;
        self.len += 1;
    }

    fn pop(&mut self) -> Option<MouseEvent> {
        if self.len == 0 {
            return None;
        }
        let ev = self.buf[self.head];
        self.head = (self.head + 1) % BUF_SIZE;
        self.len -= 1;
        Some(ev)
    }
}

static ASSEMBLER: Mutex<PacketAssembler> = Mutex::new(PacketAssembler::new());
static EVENT_BUF: Mutex<EventBuf> = Mutex::new(EventBuf::empty());

/// Called from the IRQ12 handler. Reads one raw byte and feeds the assembler;
/// pushes a decoded event to the ring buffer once a full packet lands.
pub fn handle_irq() {
    let byte: u8 = unsafe { Port::<u8>::new(PS2_DATA).read() };
    if let Some(ev) = ASSEMBLER.lock().feed(byte) {
        EVENT_BUF.lock().push(ev);
    }
}

/// Non-blocking read of the next queued mouse event, if any.
pub fn try_read() -> Option<MouseEvent> {
    EVENT_BUF.lock().pop()
}

/// Push an already-decoded event into the same ring buffer the PS/2 IRQ
/// handler feeds. Used by `drivers::usb_hid` so a USB mouse is
/// indistinguishable from a PS/2 mouse to every caller above this module.
pub fn push_event(ev: MouseEvent) {
    EVENT_BUF.lock().push(ev);
}
