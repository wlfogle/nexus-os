//! NexusOS 8259A Programmable Interrupt Controller Driver
//!
//! The BIOS maps IRQ0-15 over CPU exception vectors 0x08-0x0F / 0x70-0x77.
//! We remap them to 0x20-0x2F so they don't conflict with CPU exceptions.
//!
//! After init only IRQ0 (timer) is unmasked; all others are masked.

use x86_64::instructions::port::Port;

const MASTER_CMD:  u16 = 0x20;
const MASTER_DATA: u16 = 0x21;
const SLAVE_CMD:   u16 = 0xA0;
const SLAVE_DATA:  u16 = 0xA1;

/// Interrupt vector base for master PIC (IRQ0-7 → 0x20-0x27).
pub const PIC1_OFFSET: u8 = 0x20;
/// Interrupt vector base for slave PIC (IRQ8-15 → 0x28-0x2F).
pub const PIC2_OFFSET: u8 = 0x28;

/// IRQ numbers (relative to offset; add PIC1_OFFSET for actual INT vector).
pub const IRQ_TIMER:    u8 = 0;
pub const IRQ_KEYBOARD: u8 = 1;

// ICW1 / ICW2 / ICW3 / ICW4 constants
const ICW1_INIT:    u8 = 0x10;
const ICW1_ICW4:    u8 = 0x01;
const ICW4_8086:    u8 = 0x01;
const PIC_EOI:      u8 = 0x20;

/// Tiny I/O delay — needed between PIC init commands on real hardware.
#[inline]
unsafe fn io_wait() {
    Port::<u8>::new(0x80).write(0);
}

/// Initialise and remap both PICs.
/// On return, all IRQs are masked except IRQ0 (timer).
pub fn init() {
    unsafe {
        let mut m_cmd:  Port<u8> = Port::new(MASTER_CMD);
        let mut m_data: Port<u8> = Port::new(MASTER_DATA);
        let mut s_cmd:  Port<u8> = Port::new(SLAVE_CMD);
        let mut s_data: Port<u8> = Port::new(SLAVE_DATA);

        // ICW1 — start initialisation sequence, edge-triggered, ICW4 needed
        m_cmd.write(ICW1_INIT | ICW1_ICW4); io_wait();
        s_cmd.write(ICW1_INIT | ICW1_ICW4); io_wait();

        // ICW2 — interrupt vector offset
        m_data.write(PIC1_OFFSET); io_wait();  // master → 0x20
        s_data.write(PIC2_OFFSET); io_wait();  // slave  → 0x28

        // ICW3 — master: slave on IRQ2 (bit 2); slave: cascade identity = 2
        m_data.write(0x04); io_wait();
        s_data.write(0x02); io_wait();

        // ICW4 — 8086 mode
        m_data.write(ICW4_8086); io_wait();
        s_data.write(ICW4_8086); io_wait();

        // Mask all IRQs on master and slave …
        m_data.write(0xFF);
        s_data.write(0xFF);

        // … then unmask IRQ0 (timer) and IRQ1 (keyboard)
        unmask(IRQ_TIMER);
        unmask(IRQ_KEYBOARD);
    }
}

/// Unmask (enable) an IRQ (0–15).
pub fn unmask(irq: u8) {
    unsafe {
        if irq < 8 {
            let mut port: Port<u8> = Port::new(MASTER_DATA);
            let mask = port.read() & !(1 << irq);
            port.write(mask);
        } else {
            // Also unmask IRQ2 on master (the cascade line) so slave IRQs reach us
            let mut m: Port<u8> = Port::new(MASTER_DATA);
            let mm = m.read() & !(1 << 2);
            m.write(mm);

            let mut s: Port<u8> = Port::new(SLAVE_DATA);
            let sm = s.read() & !(1 << (irq - 8));
            s.write(sm);
        }
    }
}

/// Send End-Of-Interrupt signal.
/// Must be called at the end of every IRQ handler.
#[inline]
pub fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            Port::<u8>::new(SLAVE_CMD).write(PIC_EOI);
        }
        Port::<u8>::new(MASTER_CMD).write(PIC_EOI);
    }
}
