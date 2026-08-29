//! x86_64 Global Descriptor Table + Task State Segment
//!
//! Descriptors (identical layout on every core, so the selector values below
//! are the same everywhere — only the TSS descriptor's *embedded base
//! address* actually differs per core):
//!   0  — null
//!   1  — kernel code  (ring 0)
//!   2  — kernel data  (ring 0)
//!   3  — user data    (ring 3, needed before user data for GDT ordering)
//!   4  — user code    (ring 3)
//!   5  — TSS (64-bit, takes 2 slots)
//!
//! The TSS provides IST slot 0 — a dedicated 20 KiB stack used exclusively
//! by the double-fault handler so a kernel stack overflow can still be caught.
//!
//! Phase K5 increment 4: every core needs its *own* TSS (RSP0/IST point at
//! that core's own kernel stacks — sharing one TSS across cores would mean
//! two CPUs racing to overwrite the same RSP0 slot) and therefore its own
//! GDT (a TSS descriptor embeds its TSS's base address, so the descriptor
//! itself can't be shared either). `TSS_TABLE`/`GDT_TABLE` below are sized
//! for `super::MAX_CPUS` cores; `init(core_id)` builds and loads exactly one
//! core's own entry, called once per core (BSP now, each AP in increment 5).

use x86_64::{
    instructions::tables::load_tss,
    registers::segmentation::{CS, DS, ES, FS, GS, SS, Segment},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    PrivilegeLevel, VirtAddr,
};

use super::MAX_CPUS;

/// IST index used by the double-fault handler.
pub const DOUBLE_FAULT_IST: u16 = 0;

/// 20 KiB emergency stack for the double-fault handler, one per core.
#[repr(align(16))]
struct DoubleFaultStack([u8; 20 * 1024]);
static DOUBLE_FAULT_STACKS: [DoubleFaultStack; MAX_CPUS] =
    [const { DoubleFaultStack([0u8; 20 * 1024]) }; MAX_CPUS];

/// 16 KiB initial ring-0 interrupt stack (RSP0), one per core.
/// Used when a timer or other interrupt fires while a ring-3 process is
/// running. The scheduler updates TSS.RSP0 to each process's own kernel
/// stack on switch (via `update_rsp0`); these statics are only used before
/// each core's first user process runs.
#[repr(align(16))]
struct Rsp0Stack([u8; 16 * 1024]);
static RSP0_STACKS: [Rsp0Stack; MAX_CPUS] = [const { Rsp0Stack([0u8; 16 * 1024]) }; MAX_CPUS];

// ─── Per-core TSS + GDT ────────────────────────────────────────────────────
//
// Both `TaskStateSegment::new()` and `GlobalDescriptorTable::new()` are
// `const fn`, so these can be plain const-initialised arrays — no `Lazy`
// needed now that each entry is built eagerly, exactly once, by that
// specific core's own `init()` call rather than lazily on first access from
// whichever core happens to touch it first. `TaskStateSegment` is also
// `Copy` (simple repeat-array init below); `GlobalDescriptorTable` is only
// `Clone`, so its array uses the `[const { .. }; N]` form instead.
static mut TSS_TABLE: [TaskStateSegment; MAX_CPUS] = [TaskStateSegment::new(); MAX_CPUS];
// GlobalDescriptorTable is Clone but not Copy, so the repeat-array needs the
// `[const { .. }; N]` form (evaluates the initializer once per index rather
// than requiring a single Copy-able value) -- the same idiom already used
// above for DOUBLE_FAULT_STACKS/RSP0_STACKS and in process::TABLE.
static mut GDT_TABLE: [GlobalDescriptorTable; MAX_CPUS] =
    [const { GlobalDescriptorTable::new() }; MAX_CPUS];

// Selector values are identical on every core by construction (every core's
// GDT is built with the same 5 entries in the same order below), so unlike
// the TSS/GDT storage itself these don't need to be per-core state at all.
// (Only the three actually consumed anywhere -- kernel code/data and TSS --
// are defined; user_data/user_code are hardcoded directly as 0x1B/0x23 at
// their only call site in userspace/mod.rs, matching that file's existing
// convention rather than duplicating unused accessors here.)
const KERNEL_CODE_SELECTOR: SegmentSelector = SegmentSelector::new(1, PrivilegeLevel::Ring0);
const KERNEL_DATA_SELECTOR: SegmentSelector = SegmentSelector::new(2, PrivilegeLevel::Ring0);
const TSS_SELECTOR:         SegmentSelector = SegmentSelector::new(5, PrivilegeLevel::Ring0);

// ─── Public interface ─────────────────────────────────────────────────────────

/// Build, load, and activate this core's own GDT + TSS. Must be called
/// exactly once per core, by that core itself (`core_id` identifies which
/// slot in `TSS_TABLE`/`GDT_TABLE`/`RSP0_STACKS`/`DOUBLE_FAULT_STACKS` it
/// owns from now on). `core_id` 0 is always the BSP.
pub fn init(core_id: usize) {
    unsafe {
        // Point this core's TSS at its own RSP0/IST[0] stacks before the
        // GDT's TSS descriptor (built below) captures the TSS's address.
        TSS_TABLE[core_id].privilege_stack_table[0] = {
            let stack_start = VirtAddr::from_ptr(RSP0_STACKS[core_id].0.as_ptr());
            stack_start + RSP0_STACKS[core_id].0.len() as u64
        };
        TSS_TABLE[core_id].interrupt_stack_table[DOUBLE_FAULT_IST as usize] = {
            let stack_start = VirtAddr::from_ptr(DOUBLE_FAULT_STACKS[core_id].0.as_ptr());
            stack_start + DOUBLE_FAULT_STACKS[core_id].0.len() as u64
        };

        let gdt = &mut GDT_TABLE[core_id];
        gdt.add_entry(Descriptor::kernel_code_segment());
        gdt.add_entry(Descriptor::kernel_data_segment());
        // user_data must come before user_code (sysret requires this layout)
        gdt.add_entry(Descriptor::user_data_segment());
        gdt.add_entry(Descriptor::user_code_segment());
        gdt.add_entry(Descriptor::tss_segment(&TSS_TABLE[core_id]));

        GDT_TABLE[core_id].load();

        // Reload CS with the kernel code selector
        CS::set_reg(KERNEL_CODE_SELECTOR);
        // Set all data segments to the kernel data selector
        DS::set_reg(KERNEL_DATA_SELECTOR);
        ES::set_reg(KERNEL_DATA_SELECTOR);
        FS::set_reg(KERNEL_DATA_SELECTOR);
        GS::set_reg(KERNEL_DATA_SELECTOR);
        SS::set_reg(KERNEL_DATA_SELECTOR);
        // Load the TSS
        load_tss(TSS_SELECTOR);
    }
}

/// Return the kernel code segment selector (needed by some interrupt stubs).
/// Identical on every core — see the selector constants above.
#[inline]
pub fn kernel_code_selector() -> SegmentSelector {
    KERNEL_CODE_SELECTOR
}

/// Update a specific core's TSS.RSP0 to a new kernel stack top.
/// Called by the scheduler on every context switch so that ring-3 interrupts
/// (timer, etc.) land on the correct per-process kernel stack.
///
/// # Safety
/// Must only be called with interrupts disabled (from the timer ISR context
/// running on that same `core_id`).
pub fn update_rsp0(core_id: usize, stack_top: u64) {
    unsafe {
        TSS_TABLE[core_id].privilege_stack_table[0] = VirtAddr::new(stack_top);
    }
}
