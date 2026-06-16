# Changelog

All notable changes to the NexusOS from-scratch Rust microkernel.

## [0.6.1] - 2026-06-16

### ✨ Added — Phase 6.1: ring-3 filesystem access
- **`SYS_FS_LIST` (17)** — `fs_list(buf, cap)` lists the mounted FAT32 root
  directory, writing newline-separated entry names into a user buffer.
  Backed by new `fs::fat::list_root()`.
- **`SYS_FS_READ` (18)** — `fs_read(name, buf, cap)` reads a file by name from
  the FAT32 root into a user buffer. Bridges to existing `fs::fat::read_file()`.
- **Shell `ls` and `cat <file>` commands** in `shell_init.asm`, using a new
  2 KiB stack-resident FS buffer. `help` updated to list them.

### 🔄 Changed
- `userspace::spawn_user_init()` now maps as many user-exec code pages as the
  shell binary needs (was hard-coded to one 4 KiB page), copying each page into
  its own physical frame. Size guard raised to 16 KiB.
- Shell `version` now reports Phase 6.

## [0.6.0] - 2026-06-15

### ✨ Added — Ring-3 Interactive Shell
- **`kernel/src/userspace/shell_init.asm`** — 64-bit x86_64 NASM flat binary implementing a
  fully interactive ring-3 shell.  Commands: `help`, `version`, `uname`, `echo`, `ps`,
  `clear`, `reboot`.  Uses `SYS_READ_CHAR` (blocking keyboard read) and `SYS_WRITE`.
  Position-independent; all mutable state on the writable stack page.
- **`kernel/build.rs`** — Cargo build script that assembles `shell_init.asm` with NASM
  at compile time and embeds the flat binary into the kernel via `include_bytes!`.
- Compile-time size guard asserts binary fits in one 4 KiB code page.

### 🐛 Fixed — Boot no longer hangs in IPC demo loop
- Removed `echo-server` and `echo-client` kernel tasks that spammed the console
  every 2 seconds with IPC ping-pong output, blocking any interactive use.
- Removed `kbd-echo` kernel task that monopolised the PS/2 keyboard ring buffer,
  starving the ring-3 process of any keyboard input.
- `nexus-ai` AI Core daemon is retained (blocks on IPC recv; no console spam).

### 🔄 Changed
- `kernel/src/userspace/mod.rs` — `USER_INIT_CODE` replaced with
  `include_bytes!(concat!(env!("OUT_DIR"), "/shell_init.bin"))` (NASM-assembled binary).
- Kernel version bumped from 0.5.3 → **0.6.0**.
- Boot banner updated to reflect Phase 5 ring-3 shell availability.
- Makefile: added `disk-laptop`, `run-install-laptop`, `run-installed-laptop` targets.
- Installer: bumped version string to v0.6, fixed embedded `limine.conf` to match ISO format.
