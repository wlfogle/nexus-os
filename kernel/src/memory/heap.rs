//! NexusOS Kernel Heap
//!
//! Maps a contiguous virtual region and hands it to `linked_list_allocator`.
//! After `init()` completes, `alloc` / `Box` / `Vec` etc. are available.
//!
//! Virtual address: 0xffffe000_00000000
//! Size: 64 MiB (laptop/tiamat) | 16 MiB (bahamut)

use linked_list_allocator::LockedHeap;
use super::paging::{flags, map_page};
use super::physical::alloc_frame;

// ── Heap parameters ───────────────────────────────────────────────────────────

/// Virtual start address for the kernel heap.
/// Chosen to be well clear of kernel image and HHDM region.
const HEAP_START: u64 = 0xffffe000_0000_0000;

/// Heap size in bytes.
#[cfg(not(feature = "minimal-heap"))]
pub const HEAP_SIZE: usize = 64 * 1024 * 1024;   // 64 MiB

#[cfg(feature = "minimal-heap")]
pub const HEAP_SIZE: usize = 16 * 1024 * 1024;   // 16 MiB

// ── Global allocator ──────────────────────────────────────────────────────────

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

// ── Initialisation ────────────────────────────────────────────────────────────

/// Map heap pages and initialise the global allocator.
/// Must be called after `physical::init()` and `paging::init()`.
pub fn init() {
    let heap_end = HEAP_START + HEAP_SIZE as u64;

    let mut vaddr = HEAP_START;
    while vaddr < heap_end {
        let paddr = alloc_frame();
        map_page(vaddr, paddr, flags::KERNEL_DATA);
        vaddr += 4096;
    }

    // Safety: we have just mapped the full [HEAP_START, HEAP_START + HEAP_SIZE)
    // range and no other code has used it.
    unsafe {
        HEAP.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }
}
