# NexusOS Landing Page Content

## Hero
**NexusOS — an AI-native operating system, built from scratch.**
A Rust microkernel with no Linux, no glibc, and no distro assumptions. It boots today to an
interactive ring-3 shell (`nexus>`) in QEMU/KVM, and is dedicated to its two companions,
**Stella 🐕** and **Max Jr. 🐱**.

## What it is
- **From-scratch Rust microkernel** — Limine boot, 4-level paging, heap, preemptive scheduler,
  IPC with named ports, 19 syscalls, ring-3 userspace.
- **Storage + execution** — VirtIO-blk, FAT32, an ELF64 loader (`run`), and a self-installer
  (GPT + FAT32 ESP + kernel).
- **AI-native by design** — a native package manager (`nexuspkg`) with handlers for foreign
  formats, and AI companions wired into the OS over IPC.

## The companions (the OS is dedicated to them)
- **Stella 🐕 — Operations:** system orchestration, health, package/operations help.
- **Max Jr. 🐱 — Security:** hardening, monitoring, intrusion detection.

## Status (v0.6.x)
Phases 1–6 verified in QEMU + KVM (boot, scheduler, IPC, syscalls, ring-3 shell, FAT32, ELF
loader, installer). Next: boot from installed disk, then VirtIO-vsock → on-device AI (Ollama).

## Build
```bash
make setup            # Rust nightly targets + Limine
make laptop && make iso-laptop
make run-laptop       # boots in QEMU
```

*NexusOS — AI-native from the first instruction.*
