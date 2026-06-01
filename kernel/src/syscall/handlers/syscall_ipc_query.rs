//! SYS_IPC_QUERY (10) — Resolve a named service port to its owner PID.
//!
//! This is the Phase 5 companion to SYS_PORT_FIND (8).  Semantically identical,
//! but exposed with an AI-Core-oriented name for clarity in nexus-ai code.
//!
//! Signature: ipc_query(name_ptr: u64, name_len: u64, _reserved: u64) -> i64
//!   Returns: owner PID on success, -1 if port not registered

use crate::ipc::ports;

/// Handle SYS_IPC_QUERY syscall.
pub fn handle_ipc_query(name_ptr: u64, name_len: u64, _reserved: u64) -> i64 {
    let len = (name_len as usize).min(32);
    if len == 0 || name_ptr == 0 {
        return -22; // EINVAL
    }
    let name = unsafe { core::slice::from_raw_parts(name_ptr as *const u8, len) };
    match ports::port_find(name) {
        Some(pid) => pid as i64,
        None      => -1,
    }
}
