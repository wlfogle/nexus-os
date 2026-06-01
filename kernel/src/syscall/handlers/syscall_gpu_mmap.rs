//! SYS_GPU_MMAP (12) — Reserve a GPU buffer region in the caller's address space.
//!
//! Phase 5.0 stub: GPU driver, IOMMU integration, and DMA buffer management
//! are scheduled for Phase 5.2.  Returns ENOSYS so callers can detect support.
//!
//! Signature: gpu_mmap(size: u64, flags: u64, _reserved: u64) -> i64
//!   Returns: virtual address of mapped buffer, or -errno on error

/// Handle SYS_GPU_MMAP syscall.
///
/// # Phase 5.0 scope note
/// This syscall stub is acceptable here because it returns a well-defined
/// ENOSYS error and is only called by Phase 5.2+ GPU-accelerated inference
/// code paths.  All existing Phase 5.0 inference uses CPU-only Ollama.
pub fn handle_gpu_mmap(_size: u64, _flags: u64, _reserved: u64) -> i64 {
    // Phase 5.2: implement GPU memory mapping via VFIO/IOMMU.
    // The RTX 4080 is already in IOMMU Group 16 (isolated) and ready for
    // passthrough — see scripts/vm/vfio-bind.sh.
    -38 // ENOSYS — not yet implemented
}
