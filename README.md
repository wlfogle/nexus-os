# NexusOS Kernel

> **The world's first AI-native operating system.**  
> Built from scratch. No Linux. No glibc. No distro assumptions.

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
|------|---------|---------|
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
    ├── main.rs              entry point, Limine requests, kprintln!
    ├── panic.rs             kernel panic handler
    ├── arch/
    │   ├── x86_64/
    │   │   ├── gdt.rs       GDT + TSS (IST for double fault stack)
    │   │   ├── idt.rs       IDT, 256 entries
    │   │   └── interrupts.rs  #BP #DF #PF #GP #DE #UD #SS #AC
    │   └── aarch64/
    │       └── exceptions.rs  vector table (2KB aligned), VBAR_EL1
    ├── memory/
    │   ├── physical.rs      bitmap frame allocator from Limine mmap
    │   ├── paging.rs        4-level page tables, map_page, HHDM
    │   └── heap.rs          64 MB / 16 MB kernel heap, global allocator
    └── io/
        ├── serial.rs        COM1 serial, direct port I/O (x86_64)
        ├── uart.rs          PL011 UART MMIO at 0x09000000 (AArch64)
        ├── framebuffer.rs   8x8 bitmap font console (laptop feature)
        └── font8x8.rs       public domain ASCII bitmap font
```

## Boot Sequence

```
UEFI or BIOS firmware
  └── Limine (built from source)
        ├── 64-bit long mode (x86_64) | EL1 (AArch64)
        ├── HHDM: all physical memory mapped at a fixed offset
        ├── Passes: memory map, framebuffer info, kernel load addresses
        └── Jumps to kernel _start()
              1. Serial / UART init     (banner printed immediately)
              2. GDT + IDT             (x86_64) | VBAR_EL1 (AArch64)
              3. Physical frame allocator (bitmap over usable regions)
              4. Paging               (HHDM offset stored, map_page ready)
              5. Kernel heap          (N frames mapped, LockedHeap init)
              6. Framebuffer console  (laptop only)
              7. HLT / WFI idle loop
```

## Phase Roadmap

| Phase | Status | Scope |
|-------|--------|-------|
| 1 | In progress | Boot, memory management, CPU exceptions |
| 2 | Planned | Preemptive scheduler, timer, process table |
| 3 | Planned | IPC — message passing (microkernel core) |
| 4 | Planned | Syscall interface, ring-3 / EL0 user processes |
| 5 | Planned | **AI Core server** — user-space service, Ollama IPC |
| 6 | Planned | NexusTerminal ↔ AI Core via IPC |
| 7 | Planned | VFS, network stack |

## Legacy Code

The original Ubuntu/Grub-based 32-bit C kernel, `CMakeLists.txt`, `config.mk`,
and Arch ISO configs are archived under `legacy/`. They are not referenced by
any current build target.

---

*NexusOS — AI-native from the first instruction.*
