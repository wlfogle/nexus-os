# Phase 6: NexusTerminal — UI/IPC Bridge

**Status:** Design & Planning  
**Target Version:** v0.6.0  
**Depends on:** Phase 5.0 (AI Core + syscalls) — ✅ Complete

---

## Overview

**NexusTerminal** is a ring-3 user-space application that serves as the interactive UI for the NexusOS AI Core. It:

1. **Spawns at boot** as a standard ring-3 process (like nexus-init)
2. **Communicates with nexus-ai** daemon via IPC on the `nexus.ai` port
3. **Provides an interactive CLI** where users can:
   - Enter prompts for AI inference
   - View streaming responses
   - Manage model selection
   - Query system status
4. **Remains responsive** during long inference operations (async/threaded)

```
┌──────────────────────────────────┐
│   NexusTerminal (Ring-3)         │
│   ┌──────────────────────────┐   │
│   │  User Input (stdin)      │   │ ← Interactive prompt
│   └──────────���─┬─────────────┘   │
│                │ SYS_IPC_SEND    │
│   ┌────────────▼─────────────┐   │
│   │  IPC Client              │   │
│   └────────────┬─────────────┘   │
│                │                 │
│════════════════╬═════════════════╪═════════ IPC Queue (nexus.ai)
│                │                 │
│   ┌────────────▼─────────────┐   │
│   │  IPC Response Handler    │   │
│   └────────────┬─────────────┘   │
│                │ SYS_IPC_RECV    │
│   ┌────────────▼─────────────┐   │
│   │  Output (framebuffer)    │   │ ← Display response
│   └──────────────────────────┘   │
└──────────────────────────────────┘

         nexus-ai Daemon (Ring-3)
    ┌─────────────────────────────┐
    │  Message recv/handler       │
    │  → Ollama inference         │
    │  → Response formatting      │
    └─────────────────────────────┘
```

---

## Architecture

### 1. Directory Structure

```
userspace/nexus-terminal/
├── Cargo.toml                   # Standalone binary
├── src/
│   ├── main.rs                  # Entry point, TUI loop
│   ├── ipc_client.rs            # IPC send/recv to nexus.ai
│   ├── terminal.rs              # Terminal I/O (framebuffer or serial)
│   ├── ui.rs                    # Simple line-based UI
│   └── prompt_handler.rs        # Prompt → IPC request → response display
└── WARP.md                      # Development guide
```

### 2. Main Data Flow

**User types prompt** → **IPC send to nexus.ai** → **Wait for response** → **Display** → **Repeat**

```rust
// Pseudo-code flow
loop {
    print_prompt("nexus> ");
    let user_input = read_line();
    
    if user_input.is_empty() { continue; }
    
    // Send to nexus-ai via IPC
    let response = ipc_send_query(&user_input)?;
    
    // Display response
    println!("{}", response);
}
```

### 3. Syscalls Used

| Syscall | Purpose | Phase |
|---------|---------|-------|
| `SYS_WRITE` | Output to console | 4 |
| `SYS_GETPID` | Get own PID | 4 |
| `SYS_IPC_SEND` | Send prompt to nexus-ai | 4/5 |
| `SYS_IPC_RECV` | Receive response from nexus-ai | 4/5 |
| `SYS_IPC_QUERY` | Resolve nexus.ai port | **5** |
| `SYS_SLEEP` | Optional: delay before retry | 4 |

---

## Implementation Plan

### Phase 6.0: Basic CLI (This Sprint)

**Goal:** Spawn NexusTerminal, accept user input, send to nexus-ai, display response.

**Files:**

```
userspace/nexus-terminal/
├── Cargo.toml
└── src/
    ├── main.rs              (100 lines)
    ├── ipc_client.rs        (80 lines)
    ├── terminal.rs          (60 lines)
    └── ui.rs                (50 lines)
```

**main.rs:**
```rust
#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;

mod ipc_client;
mod terminal;
mod ui;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    terminal::init();
    terminal::println("[nexus-terminal] Starting interactive AI CLI");
    
    // Connect to nexus-ai daemon
    let ai_port = ipc_client::resolve_ai_port();
    terminal::println(&format!("[nexus-terminal] Connected to nexus.ai (port_id={})", ai_port));
    
    // Main REPL loop
    loop {
        terminal::print("nexus> ");
        if let Ok(prompt) = terminal::read_line() {
            if prompt.is_empty() { continue; }
            
            match ipc_client::query_ai(&ai_port, &prompt) {
                Ok(response) => {
                    terminal::println("");
                    terminal::println(&response);
                    terminal::println("");
                }
                Err(e) => {
                    terminal::println(&format!("Error: {}", e));
                }
            }
        }
    }
}
```

**ipc_client.rs:**
```rust
const SYS_IPC_QUERY: u64 = 7;
const SYS_IPC_SEND: u64 = 4;
const SYS_IPC_RECV: u64 = 5;

pub fn resolve_ai_port() -> u64 {
    // Call SYS_IPC_QUERY("nexus.ai") to get port ID
    let name = b"nexus.ai";
    unsafe {
        syscall3(SYS_IPC_QUERY, name.as_ptr() as u64, name.len() as u64, 0) as u64
    }
}

pub fn query_ai(port_id: u64, prompt: &str) -> Result<String, &'static str> {
    // Send prompt via SYS_IPC_SEND
    unsafe {
        syscall3(SYS_IPC_SEND, port_id, prompt.as_ptr() as u64, prompt.len() as u64);
    }
    
    // Receive response via SYS_IPC_RECV (blocking)
    let mut response = [0u8; 256];
    let len = unsafe {
        syscall3(SYS_IPC_RECV, port_id, response.as_ptr() as u64, 0)
    } as usize;
    
    core::str::from_utf8(&response[..len.min(256)])
        .map(|s| s.to_string())
        .map_err(|_| "Invalid UTF-8 response")
}
```

**terminal.rs:**
```rust
// Simple framebuffer/serial output wrapper
pub fn init() {
    // Initialize console (serial on headless, framebuffer on laptop)
}

pub fn println(s: &str) {
    unsafe { syscall3(SYS_WRITE, 1, s.as_ptr() as u64, s.len() as u64); }
}

pub fn print(s: &str) {
    unsafe { syscall3(SYS_WRITE, 1, s.as_ptr() as u64, s.len() as u64); }
}

pub fn read_line() -> Result<String, &'static str> {
    // Phase 6.0: Read from simple serial input
    // Phase 6.1: Implement full keyboard input (IRQ handler)
    // For now: accept one line per boot
    Err("stdin not implemented in Phase 6.0")
}
```

### Phase 6.1: Keyboard Input + Streaming (Future)

- Implement keyboard IRQ handler
- Stream responses character-by-character
- Add command history (↑/↓ arrows)

### Phase 6.2: Advanced UI (Future)

- Syntax highlighting for responses
- Status bar (model, latency, token count)
- Multi-line prompt editor
- Session management (save/load)

---

## Integration with Boot Sequence

**kernel/src/main.rs (Phase 5 code):**

```rust
// After nexus-ai spawned, spawn NexusTerminal
let term_pid = userspace::spawn_nexus_terminal();
kprintln!("[boot] nexus-terminal spawned as pid={}", term_pid);
```

**Expected boot log:**

```
[boot] Phase 5: AI Core initialization
[ipc:ports] Reserved: nexus.ai → port_id=100
[boot] nexus-ai spawned as pid=4
[nexus-ai] Starting AI Core daemon
[nexus-ai] Bound to port_id=100
[nexus-ai] Entering main daemon loop
[boot] nexus-terminal spawned as pid=5
[nexus-terminal] Starting interactive AI CLI
[nexus-terminal] Connected to nexus.ai (port_id=100)
nexus> 
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_ipc_query_resolves_ai_port() {
        let port_id = resolve_ai_port();
        assert!(port_id > 0);
    }
    
    #[test]
    fn test_send_receive_prompt() {
        let port_id = resolve_ai_port();
        let response = query_ai(port_id, "What is AI?").expect("send/recv failed");
        assert!(!response.is_empty());
    }
}
```

### Integration Test

**scripts/phase6-integration-test.sh:**

```bash
#!/bin/bash
# Boot with Phase 6, send mock prompt via serial, verify response

make laptop
make iso-laptop

timeout 30 qemu-system-x86_64 -m 2G -cdrom build/nexusos-laptop.iso -serial stdio -nographic \
  > /tmp/phase6-test.log 2>&1 &

sleep 5

# Verify spawns
grep -q "\[nexus-terminal\] Starting" /tmp/phase6-test.log || exit 1
grep -q "nexus> " /tmp/phase6-test.log || exit 1

echo "✓ Phase 6 boot verified"
```

---

## Command Reference (Phase 6.0)

```
nexus> help
  help               — Show this message
  status             — Query AI Core status
  models             — List available models
  use <model>        — Switch to a model
  <prompt>           — Send prompt to current model
  exit               — Quit NexusTerminal

nexus> What is NexusOS?
NexusOS is an AI-native microkernel... [response from nexus-ai]

nexus> status
  Model: mistral
  Port: 100
  Status: ready
  Last latency: 234 ms

nexus> exit
[nexus-terminal] Shutting down
```

---

## Files to Implement

- [ ] `userspace/nexus-terminal/Cargo.toml`
- [ ] `userspace/nexus-terminal/src/main.rs`
- [ ] `userspace/nexus-terminal/src/ipc_client.rs`
- [ ] `userspace/nexus-terminal/src/terminal.rs`
- [ ] `userspace/nexus-terminal/src/ui.rs`
- [ ] `userspace/nexus-terminal/src/prompt_handler.rs`
- [ ] `kernel/src/userspace/mod.rs` (add spawn_nexus_terminal function)
- [ ] `scripts/phase6-integration-test.sh`
- [ ] Update `README.md` and `WARP.md` with Phase 6 details

---

## Relation to Other Phases

**Depends on:**
- Phase 5.0: AI Core + syscalls ✅

**Blocks:**
- Phase 6.1: Full keyboard + streaming
- Phase 7: VFS + networking (can integrate NexusTerminal into desktop env)

**Parallel work:**
- Phase 5.1: HTTP client for Ollama (independent)
- Phase 5.2: GPU memory mgmt (independent)

---

## Design Decisions

1. **No_std Rust** — Stays true to OS design, all I/O via syscalls
2. **Blocking recv** — Phase 6.0 waits for full response. Phase 6.1 will support streaming.
3. **Serial-first** — Keyboard input deferred to Phase 6.1; Phase 6.0 accepts pre-entered prompts
4. **Simple parser** — Commands are single words or space-separated; complex shell parsing deferred
5. **Max prompt size: 256 bytes** — Limited by IPC message size; Phase 6.2 will add multi-part requests

---

## References

- [PHASE5_ARCHITECTURE.md](PHASE5_ARCHITECTURE.md) — AI Core syscalls
- [kernel/src/userspace/mod.rs](../kernel/src/userspace/mod.rs) — Process spawn API
- [Ollama API](https://github.com/ollama/ollama/blob/main/docs/api.md) — Response format
