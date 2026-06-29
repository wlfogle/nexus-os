# NexusOS Social Media Preview

> Current pitch. NexusOS is a **from-scratch, AI-native Rust microkernel** — **no Linux, no
> glibc, no distro base**. v0.6.x boots to an interactive ring-3 shell (`nexus>`) in QEMU/KVM.
> Dev host is Pop!_OS; the OS itself is built from `/` up. Dedicated to **Stella 🐕** and
> **Max Jr. 🐱**.

## Reddit (r/osdev, r/rust, r/linux)
**Title:** Building NexusOS — a from-scratch, AI-native Rust microkernel (boots to a ring-3 shell)

```markdown
NexusOS is a microkernel written from scratch in Rust — no Linux, no glibc, no distro base.

Where it is (v0.6.x, verified in QEMU + KVM):
- Boot via Limine → GDT/IDT, paging, heap, framebuffer
- Preemptive round-robin scheduler, PIC/PIT
- IPC (named ports) + 19 syscalls, ring-3 user processes
- PS/2 keyboard, VirtIO-blk, FAT32, ELF64 loader
- Self-installer (GPT + FAT32 ESP + kernel) and an interactive ring-3 shell: `nexus>`

AI-native plan: a native package manager (nexuspkg) with handlers for foreign formats
(.deb/.rpm/.pkg.tar.zst/flatpak/snap/appimage/pip/npm/cargo), and AI companions wired in via IPC.

Two companions the OS is dedicated to:
🐕 Stella — operations (orchestration, health)
🐱 Max Jr. — security (hardening, monitoring)

Repo: https://github.com/wlfogle/nexus-os
```

## Twitter/X
- NexusOS: a from-scratch AI-native **Rust microkernel**. No Linux. No glibc. It already boots to an interactive `nexus>` shell in QEMU/KVM. 🦀
- Built from `/` up: paging, scheduler, IPC, syscalls, ring-3 userspace, FAT32, ELF loader, self-installer — all in Rust.
- Dedicated to its two companions: **Stella 🐕 (operations)** and **Max Jr. 🐱 (security)**.
- Roadmap: disk boot → VirtIO-vsock → on-device AI; native `.nos` packages with handlers for every foreign format.

## GitHub repo description
```
NexusOS — a from-scratch, AI-native Rust microkernel (no Linux, no glibc).
v0.6.x: boots to a ring-3 shell, FAT32 + ELF loader + self-installer. Dedicated to Stella 🐕 (operations) & Max Jr. 🐱 (security).
```
