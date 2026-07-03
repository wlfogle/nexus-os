//! NexusOS Timer Subsystem

#[cfg(target_arch = "x86_64")]

pub mod pic;
#[cfg(target_arch = "x86_64")]
pub mod pit;

/// Initialise hardware timer: remap PIC interrupts then start PIT at 100 Hz.
/// Call BEFORE enabling interrupts.
pub fn init() {
    #[cfg(target_arch = "x86_64")]
    {
    pic::init();
    pit::init();
    }
}
#[cfg(target_arch = "x86_64")]

pub use pit::{ticks, millis, TIMER_HZ};

#[cfg(not(target_arch = "x86_64"))]
pub const TIMER_HZ: u64 = 0;
#[cfg(not(target_arch = "x86_64"))]
pub fn ticks() -> u64 { 0 }
#[cfg(not(target_arch = "x86_64"))]
pub fn millis() -> u64 { 0 }
