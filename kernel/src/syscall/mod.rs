#![cfg(target_arch = "x86_64")]
//! NexusOS Syscall Interface — Phase 4 + Phase 5
//!
//! Phase 5 additions: SYS_IPC_QUERY(10), SYS_IPC_TIMEOUT(11), SYS_GPU_MMAP(12)
//! Handlers live in the `handlers` submodule.
//!
//! Implements the fast `syscall`/`sysretq` path for ring-3 → ring-0 transitions.
//!
//! Calling convention (Linux-compatible):
//!   rax = syscall number
//!   rdi, rsi, rdx = arguments 1-3  (arg4 would be r10, not used in Phase 4)
//!   Return value in rax  (negative = error)
//!
//! GDT selectors (must match gdt.rs):
//!   0x08 = kernel code (ring 0)
//!   0x10 = kernel data (ring 0)
//!   0x1B = user data   (ring 3, RPL=3)
//!   0x23 = user code   (ring 3, RPL=3)
//!
//! STAR MSR encoding for sysretq:
//!   bits [47:32] = 0x0008  → kernel CS=0x08, kernel SS=0x10  (for syscall)
//!   bits [63:48] = 0x0013  → user   CS=0x23, user   SS=0x1B  (for sysretq)
//!                             (sysretq adds 16 for CS, 8 for SS)

use core::arch::global_asm;
use x86_64::registers::model_specific::Msr;
use crate::{process, scheduler, timer};
use crate::ipc;
use crate::ipc::ports;

pub mod handlers;

// ─── Syscall numbers ──────────────────────────────────────────────────────────

pub const SYS_EXIT:          u64 = 1;
pub const SYS_WRITE:         u64 = 2;  // write(fd, buf, len) — fd 1 = serial
pub const SYS_GETPID:        u64 = 3;
pub const SYS_YIELD:         u64 = 4;
pub const SYS_IPC_SEND:      u64 = 5;  // ipc_send(to, msg_ptr, len)
pub const SYS_IPC_RECV:      u64 = 6;  // ipc_recv(from, buf_ptr, buf_len)
pub const SYS_PORT_REGISTER: u64 = 7;  // port_register(name_ptr, name_len)
pub const SYS_PORT_FIND:     u64 = 8;  // port_find(name_ptr, name_len) → pid
pub const SYS_SLEEP:         u64 = 9;  // sleep(ticks)
// ── Phase 5 ─────────────────────────────────────────────────────────────────────────────
pub const SYS_IPC_QUERY:     u64 = 10; // ipc_query(name_ptr, name_len, 0) → pid
pub const SYS_IPC_TIMEOUT:   u64 = 11; // ipc_timeout(timeout_ms)
pub const SYS_GPU_MMAP:      u64 = 12; // gpu_mmap(size, flags, 0) → vaddr
pub const SYS_READ_CHAR:     u64 = 13; // read_char() → u8 (blocks until key)
pub const SYS_READ_CHAR_NB:  u64 = 14; // read_char_nb() → u8 or -1 if empty
pub const SYS_DISK_READ:     u64 = 15; // disk_read(lba, buf_ptr, num_sectors) → 0 or -err
pub const SYS_DISK_WRITE:    u64 = 16; // disk_write(lba, buf_ptr, num_sectors) → 0 or -err
// ── Phase 6.1: ring-3 filesystem access ──────────────────────────────────────
pub const SYS_FS_LIST:       u64 = 17; // fs_list(buf_ptr, cap) → bytes (newline-separated names)
pub const SYS_FS_READ:       u64 = 18; // fs_read(name_ptr (NUL-term), buf_ptr, cap) → bytes read
// ── Phase 6: program execution ────────────────────────────────────────────────
pub const SYS_EXEC:          u64 = 19; // exec(name_ptr (NUL-term)) → child exit code or -err
// ── Phase 6.1: subdirectory path-aware filesystem access ─────────────────────
pub const SYS_FS_LIST_PATH:  u64 = 20; // fs_list_path(path_ptr (NUL-term), buf_ptr, cap) → bytes
pub const SYS_FS_READ_PATH:  u64 = 21; // fs_read_path(path_ptr (NUL-term), buf_ptr, cap) → bytes read
// ── Phase 6.2: writable VFS path operations ──────────────────────────────────
pub const SYS_FS_MKDIR_PATH: u64 = 22; // fs_mkdir_path(path_ptr) → 0 or -err
pub const SYS_FS_WRITE_PATH: u64 = 23; // fs_write_path(path_ptr, data_ptr, len) → bytes
pub const SYS_FS_APPEND_PATH:u64 = 24; // fs_append_path(path_ptr, data_ptr, len) → bytes
pub const SYS_FS_REMOVE_PATH:u64 = 25; // fs_remove_path(path_ptr) → 0 or -err
// ── Phase 6.3: pointer input ──────────────────────────────────────
pub const SYS_READ_MOUSE_NB: u64 = 26; // read_mouse_nb(buf_ptr) → 1 (event written) or 0 (none)
// ── Phase 6.4: user-space heap ──────────────────────────────────
pub const SYS_BRK: u64 = 27; // brk(new_brk) → resulting brk, or -errno. brk(0) queries current brk.
// ── Phase K2: decoupled process spawn/wait/kill ──────────────────────
pub const SYS_SPAWN: u64 = 28; // spawn(name_ptr) → child pid, or -errno. Never blocks.
pub const SYS_WAIT:  u64 = 29; // wait(pid) → that process's exit code, or -errno.
pub const SYS_KILL:  u64 = 30; // kill(pid) → 0, or -errno.
// ── Phase K3: fd-based file I/O ────────────────────────────────
pub const SYS_OPEN:  u64 = 31; // open(path_ptr, flags) → fd (>=3), or -errno.
pub const SYS_CLOSE: u64 = 32; // close(fd) → 0, or -errno.
pub const SYS_READ:  u64 = 33; // read(fd, buf_ptr, len) → bytes read, or -errno. fd must be >= 3.
pub const SYS_LSEEK: u64 = 34; // lseek(fd, offset, whence) → new offset, or -errno.
// ── Phase K5: ACPI power management ─────────────────────────
pub const SYS_REBOOT:   u64 = 35; // reboot() → never returns on success; -errno on failure.
pub const SYS_SHUTDOWN: u64 = 36; // shutdown() → never returns on success; -errno on failure.

/// SYS_OPEN flags (bitmask in arg2).
pub const O_CREAT:  u64 = 1; // create the file if it doesn't exist
pub const O_TRUNC:  u64 = 2; // truncate to zero length on open
pub const O_APPEND: u64 = 4; // start the fd's offset at the current EOF

/// SYS_LSEEK whence values (arg3).
pub const SEEK_SET: u64 = 0;
pub const SEEK_CUR: u64 = 1;
pub const SEEK_END: u64 = 2;

/// Largest program image SYS_EXEC will load from disk (1 MiB).
const MAX_PROG_BYTES: usize = 1024 * 1024;

// ─── MSR addresses ───────────────────────────────────────────────────────────

const IA32_EFER:           u32 = 0xC000_0080;
const IA32_STAR:           u32 = 0xC000_0081;
const IA32_LSTAR:          u32 = 0xC000_0082;
const IA32_FMASK:          u32 = 0xC000_0084;
const IA32_GS_BASE:        u32 = 0xC000_0101;
const IA32_KERNEL_GS_BASE: u32 = 0xC000_0102;

// ─── Per-CPU data (single CPU, accessed via GS after swapgs) ─────────────────

/// Per-CPU state for syscall entry.
/// `IA32_KERNEL_GS_BASE` points here; `swapgs` makes GS.base = &PERCPU.
#[repr(C)]
pub struct PerCpu {
    /// Top of the current process's kernel stack.
    /// Syscall entry resets RSP here so each syscall starts fresh.
    pub kernel_rsp: u64,   // offset 0
    /// Scratch slot — saves user RSP during syscall prologue.
    pub user_rsp:   u64,   // offset 8
}

/// The single CPU's per-CPU data block.
#[no_mangle]
pub static mut PERCPU: PerCpu = PerCpu { kernel_rsp: 0, user_rsp: 0 };

/// Update PERCPU.kernel_rsp to the given process's kernel stack top.
/// Called by the scheduler on every context switch to a user process.
pub fn update_kernel_rsp(pid: u64) {
    use crate::process::MAX_PROCS;
    // Access the process table to get kernel_stack_top
    // We re-export a helper from process module
    if let Some(top) = process::get_kernel_stack_top(pid) {
        unsafe { PERCPU.kernel_rsp = top; }
    }
}

fn exit_current(pid: u64, code: i64) -> i64 {
    crate::kprintln!("[syscall] SYS_EXIT pid={} status={}", pid, code);
    process::exit(pid, code);
    ipc::inbox_free(pid);
    unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
    0
}

/// Shared implementation for SYS_EXEC and SYS_SPAWN: read `name` off the
/// FAT32 root, build a private address space, load it as a static ELF64,
/// and spawn it ring-3 as a child of the calling process. Never waits —
/// callers that want blocking semantics (SYS_EXEC) loop on `try_exit_code`
/// themselves afterward.
fn do_spawn(name: &str) -> Result<u64, i64> {
    let mut buf = alloc::vec::Vec::<u8>::new();
    buf.resize(MAX_PROG_BYTES, 0);
    let n = match crate::fs::fat::read_file(name, &mut buf) {
        Ok(n)  => n,
        Err(_) => return Err(-2), // ENOENT
    };

    // Build a private address space for the child: a fresh PML4 that shares
    // the kernel higher half but has an empty user half.
    let child_pml4 = crate::memory::paging::alloc_user_pml4();

    let entry = match crate::exec::load_elf(child_pml4, &buf[..n]) {
        Ok(l)  => l.entry,
        Err(e) => {
            crate::kprintln!("[exec] load_elf failed: {}", e);
            unsafe { crate::memory::paging::free_user_pml4(child_pml4); }
            return Err(-8); // ENOEXEC
        }
    };

    let personality = if entry < crate::userspace::USER_CODE_BASE {
        process::ProcessPersonality::Linux
    } else {
        process::ProcessPersonality::Nexus
    };

    crate::exec::map_user_stack(child_pml4, crate::exec::EXEC_STACK_TOP);
    let parent = scheduler::current_id();
    let child = match process::spawn_ring3(
        b"program", entry, crate::exec::EXEC_STACK_TOP, child_pml4,
        crate::exec::EXEC_HEAP_BASE, parent, personality,
    ) {
        Some(c) => c,
        None    => {
            unsafe { crate::memory::paging::free_user_pml4(child_pml4); }
            return Err(-11); // EAGAIN — no free process slot
        }
    };
    ipc::inbox_alloc(child);
    Ok(child)
}

/// Block the calling process until `child` exits, then reap it and return
/// its exit code. Shared by SYS_EXEC and SYS_WAIT.
fn wait_and_reap(child: u64) -> i64 {
    let me = scheduler::current_id();
    loop {
        if let Some(code) = process::try_exit_code(child) {
            process::reap(child);
            return code;
        }
        process::set_state(me, process::ProcessState::BlockedOnChild);
        unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
        process::set_state(me, process::ProcessState::Running);
    }
}

fn linux_syscall_dispatch(num: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    match num {
        // Linux x86_64: write(fd, buf, len) = syscall 1.
        1 => {
            let fd  = a1;
            let ptr = a2 as *const u8;
            let len = a3 as usize;
            if fd != 1 && fd != 2 { return -22; } // EINVAL
            if len > 4096 { return -7; } // E2BIG
            let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
            if let Ok(s) = core::str::from_utf8(slice) {
                crate::kprint!("{}", s);
            }
            len as i64
        }
        // Linux x86_64: getpid() = 39.
        39 => scheduler::current_id() as i64,
        // Linux x86_64: exit(status)=60, exit_group(status)=231.
        60 | 231 => exit_current(scheduler::current_id(), a1 as i64),
        _ => {
            crate::kprintln!(
                "[linux] unsupported syscall {} from pid={}",
                num,
                scheduler::current_id(),
            );
            -38 // ENOSYS
        }
    }
}

// ─── MSR / hardware setup ────────────────────────────────────────────────────

extern "C" {
    fn _nexus_syscall_entry();
}

/// Initialise syscall hardware.  Must be called after GDT is loaded.
pub fn init() {
    unsafe {
        // 1. Enable EFER.SCE (Syscall Enable, bit 0)
        let mut efer = Msr::new(IA32_EFER);
        efer.write(efer.read() | 1);

        // 2. STAR: segment selectors for syscall/sysretq
        //    bits [47:32] = kernel CS (syscall:  CS=0x08, SS=0x10)
        //    bits [63:48] = user base  (sysretq: CS=base+16=0x23, SS=base+8=0x1B)
        let star = (0x0013u64 << 48) | (0x0008u64 << 32);
        Msr::new(IA32_STAR).write(star);

        // 3. LSTAR: entry point when user executes `syscall`
        Msr::new(IA32_LSTAR).write(_nexus_syscall_entry as u64);

        // 4. FMASK: bits to clear in RFLAGS on syscall entry
        //    0x200 = IF (interrupt flag) — disable interrupts during syscall
        Msr::new(IA32_FMASK).write(0x200);

        // 5. GS base: keep GS.base = &PERCPU at ALL times (ring 0 and ring 3).
        //    The syscall stub does NOT swapgs, which avoids the swapgs
        //    re-entrancy hazard: when one ring-3 process blocks *inside* a
        //    syscall (GS would be swapped) and the scheduler runs another
        //    ring-3 process that also issues a syscall, a swapgs-based design
        //    reads the wrong shadow MSR and loads a garbage kernel stack.
        //    We also point the swapgs-shadow MSR at &PERCPU so any stray
        //    swapgs is harmless.  User programs here do not use GS.
        let percpu = &raw const PERCPU as u64;
        Msr::new(IA32_GS_BASE).write(percpu);
        Msr::new(IA32_KERNEL_GS_BASE).write(percpu);
    }
    crate::kprintln!("[syscall] STAR/LSTAR/FMASK/EFER/KERNEL_GS_BASE configured");
}

// ─── Naked syscall entry (assembly) ──────────────────────────────────────────
//
// On entry (CPU has executed `syscall`):
//   RSP  = user stack (UNCHANGED — we must switch to kernel stack)
//   RCX  = user RIP   (saved by cpu for sysretq)
//   R11  = user RFLAGS (saved by cpu for sysretq)
//   RAX  = syscall number
//   RDI  = arg1, RSI = arg2, RDX = arg3
//   Interrupts are DISABLED (masked by FMASK)
//
// Stack layout on kernel stack after prologue:
//   [RSP+0x48]  r11    (user RFLAGS)
//   [RSP+0x40]  rcx    (user RIP)
//   [RSP+0x38]  r15
//   [RSP+0x30]  r14
//   [RSP+0x28]  r13
//   [RSP+0x20]  r12
//   [RSP+0x18]  rbx
//   [RSP+0x10]  rbp
//   [RSP+0x08]  rax    (syscall return value — overwritten by dispatcher)
//   [RSP+0x00]  (alignment)

global_asm!(
    ".global _nexus_syscall_entry",
    "_nexus_syscall_entry:",

    // GS.base is permanently &PERCPU (set in init; no swapgs — see init() note).
    // Use GS-relative addressing (gs:[offset]) to access PERCPU fields.
    // PERCPU layout (must match PerCpu struct offsets):
    //   gs:[0]  = kernel_rsp   (offset 0)
    //   gs:[8]  = user_rsp     (offset 8)
    "mov qword ptr gs:[8], rsp",   // transient stash of user RSP (IF=0: no preemption)
    "mov rsp, qword ptr gs:[0]",   // switch to this process's kernel stack

    // Save the user RSP on THIS process's kernel stack — NOT the shared
    // PERCPU.user_rsp slot — so it survives the syscall blocking and another
    // process issuing its own syscall in the meantime (e.g. SYS_EXEC's child).
    "push qword ptr gs:[8]",       // [deepest] saved user RSP
    "sub rsp, 8",                  // 16-byte alignment padding (keep call aligned)

    // Save user return state (needed for sysretq)
    "push r11",          // user RFLAGS
    "push rcx",          // user RIP

    // Save callee-saved GP registers
    "push rbp",
    "push rbx",
    "push r12",
    "push r13",
    "push r14",
    "push r15",

    // Call the Rust dispatcher:
    //   nexus_syscall_dispatch(num: u64, a1: u64, a2: u64, a3: u64) -> i64
    // C calling convention: rdi=num, rsi=a1, rdx=a2, rcx=a3
    // Syscall ABI has:      rax=num, rdi=a1, rsi=a2, rdx=a3
    "mov rcx, rdx",   // a3: rdx → rcx (saved rcx is already on stack)
    "mov rdx, rsi",   // a2: rsi → rdx
    "mov rsi, rdi",   // a1: rdi → rsi
    "mov rdi, rax",   // num: rax → rdi
    "call nexus_syscall_dispatch",
    // Return value in RAX

    // Restore callee-saved GP registers
    "pop r15",
    "pop r14",
    "pop r13",
    "pop r12",
    "pop rbx",
    "pop rbp",

    // Restore user return state
    "pop rcx",           // user RIP  → RCX (used by sysretq)
    "pop r11",           // user RFLAGS → R11 (used by sysretq)

    // Restore user RSP from THIS process's kernel stack (per-process, safe
    // across blocking/preemption).  Drop the alignment padding first.
    "add rsp, 8",                  // discard alignment padding
    "pop rsp",                     // restore user RSP

    // Return to user space (restores RIP from RCX, RFLAGS from R11).
    // No swapgs: GS.base remains &PERCPU in ring 3 as well.
    "sysretq",
);

// ─── Syscall dispatcher (Rust) ────────────────────────────────────────────────

/// Called from `_nexus_syscall_entry` with C calling convention.
/// Returns the value to place in user-space RAX (negative = errno-style error).
#[no_mangle]
pub extern "C" fn nexus_syscall_dispatch(num: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    if process::get_personality(scheduler::current_id()) == process::ProcessPersonality::Linux {
        return linux_syscall_dispatch(num, a1, a2, a3);
    }
    match num {
        // ── SYS_EXIT ──────────────────────────────────────────────────────
        SYS_EXIT => {
            let pid  = scheduler::current_id();
            let code = a1 as i64;
            exit_current(pid, code)
        }

        // ── SYS_WRITE ───────────────────────────────────
        // write(fd, buf_ptr, len)
        // fd 1 = stdout (kernel serial); fd >= 3 = an fd opened via SYS_OPEN,
        // written at (and advancing) its saved offset.
        SYS_WRITE => {
            let fd  = a1;
            let ptr = a2 as *const u8;
            let len = a3 as usize;

            if fd == 1 {
                if len > 4096 { return -7; } // E2BIG
                // Safety: buf_ptr is from the calling process's user half,
                // mapped in the address space active during this syscall.
                let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
                if let Ok(s) = core::str::from_utf8(slice) {
                    crate::kprint!("{}", s);
                }
                return len as i64;
            }
            if fd < 3 { return -22; } // EINVAL — fd 0/2 not writable via this path

            let pid = scheduler::current_id();
            let (path_buf, path_len, offset) = match process::fd_info(pid, fd) {
                Some(v) => v,
                None    => return -9, // EBADF
            };
            let path = match core::str::from_utf8(&path_buf[..path_len as usize]) {
                Ok(s)  => s,
                Err(_) => return -22,
            };
            let data = unsafe { core::slice::from_raw_parts(ptr, len) };
            match crate::fs::vfs::write_at(path, offset, data) {
                Ok(n) => {
                    process::fd_set_offset(pid, fd, offset + n as u64);
                    n as i64
                }
                Err(_) => -5, // EIO
            }
        }

        // ── SYS_GETPID ────────────────────────────────────────────────────
        SYS_GETPID => scheduler::current_id() as i64,

        // ── SYS_YIELD ─────────────────────────────────────────────────────
        SYS_YIELD => {
            let pid = scheduler::current_id();
            process::set_state(pid, process::ProcessState::Ready);
            unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
            0
        }

        // ── SYS_IPC_SEND ──────────────────────────────────────────────────
        // ipc_send(to_pid, msg_ptr, msg_len)
        SYS_IPC_SEND => {
            let to  = a1;
            let ptr = a2 as *const u8;
            let len = (a3 as usize).min(ipc::MSG_DATA_SIZE);
            let mut msg = ipc::Message::new(to, 0);
            unsafe {
                core::ptr::copy_nonoverlapping(ptr, msg.data.as_mut_ptr(), len);
            }
            msg.len = len as u32;
            match ipc::ipc_send(to, msg) {
                Ok(())  => 0,
                Err(_)  => -1,
            }
        }

        // ── SYS_IPC_RECV ──────────────────────────────────────────────────
        // ipc_recv(from_filter, buf_ptr, buf_len) → bytes copied
        SYS_IPC_RECV => {
            let from_filter = a1;
            let ptr = a2 as *mut u8;
            let cap = a3 as usize;
            let mut msg = ipc::Message::new(0, 0);
            match ipc::ipc_recv(from_filter, &mut msg) {
                Ok(()) => {
                    let n = (msg.len as usize).min(cap);
                    unsafe {
                        core::ptr::copy_nonoverlapping(msg.data.as_ptr(), ptr, n);
                    }
                    n as i64
                }
                Err(_) => -1,
            }
        }

        // ── SYS_PORT_REGISTER ─────────────────────────────────────────────
        // port_register(name_ptr, name_len)
        SYS_PORT_REGISTER => {
            let ptr = a1 as *const u8;
            let len = (a2 as usize).min(32);
            let name = unsafe { core::slice::from_raw_parts(ptr, len) };
            match ports::port_register(name) {
                Ok(()) => 0,
                Err(_) => -1,
            }
        }

        // ── SYS_PORT_FIND ─────────────────────────────────────────────────
        // port_find(name_ptr, name_len) → pid or -1
        SYS_PORT_FIND => {
            let ptr = a1 as *const u8;
            let len = (a2 as usize).min(32);
            let name = unsafe { core::slice::from_raw_parts(ptr, len) };
            match ports::port_find(name) {
                Some(pid) => pid as i64,
                None      => -1,
            }
        }

        // ── SYS_SLEEP ─────────────────────────────────────────────────────
        // sleep(ticks)
        SYS_SLEEP => {
            let ticks = a1;
            let start = timer::ticks();
            while timer::ticks().wrapping_sub(start) < ticks {
                unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
            }
            0
        }

        // ── Phase 5 syscalls ──────────────────────────────────────────────────────────

        // ── SYS_IPC_QUERY ────────────────────────────────────────────────────
        SYS_IPC_QUERY => handlers::handle_ipc_query(a1, a2, a3 as u64),

        // ── SYS_IPC_TIMEOUT ────────────────────────────────────────────────
        SYS_IPC_TIMEOUT => handlers::handle_ipc_timeout(a1),

        // ── SYS_GPU_MMAP ───────────────────────────────────────────────────
        SYS_GPU_MMAP => handlers::handle_gpu_mmap(a1, a2, a3 as u64),

        // ── SYS_READ_CHAR ───────────────────────────────────────────────
        // Blocking read — blocks until a key is available
        SYS_READ_CHAR => {
            let my_id = scheduler::current_id();
            loop {
                if let Some(ch) = crate::io::keyboard::try_read() {
                    return ch as i64;
                }
                // Block this process until IRQ1 wakes it
                process::set_state(my_id, process::ProcessState::BlockedOnKey);
                unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
                process::set_state(my_id, process::ProcessState::Running);
            }
        }

        // ── SYS_READ_CHAR_NB ───────────────────────────────────
        // Non-blocking read — returns -1 immediately if no key waiting
        SYS_READ_CHAR_NB => {
            match crate::io::keyboard::try_read() {
                Some(ch) => ch as i64,
                None     => -1,
            }
        }

        // ── SYS_READ_MOUSE_NB ─────────────────────────────────
        // read_mouse_nb(buf_ptr) → 1 if an event was written to buf_ptr (5
        // bytes: dx:i16 LE, dy:i16 LE, buttons:u8 with bit0=left, bit1=right,
        // bit2=middle), or 0 if no event is queued. Non-blocking, matching
        // SYS_READ_CHAR_NB's shape rather than SYS_READ_CHAR's blocking one
        // — a full unified blocking key+mouse event queue is follow-up work.
        SYS_READ_MOUSE_NB => {
            match crate::io::mouse::try_read() {
                Some(ev) => {
                    let buf_ptr = a1 as *mut u8;
                    let dx = ev.dx.to_le_bytes();
                    let dy = ev.dy.to_le_bytes();
                    let buttons = (ev.left as u8) | ((ev.right as u8) << 1) | ((ev.middle as u8) << 2);
                    unsafe {
                        core::ptr::write(buf_ptr, dx[0]);
                        core::ptr::write(buf_ptr.add(1), dx[1]);
                        core::ptr::write(buf_ptr.add(2), dy[0]);
                        core::ptr::write(buf_ptr.add(3), dy[1]);
                        core::ptr::write(buf_ptr.add(4), buttons);
                    }
                    1
                }
                None => 0,
            }
        }

        // ── SYS_DISK_READ ────────────────────────────────────────────────
        // disk_read(lba: u64, buf_ptr: *mut u8, num_sectors: u64) → 0 or -EIO
        SYS_DISK_READ => {
            let lba         = a1;
            let buf_ptr     = a2 as *mut u8;
            let num_sectors = a3 as usize;
            if num_sectors == 0 { return 0; }
            let buf = unsafe {
                core::slice::from_raw_parts_mut(
                    buf_ptr,
                    num_sectors * crate::drivers::blockdev::SECTOR_SIZE,
                )
            };
            match crate::drivers::blockdev::read_sectors(lba, buf) {
                Ok(())  => 0,
                Err(_)  => -5, // EIO
            }
        }

        // ── SYS_DISK_WRITE ───────────────────────────────────────────────
        // disk_write(lba: u64, buf_ptr: *const u8, num_sectors: u64) → 0 or -EIO
        SYS_DISK_WRITE => {
            let lba         = a1;
            let buf_ptr     = a2 as *const u8;
            let num_sectors = a3 as usize;
            if num_sectors == 0 { return 0; }
            let buf = unsafe {
                core::slice::from_raw_parts(
                    buf_ptr,
                    num_sectors * crate::drivers::blockdev::SECTOR_SIZE,
                )
            };
            match crate::drivers::blockdev::write_sectors(lba, buf) {
                Ok(())  => 0,
                Err(_)  => -5, // EIO
            }
        }

        // ── SYS_FS_LIST ──────────────────────────────────────────────────
        // fs_list(buf_ptr, cap) → bytes written (newline-separated names)
        SYS_FS_LIST => {
            let buf_ptr = a1 as *mut u8;
            let cap     = a2 as usize;
            if cap == 0 { return 0; }
            let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, cap) };
            match crate::fs::fat::list_root(buf) {
                Ok(n)  => n as i64,
                Err(_) => -5, // EIO / not mounted
            }
        }

        // ── SYS_FS_READ ──────────────────────────────────────────────────
        // fs_read(name_ptr (NUL-terminated), buf_ptr, cap) → bytes read
        SYS_FS_READ => {
            let name_ptr = a1 as *const u8;
            let buf_ptr  = a2 as *mut u8;
            let cap      = a3 as usize;
            if cap == 0 { return 0; }
            // Bounded scan of the NUL-terminated filename (max 255 bytes).
            let mut nlen = 0usize;
            unsafe {
                while nlen < 255 && *name_ptr.add(nlen) != 0 { nlen += 1; }
            }
            let name_slice = unsafe { core::slice::from_raw_parts(name_ptr, nlen) };
            let name = match core::str::from_utf8(name_slice) {
                Ok(s)  => s,
                Err(_) => return -22, // EINVAL
            };
            let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, cap) };
            match crate::fs::fat::read_file(name, buf) {
                Ok(n)  => n as i64,
                Err(_) => -2, // ENOENT
            }
        }

        // ── SYS_FS_LIST_PATH ─────────────────────────────────────────────
        // fs_list_path(path_ptr (NUL-terminated, <=255), buf_ptr, cap)
        //   → bytes written (newline-separated names). Path is `/`-separated;
        //     `/` or empty lists the volume root.
        SYS_FS_LIST_PATH => {
            let path_ptr = a1 as *const u8;
            let buf_ptr  = a2 as *mut u8;
            let cap      = a3 as usize;
            if cap == 0 { return 0; }
            // Bounded scan of the NUL-terminated path (max 255 bytes).
            let mut plen = 0usize;
            unsafe {
                while plen < 255 && *path_ptr.add(plen) != 0 { plen += 1; }
            }
            let path = match core::str::from_utf8(
                unsafe { core::slice::from_raw_parts(path_ptr, plen) }
            ) {
                Ok(s)  => s,
                Err(_) => return -22, // EINVAL
            };
            let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, cap) };
            match crate::fs::vfs::list(path, buf) {
                Ok(n)  => n as i64,
                Err(_) => -2, // ENOENT / not a directory / not mounted
            }
        }

        // ── SYS_FS_READ_PATH ─────────────────────────────────────────────
        // fs_read_path(path_ptr (NUL-terminated, <=255), buf_ptr, cap)
        //   → bytes read. Path is `/`-separated and may name a file in any
        //     subdirectory (e.g. /EFI/BOOT/limine.conf).
        SYS_FS_READ_PATH => {
            let path_ptr = a1 as *const u8;
            let buf_ptr  = a2 as *mut u8;
            let cap      = a3 as usize;
            if cap == 0 { return 0; }
            // Bounded scan of the NUL-terminated path (max 255 bytes).
            let mut plen = 0usize;
            unsafe {
                while plen < 255 && *path_ptr.add(plen) != 0 { plen += 1; }
            }
            let path = match core::str::from_utf8(
                unsafe { core::slice::from_raw_parts(path_ptr, plen) }
            ) {
                Ok(s)  => s,
                Err(_) => return -22, // EINVAL
            };
            let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, cap) };
            match crate::fs::vfs::read(path, buf) {
                Ok(n)  => n as i64,
                Err(_) => -2, // ENOENT
            }
        }

        // ── SYS_FS_MKDIR_PATH ────────────────────────────────────────────
        // fs_mkdir_path(path_ptr (NUL-terminated, <=255)) → 0 or -err.
        SYS_FS_MKDIR_PATH => {
            let path_ptr = a1 as *const u8;
            let mut plen = 0usize;
            unsafe {
                while plen < 255 && *path_ptr.add(plen) != 0 { plen += 1; }
            }
            let path = match core::str::from_utf8(
                unsafe { core::slice::from_raw_parts(path_ptr, plen) }
            ) {
                Ok(s)  => s,
                Err(_) => return -22,
            };
            match crate::fs::vfs::mkdir(path) {
                Ok(()) => 0,
                Err(_) => -5,
            }
        }

        // ── SYS_FS_WRITE_PATH ────────────────────────────────────────────
        // fs_write_path(path_ptr, data_ptr, len) → bytes written or -err.
        SYS_FS_WRITE_PATH => {
            let path_ptr = a1 as *const u8;
            let data_ptr = a2 as *const u8;
            let len      = (a3 as usize).min(4096);
            let mut plen = 0usize;
            unsafe {
                while plen < 255 && *path_ptr.add(plen) != 0 { plen += 1; }
            }
            let path = match core::str::from_utf8(
                unsafe { core::slice::from_raw_parts(path_ptr, plen) }
            ) {
                Ok(s)  => s,
                Err(_) => return -22,
            };
            let data = unsafe { core::slice::from_raw_parts(data_ptr, len) };
            match crate::fs::vfs::write(path, data) {
                Ok(()) => len as i64,
                Err(_) => -5,
            }
        }

        // ── SYS_FS_APPEND_PATH ───────────────────────────────────────────
        // fs_append_path(path_ptr, data_ptr, len) → bytes written or -err.
        SYS_FS_APPEND_PATH => {
            let path_ptr = a1 as *const u8;
            let data_ptr = a2 as *const u8;
            let len      = (a3 as usize).min(4096);
            let mut plen = 0usize;
            unsafe {
                while plen < 255 && *path_ptr.add(plen) != 0 { plen += 1; }
            }
            let path = match core::str::from_utf8(
                unsafe { core::slice::from_raw_parts(path_ptr, plen) }
            ) {
                Ok(s)  => s,
                Err(_) => return -22,
            };
            let data = unsafe { core::slice::from_raw_parts(data_ptr, len) };
            match crate::fs::vfs::append(path, data) {
                Ok(()) => len as i64,
                Err(_) => -5,
            }
        }

        // ── SYS_FS_REMOVE_PATH ───────────────────────────────────────────
        // fs_remove_path(path_ptr (NUL-terminated, <=255)) → 0 or -err.
        SYS_FS_REMOVE_PATH => {
            let path_ptr = a1 as *const u8;
            let mut plen = 0usize;
            unsafe {
                while plen < 255 && *path_ptr.add(plen) != 0 { plen += 1; }
            }
            let path = match core::str::from_utf8(
                unsafe { core::slice::from_raw_parts(path_ptr, plen) }
            ) {
                Ok(s)  => s,
                Err(_) => return -22,
            };
            match crate::fs::vfs::remove(path) {
                Ok(()) => 0,
                Err(_) => -5,
            }
        }

        // ── SYS_EXEC ──────────────────────────────────────
        // exec(name_ptr (NUL-terminated)) → child exit code, or negative error.
        // Loads a static ELF64 from the FAT32 root, spawns it as a ring-3
        // process, and blocks the caller until the child exits. Equivalent to
        // SYS_SPAWN immediately followed by SYS_WAIT on the returned pid —
        // kept as a single syscall since "run a program and wait for it" is
        // the common case and every existing caller (the shell's `run`) wants
        // exactly that.
        SYS_EXEC => {
            let name_ptr = a1 as *const u8;
            let mut nlen = 0usize;
            unsafe {
                while nlen < 255 && *name_ptr.add(nlen) != 0 { nlen += 1; }
            }
            let name = match core::str::from_utf8(
                unsafe { core::slice::from_raw_parts(name_ptr, nlen) }
            ) {
                Ok(s)  => s,
                Err(_) => return -22, // EINVAL
            };
            match do_spawn(name) {
                Ok(child) => wait_and_reap(child),
                Err(e)    => e,
            }
        }

        // ── SYS_SPAWN ───────────────────────────────────
        // spawn(name_ptr (NUL-terminated)) → child pid, or negative error.
        // Same load path as SYS_EXEC but returns immediately — the caller is
        // free to keep running (or spawn more children) and collect the
        // result later with SYS_WAIT, from a completely different point in
        // its own execution. This is what makes background jobs possible.
        SYS_SPAWN => {
            let name_ptr = a1 as *const u8;
            let mut nlen = 0usize;
            unsafe {
                while nlen < 255 && *name_ptr.add(nlen) != 0 { nlen += 1; }
            }
            let name = match core::str::from_utf8(
                unsafe { core::slice::from_raw_parts(name_ptr, nlen) }
            ) {
                Ok(s)  => s,
                Err(_) => return -22, // EINVAL
            };
            match do_spawn(name) {
                Ok(child) => child as i64,
                Err(e)    => e,
            }
        }

        // ── SYS_WAIT ────────────────────────────────────
        // wait(pid) → that process's exit code, or -10 (ECHILD) if `pid` is
        // not a live-or-exited-but-unreaped process. Blocks until `pid`
        // exits if it's still running — works whether `pid` already exited
        // (a SYS_SPAWN result collected late) or exits sometime after this
        // call, since the exit code lives on the child's own Zombie slot
        // rather than a wait registration that had to be armed in advance.
        SYS_WAIT => {
            let pid = a1;
            if pid == 0 || process::get_state(pid) == process::ProcessState::Dead {
                return -10; // ECHILD — never existed, or already reaped
            }
            wait_and_reap(pid)
        }

        // ── SYS_KILL ────────────────────────────────────
        // kill(pid) → 0, or -err. Forcibly terminates `pid` from outside its
        // own context (unlike SYS_EXIT, which a process calls on itself).
        // Safe on a single core: the caller is by definition the only process
        // running right now, so `pid` is guaranteed Ready/Running-but-not-
        // scheduled/Blocked*, never mid-execution — there is no "the target
        // was between instructions" race to worry about. No permission model
        // exists yet (any process may kill any other); that's a known gap
        // for whenever multiple mutually-untrusting programs are a real
        // scenario, not before.
        SYS_KILL => {
            let pid = a1;
            let me  = scheduler::current_id();
            if pid == me {
                return -22; // EINVAL — use SYS_EXIT to terminate yourself
            }
            match process::get_state(pid) {
                process::ProcessState::Dead   => -3,  // ESRCH — no such process
                process::ProcessState::Zombie => 0,   // already exited — idempotent
                _ => {
                    ipc::inbox_free(pid);
                    process::exit(pid, -9); // -9 mirrors SIGKILL as a sentinel exit code
                    0
                }
            }
        }

        // ── SYS_BRK ──────────────────────────────────────
        // brk(new_brk) → resulting brk (u64), or -errno.  brk(0) is a pure
        // query (returns the current brk without changing anything) since 0
        // is never a valid heap address — every heap region starts well above
        // the null/low guard page (see USER_HEAP_BASE / EXEC_HEAP_BASE).
        //
        // Grows/shrinks the process's own heap mapping in-place: pages between
        // the page-aligned extents of the old and new brk are mapped (via a
        // freshly zeroed frame) or unmapped-and-freed. Never touches any page
        // outside that delta, so a no-op brk (new_brk == current brk) does
        // zero allocation work.
        SYS_BRK => {
            let pid = scheduler::current_id();
            let (base, cur_brk) = process::get_heap(pid);
            let pml4 = process::get_pml4(pid);
            if base == 0 || pml4 == 0 {
                return -38; // ENOSYS — process has no reserved heap region
            }
            let requested = a1;
            if requested == 0 {
                return cur_brk as i64;
            }
            // Per-process heap cap. Generous for a shell/small native program;
            // revisit once real userspace programs need more.
            const HEAP_MAX: u64 = 64 * 1024 * 1024;
            if requested < base || requested > base + HEAP_MAX {
                return -12; // ENOMEM
            }

            const PAGE: u64 = 4096;
            let page_align_up = |x: u64| (x + PAGE - 1) & !(PAGE - 1);
            let old_end = page_align_up(cur_brk);
            let new_end = page_align_up(requested);

            if new_end > old_end {
                let mut v = old_end;
                while v < new_end {
                    let frame = crate::memory::physical::alloc_frame();
                    unsafe {
                        core::ptr::write_bytes(
                            crate::memory::paging::phys_to_virt(frame) as *mut u8,
                            0,
                            PAGE as usize,
                        );
                    }
                    crate::memory::paging::map_page_in(
                        pml4, v, frame,
                        crate::memory::paging::flags::PRESENT
                            | crate::memory::paging::flags::WRITABLE
                            | crate::memory::paging::flags::USER
                            | crate::memory::paging::flags::NO_EXECUTE,
                    );
                    v += PAGE;
                }
            } else if new_end < old_end {
                let mut v = new_end;
                while v < old_end {
                    if let Some(phys) = crate::memory::paging::unmap_page_in(pml4, v) {
                        unsafe { crate::memory::physical::free_frame(phys); }
                    }
                    v += PAGE;
                }
            }

            process::set_heap_brk(pid, requested);
            requested as i64
        }

        // ── SYS_OPEN ───────────────────────────────────
        // open(path_ptr (NUL-term, <=255), flags) → fd (>= 3), or -errno.
        // flags: O_CREAT=1, O_TRUNC=2, O_APPEND=4 (see constants above).
        SYS_OPEN => {
            let path_ptr = a1 as *const u8;
            let flags    = a2;
            let mut plen = 0usize;
            unsafe {
                while plen < 255 && *path_ptr.add(plen) != 0 { plen += 1; }
            }
            let path = match core::str::from_utf8(
                unsafe { core::slice::from_raw_parts(path_ptr, plen) }
            ) {
                Ok(s)  => s,
                Err(_) => return -22, // EINVAL
            };
            let create   = flags & O_CREAT  != 0;
            let truncate = flags & O_TRUNC  != 0;
            let append   = flags & O_APPEND != 0;
            let rel = match crate::fs::vfs::open(path, create, truncate) {
                Ok(rel) => rel,
                Err(_)  => return -2, // ENOENT
            };
            let start_offset = if append {
                crate::fs::vfs::size(path).unwrap_or(0)
            } else {
                0
            };
            let pid = scheduler::current_id();
            match process::open_fd(pid, rel, start_offset) {
                Some(fd) => fd as i64,
                None     => -24, // EMFILE — no free fd slot
            }
        }

        // ── SYS_CLOSE ──────────────────────────────────
        // close(fd) → 0, or -errno.
        SYS_CLOSE => {
            let pid = scheduler::current_id();
            if process::close_fd(pid, a1) { 0 } else { -9 } // EBADF
        }

        // ── SYS_READ ───────────────────────────────────
        // read(fd, buf_ptr, len) → bytes read, or -errno. fd must be an fd
        // opened via SYS_OPEN (>= 3) — keyboard input still goes through
        // SYS_READ_CHAR/_NB, not this syscall.
        SYS_READ => {
            let pid     = scheduler::current_id();
            let fd      = a1;
            let buf_ptr = a2 as *mut u8;
            let len     = a3 as usize;
            let (path_buf, path_len, offset) = match process::fd_info(pid, fd) {
                Some(v) => v,
                None    => return -9, // EBADF
            };
            let path = match core::str::from_utf8(&path_buf[..path_len as usize]) {
                Ok(s)  => s,
                Err(_) => return -22,
            };
            let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, len) };
            match crate::fs::vfs::read_at(path, offset, buf) {
                Ok(n) => {
                    process::fd_set_offset(pid, fd, offset + n as u64);
                    n as i64
                }
                Err(_) => -5, // EIO
            }
        }

        // ── SYS_LSEEK ──────────────────────────────────
        // lseek(fd, offset, whence) → new offset, or -errno.
        // whence: SEEK_SET=0, SEEK_CUR=1, SEEK_END=2.
        SYS_LSEEK => {
            let pid     = scheduler::current_id();
            let fd      = a1;
            let off_arg = a2 as i64;
            let whence  = a3;
            let (path_buf, path_len, cur_offset) = match process::fd_info(pid, fd) {
                Some(v) => v,
                None    => return -9, // EBADF
            };
            let path = match core::str::from_utf8(&path_buf[..path_len as usize]) {
                Ok(s)  => s,
                Err(_) => return -22,
            };
            let base: i64 = match whence {
                SEEK_SET => 0,
                SEEK_CUR => cur_offset as i64,
                SEEK_END => match crate::fs::vfs::size(path) {
                    Ok(s)  => s as i64,
                    Err(_) => return -5,
                },
                _ => return -22,
            };
            let new_offset = base + off_arg;
            if new_offset < 0 { return -22; }
            process::fd_set_offset(pid, fd, new_offset as u64);
            new_offset
        }

        // ── SYS_REBOOT ───────────────────────────────────
        // reboot() → never returns — crate::acpi::reboot() always finds a way
        // (ACPI reset register, or the universal 8042 controller pulse
        // fallback) or halts as a last resort. No failure path exists here.
        SYS_REBOOT => {
            crate::acpi::reboot();
        }

        // ── SYS_SHUTDOWN ─────────────────────────────
        // shutdown() → never returns on success; -1 if ACPI S5 power-off
        // isn't available/parseable on this machine (no universal fallback
        // exists for power-off the way the 8042 pulse covers reboot).
        SYS_SHUTDOWN => {
            match crate::acpi::shutdown() {
                Ok(()) => 0,
                Err(e) => {
                    crate::kprintln!("[syscall] SYS_SHUTDOWN failed: {}", e);
                    -1
                }
            }
        }

        _ => {
            crate::kprintln!("[syscall] unknown syscall {} from pid={}",
                             num, scheduler::current_id());
            -38  // ENOSYS
        }
    }
}
