//! NexusOS Process Management — Phase 2
//!
//! Supports kernel threads only (ring 0).  User-space processes come in Phase 4.
//!
//! Each process has a 16 KiB kernel stack.  The stack pointer saved in the PCB
//! points into that stack at the position that the context-switch code expects:
//!
//!   [RSP] → saved_rax, saved_rbx, …, saved_r15,
//!             then the CPU-pushed interrupt frame: RIP, CS, RFLAGS, RSP, SS
//!
//! When context_switch restores a process it pops the saved registers and then
//! executes IRETQ which pops the CPU interrupt frame.

use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

/// Kernel stack size per process: 16 KiB.
pub const KSTACK_SIZE: usize = 16 * 1024;

/// Maximum simultaneous processes.
pub const MAX_PROCS: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessState {
    Ready,
    Running,
    Dead,
    /// Waiting to receive a message — scheduler skips this process.
    BlockedOnRecv,
    /// Waiting to send (destination inbox full) — scheduler skips this process.
    BlockedOnSend,
    /// Waiting for keyboard input — scheduler skips; IRQ1 handler wakes.
    BlockedOnKey,
    /// Waiting for a spawned child process to exit (SYS_EXEC/SYS_WAIT) —
    /// scheduler skips this process while it polls for the child's exit.
    BlockedOnChild,
    /// Exited but not yet reaped: a ring-3 process that owned a private
    /// address space finished (SYS_EXIT or SYS_KILL) but nothing has called
    /// SYS_WAIT on it yet. Kept out of the Dead/reusable pool so its
    /// pml4_phys survives until `reap()` frees the address space —
    /// reusing the slot (and silently overwriting pml4_phys) before that
    /// would leak the exited process's entire address space. Processes with
    /// no address space (pml4_phys == 0, e.g. kernel threads) skip this
    /// state and go straight to Dead since there's nothing to reap.
    Zombie,
}

/// Syscall personality for a ring-3 process.
/// Nexus-native binaries use the NexusOS syscall table (SYS_WRITE=2, etc.).
/// Linux ABI binaries use Linux x86_64 syscall numbers (write=1, exit=60).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessPersonality {
    Nexus,
    Linux,
}

/// Process Control Block.
#[repr(C)]
pub struct Process {
    pub id:    u64,
    pub state: ProcessState,
    /// Saved kernel stack pointer (updated on every context switch out).
    pub rsp:   u64,
    /// Top of the kernel stack (fixed; used to reset RSP on syscall entry).
    pub kernel_stack_top: u64,
    pub name:  [u8; 32],
    /// PID of the process that spawned this one via SYS_EXEC/SYS_SPAWN
    /// (0 = none, e.g. the top-level shell or a kernel thread).
    pub parent: u64,
    /// This process's own exit code, valid once `state == Zombie` (or after
    /// reaping, until the slot is reused). Set once by `exit()`.
    pub exit_code: i64,
    /// Physical address of this process's PML4 (its private address space).
    /// 0 means "none" — the process runs in the shared kernel address space
    /// (all kernel threads).  The scheduler loads this into CR3 on switch.
    pub pml4_phys: u64,
    /// Syscall ABI used by this process.
    pub personality: ProcessPersonality,
    /// Lowest address of this process's heap region (Phase 6.4: SYS_BRK).
    /// 0 means "no heap" — kernel threads and any ring-3 process spawned
    /// without a reserved heap region (neither currently exists, but the
    /// zero-default keeps SYS_BRK safely rejecting such processes).
    pub heap_base: u64,
    /// Current program break: [heap_base, heap_brk) is the logical heap size
    /// requested so far. Physical pages are only mapped up to the page-aligned
    /// extent of heap_brk — see SYS_BRK in syscall/mod.rs.
    pub heap_brk: u64,
    /// Kernel stack storage (lives inside the PCB for simplicity).
    pub stack: [u8; KSTACK_SIZE],
}

impl Process {
    const fn zero() -> Self {
        Self {
            id:               0,
            state:            ProcessState::Dead,
            rsp:              0,
            kernel_stack_top: 0,
            name:             [0u8; 32],
            parent:           0,
            exit_code:        0,
            pml4_phys:        0,
            personality:      ProcessPersonality::Nexus,
            heap_base:        0,
            heap_brk:         0,
            stack:            [0u8; KSTACK_SIZE],
        }
    }

    pub fn name_str(&self) -> &str {
        let len = self.name.iter().position(|&b| b == 0).unwrap_or(32);
        core::str::from_utf8(&self.name[..len]).unwrap_or("?")
    }
}

// ─── Process table ────────────────────────────────────────────────────────────

// The table lives in BSS — zero-initialised, no heap needed.
// Safety: only accessed through the Mutex.
static TABLE: Mutex<[Process; MAX_PROCS]> = Mutex::new(
    // const initialisation — all slots start Dead
    [const { Process::zero() }; MAX_PROCS]
);

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

// ─── Public API ───────────────────────────────────────────────────────────────

/// Allocate a new process slot and return its ID.
/// Sets up the kernel stack so the first "context restore" will jump to `entry`.
///
/// Stack layout after setup (high → low address, RSP points at rax slot):
///   [high]  SS=0x10, RSP=stack_top, RFLAGS=0x202 (IF set), CS=0x08, RIP=entry
///           r15=0, r14=0, …, rax=0, rbx=0, rcx=0, rdx=0, rsi=0, rdi=0, rbp=0
///   [low]  ← RSP saved here
pub fn spawn(name: &[u8], entry: u64) -> Option<u64> {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let mut table = TABLE.lock();

    let slot = table.iter_mut().find(|p| p.state == ProcessState::Dead)?;

    // Set name
    let len = name.len().min(31);
    slot.name[..len].copy_from_slice(&name[..len]);
    slot.name[len] = 0;

    // Stack grows downward — start from the top of the stack array
    let stack_top = (slot.stack.as_ptr() as u64) + KSTACK_SIZE as u64;

    // Align to 16 bytes (required by x86_64 ABI before CALL)
    let mut sp = stack_top & !0xF;

    // Push a fake CPU interrupt frame so IRETQ launches the entry point.
    // The frame pushed by the CPU on interrupt: SS, RSP, RFLAGS, CS, RIP
    // We use kernel segments: CS=0x08 (kernel code), SS=0x10 (kernel data)
    macro_rules! push64 {
        ($sp:expr, $val:expr) => {{
            $sp -= 8;
            unsafe { *($sp as *mut u64) = $val };
        }};
    }

    push64!(sp, 0x10);          // SS  — kernel data selector
    push64!(sp, stack_top - 8); // RSP — arbitrary initial stack pointer
    push64!(sp, 0x202);         // RFLAGS — IF=1 (interrupts enabled)
    push64!(sp, 0x08);          // CS  — kernel code selector
    push64!(sp, entry);         // RIP — entry point

    // Push saved general-purpose registers (all zero — fresh process)
    // Order must match what context_switch_asm pops: rbp,rax,rbx,rcx,rdx,rsi,rdi,r8-r15
    for _ in 0..15 {
        push64!(sp, 0);
    }

    slot.id               = id;
    slot.rsp              = sp;
    slot.kernel_stack_top = stack_top;  // syscall handler resets to here
    slot.state            = ProcessState::Ready;

    Some(id)
}

/// Syscall personality of a process. Defaults to Nexus for unknown/dead IDs.
pub fn get_personality(id: u64) -> ProcessPersonality {
    let table = TABLE.lock();
    table
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.personality)
        .unwrap_or(ProcessPersonality::Nexus)
}

/// Get the saved RSP of the currently-running process.
/// Called from the context switch to snapshot where we are.
pub fn get_rsp(id: u64) -> Option<u64> {
    let table = TABLE.lock();
    table.iter().find(|p| p.id == id).map(|p| p.rsp)
}

/// Update the saved RSP of a process (called after saving context).
pub fn set_rsp(id: u64, rsp: u64) {
    let mut table = TABLE.lock();
    if let Some(p) = table.iter_mut().find(|p| p.id == id) {
        p.rsp = rsp;
    }
}

/// Set process state.
pub fn set_state(id: u64, state: ProcessState) {
    let mut table = TABLE.lock();
    if let Some(p) = table.iter_mut().find(|p| p.id == id) {
        p.state = state;
    }
}

/// Get process state.
pub fn get_state(id: u64) -> ProcessState {
    let table = TABLE.lock();
    table.iter()
        .find(|p| p.id == id)
        .map(|p| p.state)
        .unwrap_or(ProcessState::Dead)
}

/// Spawn a user-space (ring-3) process in its own address space.
/// The initial IRETQ frame uses user segment selectors so the CPU performs a
/// full privilege-level switch on first schedule.  `pml4_phys` is the private
/// PML4 the scheduler loads into CR3 when this process runs (0 = shared kernel
/// address space).
pub fn spawn_ring3(
    name: &[u8],
    user_rip: u64,
    user_rsp: u64,
    pml4_phys: u64,
    heap_base: u64,
    parent: u64,
    personality: ProcessPersonality,
) -> Option<u64> {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let mut table = TABLE.lock();
    let slot = table.iter_mut().find(|p| p.state == ProcessState::Dead)?;

    let len = name.len().min(31);
    slot.name[..len].copy_from_slice(&name[..len]);
    slot.name[len] = 0;

    let stack_top = (slot.stack.as_ptr() as u64) + KSTACK_SIZE as u64;
    let mut sp = stack_top & !0xF;

    macro_rules! push64 {
        ($sp:expr, $val:expr) => {{
            $sp -= 8;
            unsafe { *($sp as *mut u64) = $val };
        }};
    }

    // Ring-3 IRETQ frame: SS, RSP, RFLAGS, CS, RIP
    push64!(sp, 0x1B);     // SS  — user data, RPL=3
    push64!(sp, user_rsp); // RSP — user stack
    push64!(sp, 0x202);    // RFLAGS — IF=1
    push64!(sp, 0x23);     // CS  — user code, RPL=3
    push64!(sp, user_rip); // RIP — user entry point

    // Saved GP registers (all zero for fresh process)
    for _ in 0..15 {
        push64!(sp, 0);
    }

    slot.id               = id;
    slot.rsp              = sp;
    slot.kernel_stack_top = stack_top;
    slot.pml4_phys        = pml4_phys;
    slot.personality      = personality;
    slot.heap_base        = heap_base;
    slot.heap_brk         = heap_base;
    slot.parent           = parent;
    slot.state            = ProcessState::Ready;

    Some(id)
}

/// Physical address of a process's private PML4 (0 = shared kernel space).
pub fn get_pml4(id: u64) -> u64 {
    let table = TABLE.lock();
    table.iter().find(|p| p.id == id).map(|p| p.pml4_phys).unwrap_or(0)
}

/// Return `(heap_base, heap_brk)` for a process. `(0, 0)` if it has no heap
/// region (e.g. a kernel thread, which never issues SYS_BRK anyway).
pub fn get_heap(id: u64) -> (u64, u64) {
    let table = TABLE.lock();
    table.iter()
        .find(|p| p.id == id)
        .map(|p| (p.heap_base, p.heap_brk))
        .unwrap_or((0, 0))
}

/// Update the current program break after SYS_BRK (un)maps the backing pages.
pub fn set_heap_brk(id: u64, new_brk: u64) {
    let mut table = TABLE.lock();
    if let Some(p) = table.iter_mut().find(|p| p.id == id) {
        p.heap_brk = new_brk;
    }
}

/// Return IDs of all processes in BlockedOnKey state.
pub fn blocked_on_key_ids(buf: &mut [u64]) -> usize {
    let table = TABLE.lock();
    let mut n = 0;
    for p in table.iter() {
        if n >= buf.len() { break; }
        if p.state == ProcessState::BlockedOnKey {
            buf[n] = p.id;
            n += 1;
        }
    }
    n
}

/// Get a process's kernel stack top address (for syscall PERCPU update).
pub fn get_kernel_stack_top(id: u64) -> Option<u64> {
    let table = TABLE.lock();
    table.iter()
        .find(|p| p.id == id)
        .map(|p| p.kernel_stack_top)
}

// ─── Exit / wait / reap (Phase 6 SYS_EXEC, Phase K2 SYS_SPAWN/SYS_WAIT/SYS_KILL) ─

/// Return the parent PID of `child` (0 if none).
pub fn get_parent(child: u64) -> u64 {
    let table = TABLE.lock();
    table.iter().find(|p| p.id == child).map(|p| p.parent).unwrap_or(0)
}

/// Mark `pid` as exited with `code`, called from SYS_EXIT (on itself) or
/// SYS_KILL (on another process). Transitions to `Zombie` if `pid` owns a
/// private address space (so a later `reap()` can free it), or straight to
/// `Dead` if it doesn't (kernel threads — nothing to reap).
///
/// Does not look up or notify any parent: a waiter discovers this by polling
/// `try_exit_code(pid)` directly, which works whether SYS_WAIT was already
/// blocked on `pid` or is called well after `pid` exited (the whole point of
/// decoupling SYS_SPAWN from SYS_WAIT — a background child's result must
/// survive until someone asks for it, not just if someone was already asking
/// at the exact instant it exited).
pub fn exit(pid: u64, code: i64) {
    let mut table = TABLE.lock();
    if let Some(p) = table.iter_mut().find(|p| p.id == pid) {
        p.exit_code = code;
        p.state = if p.pml4_phys != 0 { ProcessState::Zombie } else { ProcessState::Dead };
    }
}

/// Non-blocking: if `pid` has exited (state == Zombie), return its exit
/// code. Returns `None` if it's still running or doesn't exist / was
/// already reaped. Does not reap — call `reap(pid)` once you're done
/// needing anything else about it.
pub fn try_exit_code(pid: u64) -> Option<i64> {
    let table = TABLE.lock();
    table.iter()
        .find(|p| p.id == pid && p.state == ProcessState::Zombie)
        .map(|p| p.exit_code)
}

/// Reap a Zombie: free its private address space (if any) and return its
/// slot to the Dead pool so a future spawn can reuse it. Frees the PML4
/// with the table lock released (paging::free_user_pml4 walks the whole
/// address space and takes the physical allocator's own lock — no need to
/// hold the process table for that), then re-locks briefly to zero the slot.
pub fn reap(pid: u64) {
    let pml4 = {
        let table = TABLE.lock();
        table.iter().find(|p| p.id == pid).map(|p| p.pml4_phys).unwrap_or(0)
    };
    if pml4 != 0 {
        unsafe { crate::memory::paging::free_user_pml4(pml4); }
    }
    let mut table = TABLE.lock();
    if let Some(p) = table.iter_mut().find(|p| p.id == pid) {
        *p = Process::zero();
    }
}

/// Return IDs of all Ready or Running processes, in table order.
pub fn ready_ids(buf: &mut [u64]) -> usize {
    let table = TABLE.lock();
    let mut n = 0;
    for p in table.iter() {
        if n >= buf.len() { break; }
        if p.state == ProcessState::Ready || p.state == ProcessState::Running {
            buf[n] = p.id;
            n += 1;
        }
    }
    n
}
