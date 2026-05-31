//! NexusOS IPC — Message-Passing Subsystem (Phase 3)
//!
//! The microkernel's core communication mechanism.  Every subsystem —
//! AI Core, filesystem, network, NexusTerminal — communicates through
//! this interface rather than direct function calls.
//!
//! Design:
//!   • Fixed-size messages (header + 240-byte payload)
//!   • Per-process inbox ring-buffer (depth = 8 messages)
//!   • Blocking send / receive with scheduler integration
//!   • Non-blocking try_send / try_recv for polling
//!   • Named port registry for service discovery
//!
//! Blocking model (single-CPU):
//!   ipc_recv on empty inbox → set BlockedOnRecv, `sti;hlt` loop
//!   ipc_send on full inbox  → set BlockedOnSend, `sti;hlt` loop
//!   Sender wakes receiver by setting its state back to Ready.
//!   The timer ISR fires, scheduler picks the now-Ready receiver.

pub mod ports;

use spin::Mutex;
use crate::process::{self, ProcessState};
use crate::scheduler;

// ─── Message format ───────────────────────────────────────────────────────────

/// Maximum payload size in bytes.
pub const MSG_DATA_SIZE: usize = 240;

/// Message queue depth per process.
pub const QUEUE_DEPTH: usize = 8;

/// Wildcard sender filter for `ipc_recv` — accept messages from any process.
pub const ANY: u64 = 0;

// ── Message type constants ────────────────────────────────────────────────────
// Low 16 bits: category.  High 16 bits: subtype.

pub const MSG_PING:        u32 = 0x0001; // heartbeat request
pub const MSG_PONG:        u32 = 0x0002; // heartbeat response
pub const MSG_LOG:         u32 = 0x0010; // log a string (data = UTF-8 bytes)
pub const MSG_AI_REQUEST:  u32 = 0x0100; // send prompt to AI Core
pub const MSG_AI_RESPONSE: u32 = 0x0101; // AI Core response
pub const MSG_FS_READ:     u32 = 0x0200; // filesystem read request
pub const MSG_FS_WRITE:    u32 = 0x0201; // filesystem write request
pub const MSG_FS_REPLY:    u32 = 0x0202; // filesystem operation reply
pub const MSG_NET_SEND:    u32 = 0x0300; // network send request
pub const MSG_NET_RECV:    u32 = 0x0301; // network received packet

/// IPC message — fixed-size, copyable.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Message {
    /// Sender's process ID (filled in automatically by `ipc_send`).
    pub from:     u64,
    /// Intended recipient's process ID.
    pub to:       u64,
    /// Message type tag — use the `MSG_*` constants above.
    pub msg_type: u32,
    /// Number of valid bytes in `data` (0 = no payload).
    pub len:      u32,
    /// Payload — interpret based on `msg_type`.
    pub data:     [u8; MSG_DATA_SIZE],
}

impl Message {
    /// Construct a new message with no payload.
    pub const fn new(to: u64, msg_type: u32) -> Self {
        Self {
            from: 0,
            to,
            msg_type,
            len: 0,
            data: [0u8; MSG_DATA_SIZE],
        }
    }

    /// Construct a message with a text payload (truncated to MSG_DATA_SIZE).
    pub fn with_str(to: u64, msg_type: u32, text: &str) -> Self {
        let mut m = Self::new(to, msg_type);
        let bytes = text.as_bytes();
        let n = bytes.len().min(MSG_DATA_SIZE);
        m.data[..n].copy_from_slice(&bytes[..n]);
        m.len = n as u32;
        m
    }

    /// Return the payload as a UTF-8 string slice (best-effort).
    pub fn as_str(&self) -> &str {
        let n = (self.len as usize).min(MSG_DATA_SIZE);
        core::str::from_utf8(&self.data[..n]).unwrap_or("<invalid utf8>")
    }
}

// ─── Per-process inbox ────────────────────────────────────────────────────────

struct Inbox {
    owner: u64,                       // process ID that owns this inbox (0 = free)
    buf:   [Message; QUEUE_DEPTH],
    head:  usize,                     // next slot to read
    tail:  usize,                     // next slot to write
    len:   usize,                     // current number of messages
}

impl Inbox {
    const fn empty() -> Self {
        Self {
            owner: 0,
            buf:   [const { Message::new(0, 0) }; QUEUE_DEPTH],
            head:  0,
            tail:  0,
            len:   0,
        }
    }

    fn push(&mut self, msg: Message) -> bool {
        if self.len == QUEUE_DEPTH { return false; }
        self.buf[self.tail] = msg;
        self.tail = (self.tail + 1) % QUEUE_DEPTH;
        self.len += 1;
        true
    }

    /// Pop a message matching `from_filter` (or ANY).
    fn pop(&mut self, from_filter: u64) -> Option<Message> {
        if self.len == 0 { return None; }

        if from_filter == ANY {
            // Fast path — take the head
            let msg = self.buf[self.head];
            self.head = (self.head + 1) % QUEUE_DEPTH;
            self.len -= 1;
            return Some(msg);
        }

        // Search for a matching message
        for i in 0..self.len {
            let idx = (self.head + i) % QUEUE_DEPTH;
            if self.buf[idx].from == from_filter {
                let msg = self.buf[idx];
                // Compact the ring: shift remaining entries left
                for j in i..self.len - 1 {
                    let a = (self.head + j) % QUEUE_DEPTH;
                    let b = (self.head + j + 1) % QUEUE_DEPTH;
                    self.buf[a] = self.buf[b];
                }
                self.len -= 1;
                self.tail = (self.tail + QUEUE_DEPTH - 1) % QUEUE_DEPTH;
                return Some(msg);
            }
        }
        None
    }

    fn is_full(&self) -> bool { self.len == QUEUE_DEPTH }
}

// ─── Inbox table ──────────────────────────────────────────────────────────────
// One inbox slot per possible process ID (indexed by pid-1).
// MAX_INBOXES must equal process::MAX_PROCS.

const MAX_INBOXES: usize = 64;

static INBOXES: Mutex<[Inbox; MAX_INBOXES]> = Mutex::new(
    [const { Inbox::empty() }; MAX_INBOXES]
);

/// Allocate an inbox for a newly-spawned process.
/// Called by the IPC layer after a process is created.
pub fn inbox_alloc(pid: u64) {
    let idx = ((pid - 1) as usize) % MAX_INBOXES;
    let mut inboxes = INBOXES.lock();
    inboxes[idx].owner = pid;
    inboxes[idx].head  = 0;
    inboxes[idx].tail  = 0;
    inboxes[idx].len   = 0;
}

/// Free a process's inbox when it exits.
pub fn inbox_free(pid: u64) {
    let idx = ((pid - 1) as usize) % MAX_INBOXES;
    INBOXES.lock()[idx].owner = 0;
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn inbox_idx(pid: u64) -> Option<usize> {
    if pid == 0 { return None; }
    let idx = ((pid - 1) as usize) % MAX_INBOXES;
    // Verify ownership to detect stale reuse
    let inboxes = INBOXES.lock();
    if inboxes[idx].owner == pid { Some(idx) } else { None }
}

fn enqueue(to: u64, msg: Message) -> bool {
    let idx = match inbox_idx(to) {
        Some(i) => i,
        None    => return false,
    };
    INBOXES.lock()[idx].push(msg)
}

fn dequeue(my_pid: u64, from_filter: u64) -> Option<Message> {
    let idx = match inbox_idx(my_pid) {
        Some(i) => i,
        None    => return None,
    };
    INBOXES.lock()[idx].pop(from_filter)
}

fn inbox_full(pid: u64) -> bool {
    match inbox_idx(pid) {
        Some(i) => INBOXES.lock()[i].is_full(),
        None    => true,
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// IPC error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    /// Destination process does not exist or has no inbox.
    NoSuchProcess,
    /// Inbox is full (non-blocking send only).
    QueueFull,
    /// No message available (non-blocking receive only).
    WouldBlock,
}

/// Send a message, blocking until the destination's inbox has space.
///
/// Automatically fills `msg.from` with the caller's PID.
/// Returns `Err(NoSuchProcess)` if the destination has no inbox.
pub fn ipc_send(to: u64, mut msg: Message) -> Result<(), IpcError> {
    let my_id = scheduler::current_id();
    msg.from = my_id;
    msg.to   = to;

    // Fast path — queue has space
    if enqueue(to, msg) {
        // Wake receiver if it was blocked waiting for a message
        if process::get_state(to) == ProcessState::BlockedOnRecv {
            process::set_state(to, ProcessState::Ready);
        }
        return Ok(());
    }

    // Slow path — inbox full, block until space opens
    if inbox_idx(to).is_none() {
        return Err(IpcError::NoSuchProcess);
    }

    loop {
        process::set_state(my_id, ProcessState::BlockedOnSend);
        // Yield CPU — timer will preempt us; receiver drains the queue
        unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }
        process::set_state(my_id, ProcessState::Running);

        if enqueue(to, msg) {
            if process::get_state(to) == ProcessState::BlockedOnRecv {
                process::set_state(to, ProcessState::Ready);
            }
            return Ok(());
        }
    }
}

/// Receive a message, blocking until one arrives.
///
/// `from_filter` — accept only messages from this PID, or `ANY` (0) for all.
pub fn ipc_recv(from_filter: u64, buf: &mut Message) -> Result<(), IpcError> {
    let my_id = scheduler::current_id();

    // Fast path — message already waiting
    if let Some(msg) = dequeue(my_id, from_filter) {
        *buf = msg;
        return Ok(());
    }

    // Slow path — inbox empty, block until sender wakes us
    process::set_state(my_id, ProcessState::BlockedOnRecv);
    loop {
        // Sleep until the next timer interrupt; sender will set us Ready
        unsafe { core::arch::asm!("sti; hlt; cli", options(nomem, nostack)); }

        if let Some(msg) = dequeue(my_id, from_filter) {
            process::set_state(my_id, ProcessState::Running);
            *buf = msg;
            return Ok(());
        }
        // Not yet — stay blocked
        process::set_state(my_id, ProcessState::BlockedOnRecv);
    }
}

/// Non-blocking send.  Returns `Err(QueueFull)` immediately if inbox is full.
pub fn ipc_try_send(to: u64, mut msg: Message) -> Result<(), IpcError> {
    let my_id = scheduler::current_id();
    msg.from = my_id;
    msg.to   = to;

    if inbox_idx(to).is_none() {
        return Err(IpcError::NoSuchProcess);
    }
    if enqueue(to, msg) {
        if process::get_state(to) == ProcessState::BlockedOnRecv {
            process::set_state(to, ProcessState::Ready);
        }
        Ok(())
    } else {
        Err(IpcError::QueueFull)
    }
}

/// Non-blocking receive.  Returns `Err(WouldBlock)` immediately if no message.
pub fn ipc_try_recv(from_filter: u64, buf: &mut Message) -> Result<(), IpcError> {
    let my_id = scheduler::current_id();
    match dequeue(my_id, from_filter) {
        Some(msg) => { *buf = msg; Ok(()) }
        None      => Err(IpcError::WouldBlock),
    }
}

/// Wake any processes blocked trying to send to `pid` (call after draining inbox).
pub fn wake_senders(_pid: u64) {
    // Phase 3: processes blocked on send poll after hlt — no explicit wake needed.
    // Phase 4 will use a proper wait queue per inbox.
}
