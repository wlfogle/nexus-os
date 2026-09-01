//! NexusOS Round-Robin Preemptive Scheduler — Phase 2
//!
//! Called from the timer IRQ handler every 10 ms.
//! Selects the next Ready process and performs a context switch.
//!
//! Context switch protocol (x86_64, kernel threads only):
//!   1. Timer ISR (naked) pushes all GP registers onto the *current* stack.
//!   2. Calls `scheduler_tick(rsp)` with the current RSP value.
//!   3. `scheduler_tick` saves the RSP into the current PCB, picks next process,
//!      updates the current-process pointer, and returns the *next* RSP.
//!   4. The naked ISR loads the returned RSP into RSP, pops GP registers,
//!      and executes IRETQ — landing in the next process.

use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use crate::process::{self, ProcessState};
use crate::ipc;

/// Number of per-core `CURRENT` slots. x86_64 sizes this to the same
/// `MAX_CPUS` the GDT/TSS/PERCPU arrays use (Phase K5 increments 4-5);
/// every other architecture (aarch64/bahamut) doesn't run the scheduler on
/// more than one core today, so a single slot is both correct and cheap.
#[cfg(target_arch = "x86_64")]
const MAX_CORES: usize = crate::arch::x86_64::MAX_CPUS;
#[cfg(not(target_arch = "x86_64"))]
const MAX_CORES: usize = 1;

/// This core's dense index, for indexing `CURRENT` below. On x86_64 this is
/// the same GS-relative id `syscall::current_core_id()` already provides
/// (populated by that core's own `syscall::init(core_id)` call); every
/// other architecture has exactly one schedulable core today.
#[cfg(target_arch = "x86_64")]
#[inline]
fn this_core() -> usize { crate::syscall::current_core_id() }
#[cfg(not(target_arch = "x86_64"))]
#[inline]
fn this_core() -> usize { 0 }

/// ID of the process currently running *on each core* (0 = no process /
/// boot context on that core). A single global here — correct only because
/// exactly one core ever ran the scheduler — was Phase K5's last real
/// single-core assumption left in this file; see `scheduler_tick` for the
/// matching per-core dispatch.
static CURRENT: [AtomicU64; MAX_CORES] = [const { AtomicU64::new(0) }; MAX_CORES];

/// Round-robin cursor — index of the last-picked slot. Shared across cores
/// on purpose (one global rotation through the ready set, not a separate
/// rotation per core), but every read-compute-write of it must happen
/// inside `PICK_LOCK` — see `scheduler_tick`'s comment for why.
static CURSOR: AtomicU64 = AtomicU64::new(0);

/// Serializes the "pick the next ready process" decision across cores.
/// `process::TABLE`'s own lock protects individual field reads/writes, but
/// it does not make the round-robin *decision* atomic across the whole
/// scan-then-claim sequence: two cores concurrently reading `CURSOR`,
/// independently computing the same "next" id, and both marking it
/// `Running` would double-schedule that process on two cores at once — a
/// real bug with real concurrent callers, not a theoretical one now that
/// APs exist. This lock is always acquired before any `process::TABLE`
/// lock and never the reverse, so there is no lock-ordering / deadlock risk.
static PICK_LOCK: Mutex<()> = Mutex::new(());

/// Initialise the scheduler: register the boot context as process 0 (idle).
/// Called once, by the BSP, before any AP exists — so core 0's slot is the
/// only one that needs seeding here.
pub fn init() {
    let id = process::spawn(b"idle", idle_entry as u64)
        .expect("scheduler: could not spawn idle process");
    ipc::inbox_alloc(id);
    CURRENT[0].store(id, Ordering::SeqCst);
    crate::kprintln!("[sched] Scheduler initialized, idle process id={}", id);
}

/// Spawn a kernel thread and allocate its IPC inbox.
pub fn spawn(name: &[u8], entry: extern "C" fn() -> !) -> Option<u64> {
    let id = process::spawn(name, entry as u64)?;
    ipc::inbox_alloc(id);
    Some(id)
}

/// Called from the timer ISR with the RSP *after* all GP registers were pushed.
/// Returns the RSP to switch to.
///
/// # Safety
/// Must only be called from the naked timer ISR with interrupts disabled.
#[no_mangle]
pub unsafe extern "C" fn scheduler_tick(current_rsp: u64) -> u64 {
    let core = this_core();

    // Only the BSP's timer interrupt is actually sourced from the PIT
    // hardware (the I/O APIC routes it exclusively to the BSP's LAPIC id —
    // see main.rs); an AP calling this via its own LAPIC timer must not
    // also advance the PIT tick counter, or `timer::ticks()`/`millis()`
    // would run faster than real time in proportion to core count.
    #[cfg(target_arch = "x86_64")]
    if core == 0 {
        crate::timer::pit::tick();
    }

    let cur_id = CURRENT[core].load(Ordering::SeqCst);

    // Save the current process's stack pointer
    if cur_id != 0 {
        process::set_rsp(cur_id, current_rsp);
        if process::get_state(cur_id) == ProcessState::Running {
            process::set_state(cur_id, ProcessState::Ready);
        }
    }

    // Pick the next ready process. The whole scan-decide-claim sequence
    // runs under PICK_LOCK so no other core's concurrent tick can observe
    // the same pre-claim snapshot and pick the same process — see
    // PICK_LOCK's own doc comment for why that's a real race, not a
    // theoretical one, now that more than one core can call this function.
    let next_id = {
        let _guard = PICK_LOCK.lock();

        let mut ids = [0u64; 64];
        let n = process::ready_ids(&mut ids);

        let next_id = if n == 0 {
            // No ready processes — keep current (or spin in idle)
            cur_id
        } else {
            // Advance cursor
            let cursor = CURSOR.load(Ordering::SeqCst);
            // Find next after cursor
            let start = ids.iter().position(|&id| id > cursor).unwrap_or(0);
            let next = ids[start];
            CURSOR.store(next, Ordering::SeqCst);
            next
        };

        process::set_state(next_id, ProcessState::Running);
        next_id
    };
    CURRENT[core].store(next_id, Ordering::SeqCst);

    // next_id == 0 means there is truly nothing schedulable on this core
    // right now (only reachable when this core's own `cur_id` was also 0 —
    // e.g. an AP's very first tick landing at the exact instant every real
    // process happens to be Running-elsewhere/Blocked). id 0 is never a
    // real process (`process::spawn*` start IDs at 1) and every
    // `process::*` lookup now treats it as "not found" on purpose — so
    // there is nothing to update or switch into. Just resume exactly where
    // this tick was invoked from.
    if next_id == 0 {
        return current_rsp;
    }

    // Update PERCPU.kernel_rsp (for syscall entry) and TSS.RSP0 (for ring-3
    // interrupts) on the core actually running this tick.
    #[cfg(target_arch = "x86_64")]
    crate::syscall::update_kernel_rsp(core, next_id);
    #[cfg(target_arch = "x86_64")]
    if let Some(top) = crate::process::get_kernel_stack_top(next_id) {
        crate::arch::x86_64::gdt::update_rsp0(core, top);
    }

    // Switch into the next process's address space.  User processes carry a
    // private PML4; kernel threads (pml4 == 0) run in the shared kernel PML4.
    // Every PML4 maps the full kernel higher half identically, so the kernel
    // stack we are running on and the code after this call remain valid across
    // the CR3 load.  The load also flushes stale user TLB entries.
    let next_pml4 = process::get_pml4(next_id);
    let target = if next_pml4 == 0 {
        crate::memory::paging::kernel_pml4_phys()
    } else {
        next_pml4
    };
    crate::memory::paging::switch_address_space(target);

    // Return the next process's saved RSP
    process::get_rsp(next_id).unwrap_or(current_rsp)
}

/// Return the ID of the process currently running on *this* core.
pub fn current_id() -> u64 {
    CURRENT[this_core()].load(Ordering::Relaxed)
}

/// Idle process — runs when nothing else is runnable.
/// Uses HLT to save power; the next timer tick will preempt it.
extern "C" fn idle_entry() -> ! {
    loop {
        unsafe {
            core::arch::asm!("sti; hlt", options(nomem, nostack));
        }
    }
}
