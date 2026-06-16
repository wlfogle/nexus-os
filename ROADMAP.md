# NexusOS Roadmap

NexusOS is a **from-scratch Rust microkernel** — the world's first AI-native OS.
No Linux. No glibc. No distro. One codebase, three build targets (laptop / tiamat / bahamut).

Build: `make laptop && make iso-laptop` → bootable install ISO.
First boot onto a blank VirtIO disk runs `task_installer` automatically
(GPT + FAT32 ESP + Limine UEFI bootloader + kernel ELF written to disk).

---

## Current state — v0.6.0 (2026-06-15)

Phases 1–5 verified on QEMU + KVM. Ring-3 interactive shell boots.

| Phase | Status | Scope |
|-------|--------|-------|
| 1 | ✓ Done | Boot, GDT/IDT, physical memory, paging, heap, framebuffer |
| 2 | ✓ Done | Preemptive round-robin scheduler, 8259A PIC, 8253 PIT 100 Hz |
| 3 | ✓ Done | IPC ring-buffers, blocking send/recv, named port registry |
| 4 | ✓ Done | `syscall`/`sysretq` fast path, ring-3 user process via IRETQ |
| 5 | ✓ Done | AI Core daemon (nexus.ai), PS/2 keyboard, VirtIO-blk, FAT32, self-installer, ring-3 shell (`nexus>`) |

Syscall table (18 implemented, all in `kernel/src/syscall/mod.rs`):

| # | Name | Description |
|---|------|-------------|
| 1 | SYS_EXIT | exit(code) |
| 2 | SYS_WRITE | write(fd=1, buf, len) |
| 3 | SYS_GETPID | → pid |
| 4 | SYS_YIELD | yield to scheduler |
| 5 | SYS_IPC_SEND | ipc_send(to_pid, buf, len) |
| 6 | SYS_IPC_RECV | ipc_recv(from_filter, buf, cap) |
| 7 | SYS_PORT_REGISTER | port_register(name, len) |
| 8 | SYS_PORT_FIND | port_find(name, len) → pid |
| 9 | SYS_SLEEP | sleep(ticks) |
| 10 | SYS_IPC_QUERY | ipc_query(name, len, 0) → pid |
| 11 | SYS_IPC_TIMEOUT | ipc_timeout(ms) |
| 12 | SYS_GPU_MMAP | gpu_mmap(size, flags, 0) → vaddr (Phase 5.2 stub) |
| 13 | SYS_READ_CHAR | → u8, blocks until keypress (IRQ1-woken) |
| 14 | SYS_READ_CHAR_NB | → u8 or -1 if no key queued |
| 15 | SYS_DISK_READ | disk_read(lba, buf, sectors) |
| 16 | SYS_DISK_WRITE | disk_write(lba, buf, sectors) |
| 17 | SYS_FS_LIST | fs_list(buf, cap) → bytes (newline-separated names) |
| 18 | SYS_FS_READ | fs_read(name, buf, cap) → bytes read |

---

## Near-term

### Phase 5.4 — Disk boot + VirtIO-vsock → Ollama

1. Boot from installed QCOW2 disk (GPT ESP, no ISO) verified end-to-end in QEMU with OVMF.
2. VirtIO-vsock driver so the ring-3 nexus-ai daemon opens a host-forwarded socket to Ollama.
3. nexus-ai replaces the mock reply with a real HTTP POST to `http://localhost:11434/api/generate`.
4. Shell `ai <prompt>` command: IPC to nexus.ai port, prints LLM response to serial.

### Phase 5.5 — Shell polish

1. Command history (up-arrow recall), tab completion for built-ins.
2. `uptime` and `mem` built-ins querying kernel stats via SYS_IPC_QUERY.
3. Framebuffer mirror: shell output visible on screen (laptop), not only on serial.

---

## Mid-term

### Phase 6 — Linux ELF personality server

Load and execute Linux x86_64 static ELF binaries by translating Linux
syscalls → NexusOS IPC. Milestone: `echo`, `ls`, `cat` static ELFs run natively.

### Phase 6.1 — FAT32 from ring-3 *(partially done: v0.6.1)*

The ring-3 shell can now `ls` the disk root and `cat` a file via two new
syscalls (SYS_FS_LIST=17, SYS_FS_READ=18) bridging to the kernel FAT32 driver.
Remaining: subdirectory path parsing (`ls /boot`) and a user-space VFS layer
built on SYS_DISK_READ.

### Phase 6.2 — Network stack

VirtIO-net driver + smoltcp (`no_std`) TCP/IP. Unlocks Ollama HTTP
without vsock dependency.

---

## Long-term

| Phase | Scope |
|-------|-------|
| 7 | BSD personality server — run BSD/POSIX ELF binaries |
| 8 | Windows personality server — PE/COFF, Win32 API subset |
| 9 | macOS personality server — Mach-O, Darwin API subset |
| 10 | NexusStore v1.0 — install anything from any source |
| 11 | Bare-metal daily driver on i9-13900HX + RTX 4080 |

---

## Policy

- **Zero stub code.** Every merged function is complete and working.
- **"Done" means** the behaviour is demonstrable from the ISO or an
  installed disk — not "code exists but untested".
- All commits include `Co-Authored-By: Oz <oz-agent@warp.dev>` where applicable.
