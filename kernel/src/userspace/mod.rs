//! NexusOS User-Space Process Management — Phase 4
//!
//! Spawns the first ring-3 (user-space) process.
//!
//! Memory layout for user processes:
//!   Code:  USER_CODE_BASE  = 0x0000_0040_0000  (4 MB, mapped PRESENT|USER|EXEC)
//!   Stack: USER_STACK_TOP  = 0x0000_7FFF_F000  (1 page below 128 TB boundary)
//!
//! Phase 4 shares the kernel's page tables — no separate address space yet.
//! The user code page is marked user-accessible; the kernel remains protected.
//! Separate page tables per process come in Phase 5.
//!
//! First entry uses IRETQ (not sysretq) because we're going ring-0 → ring-3
//! for the FIRST time, not returning from a syscall.

use crate::memory::{paging, physical};
use crate::process;
use crate::ipc;

/// User-space code page.  Placed above 512 MB to avoid Limine's identity map.
/// Limine typically identity-maps only the first 128–512 MB for boot.
// PML4[1] starts at 512 GB.  Limine only identity-maps physical RAM inside
// PML4[0] (0–512 GB) using 1 GB huge pages.  PML4[1] is completely free.
pub const USER_CODE_BASE:  u64 = 0x0000_0080_0000_0000;  // 512 GB (PML4[1])

/// User-space stack top — same PML4 entry, different PDPT entry.
pub const USER_STACK_TOP:  u64 = 0x0000_0080_1000_0000;  // 512 GB + 256 MB

/// User page flags: Present + Writable + User-accessible + No-Execute for data.
const USER_DATA_FLAGS: u64 = paging::flags::PRESENT
    | paging::flags::WRITABLE
    | paging::flags::USER
    | paging::flags::NO_EXECUTE;

/// User code flags: Present + User-accessible (executable, so no NX).
const USER_CODE_FLAGS: u64 = paging::flags::PRESENT | paging::flags::USER;

// ─── Embedded user-space shell ───────────────────────────────────────────────
//
// `shell_init.bin` is a flat 64-bit x86_64 binary assembled at build time
// from `src/userspace/shell_init.asm` by `build.rs` using NASM (-f bin).
//
// It is position-independent (all data references use RIP-relative [rel …]
// addressing) and fits in a single 4 KiB code page.  Mutable state
// (cmd_buf, num_buf) lives on the writable stack page.
//
// Syscalls used (all implemented in kernel/src/syscall/mod.rs):
//   SYS_EXIT=1  SYS_WRITE=2  SYS_GETPID=3  SYS_SLEEP=9  SYS_READ_CHAR=13
const USER_INIT_CODE: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/shell_init.bin"));

// Compile-time guard: the kernel maps exactly one 4 KiB page for the shell.
// If the binary ever exceeds this, add additional page mappings in
// spawn_user_init() before increasing this limit.
const _SHELL_SIZE_CHECK: () = assert!(
    USER_INIT_CODE.len() <= 4096,
    "shell_init.bin exceeds one code page (4096 bytes) — \
     add more page mappings in spawn_user_init()"
);

// ─── Public API ───────────────────────────────────────────────────────────────

/// Spawn the first user-space (ring-3) process.
///
/// 1. Maps one page at USER_CODE_BASE as user-executable.
/// 2. Copies USER_INIT_CODE there.
/// 3. Maps one page at USER_STACK_TOP - PAGE_SIZE as user-writable.
/// 4. Creates a process PCB whose initial "interrupt frame" has:
///      CS = 0x23 (user code, RPL=3)
///      SS = 0x1B (user data, RPL=3)
///      RIP = USER_CODE_BASE
///      RSP = USER_STACK_TOP
///      RFLAGS = 0x202 (interrupts enabled)
///    When the scheduler does IRETQ for this process, the CPL change
///    from ring-0 → ring-3 is handled automatically by the CPU.
pub fn spawn_user_init() -> u64 {
    const PAGE_SIZE: u64 = 4096;

    // ── 1. Allocate + map code page ──────────────────────────────────────
    let code_phys = physical::alloc_frame();
    paging::map_page(USER_CODE_BASE, code_phys, USER_CODE_FLAGS);

    // Copy init code to the mapped page
    let code_virt = paging::phys_to_virt(code_phys) as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(
            USER_INIT_CODE.as_ptr(),
            code_virt,
            USER_INIT_CODE.len(),
        );
        // Zero the rest of the page
        core::ptr::write_bytes(
            code_virt.add(USER_INIT_CODE.len()),
            0,
            PAGE_SIZE as usize - USER_INIT_CODE.len(),
        );
    }

    // ── 2. Allocate + map stack page ─────────────────────────────────────
    let stack_phys = physical::alloc_frame();
    paging::map_page(USER_STACK_TOP - PAGE_SIZE, stack_phys, USER_DATA_FLAGS);

    // ── 3. Create PCB with ring-3 initial interrupt frame ─────────────────
    // We call process::spawn_user which sets up CS=0x23/SS=0x1B
    let id = spawn_user_process(b"nexus-init", USER_CODE_BASE, USER_STACK_TOP)
        .expect("failed to spawn user init");

    // ── 4. Allocate IPC inbox ─────────────────────────────────────────────
    ipc::inbox_alloc(id);

    // ── 5. Prime PERCPU with this process's kernel stack top ──────────────
    crate::syscall::update_kernel_rsp(id);

    id
}

/// Allocate a process with a ring-3 initial interrupt frame.
fn spawn_user_process(name: &[u8], user_rip: u64, user_rsp_top: u64) -> Option<u64> {
    use process::MAX_PROCS;
    use core::sync::atomic::Ordering;

    // We build the PCB manually via the same mechanism as process::spawn,
    // but with ring-3 CS/SS selectors and user RSP.
    //
    // The fake "interrupt frame" the timer ISR will IRETQ into:
    //   SS     = 0x1B (user data, RPL=3)
    //   RSP    = user_rsp_top
    //   RFLAGS = 0x202 (IF=1)
    //   CS     = 0x23 (user code, RPL=3)
    //   RIP    = user_rip
    //
    // The CPU, seeing CS.RPL=3 ≠ current CPL=0, performs a full
    // privilege-level switch, popping SS and RSP from the frame.

    // We need access to the process table internals — use the public spawn
    // then patch the stack frame.  Easier: just call process::spawn_ring3.
    process::spawn_ring3(name, user_rip, user_rsp_top)
}
