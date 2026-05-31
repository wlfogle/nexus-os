//! NexusOS Virtual Memory / Paging
//!
//! Both x86_64 and AArch64 use 4-level, 48-bit virtual address, 4 KiB pages.
//! Limine has already set up a working page table before calling `_start`.
//! Phase 1 goals:
//!   • Verify HHDM is present
//!   • Record the HHDM offset for later use
//!   • Provide `map_page` / `unmap_page` used by heap::init
//!
//! Full recursive-mapping or self-referential PML4 is Phase 2.

use core::sync::atomic::{AtomicU64, Ordering};

// ── HHDM offset — set once in init, read-only afterwards ─────────────────────

static HHDM_OFFSET: AtomicU64 = AtomicU64::new(0);

/// Convert a physical address to a kernel virtual address via HHDM.
#[inline]
pub fn phys_to_virt(phys: u64) -> u64 {
    phys + HHDM_OFFSET.load(Ordering::Relaxed)
}

/// Convert a kernel HHDM virtual address to a physical address.
#[inline]
pub fn virt_to_phys(virt: u64) -> u64 {
    virt - HHDM_OFFSET.load(Ordering::Relaxed)
}

// ── Page flags ────────────────────────────────────────────────────────────────

pub mod flags {
    pub const PRESENT:        u64 = 1 << 0;
    pub const WRITABLE:       u64 = 1 << 1;
    pub const USER:           u64 = 1 << 2;
    pub const WRITE_THROUGH:  u64 = 1 << 3;
    pub const NO_CACHE:       u64 = 1 << 4;
    pub const ACCESSED:       u64 = 1 << 5;
    pub const DIRTY:          u64 = 1 << 6;
    pub const HUGE:           u64 = 1 << 7;
    pub const GLOBAL:         u64 = 1 << 8;

    // x86_64 NX bit / AArch64 UXN+PXN
    pub const NO_EXECUTE:     u64 = 1 << 63;

    pub const KERNEL_CODE: u64 = PRESENT | ACCESSED;
    pub const KERNEL_DATA: u64 = PRESENT | WRITABLE | NO_EXECUTE;
    pub const KERNEL_RO:   u64 = PRESENT | NO_EXECUTE;
}

// ── Page table entry helpers (architecture-generic 4-level layout) ────────────

const PAGE_SIZE: u64 = 4096;
const ENTRY_COUNT: usize = 512;

/// A single page table (512 × 8-byte entries).
#[repr(C, align(4096))]
struct PageTable([u64; ENTRY_COUNT]);

impl PageTable {
    fn new_zeroed() -> &'static mut Self {
        let phys = super::physical::alloc_frame();
        let virt = phys_to_virt(phys) as *mut PageTable;
        unsafe {
            // Zero the new table
            virt.write_bytes(0, 1);
            &mut *virt
        }
    }
}

// ── Index extraction (48-bit VA, 4-level) ─────────────────────────────────────

#[inline] fn pml4_idx(virt: u64) -> usize { ((virt >> 39) & 0x1FF) as usize }
#[inline] fn pdpt_idx(virt: u64) -> usize { ((virt >> 30) & 0x1FF) as usize }
#[inline] fn pd_idx  (virt: u64) -> usize { ((virt >> 21) & 0x1FF) as usize }
#[inline] fn pt_idx  (virt: u64) -> usize { ((virt >> 12) & 0x1FF) as usize }

/// Read the current active PML4 physical address.
#[cfg(target_arch = "x86_64")]
fn active_pml4_phys() -> u64 {
    let cr3: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    }
    cr3 & !0xFFF   // low 12 bits are flags, not part of the address
}

#[cfg(target_arch = "aarch64")]
fn active_pml4_phys() -> u64 {
    let ttbr1: u64;
    unsafe {
        core::arch::asm!("mrs {}, ttbr1_el1", out(reg) ttbr1, options(nomem, nostack));
    }
    ttbr1 & !0xFFF
}

// ── Public interface ──────────────────────────────────────────────────────────

/// Initialise paging — record HHDM offset, verify HHDM is accessible.
pub fn init(hhdm_offset: u64) {
    HHDM_OFFSET.store(hhdm_offset, Ordering::Relaxed);
}

/// Map a single 4 KiB virtual page to a physical frame.
///
/// Creates intermediate tables as needed by allocating frames from the
/// physical allocator.  Intermediate table entries use PRESENT | WRITABLE.
///
/// # Panics
/// Panics if the virtual address is already mapped to a different frame.
pub fn map_page(virt: u64, phys: u64, page_flags: u64) {
    assert_eq!(virt & 0xFFF, 0, "map_page: virt not page-aligned");
    assert_eq!(phys & 0xFFF, 0, "map_page: phys not page-aligned");

    let pml4_phys = active_pml4_phys();
    let pml4 = unsafe { &mut *(phys_to_virt(pml4_phys) as *mut PageTable) };

    // For user-accessible pages, intermediate entries must also carry the USER
    // bit — x86_64 denies user-mode access if ANY level has U/S=0.
    let inter_flags = flags::PRESENT | flags::WRITABLE
        | (page_flags & flags::USER);   // propagate USER to PML4/PDPT/PD

    let l4 = pml4_idx(virt);
    if pml4.0[l4] & flags::PRESENT == 0 {
        let new_tbl = super::physical::alloc_frame();
        unsafe { (phys_to_virt(new_tbl) as *mut PageTable).write_bytes(0, 1) };
        pml4.0[l4] = new_tbl | inter_flags;
    } else if page_flags & flags::USER != 0 {
        pml4.0[l4] |= flags::USER; // set USER on existing entry
    }
    let pdpt_phys = pml4.0[l4] & !0xFFF;
    let pdpt = unsafe { &mut *(phys_to_virt(pdpt_phys) as *mut PageTable) };

    let l3 = pdpt_idx(virt);
    if pdpt.0[l3] & flags::PRESENT == 0 {
        let new_tbl = super::physical::alloc_frame();
        unsafe { (phys_to_virt(new_tbl) as *mut PageTable).write_bytes(0, 1) };
        pdpt.0[l3] = new_tbl | inter_flags;
    } else {
        assert_eq!(pdpt.0[l3] & flags::HUGE, 0,
            "map_page: PDPT[{l3}] at virt {virt:#x} is a 1 GB huge page — pick a higher user address",
            l3 = l3, virt = virt);
        if page_flags & flags::USER != 0 { pdpt.0[l3] |= flags::USER; }
    }
    let pd_phys = pdpt.0[l3] & !0xFFF;
    let pd = unsafe { &mut *(phys_to_virt(pd_phys) as *mut PageTable) };

    let l2 = pd_idx(virt);
    if pd.0[l2] & flags::PRESENT == 0 {
        let new_tbl = super::physical::alloc_frame();
        unsafe { (phys_to_virt(new_tbl) as *mut PageTable).write_bytes(0, 1) };
        pd.0[l2] = new_tbl | inter_flags;
    } else {
        assert_eq!(pd.0[l2] & flags::HUGE, 0,
            "map_page: PD[{l2}] at virt {virt:#x} is a 2 MB huge page — pick a higher user address",
            l2 = l2, virt = virt);
        if page_flags & flags::USER != 0 { pd.0[l2] |= flags::USER; }
    }
    let pt_phys = pd.0[l2] & !0xFFF;
    let pt = unsafe { &mut *(phys_to_virt(pt_phys) as *mut PageTable) };

    let l1 = pt_idx(virt);
    let existing = pt.0[l1];
    if existing & flags::PRESENT != 0 {
        let existing_phys = existing & !0xFFF;
        assert_eq!(
            existing_phys, phys,
            "map_page: {:#x} already mapped to {:#x}, tried to map to {:#x}",
            virt, existing_phys, phys
        );
        return; // idempotent
    }

    pt.0[l1] = phys | page_flags;

    // TLB invalidation
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            "dsb ishst",
            "tlbi vaae1is, {}",
            "dsb ish",
            "isb",
            in(reg) virt >> 12,
            options(nostack),
        );
    }
}
