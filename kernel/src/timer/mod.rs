//! NexusOS Timer Subsystem (PIC + PIT)

pub mod pic;
pub mod pit;

/// Initialise hardware timer: remap PIC interrupts then start PIT at 100 Hz.
/// Call BEFORE enabling interrupts.
pub fn init() {
    pic::init();
    pit::init();
}

pub use pit::{ticks, millis, TIMER_HZ};
