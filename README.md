# NexusOS Kernel

> **The world's first AI-native operating system.**  
> Built from scratch. No Linux. No glibc. No distro assumptions.

**Current version: v0.6.0** — Phases 1–5 complete and verified. Ring-3 interactive shell (`nexus>`) boots. Installer writes GPT + FAT32 ESP + kernel to VirtIO disk.

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

# 2. Build + ISO (laptop target)
make laptop && make iso-laptop   # → build/nexusos-laptop.iso

# 3. Install to a disk image and boot (QEMU)
make disk-laptop                  # creates build/nexusos-laptop.qcow2 (8 GB)
make run-install-laptop           # boot ISO + disk: installer runs, writes kernel to disk
make run-installed-laptop         # boot from installed disk only (OVMF)

# 4. Write to USB for bare-metal
sudo dd if=build/nexusos-laptop.iso of=/dev/sdX bs=4M status=progress oflag=sync

# Other targets
make tiamat && make iso-tiamat
make bahamut && make iso-bahamut
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
    ├── drivers/
    │   ├── pci.rs           legacy PCI I/O scanner (0xCF8/0xCFC, BAR0)
    │   └── virtio/
    │       ├── mod.rs       VirtIO legacy I/O register helpers
    │       └── blk.rs       VirtIO-blk driver (8-entry virtqueue, polling)
    ├── userspace/
    │   ├── mod.rs           ring-3 process spawn, page mapping, USER_CODE_BASE
    │   └── shell_init.asm   ring-3 interactive shell (NASM flat bin, built by build.rs)
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
             13. First ring-3 process (IRETQ → USER_CODE_BASE)
             14. 19 syscalls live (SYS_EXIT … SYS_EXEC)
             15. Idle loop (preempted every 10 ms)

              Phase 5 — AI Core + Drivers + Shell
             16. PCI scan → VirtIO-blk detect → disk capacity
             17. Register system ports (nexus.ai, nexus.fs, nexus.gpu)
             18. Spawn nexus-ai daemon, IPC on nexus.ai port
             19. PS/2 keyboard (IRQ1, ring-buffer, BlockedOnKey)
             20. FAT32 via fatfs crate (sector-buffered DiskIo adapter)
             21. Installer: GPT + FAT32 ESP + BOOTX64.EFI + kernel ELF → disk
             22. ring-3 shell: `nexus>` prompt, SYS_READ_CHAR, 10 commands
                 (help/version/uname/echo/ls/cat/run/ps/clear/reboot)

              Phase 6 — Filesystem + Program Execution
             23. ls / cat from ring-3 (SYS_FS_LIST / SYS_FS_READ)
             24. ELF64 loader + SYS_EXEC: `run HELLO.ELF` runs a ring-3 program
```

## Phase Roadmap

| Phase | Status | Scope | Verified |
|-------|--------|-------|----------|
| 1 | **Done** | Boot, GDT/IDT, physical memory, paging, heap, framebuffer | `[pmem] 4090 MiB usable` |
| 2 | **Done** | Preemptive scheduler, 8259A PIC, 8253 PIT, round-robin | `[task-demo] alive at 9s` |
| 3 | **Done** | IPC ring-buffers, blocking send/recv, named port registry | `ping#00007 round-trip OK` |
| 4 | **Done** | `syscall`/`sysretq`, ring-3 user process, SYS_WRITE/SLEEP/IPC | `Hello from ring 3!` |
| 5 | **Done** | AI Core (nexus.ai), PS/2 keyboard, VirtIO-blk, FAT32, installer, ring-3 shell | `nexus>` prompt |
| 6.1 | **Done** | Ring-3 filesystem access — `ls` / `cat` via SYS_FS_LIST/SYS_FS_READ | `nexus> ls` lists ESP |
| 6 | **Core done** | ELF64 loader + `SYS_EXEC` + parent/child wait; `run HELLO.ELF` | ring-3 ELF prints + exits |
| 5.4 | **Next** | Boot from installed disk (OVMF); VirtIO-vsock → Ollama HTTP | — |

## Test VM

QEMU + KVM scripts live under `scripts/vm/`:

```bash
# Quick ISO test
./scripts/vm/nexusos-vm-test.sh

# RTX 4080 GPU passthrough (IOMMU Group 16)
sudo ./scripts/vm/vfio-bind.sh
./scripts/vm/nexusos-vm-gpu.sh --cdrom   # install run
./scripts/vm/nexusos-vm-gpu.sh           # boot installed
sudo ./scripts/vm/vfio-unbind.sh
```

Persistent VM disk: `/media/loufogle/Data/vms/nexusos/nexusos.qcow2` (40 GB).
Libvirt domain `nexusos` is registered in virt-manager.

---

*NexusOS — AI-native from the first instruction.*