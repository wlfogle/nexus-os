# Phase 5: AI Core Architecture — Implementation Reference

**Status:** Implemented (v0.6.0)  
**Phases Complete:** 1–5

---

## Overview

Phase 5 delivers **AI inference as a first-class system service** via IPC.
The kernel provides the syscall interface; the ring-3 `nexus-ai` daemon handles
Ollama HTTP integration and message routing.

Phase 5.0 (mock replies) is complete and verified.  
Phase 5.1 (real Ollama HTTP via VirtIO-vsock) is the immediate next step.

**Achieved:** Ring-3 processes can query the AI Core for LLM inference via IPC.
The `nexus>` shell boots and is interactive. The self-installer writes the full
stack to a VirtIO disk on first boot.

```
┌─────────────────────────────────────────────┐
│  Ring-3 User Application (nexus-init, etc) │
│         SYS_IPC_SEND("Hello")               │
└──────────────────┬──────────────────────────┘
                   │ IPC Message Queue
                   ▼
     ┌───────────────────────────────┐
     │   nexus.ai Named Port         │
     │  (Ring-3 AI Core Daemon)      │
     └──────────────┬────────────────┘
                    │ HTTP (localhost:11434)
                    ▼
          ┌─────────────────────┐
          │  Ollama (local GPU) │
          │  Model inference    │
          └─────────────────────┘
```

---

## Kernel Changes (kernel/)

### 1. Syscalls (kernel/src/syscall/mod.rs) — all implemented

```rust
// Phase 4 (base)
pub const SYS_EXIT:          u64 = 1;
pub const SYS_WRITE:         u64 = 2;   // fd=1 → serial
pub const SYS_GETPID:        u64 = 3;
pub const SYS_YIELD:         u64 = 4;
pub const SYS_IPC_SEND:      u64 = 5;
pub const SYS_IPC_RECV:      u64 = 6;
pub const SYS_PORT_REGISTER: u64 = 7;
pub const SYS_PORT_FIND:     u64 = 8;
pub const SYS_SLEEP:         u64 = 9;
// Phase 5
pub const SYS_IPC_QUERY:     u64 = 10;
pub const SYS_IPC_TIMEOUT:   u64 = 11;
pub const SYS_GPU_MMAP:      u64 = 12;  // Phase 5.2 stub (reserved region)
pub const SYS_READ_CHAR:     u64 = 13;  // blocks on BlockedOnKey, woken by IRQ1
pub const SYS_READ_CHAR_NB:  u64 = 14;  // non-blocking
pub const SYS_DISK_READ:     u64 = 15;  // VirtIO-blk sector read
pub const SYS_DISK_WRITE:    u64 = 16;  // VirtIO-blk sector write
```

### 2. Syscall Handlers (kernel/src/syscall/handlers/)

**`syscall_ipc_query.rs`** — Resolve service name → port ID

```rust
pub fn handle_ipc_query(name: &str) -> Result<u64, i32> {
    // Call into ipc/ports.rs port registry
    match get_port_id(name) {
        Some(id) => Ok(id as u64),
        None => Err(-2), // ENOENT
    }
}
```

**`syscall_ipc_timeout.rs`** — Set per-process recv timeout

```rust
pub fn handle_ipc_timeout(timeout_ms: u64) -> Result<u64, i32> {
    let current = CURRENT_PROCESS.read();
    current.set_recv_timeout(timeout_ms);
    Ok(0)
}
```

**`syscall_gpu_mmap.rs`** — Reserve GPU buffer region (stub for Phase 5)

```rust
pub fn handle_gpu_mmap(size: u64, flags: u32) -> Result<u64, i32> {
    // Phase 5.2: Full GPU memory management
    // For now: allocate dummy memory in VRAM zone
    Ok(0x0000_7000_0000_0000) // Placeholder GPU region start
}
```

### 3. Port Registry Extension (kernel/src/ipc/ports.rs)

Ensure the named port registry has these reserved names:

```rust
pub const RESERVED_PORTS: &[&str] = &[
    "nexus.ai",         // AI Core inference
    "nexus.fs",         // VFS (Phase 7)
    "nexus.gpu",        // GPU scheduler (Phase 5.2)
    "nexus.net",        // Network stack (Phase 7)
];

pub fn register_system_ports() {
    for port_name in RESERVED_PORTS {
        let port_id = PORT_REGISTRY.insert(port_name, PortDescriptor::new());
        log!("[ipc:ports] Reserved: {} → port_id={}", port_name, port_id);
    }
}
```

### 4. Kernel Entry Point Update (kernel/src/main.rs)

In Phase 4 boot sequence, add **Phase 5 init** after Phase 4:

```rust
// Phase 5 — AI Core
log!("[boot] Phase 5: AI Core initialization");
ipc::ports::register_system_ports();
log!("[boot] Reserved ports: nexus.ai, nexus.fs, nexus.gpu, nexus.net");

// Spawn nexus-ai daemon into ring-3
let ai_proc = process::spawn_process(
    "nexus-ai",
    &include_bytes!("../../userspace/nexus-ai/nexus-ai-init.bin")[..],
    ProcessFlags::RING3 | ProcessFlags::DAEMON,
)?;
log!("[boot] nexus-ai spawned as PID {}", ai_proc.pid);

// Continue to idle loop
idle_loop();
```

### 5. Update Cargo Features (kernel/Cargo.toml)

```toml
[features]
laptop = ["framebuffer", "acpi", "ai-core"]
tiamat = ["acpi", "ai-core"]
bahamut = ["serial-only", "ai-core"]
ai-core = []  # Enable Phase 5 syscalls and port registry
```

---

## User-Space AI Core (userspace/nexus-ai/)

### Directory Structure

```
userspace/nexus-ai/
├── Cargo.toml                   # Standalone binary
├── src/
│   ├── main.rs                  # Entry point, daemon loop
│   ├── ipc.rs                   # IPC message handling
│   ├── ollama_client.rs         # HTTP client to localhost:11434
│   ├── inference.rs             # Request → Ollama → Response
│   └── gpu.rs                   # GPU state (Phase 5.2)
└── nexus-ai-init.bin            # Compiled binary (embedded in kernel)
```

### 1. Cargo.toml

```toml
[package]
name = "nexus-ai"
version = "0.1.0"
edition = "2021"

[dependencies]
nexus-syscall = { path = "../../kernel/src/syscall_abi" }
```

### 2. main.rs

```rust
#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;
use alloc::string::String;

mod ipc;
mod ollama_client;
mod inference;

/// nexus-ai entry point (ring-3, spawned by kernel)
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println_ipc("[nexus-ai] Starting AI Core daemon");
    
    // Register on nexus.ai port (kernel already created it)
    let port_id = ipc::resolve_port("nexus.ai")
        .expect("nexus.ai port not registered");
    println_ipc(&format!("[nexus-ai] Bound to port_id={}", port_id));
    
    // Main daemon loop
    loop {
        match ipc::recv_message(port_id, 0) {  // blocking recv
            Ok(msg) => {
                let response = inference::handle_request(&msg.payload);
                ipc::send_message(msg.sender_pid, &response)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln_ipc(&format!("[nexus-ai] recv error: {}", e));
            }
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    eprintln_ipc(&format!("[nexus-ai] PANIC: {:?}", info));
    loop {}
}

fn println_ipc(s: &str) {
    // Use SYS_WRITE(1, buf, len) for serial output
    unsafe {
        syscall!(SYS_WRITE, 1, s.as_ptr(), s.len());
    }
}

fn eprintln_ipc(s: &str) {
    println_ipc(s);
}
```

### 3. ipc.rs

```rust
use alloc::string::String;

#[repr(C)]
pub struct IpcMessage {
    pub sender_pid: u64,
    pub payload: [u8; 256],  // Max message size
    pub len: usize,
}

pub fn resolve_port(name: &str) -> Result<u64, i32> {
    let name_ptr = name.as_ptr();
    let name_len = name.len();
    
    let port_id: i64 = unsafe {
        syscall!(SYS_IPC_QUERY, name_ptr, name_len)
    };
    
    if port_id < 0 {
        Err(port_id as i32)
    } else {
        Ok(port_id as u64)
    }
}

pub fn recv_message(port_id: u64, timeout_ms: u64) -> Result<IpcMessage, i32> {
    if timeout_ms > 0 {
        unsafe { syscall!(SYS_IPC_TIMEOUT, timeout_ms); }
    }
    
    let mut msg: IpcMessage = Default::default();
    let ret: i64 = unsafe {
        syscall!(SYS_IPC_RECV, port_id, &msg as *const _ as usize)
    };
    
    if ret < 0 {
        Err(ret as i32)
    } else {
        Ok(msg)
    }
}

pub fn send_message(pid: u64, payload: &str) -> Result<(), i32> {
    let ret: i64 = unsafe {
        syscall!(SYS_IPC_SEND, pid, payload.as_ptr(), payload.len())
    };
    
    if ret < 0 {
        Err(ret as i32)
    } else {
        Ok(())
    }
}
```

### 4. ollama_client.rs

```rust
use alloc::string::{String, ToString};
use alloc::format;

/// Minimal HTTP client wrapper for Ollama API
pub struct OllamaClient {
    host: &'static str,
}

impl OllamaClient {
    pub fn new(host: &'static str) -> Self {
        OllamaClient { host }
    }
    
    /// POST /api/generate with prompt
    pub fn generate(&self, model: &str, prompt: &str) -> Result<String, &'static str> {
        // Phase 5: Implement minimal HTTP POST to Ollama
        // For now, return mock response
        Ok(format!("mock_response_to: {}", prompt))
    }
}
```

### 5. inference.rs

```rust
use alloc::string::String;
use alloc::format;
use crate::ollama_client::OllamaClient;

pub fn handle_request(payload: &[u8]) -> String {
    let request = core::str::from_utf8(payload)
        .unwrap_or("invalid_utf8");
    
    let client = OllamaClient::new("http://localhost:11434");
    
    match client.generate("mistral", request) {
        Ok(response) => response,
        Err(e) => format!("error: {}", e),
    }
}
```

### 6. Build Integration (kernel/src/userspace/mod.rs)

Include nexus-ai as an embedded binary:

```rust
pub mod nexus_ai {
    pub const BINARY: &[u8] = include_bytes!(
        "../../target/release/nexus-ai/nexus-ai-init.bin"
    );
}
```

---

## Testing Strategy (Phase 5)

### 5.1 Kernel Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ipc_query_resolve() {
        register_system_ports();
        let port_id = handle_ipc_query("nexus.ai").expect("port not found");
        assert!(port_id > 0);
    }
    
    #[test]
    fn test_ipc_timeout_syscall() {
        let result = handle_ipc_timeout(1000);
        assert_eq!(result, Ok(0));
    }
}
```

### 5.2 Integration Test (VM)

**test-ai-core.sh** — Boot kernel, spawn nexus-ai, send IPC query

```bash
#!/bin/bash
set -e

# Build and boot kernel with AI-Core
make laptop AI_CORE=1
make iso-laptop
make run-laptop > /tmp/ai-core-test.log 2>&1 &

# Wait for kernel boot (check serial output)
sleep 5

# Verify phase 5 messages
if grep -q "Phase 5: AI Core initialization" /tmp/ai-core-test.log; then
    echo "✓ Phase 5 boot sequence OK"
else
    echo "✗ Phase 5 boot failed"
    cat /tmp/ai-core-test.log
    exit 1
fi

# Verify nexus-ai daemon spawned
if grep -q "nexus-ai spawned as PID" /tmp/ai-core-test.log; then
    echo "✓ nexus-ai daemon spawned"
else
    echo "✗ nexus-ai spawn failed"
    exit 1
fi

# Verify reserved ports registered
for port in "nexus.ai" "nexus.fs" "nexus.gpu"; do
    if grep -q "Reserved: $port" /tmp/ai-core-test.log; then
        echo "✓ Port $port reserved"
    else
        echo "✗ Port $port not reserved"
        exit 1
    fi
done

echo "✓ All Phase 5 tests passed"
```

---

## IPC Protocol Spec

### Message Format

```c
struct ipc_message {
    uint64_t sender_pid;      // Set by kernel on send
    uint8_t  payload[256];    // User data
    uint16_t len;             // Payload length
    uint16_t flags;           // Reserved
};
```

### nexus.ai API

**Request (JSON in payload):**
```json
{
  "model": "mistral",
  "prompt": "What is AI?",
  "max_tokens": 256
}
```

**Response (JSON in payload):**
```json
{
  "model": "mistral",
  "response": "AI is artificial intelligence...",
  "stop_reason": "stop",
  "eval_count": 42
}
```

---

## Phases 5.1–5.3 (Future)

| Phase | Scope | Syscalls | Status |
|-------|-------|----------|--------|
| 5.0 | **Core** (this doc) | SYS_IPC_QUERY/TIMEOUT, SYS_GPU_MMAP | Ready to implement |
| 5.1 | GPU memory abstraction | SYS_GPU_ALLOC, SYS_GPU_FREE | Design |
| 5.2 | Local inference engine | GPU-resident model cache | Design |
| 5.3 | Multi-model scheduling | Load balancing, priority queue | Backlog |

---

## Implementation Status

- [x] Phase 5 syscalls added (SYS_IPC_QUERY, SYS_IPC_TIMEOUT, SYS_GPU_MMAP, SYS_READ_CHAR, SYS_READ_CHAR_NB, SYS_DISK_READ, SYS_DISK_WRITE)
- [x] syscall handlers implemented
- [x] ipc/ports.rs reserved port registry (nexus.ai, nexus.fs, nexus.gpu, nexus.net)
- [x] nexus-ai kernel thread spawned at boot, IPC on nexus.ai port
- [x] PS/2 keyboard driver (IRQ1, ring-buffer, BlockedOnKey → wake_blocked_on_key)
- [x] VirtIO-blk disk driver (legacy I/O-port BAR0, 8-entry virtqueue)
- [x] FAT32 filesystem (fatfs crate, sector-buffered DiskIo)
- [x] Self-installer (GPT + FAT32 ESP + BOOTX64.EFI + limine.conf + kernel ELF → disk)
- [x] ring-3 interactive shell (shell_init.asm NASM binary, 7 commands, SYS_READ_CHAR)
- [ ] Phase 5.4: VirtIO-vsock driver
- [ ] Phase 5.4: nexus-ai real HTTP POST to Ollama via vsock
- [ ] Phase 5.4: shell `ai <prompt>` command
- [ ] Phase 5.5: shell history, tab completion, framebuffer mirror

---

## References

- [Phase 4 Syscall Implementation](kernel/src/syscall/)
- [IPC Infrastructure](kernel/src/ipc/)
- [Process Management](kernel/src/process/)
- [Ollama HTTP API](https://github.com/ollama/ollama/blob/main/docs/api.md)