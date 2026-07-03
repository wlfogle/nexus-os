//! NexusOS Filesystem Layer
//!
//! Phase 5.2: FAT32 support via the `fatfs` crate (0.4 git).

#[cfg(target_arch = "x86_64")]
pub mod fat;
#[cfg(target_arch = "x86_64")]
pub mod vfs;
