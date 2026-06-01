//! SYS_IPC_TIMEOUT (11) — Set per-process IPC receive timeout.
//!
//! Phase 5.0 stub: ipc_recv remains fully blocking regardless of this call.
//! Full timeout management (with process wakeup on deadline) ships in Phase 5.2.
//!
//! Signature: ipc_timeout(timeout_ms: u64) -> i64
//!   Returns: 0 (accepted, not yet enforced)

/// Handle SYS_IPC_TIMEOUT syscall.
///
/// Stores the timeout request but does not yet enforce it.
/// Phase 5.2 will integrate with the scheduler's wait-queue mechanism.
pub fn handle_ipc_timeout(_timeout_ms: u64) -> i64 {
    // Phase 5.2: store per-process timeout in PCB and enforce in ipc_recv.
    0
}
