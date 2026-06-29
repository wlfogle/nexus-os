# NexusOS — AI-Native Rust Microkernel

> **The world's first AI-native operating system, built from scratch.** No Linux. No glibc.
> No distro base. v0.6.x boots to an interactive ring-3 shell (`nexus>`) in QEMU/KVM.
> Dedicated to **Stella 🐕** and **Max Jr. 🐱**.

## The companions
- **Stella 🐕 — Operations:** orchestration, health, package/operations assistance.
- **Max Jr. 🐱 — Security:** hardening, monitoring, intrusion detection.

(`core/services/`: `stella.py`, `maxjr.py`, `nexus-orchestrator.py` + systemd units.)

## What works today (v0.6.x, QEMU + KVM)
- Limine boot → GDT/IDT/TSS, physical + 4-level paging, kernel heap, framebuffer console
- Preemptive round-robin scheduler, 8259A PIC, 8253 PIT
- IPC ring-buffers + named port registry, 19 syscalls, ring-3 user processes
- PS/2 keyboard, VirtIO-blk, FAT32, ELF64 loader (`run HELLO.ELF`)
- Self-installer: GPT + FAT32 ESP + kernel; interactive `nexus>` shell

## Architecture
Three build targets, one Rust codebase: `laptop` (x86_64 full), `tiamat` (x86_64 server),
`bahamut` (AArch64 edge). See `README.md` and `ROADMAP.md`.

## Package model (in progress)
`nexuspkg` is the native package manager. NexusOS ships its own package format and provides
**handlers for foreign formats** (`.deb`, `.rpm`, `.pkg.tar.zst`, flatpak, snap, appimage,
pip, npm, cargo) so software from any ecosystem can be imported.

## Build
```bash
make setup
make laptop && make iso-laptop && make run-laptop
```

## Roadmap (next)
Boot from installed disk (OVMF) → VirtIO-vsock → on-device AI (Ollama) → personality servers
(Linux/BSD/Windows/macOS ABIs). Full plan in `nexus-os-roadmap.md`.

## License
GPL-3.0+.

*NexusOS — AI-native from the first instruction. For Stella 🐕 & Max Jr. 🐱.*
