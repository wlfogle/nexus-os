//! AArch64 CPU Initialisation (Bahamut)

pub mod exceptions;

pub fn init() {
    exceptions::init();
}
