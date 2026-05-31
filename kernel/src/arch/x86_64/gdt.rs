//! x86_64 Global Descriptor Table + Task State Segment
//!
//! Descriptors:
//!   0  — null
//!   1  — kernel code  (ring 0)
//!   2  — kernel data  (ring 0)
//!   3  — user data    (ring 3, needed before user data for GDT ordering)
//!   4  — user code    (ring 3)
//!   5  — TSS (64-bit, takes 2 slots)
//!
//! The TSS provides IST slot 0 — a dedicated 20 KiB stack used exclusively
//! by the double-fault handler so a kernel stack overflow can still be caught.

use spin::Lazy;
use x86_64::{
    instructions::tables::load_tss,
    registers::segmentation::{CS, DS, ES, FS, GS, SS, Segment},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

/// IST index used by the double-fault handler.
pub const DOUBLE_FAULT_IST: u16 = 0;

/// 20 KiB emergency stack for the double-fault handler.
#[repr(align(16))]
struct DoubleFaultStack([u8; 20 * 1024]);
static DOUBLE_FAULT_STACK: DoubleFaultStack = DoubleFaultStack([0u8; 20 * 1024]);

/// 16 KiB initial ring-0 interrupt stack (RSP0).
/// Used when a timer or other interrupt fires while a ring-3 process is running.
/// The scheduler updates TSS.RSP0 to each process's own kernel stack on switch
/// (via `update_rsp0`); this static is only used before the first user process runs.
#[repr(align(16))]
struct Rsp0Stack([u8; 16 * 1024]);
static RSP0_STACK: Rsp0Stack = Rsp0Stack([0u8; 16 * 1024]);

// ─── TSS ─────────────────────────────────────────────────────────────────────────────────

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    // RSP0: kernel stack for interrupts from ring 3.
    // The scheduler updates this on every context switch (see update_rsp0).
    tss.privilege_stack_table[0] = {
        let stack_start = VirtAddr::from_ptr(RSP0_STACK.0.as_ptr());
        stack_start + RSP0_STACK.0.len() as u64
    };

    // IST[0]: emergency stack for the double-fault handler.
    tss.interrupt_stack_table[DOUBLE_FAULT_IST as usize] = {
        let stack_start = VirtAddr::from_ptr(DOUBLE_FAULT_STACK.0.as_ptr());
        stack_start + DOUBLE_FAULT_STACK.0.len() as u64
    };

    tss
});

// ─── GDT ─────────────────────────────────────────────────────────────────────

struct GdtSelectors {
    kernel_code: SegmentSelector,
    kernel_data: SegmentSelector,
    user_data:   SegmentSelector,
    user_code:   SegmentSelector,
    tss:         SegmentSelector,
}

static GDT: Lazy<(GlobalDescriptorTable, GdtSelectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let kernel_code = gdt.add_entry(Descriptor::kernel_code_segment());
    let kernel_data = gdt.add_entry(Descriptor::kernel_data_segment());
    // user_data must come before user_code (sysret requires this layout)
    let user_data   = gdt.add_entry(Descriptor::user_data_segment());
    let user_code   = gdt.add_entry(Descriptor::user_code_segment());
    let tss         = gdt.add_entry(Descriptor::tss_segment(&TSS));

    (gdt, GdtSelectors { kernel_code, kernel_data, user_data, user_code, tss })
});

// ─── Public interface ─────────────────────────────────────────────────────────

/// Load the GDT and reload all segment registers.
pub fn init() {
    GDT.0.load();

    unsafe {
        // Reload CS with the kernel code selector
        CS::set_reg(GDT.1.kernel_code);
        // Set all data segments to the kernel data selector
        DS::set_reg(GDT.1.kernel_data);
        ES::set_reg(GDT.1.kernel_data);
        FS::set_reg(GDT.1.kernel_data);
        GS::set_reg(GDT.1.kernel_data);
        SS::set_reg(GDT.1.kernel_data);
        // Load the TSS
        load_tss(GDT.1.tss);
    }
}

/// Return the kernel code segment selector (needed by some interrupt stubs).
#[inline]
pub fn kernel_code_selector() -> SegmentSelector {
    GDT.1.kernel_code
}

/// Update TSS.RSP0 to a new kernel stack top.
/// Called by the scheduler on every context switch so that ring-3 interrupts
/// (timer, etc.) land on the correct per-process kernel stack.
///
/// # Safety
/// Must only be called with interrupts disabled (from the timer ISR context).
pub fn update_rsp0(stack_top: u64) {
    unsafe {
        // TSS is behind Lazy<> — we take a raw mutable pointer to update RSP0.
        // This is safe because interrupts are disabled during the call.
        let tss_ptr = &*TSS as *const TaskStateSegment as *mut TaskStateSegment;
        (*tss_ptr).privilege_stack_table[0] = VirtAddr::new(stack_top);
    }
}
