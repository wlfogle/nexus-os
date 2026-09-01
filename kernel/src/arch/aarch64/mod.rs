//! AArch64 CPU Initialisation (Bahamut)

pub mod exceptions;
pub mod platform;

pub fn init() {
    exceptions::init();
}
