#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;
use alloc::string::String;
use alloc::format;

mod ipc;
mod ollama_client;
mod inference;

// Syscall numbers (must match kernel/src/syscall/mod.rs)
// Phase 4 syscalls:
const SYS_EXIT:          u64 = 1;
const SYS_WRITE:         u64 = 2;
const SYS_GETPID:        u64 = 3;
const SYS_YIELD:         u64 = 4;
const SYS_IPC_SEND:      u64 = 5;
const SYS_IPC_RECV:      u64 = 6;
const SYS_PORT_REGISTER: u64 = 7;
const SYS_PORT_FIND:     u64 = 8;
const SYS_SLEEP:         u64 = 9;
// Phase 5 syscalls:
const SYS_IPC_QUERY:     u64 = 10;
const SYS_IPC_TIMEOUT:   u64 = 11;
const SYS_GPU_MMAP:      u64 = 12;

/// Make a syscall with 3 arguments
#[inline(always)]
unsafe fn syscall3(sysno: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let result: u64;
    core::arch::asm!(
        "syscall",
        in("rax") sysno,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        out("rax") result,
        clobber_abi("C"),
    );
    result
}

/// Make a syscall with 1 argument
#[inline(always)]
unsafe fn syscall1(sysno: u64, arg1: u64) -> u64 {
    let result: u64;
    core::arch::asm!(
        "syscall",
        in("rax") sysno,
        in("rdi") arg1,
        out("rax") result,
        clobber_abi("C"),
    );
    result
}

/// Write to serial console
fn println_ipc(s: &str) {
    unsafe {
        syscall3(SYS_WRITE, 1, s.as_ptr() as u64, s.len() as u64);
    }
}

/// Entry point for nexus-ai daemon
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println_ipc("[nexus-ai] Starting AI Core daemon\n");
    
    // Get own PID
    let pid = unsafe { syscall1(SYS_GETPID, 0) };
    let pid_str = format!("[nexus-ai] PID: {}\n", pid);
    println_ipc(&pid_str);
    
    // Resolve nexus.ai port
    let port_name = b"nexus.ai";
    let port_id_result = unsafe {
        syscall3(SYS_IPC_QUERY, port_name.as_ptr() as u64, port_name.len() as u64, 0)
    } as i64;
    
    if port_id_result < 0 {
        let err_str = format!("[nexus-ai] Failed to resolve nexus.ai port: error {}\n", port_id_result);
        println_ipc(&err_str);
        loop {}
    }
    
    let port_id = port_id_result as u64;
    let port_str = format!("[nexus-ai] Bound to port_id={}\n", port_id);
    println_ipc(&port_str);
    
    println_ipc("[nexus-ai] Entering main daemon loop\n");
    
    // Main daemon loop
    loop {
        // Set blocking recv (timeout = 0 = infinite)
        unsafe { syscall1(SYS_IPC_TIMEOUT, 0); }
        
        // Receive message from IPC queue (blocking)
        let mut msg: ipc::IpcMessage = ipc::IpcMessage::new();
        let recv_result = unsafe {
            syscall3(SYS_IPC_RECV, port_id, &mut msg as *mut _ as u64, 0)
        } as i64;
        
        if recv_result < 0 {
            let err_str = format!("[nexus-ai] recv error: {}\n", recv_result);
            println_ipc(&err_str);
            continue;
        }
        
        // Handle the request
        let request_str = core::str::from_utf8(&msg.payload[..msg.len.min(256)])
            .unwrap_or("invalid_utf8");
        let req_log = format!("[nexus-ai] Received from PID {}: {}\n", msg.sender_pid, request_str);
        println_ipc(&req_log);
        
        // Generate response
        let response = inference::handle_request(&msg.payload[..msg.len]);
        let resp_log = format!("[nexus-ai] Responding: {}\n", response);
        println_ipc(&resp_log);
        
        // Send response back to sender
        let send_result = unsafe {
            syscall3(SYS_IPC_SEND, msg.sender_pid, response.as_ptr() as u64, response.len() as u64)
        } as i64;
        
        if send_result < 0 {
            let err_str = format!("[nexus-ai] send error: {}\n", send_result);
            println_ipc(&err_str);
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let msg = format!("[nexus-ai] PANIC: {:?}\n", info);
    println_ipc(&msg);
    loop {}
}