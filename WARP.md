# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in
this repository.

## Project Overview

NexusOS is a **from-scratch Rust microkernel** — the world's first AI-native
operating system.  No Linux.  No glibc.  No distro assumptions.

**Current state: v0.4.0 — Phases 1–4 verified on bare metal (QEMU + KVM).**

The old Ubuntu/distro material is preserved under `legacy/` but is never built.

---

<!-- Legacy description preserved below for historical reference only -->

_ORIGINAL (pre-kernel) WARP.md content has been superseded._
_All active kernel work is in `kernel/`.  See README.md for build instructions._
_See below for kernel-specific guidance._

---

## Kernel Build

### Prerequisites (one-time)

```bash
make setup          # installs Rust nightly, adds targets, builds Limine
```

Requires only: `rustup`, `make`, `xorriso`.  No distro packages.

### Build + run

```bash
make laptop && make iso-laptop && make run-laptop   # x86_64 full
make tiamat && make iso-tiamat                     # x86_64 server
make bahamut && make iso-bahamut                   # AArch64
```

## Active Kernel Modules

| Module | Phase | Description |
|--------|-------|-------------|
| `arch/x86_64/{gdt,idt,interrupts,timer_isr}` | 1 | CPU structures, naked timer ISR |
| `arch/aarch64/exceptions` | 1 | AArch64 vector table, VBAR_EL1 |
| `memory/{physical,paging,heap}` | 1 | Bitmap allocator (huge-page aware), 4-level paging, heap |
| `io/{serial,uart,framebuffer}` | 1 | Serial/UART, 2x-scaled framebuffer console |
| `timer/{pic,pit}` | 2 | 8259A PIC remap, 8253 PIT 100 Hz |
| `process` | 2 | PCB, ring-0/ring-3 spawn, BlockedOnRecv/Send states |
| `scheduler` | 2 | Round-robin preemptive, TSS.RSP0 + PERCPU updates |
| `ipc/{mod,ports}` | 3 | Message queues (depth=8), blocking send/recv, named ports |
| `syscall` | 4 | STAR/LSTAR/FMASK/EFER, GS-relative naked entry, 9 syscalls |
| `userspace` | 4 | Ring-3 page mapping in PML4[1], nexus-init machine code |

## Key Gotchas

1. **Limine huge pages** — `map_page` must detect 1 GB/2 MB huge-page entries
   (HUGE flag) and refuse to walk through them.  User addresses must be in
   PML4[1] (512 GB+) where Limine has no identity-map entries.

2. **Limine v12 config syntax** — `key: value` (colon-space), NOT `key=value`.
   Entry headers are `/Name`, NOT `:Name`.

3. **TSS.RSP0** — must be updated to each process’s kernel stack top on every
   context switch so timer interrupts from ring 3 land on the right stack.

4. **Syscall PERCPU** — use `gs:[0]` / `gs:[8]` (GS-relative addressing) in
   the naked syscall entry stub, not `[PERCPU+N]` absolute symbol references.

5. **VM** — Test VM at `/media/loufogle/Data/vms/nexusos/`, scripts in
   `scripts/vm/`.  RTX 4080 is in IOMMU Group 16 (isolated), passthrough
   ready via `scripts/vm/vfio-bind.sh`.

## Host Environment

Pop!_OS 22.04 on Intel i9-13900HX + RTX 4080 + 64 GB DDR5.
Preferred package manager: `nala` (not raw `apt`).

## Code Quality Standards

Every function must be complete and working.  No stubs, no TODOs, no zombie code.

- **NO** `TODO`, `FIXME`, `XXX`, `HACK`, `stub`, or `unimplemented` markers
- **NO** incomplete functions or dead code paths
- Code compiles clean and runs correctly before committing
