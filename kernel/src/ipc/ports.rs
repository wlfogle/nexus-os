//! NexusOS IPC Port Registry
//!
//! Named service endpoints.  A server process registers a port by name;
//! clients look up the PID and send messages to it.
//!
//! Well-known port names:
//!   "nexus.ai"    — AI Core server (Ollama integration, Phase 5)
//!   "nexus.fs"    — Filesystem server (Phase 7)
//!   "nexus.net"   — Network stack (Phase 7)
//!   "nexus.log"   — Kernel logging aggregator
//!   "nexus.term"  — NexusTerminal IPC bridge (Phase 6)

use spin::Mutex;
use super::{ipc_send, Message, IpcError, ANY};
use crate::scheduler;

const MAX_PORTS: usize = 32;
const PORT_NAME_LEN: usize = 32;

#[derive(Clone, Copy)]
struct PortEntry {
    name:    [u8; PORT_NAME_LEN],
    name_len: usize,
    owner:   u64,  // PID of the registered server (0 = free slot)
}

impl PortEntry {
    const fn empty() -> Self {
        Self { name: [0u8; PORT_NAME_LEN], name_len: 0, owner: 0 }
    }

    fn matches(&self, name: &[u8]) -> bool {
        self.owner != 0
            && self.name_len == name.len()
            && self.name[..self.name_len] == *name
    }
}

static REGISTRY: Mutex<[PortEntry; MAX_PORTS]> = Mutex::new(
    [const { PortEntry::empty() }; MAX_PORTS]
);

// ─── Public API ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortError {
    /// Port name already registered by another process.
    AlreadyRegistered,
    /// Port name too long (max PORT_NAME_LEN bytes).
    NameTooLong,
    /// No process has registered this port.
    NotFound,
    /// Port table is full (increase MAX_PORTS).
    TableFull,
    /// IPC error while sending to port.
    Ipc(IpcError),
}

/// Register a port under `name` for the calling process.
/// Returns `Err(AlreadyRegistered)` if another process owns it.
pub fn port_register(name: &[u8]) -> Result<(), PortError> {
    if name.len() > PORT_NAME_LEN { return Err(PortError::NameTooLong); }
    let my_pid = scheduler::current_id();
    let mut reg = REGISTRY.lock();

    // Check for existing registration
    if reg.iter().any(|e| e.matches(name)) {
        return Err(PortError::AlreadyRegistered);
    }

    // Find a free slot
    let slot = reg.iter_mut().find(|e| e.owner == 0)
        .ok_or(PortError::TableFull)?;

    let n = name.len();
    slot.name[..n].copy_from_slice(name);
    slot.name_len = n;
    slot.owner    = my_pid;
    Ok(())
}

/// Unregister a port.  Only the owning process may do this.
pub fn port_unregister(name: &[u8]) -> Result<(), PortError> {
    let my_pid = scheduler::current_id();
    let mut reg = REGISTRY.lock();
    let slot = reg.iter_mut()
        .find(|e| e.matches(name) && e.owner == my_pid)
        .ok_or(PortError::NotFound)?;
    *slot = PortEntry::empty();
    Ok(())
}

/// Look up the PID of the process that owns `name`.
pub fn port_find(name: &[u8]) -> Option<u64> {
    REGISTRY.lock().iter().find(|e| e.matches(name)).map(|e| e.owner)
}

/// Send a message to a named port.
/// Looks up the owner PID and calls `ipc_send`.
pub fn port_send(name: &[u8], msg: Message) -> Result<(), PortError> {
    let owner = port_find(name).ok_or(PortError::NotFound)?;
    ipc_send(owner, msg).map_err(PortError::Ipc)
}

/// List all registered ports — writes (name, owner_pid) pairs into `buf`.
/// Returns the number of entries written.
pub fn port_list(buf: &mut [(&'static str, u64)]) -> usize {
    // Can't return &str pointing into the static table safely without unsafe,
    // so we just print directly to serial here.
    let reg = REGISTRY.lock();
    let mut n = 0;
    for e in reg.iter() {
        if e.owner == 0 { continue; }
        crate::kprintln!("  port: {:?}  owner={}",
            core::str::from_utf8(&e.name[..e.name_len]).unwrap_or("?"),
            e.owner);
        n += 1;
        if n >= buf.len() { break; }
    }
    n
}
