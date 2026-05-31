//! NexusOS Physical Frame Allocator — Bitmap Implementation
//!
//! Manages physical 4 KiB pages (frames) using a static bitmap.
//! One bit = one frame: 0 = free, 1 = used.
//!
//! Bitmap capacity:
//!   laptop/tiamat: up to 64 GB  (2 MB bitmap in BSS, zero-cost in binary)
//!   bahamut:       up to  2 GB  (64 KB bitmap)

use spin::Mutex;
use limine::{MemmapResponse, MemoryMapEntryType};

const PAGE_SIZE: u64 = 4096;

// ── Bitmap sizing ──────────────────────────────────────────────────────────────

#[cfg(not(feature = "minimal-heap"))]
const MAX_FRAMES: usize = 1 << 24;   // 16 M frames = 64 GB

#[cfg(feature = "minimal-heap")]
const MAX_FRAMES: usize = 1 << 19;   // 512 K frames = 2 GB

const BITMAP_WORDS: usize = MAX_FRAMES / 64;

// ── Static bitmap ─────────────────────────────────────────────────────────────
// Lives in .bss — not stored in the ELF, zeroed by the bootloader.
// All bits start at 0 (free); init() marks reserved/used regions as 1.

static mut BITMAP: [u64; BITMAP_WORDS] = [0u64; BITMAP_WORDS];

struct FrameAllocator {
    total:  usize,
    free:   usize,
}

static ALLOCATOR: Mutex<FrameAllocator> = Mutex::new(FrameAllocator {
    total: 0,
    free:  0,
});

// ── Bitmap helpers ────────────────────────────────────────────────────────────

#[inline]
fn set_used(frame: usize) {
    unsafe { BITMAP[frame / 64] |= 1u64 << (frame % 64) };
}

#[inline]
fn set_free(frame: usize) {
    unsafe { BITMAP[frame / 64] &= !(1u64 << (frame % 64)) };
}

#[inline]
fn is_used(frame: usize) -> bool {
    unsafe { (BITMAP[frame / 64] >> (frame % 64)) & 1 != 0 }
}

// ── Public interface ──────────────────────────────────────────────────────────

/// Initialise from the Limine memory map.
/// Marks all frames as used, then frees only the `Usable` regions.
pub fn init(mmap: &MemmapResponse, _hhdm_offset: u64) {
    let mut alloc = ALLOCATOR.lock();

    // Start with all frames marked as used (safe default).
    unsafe { BITMAP.iter_mut().for_each(|w| *w = u64::MAX) };

    let entries = mmap.memmap();
    let mut total = 0usize;
    let mut free  = 0usize;

    for entry_ptr in entries.iter() {
        // NonNullPtr<MemmapEntry> implements Deref<Target = MemmapEntry>
        let entry = &**entry_ptr;

        let start_frame = (entry.base / PAGE_SIZE) as usize;
        let frames      = (entry.len  / PAGE_SIZE) as usize;

        if start_frame + frames > MAX_FRAMES {
            // Beyond our bitmap capacity; skip or truncate
            continue;
        }

        total += frames;

        if entry.typ == MemoryMapEntryType::Usable {
            for f in start_frame..start_frame + frames {
                set_free(f);
            }
            free += frames;
        }
    }

    alloc.total = total;
    alloc.free  = free;

    crate::kprintln!(
        "[pmem] {} MiB usable / {} MiB total",
        (free  * 4096) / (1024 * 1024),
        (total * 4096) / (1024 * 1024),
    );
}

/// Allocate a single free physical frame.
/// Returns the physical address of the frame, or panics if OOM.
pub fn alloc_frame() -> u64 {
    let mut alloc = ALLOCATOR.lock();

    for word_idx in 0..BITMAP_WORDS {
        let word = unsafe { BITMAP[word_idx] };
        if word == u64::MAX { continue; }  // all used

        // Find first free bit
        let bit = word.trailing_ones() as usize;
        let frame = word_idx * 64 + bit;
        if frame >= MAX_FRAMES { break; }

        unsafe { BITMAP[word_idx] |= 1u64 << bit; }
        alloc.free -= 1;
        return (frame as u64) * PAGE_SIZE;
    }

    panic!("alloc_frame: out of physical memory ({} free)", alloc.free);
}

/// Free a physical frame previously returned by `alloc_frame`.
/// # Safety
/// The caller must ensure the frame is no longer mapped anywhere.
pub unsafe fn free_frame(phys_addr: u64) {
    let frame = (phys_addr / PAGE_SIZE) as usize;
    assert!(frame < MAX_FRAMES, "free_frame: address out of range");
    assert!(is_used(frame), "free_frame: double-free detected");

    set_free(frame);
    ALLOCATOR.lock().free += 1;
}

/// Number of free frames remaining.
pub fn free_frames() -> usize {
    ALLOCATOR.lock().free
}
