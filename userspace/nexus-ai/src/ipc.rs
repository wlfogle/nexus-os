/// IPC message structure matching kernel definition
#[repr(C)]
#[derive(Copy, Clone)]
pub struct IpcMessage {
    pub sender_pid: u64,
    pub payload: [u8; 256],
    pub len: usize,
    pub flags: u16,
}

impl IpcMessage {
    pub fn new() -> Self {
        IpcMessage {
            sender_pid: 0,
            payload: [0u8; 256],
            len: 0,
            flags: 0,
        }
    }
}

impl Default for IpcMessage {
    fn default() -> Self {
        Self::new()
    }
}