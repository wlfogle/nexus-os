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
use crate::process::{self, ProcessState};
use crate::ipc;

/// ID of the currently-running process (0 = no process / boot context).
static CURRENT: AtomicU64 = AtomicU64::new(0);

/// Round-robin cursor — index of the last-picked slot.
static CURSOR: AtomicU64 = AtomicU64::new(0);

/// Initialise the scheduler: register the boot context as process 0 (idle).
pub fn init() {
    let id = process::spawn(b"idle", idle_entry as u64)
        .expect("scheduler: could not spawn idle process");
    ipc::inbox_alloc(id);
    CURRENT.store(id, Ordering::SeqCst);
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
    crate::timer::pit::tick();

    let cur_id = CURRENT.load(Ordering::SeqCst);

    // Save the current process's stack pointer
    if cur_id != 0 {
        process::set_rsp(cur_id, current_rsp);
        if process::get_state(cur_id) == ProcessState::Running {
            process::set_state(cur_id, ProcessState::Ready);
        }
    }

    // Find next ready process (round-robin)
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
    CURRENT.store(next_id, Ordering::SeqCst);

    // Update PERCPU.kernel_rsp (for syscall entry) and TSS.RSP0 (for ring-3 interrupts)
    crate::syscall::update_kernel_rsp(next_id);
    if let Some(top) = crate::process::get_kernel_stack_top(next_id) {
        crate::arch::x86_64::gdt::update_rsp0(top);
    }

    // Return the next process's saved RSP
    process::get_rsp(next_id).unwrap_or(current_rsp)
}

/// Return the ID of the currently-running process.
pub fn current_id() -> u64 {
    CURRENT.load(Ordering::Relaxed)
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
