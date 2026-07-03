# Contributing to NexusOS

NexusOS is a **from-scratch Rust microkernel** — no Linux, no glibc, no POSIX.
If you've ever wanted to work on a real OS kernel, this is your chance.

**Current state:** Phases 1–6 (core) complete — the kernel boots, schedules
processes preemptively, has IPC, syscalls, ring-3 userspace, a PS/2 keyboard
driver, VirtIO-blk disk I/O, a FAT32 filesystem, an ELF64 loader (`run`), and a
self-installer (GPT + FAT32 ESP + kernel). See `README.md` for the full boot
sequence.

## Where you can help

### Kernel (Rust, `#![no_std]`)

These are the highest-impact areas right now:

| Area | Difficulty | Description |
|------|-----------|-------------|
| **FAT32 write support** | Medium | Extend the read-only FAT32 driver to support file creation and writes |
| **AHCI / NVMe drivers** | Hard | Real hardware disk drivers (currently VirtIO-blk only) |
| **VirtIO-net** | Medium | Network driver — required before any networking stack |
| **TCP/IP stack** | Hard | Minimal IP + TCP + UDP — needed for AI Core to reach Ollama |
| **USB HID** | Hard | USB keyboard/mouse (currently PS/2 only) |
| **ACPI** | Medium | Power management, shutdown, device enumeration |
| **SMP** | Hard | Multi-core scheduler (currently single-core) |
| **ext4 read support** | Medium | Second filesystem for real-world disk access |

### Userspace

| Area | Difficulty | Description |
|------|-----------|-------------|
| **NexusTerminal** | Medium | Terminal emulator in userspace (see `PHASE6_NEXUSTERMINAL.md`) |
| **Shell** | Medium | Basic command interpreter for NexusTerminal |
| **Personality servers** | Hard | Linux/BSD/macOS/Win32 ABI translation layers (the long-term vision) |
| **Init system** | Medium | Service manager replacing the current hardcoded boot sequence |

### Documentation

| Area | Description |
|------|-------------|
| **Architecture docs** | Explain how paging, IPC, or the scheduler work — help newcomers understand the code |
| **Build guides** | Test the build on different Linux distros, document issues |
| **Syscall reference** | Document every syscall number, arguments, return values |
| **Driver writing guide** | How to add a new driver to NexusOS |

### Testing

| Area | Description |
|------|-------------|
| **QEMU testing** | Run the kernel on different QEMU configs, report what breaks |
| **Hardware testing** | Boot the ISO on real hardware, report results |
| **Fuzzing** | Fuzz syscall inputs from ring-3 |

## Build setup

You need four things:

```bash
# 1. Rust nightly (via rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. make, xorriso
# Debian/Ubuntu/Pop!_OS:
sudo apt install make xorriso
# Fedora:
sudo dnf install make xorriso
# Arch:
sudo pacman -S make xorriso

# 3. QEMU (for testing)
sudo apt install qemu-system-x86 qemu-system-aarch64

# 4. Clone and build
git clone https://github.com/wlfogle/nexus-os.git
cd nexus-os
make setup    # installs Rust targets, builds Limine bootloader
make laptop   # build x86_64 kernel
make iso-laptop
make run-laptop   # boots in QEMU
```

Three build targets from one codebase:

| Target | Arch | Use case |
|--------|------|----------|
| `laptop` | x86_64 | Full: framebuffer + serial + AI hooks |
| `tiamat` | x86_64 | Headless server: serial only |
| `bahamut` | AArch64 | Raspberry Pi / edge: minimal heap |

## Good first issues

If you're new to OS development, start here:

- **Add a new syscall** — follow the pattern in `kernel/src/syscall/`. Pick
  something simple like `SYS_UPTIME` (return tick count) or `SYS_GETPID`
  improvements.
- **Improve serial output formatting** — add color codes, timestamps, or log
  levels to the serial console.
- **Write a userspace test program** — create a ring-3 program that exercises
  IPC or disk syscalls and reports pass/fail.
- **Document a subsystem** — pick any module in `kernel/src/` and write a
  `docs/` page explaining how it works.
- **Test on your hardware** — boot the ISO and report what happens (GPU,
  serial output, crashes).

## Code standards

- **Rust, `#![no_std]`, nightly** — no standard library, no allocator until
  the heap is initialized.
- **No stubs or zombie code.** Every function must be complete and working.
  The only exception: syscall stubs for future phases must have inline
  comments explaining scope.
- **No `TODO`, `FIXME`, `XXX`, `HACK`, or `unimplemented!()` in committed
  code.**
- **Code must compile clean** — `make laptop` with no warnings before you
  open a PR.
- Kernel code uses `unsafe` where hardware access demands it. Every `unsafe`
  block must have a `// SAFETY:` comment explaining why it's sound.

## Commit messages

```
type(scope): description

Examples:
feat(kernel/drivers): add AHCI port detection
fix(scheduler): prevent double-free of process stacks
docs(syscall): document SYS_DISK_READ arguments
test(ipc): add ring-3 IPC stress test
```

## Project structure

```
kernel/src/
├── main.rs          — boot sequence, kernel tasks
├── arch/            — x86_64 and AArch64 CPU setup (GDT, IDT, exceptions)
├── memory/          — physical allocator, paging, heap
├── io/              — serial, UART, framebuffer
├── timer/           — PIC, PIT
├── process/         — PCB, kernel stacks, ring-3 spawn
├── scheduler/       — round-robin preemptive scheduler
├── ipc/             — message queues, named port registry
├── syscall/         — STAR/LSTAR entry, syscall dispatch
├── drivers/         — PCI, VirtIO-blk (add yours here)
└── userspace/       — ring-3 init process, page mapping

userspace/           — userspace daemons (nexus-ai, etc.)
scripts/             — build scripts, VM helpers, integration tests
docs/                — architecture documentation
```

## Workflow

1. Fork the repo
2. Create a feature branch: `git checkout -b feat/your-feature`
3. Build and test: `make laptop && make iso-laptop && make run-laptop`
4. Check serial output in QEMU for your changes
5. Commit (see format above)
6. Open a PR against `main`

## Communication

- **GitHub Issues** — bugs, feature proposals, questions
- **GitHub Discussions** — architecture discussions, design proposals
- **PRs** — code review happens here

## License

Contributions are licensed under GPL-3.0+. By submitting a PR, you agree
your code falls under this license.

---

*NexusOS — AI-native from the first instruction.*
