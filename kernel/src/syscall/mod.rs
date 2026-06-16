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

// ─── MSR addresses ───────────────────────────────────────────────────────────

const IA32_EFER:           u32 = 0xC000_0080;
const IA32_STAR:           u32 = 0xC000_0081;
const IA32_LSTAR:          u32 = 0xC000_0082;
const IA32_FMASK:          u32 = 0xC000_0084;
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

        // 5. KERNEL_GS_BASE: points to our PerCpu struct
        //    `swapgs` on syscall entry makes GS.base = &PERCPU
        Msr::new(IA32_KERNEL_GS_BASE).write(&raw const PERCPU as u64);
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

    // After swapgs, GS.base = &PERCPU (IA32_KERNEL_GS_BASE).
    // Use GS-relative addressing (gs:[offset]) to access PERCPU fields —
    // this is the correct x86_64 idiom and avoids any linker symbol ambiguity.
    "swapgs",

    // PERCPU layout (must match PerCpu struct offsets):
    //   gs:[0]  = kernel_rsp   (offset 0)
    //   gs:[8]  = user_rsp     (offset 8)
    "mov qword ptr gs:[8], rsp",   // save user RSP to PERCPU.user_rsp
    "mov rsp, qword ptr gs:[0]",   // load kernel RSP from PERCPU.kernel_rsp

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

    // Restore user RSP (GS still active — use GS-relative read)
    "mov rsp, qword ptr gs:[8]",

    // Swap GS back to user GS before returning
    "swapgs",

    // Return to user space (restores RIP from RCX, RFLAGS from R11)
    "sysretq",
);

// ─── Syscall dispatcher (Rust) ────────────────────────────────────────────────

/// Called from `_nexus_syscall_entry` with C calling convention.
/// Returns the value to place in user-space RAX (negative = errno-style error).
#[no_mangle]
pub extern "C" fn nexus_syscall_dispatch(num: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    match num {
        // ── SYS_EXIT ──────────────────────────────────────────────────────
        SYS_EXIT => {
            let pid = scheduler::current_id();
            crate::kprintln!("[syscall] SYS_EXIT pid={} status={}", pid, a1 as i64);
            process::set_state(pid, process::ProcessState::Dead);
            ipc::inbox_free(pid);
            // Yield — scheduler will pick another process
            unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
            0
        }

        // ── SYS_WRITE ─────────────────────────────────────────────────────
        // write(fd, buf_ptr, len)
        // fd 1 = stdout (kernel serial)
        SYS_WRITE => {
            let fd  = a1;
            let ptr = a2 as *const u8;
            let len = a3 as usize;

            if fd != 1 { return -22; } // EINVAL
            if len > 4096 { return -7; } // E2BIG

            // Safety: buf_ptr is from user space.  Phase 4 runs user in same
            // address space as kernel (no separate page tables yet — Phase 5).
            let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
            if let Ok(s) = core::str::from_utf8(slice) {
                crate::kprint!("{}", s);
            }
            len as i64
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

        // ── SYS_READ_CHAR_NB ─────────────────────────────────────────────
        // Non-blocking read — returns -1 immediately if no key waiting
        SYS_READ_CHAR_NB => {
            match crate::io::keyboard::try_read() {
                Some(ch) => ch as i64,
                None     => -1,
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
                    num_sectors * crate::drivers::virtio::blk::SECTOR_SIZE,
                )
            };
            match crate::drivers::virtio::blk::read_sectors(lba, buf) {
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
                    num_sectors * crate::drivers::virtio::blk::SECTOR_SIZE,
                )
            };
            match crate::drivers::virtio::blk::write_sectors(lba, buf) {
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

        _ => {
            crate::kprintln!("[syscall] unknown syscall {} from pid={}",
                             num, scheduler::current_id());
            -38  // ENOSYS
        }
    }
}
