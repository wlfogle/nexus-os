# NexusOS Roadmap

NexusOS is a **from-scratch Rust microkernel** — the world's first AI-native OS.
No Linux. No glibc. No distro. One codebase, three build targets (laptop / tiamat / bahamut).

Build: `make laptop && make iso-laptop` → bootable install ISO.
First boot onto a blank VirtIO disk runs `task_installer` automatically
(GPT + FAT32 ESP + Limine UEFI bootloader + kernel ELF written to disk).
Target installer ISOs are built separately:
- `make iso-laptop` — control-center profile.
- `make iso-tiamat` — media/file-share host profile.
- `make iso-bahamut` — AArch64 Bahamut Lite network-edge profile.

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

Current role profiles:
- **Laptop** — control center for NexusTerminal, Cockpit, Ollama, Stella, Max Jr., and orchestration.
- **Tiamat** — media/file-share host for nexus-mediastack/service roots.
- **Bahamut** — NexusOS Lite AArch64 network-edge profile for Pi 4 / DietPi-class edge duties.

Syscall table (19 implemented, all in `kernel/src/syscall/mod.rs`):

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
| 19 | SYS_EXEC | exec(name) → child exit code (loads + runs a static ELF64) |
| 20 | SYS_FS_LIST_PATH | fs_list_path(path, buf, cap) → bytes (subdir listing) |
| 21 | SYS_FS_READ_PATH | fs_read_path(path, buf, cap) → bytes read (subdir file) |
| 22 | SYS_FS_MKDIR_PATH | fs_mkdir_path(path) → 0 or -errno (create directory) |
| 23 | SYS_FS_WRITE_PATH | fs_write_path(path, buf, len) → bytes written (create/overwrite) |
| 24 | SYS_FS_APPEND_PATH | fs_append_path(path, buf, len) → bytes written (append/create) |
| 25 | SYS_FS_REMOVE_PATH | fs_remove_path(path) → 0 or -errno (remove file/empty dir) |

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

### Phase 6 — ELF program execution *(core done: v0.6.2)*

The kernel now loads and runs **static ELF64** programs from the FAT32 disk:
`exec/mod.rs` parses the ELF header + `PT_LOAD` segments, maps them into the
user half, and `SYS_EXEC` (19) spawns the image as a ring-3 process while the
caller blocks on a real parent/child wait.  The shell `run HELLO.ELF` command
executes the bundled reference program (`userspace/hello.asm`, linked at
0x8040000000, written to the ESP by the installer).

Remaining for full Phase 6: a **Linux** personality server (translate Linux
syscalls → NexusOS IPC so unmodified Linux ELFs run), per-process address
spaces (CR3 switching) so programs can link at the conventional base, and
dynamic-linker support.

### Phase 6.1 — FAT32 from ring-3 *(subdir paths done: v0.6.2)*
The ring-3 shell can `ls` the disk root and `cat` a file via SYS_FS_LIST=17 /
SYS_FS_READ=18, and now also list/read files in **subdirectories** via the
path-aware VFS layer (`kernel/src/fs/vfs.rs`) and two new syscalls
(SYS_FS_LIST_PATH=20, SYS_FS_READ_PATH=21).  `fatfs` resolves `/`-separated
components from `root_dir()` (`open_dir`/`open_file`), so `ls /EFI/BOOT`,
`ls /boot`, and `cat /EFI/BOOT/limine.conf` all work from the `nexus>` prompt.
Write-path operations are now live too: SYS_FS_MKDIR_PATH=22,
SYS_FS_WRITE_PATH=23, SYS_FS_APPEND_PATH=24, SYS_FS_REMOVE_PATH=25 back the
shell `mkdir <p>`, `write <p> <text>`, `append <p> <text>`, and `rm <p>`
commands.  Writes commit through `fatfs` to the FAT32 disk, so files created
from the `nexus>` prompt persist across reboots.

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
