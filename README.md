# NexusOS Kernel

> **The world's first AI-native operating system.**  
> Built from scratch. No Linux. No glibc. No distro assumptions.

**Current version: v0.4.0** — Phases 1–4 complete and verified on bare metal.

## Architecture

NexusOS is a **Rust microkernel** that boots via the [Limine](https://github.com/limine-bootloader/limine) protocol.
Three build targets — one codebase, zero shared OS assumptions.

| Target | Architecture | Memory | Features |
|--------|-------------|--------|----------|
| **laptop** | x86_64 | 4–64 GB | Framebuffer, AI hooks, full ACPI |
| **tiamat** | x86_64 | 8–64 GB | Headless server, service hooks |
| **bahamut** | AArch64 | 2 GB | Serial-only, minimal heap, edge node |

## Build Requirements

Only four things — none are distro-specific:

| Tool | Purpose | Install |
|------|---------|----------|
| `rustup` | Rust toolchain + targets | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| `make` | Build orchestration | any Linux (already present) |
| `xorriso` | ISO creation | `apt/dnf/pacman install xorriso` or build from source |
| `qemu-system-*` | Testing (optional) | install for your platform |

## Quick Start

```bash
# 1. One-time setup — installs Rust nightly targets + builds Limine from source
make setup

# 2. Build all three kernels
make all
# or individually:
make laptop
make tiamat
make bahamut

# 3. Create bootable ISOs
make iso-laptop
make iso-tiamat
make iso-bahamut

# 4. Test in QEMU
make run-laptop
make run-tiamat
make run-bahamut
```

Output files land in `build/`:
```
build/nexus-kernel-laptop    # ELF binary (x86_64 full)
build/nexus-kernel-tiamat    # ELF binary (x86_64 server)
build/nexus-kernel-bahamut   # ELF binary (AArch64)
build/nexusos-laptop.iso     # Bootable hybrid UEFI+BIOS ISO
build/nexusos-tiamat.iso
build/nexusos-bahamut.iso    # AArch64 UEFI ISO
```

## Kernel Source Structure

```
kernel/
├── Cargo.toml               features: laptop / tiamat / bahamut
├── rust-toolchain.toml      nightly pinned
├── .cargo/config.toml       build-std, linker scripts per arch
├── nexus-x86_64.ld          higher-half linker script (x86_64)
├── nexus-aarch64.ld         higher-half linker script (AArch64)
└── src/
    ├── main.rs              entry point, boot sequence, kernel tasks
    ├── panic.rs             kernel panic handler
    ├── arch/
    │   ├── x86_64/
    │   │   ├── gdt.rs       GDT + TSS (RSP0 for ring-3 interrupts, IST for #DF)
    │   │   ├── idt.rs       IDT 256 entries + IRQ0 timer wired
    │   │   ├── interrupts.rs  CPU exception handlers
    │   │   └── timer_isr.rs  naked timer ISR — context switch heart
    │   └── aarch64/
    │       └── exceptions.rs  vector table (2KB aligned), VBAR_EL1
    ├── memory/
    │   ├── physical.rs      bitmap frame allocator (huge-page aware)
    │   ├── paging.rs        4-level page tables, HHDM, user page mapping
    │   └── heap.rs          64 MB / 16 MB kernel heap, global allocator
    ├── io/
    │   ├── serial.rs        COM1 serial, direct port I/O (x86_64)
    │   ├── uart.rs          PL011 UART MMIO (AArch64)
    │   ├── framebuffer.rs   2x-scaled 8×8 bitmap console (laptop)
    │   └── font8x8.rs       public domain ASCII bitmap font
    ├── timer/
    │   ├── pic.rs           8259A PIC — remaps IRQs to 0x20–0x2F
    │   └── pit.rs           8253 PIT — 100 Hz tick counter
    ├── process/             PCB, kernel stack, ring-3 spawn, state machine
    ├── scheduler/           round-robin preemptive scheduler + PERCPU update
    ├── ipc/
    │   ├── mod.rs           Message, inbox queues, blocking send/recv
    │   └── ports.rs         named port registry (nexus.ai, nexus.fs, ...)
    ├── syscall/             STAR/LSTAR/FMASK/EFER + GS-relative entry stub
    └── userspace/           ring-3 process spawn, page mapping, init code
```

## Boot Sequence

```
UEFI or BIOS firmware
  └── Limine v12 (built from source, UEFI + BIOS hybrid)
        ├── 64-bit long mode (x86_64) | EL1 (AArch64)
        ├── HHDM: all physical memory mapped at HHDM_OFFSET
        ├── Passes: memory map, framebuffer, kernel load addresses
        └── Jumps to kernel _start()

              Phase 1 — Foundation
              1. Serial / UART init     (banner immediately)
              2. GDT + IDT + TSS.RSP0  (x86_64) | VBAR_EL1 (AArch64)
              3. Physical frame allocator (bitmap, huge-page aware)
              4. 4-level paging        (HHDM stored, map_page ready)
              5. Kernel heap           (64 MB / 16 MB LockedHeap)
              6. Framebuffer console   (laptop, 2x scaled 8×8 font)

              Phase 2 — Preemptive Scheduler
              7. PIC remap + PIT 100 Hz
              8. Process table + kernel stacks
              9. Round-robin scheduler (timer ISR context switch)

              Phase 3 — IPC
             10. Per-process inbox queues + blocking send/recv
             11. Named port registry (service discovery)

              Phase 4 — Syscall + User Space
             12. STAR/LSTAR/FMASK/EFER + swapgs PERCPU entry
             13. First ring-3 process (nexus-init via IRETQ)
             14. SYS_WRITE, SYS_SLEEP, SYS_GETPID, SYS_IPC_* live
             15. Idle loop (preempted every 10 ms)

              Phase 5 — AI Core
             16. Register system ports (nexus.ai, nexus.fs, nexus.gpu)
             17. Spawn nexus-ai daemon into ring-3
             18. AI Core listening on nexus.ai port for IPC queries
```

## Phase Roadmap

| Phase | Status | Scope | Verified |
|-------|--------|-------|----------|
| 1 | **Done** | Boot, GDT/IDT, physical memory, paging, heap, framebuffer | `[pmem] 4090 MiB usable` |
| 2 | **Done** | Preemptive scheduler, 8259A PIC, 8253 PIT, round-robin | `[task-demo] alive at 9s` |
| 3 | **Done** | IPC ring-buffers, blocking send/recv, named port registry | `ping#00007 round-trip OK` |
| 4 | **Done** | `syscall`/`sysretq`, ring-3 user process, SYS_WRITE/SLEEP/IPC | `Hello from ring 3!` |
| 5 | **In Progress** | **AI Core server** — user-space Ollama IPC, `nexus.ai` port, SYS_IPC_QUERY | See [PHASE5_ARCHITECTURE.md](PHASE5_ARCHITECTURE.md) |
| 6 | Planned | NexusTerminal ↔ AI Core via IPC |
| 7 | Planned | VFS, network stack, NexusOS installer |

## Test VM

A QEMU + KVM test VM is pre-configured under `scripts/vm/`:

```bash
# Quick ISO test (no GPU rebinding)
./scripts/vm/nexusos-vm-test.sh

# RTX 4080 GPU passthrough (IOMMU Group 16, isolated)
sudo ./scripts/vm/vfio-bind.sh
./scripts/vm/nexusos-vm-gpu.sh --cdrom   # install
./scripts/vm/nexusos-vm-gpu.sh           # boot installed
sudo ./scripts/vm/vfio-unbind.sh
```

VM disk image lives at `/media/loufogle/Data/vms/nexusos/nexusos.qcow2` (40 GB).
A `libvirt` domain `nexusos` is registered in virt-manager for GUI access.

## Phase 5 Development

See [PHASE5_ARCHITECTURE.md](PHASE5_ARCHITECTURE.md) for:
- Kernel syscall additions (SYS_IPC_QUERY, SYS_IPC_TIMEOUT, SYS_GPU_MMAP)
- nexus-ai daemon implementation
- IPC protocol specification
- Phase 5.0–5.3 roadmap

**Quick start for Phase 5:**

```bash
# Create feature branch
git checkout -b phase/5-ai-core

# Build with AI-Core (adds syscalls + port registry)
cd kernel && make setup && make laptop && make iso-laptop && make run-laptop

# In VM, nexus-ai should spawn and listen on nexus.ai port
```

## Legacy Code

The original Ubuntu/Grub-based 32-bit C kernel, `CMakeLists.txt`, `config.mk`,
and Arch ISO configs are archived under `legacy/`. They are not referenced by
any current build target.

---

*NexusOS — AI-native from the first instruction.*