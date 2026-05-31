//! NexusOS 8253/8254 Programmable Interval Timer Driver
//!
//! Programs channel 0 (IRQ0) to fire at TIMER_HZ.
//! The PIT input clock is 1,193,182 Hz.

use x86_64::instructions::port::Port;
use core::sync::atomic::{AtomicU64, Ordering};

const PIT_CH0:  u16 = 0x40;  // Channel 0 data port
const PIT_CMD:  u16 = 0x43;  // Mode/Command register

/// PIT input clock frequency in Hz.
const PIT_CLOCK: u32 = 1_193_182;

/// Desired timer frequency in Hz.
pub const TIMER_HZ: u32 = 100;   // 10 ms per tick

/// Global tick counter — incremented every timer interrupt.
static TICKS: AtomicU64 = AtomicU64::new(0);

/// Initialise PIT channel 0 at TIMER_HZ.
pub fn init() {
    // Divisor = PIT_CLOCK / TIMER_HZ
    let divisor = (PIT_CLOCK / TIMER_HZ) as u16;

    unsafe {
        let mut cmd:  Port<u8> = Port::new(PIT_CMD);
        let mut ch0:  Port<u8> = Port::new(PIT_CH0);

        // Channel 0, access lo/hi bytes, mode 2 (rate generator), binary
        cmd.write(0x34);

        // Send divisor low byte then high byte
        ch0.write((divisor & 0xFF) as u8);
        ch0.write((divisor >> 8) as u8);
    }
}

/// Increment tick counter.  Called by the timer interrupt handler.
#[inline]
pub fn tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

/// Return the current tick count.
#[inline]
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/// Return elapsed milliseconds since boot.
#[inline]
pub fn millis() -> u64 {
    ticks() * (1000 / TIMER_HZ as u64)
}
