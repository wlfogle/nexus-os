//! Phase 5 syscall handlers for AI Core and system services.
//!
//! New syscalls added in Phase 5 (numbers continue from Phase 4):
//!   SYS_IPC_QUERY   = 10  -- resolve port name -> owner PID
//!   SYS_IPC_TIMEOUT = 11  -- set per-process recv timeout (stub Phase 5.0)
//!   SYS_GPU_MMAP    = 12  -- reserve GPU buffer region   (stub Phase 5.0)

pub mod syscall_ipc_query;
pub mod syscall_ipc_timeout;
pub mod syscall_gpu_mmap;

pub use syscall_ipc_query::handle_ipc_query;
pub use syscall_ipc_timeout::handle_ipc_timeout;
pub use syscall_gpu_mmap::handle_gpu_mmap;
