//! NexusOS Filesystem Layer
//!
//! Phase 5.2: FAT32 support via the `fatfs` crate (0.4 git).
//! Available on all targets; disk backend selected at compile time.

pub mod fat;
#[cfg(target_arch = "x86_64")]
pub mod vfs;
