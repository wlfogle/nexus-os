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

// ─── Embedded user-space init program ────────────────────────────────────────
//
// The "nexus-init" program runs in ring 3.  It demonstrates the full
// ring-3 → ring-0 → ring-3 syscall round-trip.
//
// Program logic (hand-assembled x86_64):
//   1. mov rax, SYS_GETPID (3); syscall  → rax = our PID
//   2. loop:
//        mov rax, SYS_WRITE (2)
//        mov rdi, 1          (stdout)
//        lea rsi, [rip+msg]
//        mov rdx, msg_len
//        syscall
//        mov rax, SYS_SLEEP (9)
//        mov rdi, 200        (~2 seconds)
//        syscall
//        jmp loop
//
// Machine code generated from the above (position-independent):

// Minimal test code to isolate the fault:
// [0-6] mov rax, 3 (SYS_GETPID)
// [7-8] syscall
// [9]   f4 hlt  (ring-3 HLT causes #GP — proves we got past the syscall)
//
// OLD LAYOUT COMMENT (kept for reference):
//
//   [0-6]   mov rax, 3 (SYS_GETPID)
//   [7-8]   syscall                          RIP_after = 9
//   [9-11]  mov r12, rax
//   [12-18] mov rax, 2 (SYS_WRITE)  ← LOOP_START
//   [19-25] mov rdi, 1
//   [26-32] lea rsi, [rip+30]               rip_after=33 + 30 = 63 = msg start ✓
//   [33-39] mov rdx, 32 (msg length)
//   [40-41] syscall                          RIP_after = 42 (code continues below)
//   [42-48] mov rax, 9 (SYS_SLEEP)
//   [49-55] mov rdi, 200
//   [56-57] syscall                          RIP_after = 58 (code continues)
//   [58-62] jmp LOOP_START                  rel = 12 - 63 = -51 = 0xFFFF_FFCD
//   [63-94] msg (32 bytes)  ← data only, never executed
// User-space init — ring-3 process that calls SYS_WRITE every 2 seconds.
//
// Code is position-independent; msg comes AFTER the jmp so all syscall
// return addresses land in executable code, not data.
//
// Byte layout:
//   [0-6]   mov rax, 3 (SYS_GETPID)
//   [7-8]   syscall                      rip_after=9
//   [9-11]  mov r12, rax                  (save our pid)
//   [12-18] mov rax, 2 (SYS_WRITE)   ← LOOP_START
//   [19-25] mov rdi, 1                   (fd = stdout)
//   [26-32] lea rsi, [rip+30]            rip_after=33, target=63 = msg
//   [33-39] mov rdx, 32                  (msg length)
//   [40-41] syscall                      rip_after=42 (code)
//   [42-48] mov rax, 9 (SYS_SLEEP)
//   [49-55] mov rdi, 200                 (~2 seconds)
//   [56-57] syscall                      rip_after=58 (code)
//   [58-62] jmp LOOP_START              rel = 12 - 63 = -51
//   [63-94] msg (32 bytes, data only)
const USER_INIT_CODE: &[u8] = &[
    // SYS_GETPID
    0x48, 0xc7, 0xc0, 0x03, 0x00, 0x00, 0x00,  // [0]  mov rax, 3
    0x0f, 0x05,                                  // [7]  syscall
    0x49, 0x89, 0xc4,                            // [9]  mov r12, rax
    // SYS_WRITE loop
    0x48, 0xc7, 0xc0, 0x02, 0x00, 0x00, 0x00,  // [12] mov rax, 2
    0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,  // [19] mov rdi, 1
    0x48, 0x8d, 0x35, 0x1e, 0x00, 0x00, 0x00,  // [26] lea rsi, [rip+30] → 63
    0x48, 0xc7, 0xc2, 0x20, 0x00, 0x00, 0x00,  // [33] mov rdx, 32
    0x0f, 0x05,                                  // [40] syscall → rip=42
    // SYS_SLEEP
    0x48, 0xc7, 0xc0, 0x09, 0x00, 0x00, 0x00,  // [42] mov rax, 9
    0x48, 0xc7, 0xc7, 0xc8, 0x00, 0x00, 0x00,  // [49] mov rdi, 200
    0x0f, 0x05,                                  // [56] syscall → rip=58
    // jmp LOOP_START: rel = 12 - (58+5) = 12 - 63 = -51 = 0xFFFF_FFCD
    0xe9, 0xcd, 0xff, 0xff, 0xff,               // [58] jmp -51
    // msg data (32 bytes, never executed, only referenced by lea)
    b'[', b'n', b'e', b'x', b'u', b's', b'-', b'i', b'n', b'i', b't',
    b']', b' ', b'H', b'e', b'l', b'l', b'o', b' ', b'f', b'r',
    b'o', b'm', b' ', b'r', b'i', b'n', b'g', b' ', b'3', b'!', b'\n',
];

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
