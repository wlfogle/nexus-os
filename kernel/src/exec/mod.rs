//! NexusOS Program Loader — Phase 6
//!
//! A minimal but complete static **ELF64** loader.  It parses an in-memory ELF
//! image (already read from the FAT32 disk by the caller), maps every `PT_LOAD`
//! segment into the target process's address space, zeroes any `.bss` tail, and
//! returns the entry point so the caller can spawn a ring-3 process.
//!
//! # Address-space model
//!
//! Each executed program runs in its **own** address space: the caller builds a
//! fresh PML4 with [`crate::memory::paging::alloc_user_pml4`] (sharing the
//! kernel higher half but with a private user half) and passes its physical
//! address to [`load_elf`] and [`map_user_stack`], which map through the HHDM
//! into that table via `map_page_in`.  The program image links at [`EXEC_BASE`]
//! and its stack uses [`EXEC_STACK_TOP`]; both live in PML4[1], which Limine
//! never identity-maps.  Because the user half is private, a program never
//! collides with the resident shell even when they pick overlapping addresses.
//!
//! # Re-entrancy
//!
//! `run` is synchronous: the shell blocks until the child exits, so only one
//! loaded program exists at a time.  The child's address space is reclaimed in
//! full (see `free_user_pml4`) once it exits, so repeated `run` invocations do
//! not leak frames.

use crate::memory::{paging, physical};

/// Recommended link base for NexusOS user programs (PML4[1], PDPT[1]).
/// The bundled `hello.elf` is linked here (`ld -Ttext=0x8040000000`).
pub const EXEC_BASE: u64 = 0x0000_0080_4000_0000;

/// Top of the stack handed to executed programs (PML4[1], PDPT[5]).
pub const EXEC_STACK_TOP: u64 = 0x0000_0080_5000_0000;

/// Base of an executed program's SYS_BRK heap region (Phase 6.4).  Placed
/// 256 MB past EXEC_STACK_TOP (1.5 GB into PML4[1]'s PDPT[1], the same 1 GB
/// PDPT entry EXEC_BASE and EXEC_STACK_TOP already live in) — clear of both
/// the code page(s) near EXEC_BASE and the single fixed stack page just below
/// EXEC_STACK_TOP, with room to grow up to HEAP_MAX before hitting PDPT[2].
pub const EXEC_HEAP_BASE: u64 = EXEC_STACK_TOP + 0x1000_0000;

/// Lowest legal user virtual address.  Keep the null/low guard page unmapped,
/// but allow conventional static Linux ELF bases such as 0x400000 now that each
/// process owns a private PML4[0].
const USER_REGION_LO: u64 = 0x0000_0000_0040_0000;
/// One past the highest legal user virtual address we accept for a segment.
const USER_REGION_HI: u64 = 0x0000_0100_0000_0000;

const PAGE: u64 = 4096;

// ── ELF constants ────────────────────────────────────────────────────────────

const PT_LOAD: u32 = 1;
const PF_X:    u32 = 1; // segment is executable

// ── Public API ────────────────────────────────────────────────────────────────

/// Result of a successful load.
pub struct Loaded {
    /// Virtual entry point (`e_entry`).
    pub entry: u64,
}

/// Parse and map a static ELF64 image into the address space rooted at
/// `pml4_phys`.  Returns the entry point on success.
///
/// The image is mapped into the target process's private address space using
/// user-accessible pages.  File contents are copied through each frame's HHDM
/// alias — never through the read-only user virtual address — so this works
/// regardless of CR0.WP and regardless of which PML4 is currently active.
pub fn load_elf(pml4_phys: u64, data: &[u8]) -> Result<Loaded, &'static str> {
    // ── ELF header validation ────────────────────────────────────────────────
    if data.len() < 64 {
        return Err("elf: image smaller than ELF header");
    }
    if &data[0..4] != b"\x7FELF" {
        return Err("elf: bad magic");
    }
    if data[4] != 2 {
        return Err("elf: not ELFCLASS64");
    }
    if data[5] != 1 {
        return Err("elf: not little-endian");
    }
    let e_machine = u16::from_le_bytes([data[18], data[19]]);
    if e_machine != 0x3E {
        return Err("elf: not x86-64");
    }
    let e_entry     = read_u64(data, 24);
    let e_phoff     = read_u64(data, 32) as usize;
    let e_phentsize = u16::from_le_bytes([data[54], data[55]]) as usize;
    let e_phnum     = u16::from_le_bytes([data[56], data[57]]) as usize;
    if e_phentsize < 56 {
        return Err("elf: program-header entry too small");
    }
    if e_phnum == 0 {
        return Err("elf: no program headers");
    }

    // ── Walk program headers, map every PT_LOAD ───────────────────────────────
    let mut loaded_any = false;
    for i in 0..e_phnum {
        let off = e_phoff + i * e_phentsize;
        if off + 56 > data.len() {
            return Err("elf: program header out of bounds");
        }
        let ph = &data[off..off + 56];
        let p_type = u32::from_le_bytes([ph[0], ph[1], ph[2], ph[3]]);
        if p_type != PT_LOAD {
            continue;
        }
        let p_flags  = u32::from_le_bytes([ph[4], ph[5], ph[6], ph[7]]);
        let p_offset = read_u64(ph, 8) as usize;
        let p_vaddr  = read_u64(ph, 16);
        let p_filesz = read_u64(ph, 32) as usize;
        let p_memsz  = read_u64(ph, 40) as usize;

        if p_memsz == 0 {
            continue;
        }
        if p_vaddr < USER_REGION_LO || p_vaddr.saturating_add(p_memsz as u64) > USER_REGION_HI {
            return Err("elf: segment outside user region (link at >= 0x400000)");
        }
        if p_filesz > p_memsz {
            return Err("elf: filesz exceeds memsz");
        }
        if p_filesz > 0 && p_offset + p_filesz > data.len() {
            return Err("elf: segment file range out of bounds");
        }

        // Executable segments are mapped read+execute+user; everything else is
        // writable+user+NX.  (A fully separate W^X split per overlapping page is
        // deferred; the bundled program has a single segment.)
        let flags = if p_flags & PF_X != 0 {
            paging::flags::PRESENT | paging::flags::USER
        } else {
            paging::flags::PRESENT | paging::flags::WRITABLE
                | paging::flags::USER | paging::flags::NO_EXECUTE
        };

        let file_lo = p_vaddr;
        let file_hi = p_vaddr + p_filesz as u64;
        let vstart  = p_vaddr & !(PAGE - 1);
        let vend    = p_vaddr + p_memsz as u64;

        let mut v = vstart;
        while v < vend {
            // Free any frame previously mapped here (repeat-run safety), then
            // allocate a fresh, zeroed frame and copy this page's file bytes
            // into it via the HHDM alias (writable kernel mapping).
            if let Some(old) = paging::unmap_page_in(pml4_phys, v) {
                unsafe { physical::free_frame(old); }
            }
            let frame = physical::alloc_frame();
            let dst   = paging::phys_to_virt(frame) as *mut u8;
            unsafe { core::ptr::write_bytes(dst, 0, PAGE as usize); }

            let copy_start = core::cmp::max(v, file_lo);
            let copy_end   = core::cmp::min(v + PAGE, file_hi);
            if copy_start < copy_end {
                let in_page = (copy_start - v) as usize;
                let in_file = p_offset + (copy_start - file_lo) as usize;
                let n       = (copy_end - copy_start) as usize;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        data.as_ptr().add(in_file),
                        dst.add(in_page),
                        n,
                    );
                }
            }

            paging::map_page_in(pml4_phys, v, frame, flags);
            v += PAGE;
        }
        loaded_any = true;
    }

    if !loaded_any {
        return Err("elf: no loadable segments");
    }
    if e_entry < USER_REGION_LO || e_entry >= USER_REGION_HI {
        return Err("elf: entry point outside user region");
    }

    Ok(Loaded { entry: e_entry })
}

/// (Re)map a single fresh, zeroed, user-writable stack page just below `top`
/// in the address space rooted at `pml4_phys`.  Returns the lowest stack
/// address mapped.
pub fn map_user_stack(pml4_phys: u64, top: u64) -> u64 {
    let page = top - PAGE;
    if let Some(old) = paging::unmap_page_in(pml4_phys, page) {
        unsafe { physical::free_frame(old); }
    }
    let frame = physical::alloc_frame();
    unsafe { core::ptr::write_bytes(paging::phys_to_virt(frame) as *mut u8, 0, PAGE as usize); }
    paging::map_page_in(
        pml4_phys,
        page,
        frame,
        paging::flags::PRESENT | paging::flags::WRITABLE
            | paging::flags::USER | paging::flags::NO_EXECUTE,
    );
    page
}

#[inline]
fn read_u64(buf: &[u8], off: usize) -> u64 {
    u64::from_le_bytes([
        buf[off], buf[off + 1], buf[off + 2], buf[off + 3],
        buf[off + 4], buf[off + 5], buf[off + 6], buf[off + 7],
    ])
}
