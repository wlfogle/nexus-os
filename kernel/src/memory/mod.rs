//! NexusOS Memory Management
//!
//! Initialisation order (enforced by main.rs):
//!   1. physical::init()  — bitmap frame allocator from Limine memory map
//!   2. paging::init()    — page table setup, HHDM verification
//!   3. heap::init()      — kernel heap mapped + registered as global allocator

pub mod physical;
pub mod paging;
pub mod heap;
