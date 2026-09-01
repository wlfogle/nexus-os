# WARP.md — nexus-os

**Read this file at the start of every session. Read `docs/NEXUS-OS-SOURCE-OF-TRUTH.md` first — it wins over all other docs.**

## Infrastructure Quick Reference (2026-07-28)

- **ISP:** Spectrum gigabit `74.134.128.100`. Archer AX55 Pro Router mode `192.168.12.254`. **DMZ → 192.168.12.222**.
- **Tiamat** `192.168.12.242` — Proxmox VE host. All VMs/CTs live here. Gateway: `192.168.12.222` (OpenWrt).
- **CT-300** `192.168.12.30` — all media services. Gateway `192.168.12.222` (OpenWrt). wg0 split-tunnel (10.92.29.5).
- **Bahamut** `192.168.12.244` — AdGuard DNS, Vaultwarden, Caddy/DuckDNS. Gateway `192.168.12.222`. **PiVPN = mgmt tool only** (WG server moved to OpenWrt).
- **OpenWrt VM-100** `192.168.12.222` — **Primary router/gateway + WireGuard SERVER** (10.92.29.1:51820). LuCI on :80 and :8101. Full routing migration COMPLETE (2026-07-28).
- **Laptop** `192.168.1.188` (wired) / `192.168.12.172` (WiFi) — admin/dev workstation.
- SSH aliases: `ssh tiamat`, `ssh bahamut`, `ssh openwrt`, `ssh mediastack`
- Active modules: **nexus-mediastack** (live media stack), **bulletproof-mediastack** (forerunner/reference only)
- **ALL repos live in `/media/loufogle/Data/Repos/<name>`** (2026-08-07). Home holds no
  repos. Clone new work there; do not clone into `~`. Git bundles preserving
  local-only state that could not be pushed are in `Repos/_bundles/<repo>/`.
- **JDownloader2**: runs on Tiamat at `/root/JDownloader2/`, systemd service `jdownloader2.service`, requires `default-jre-headless` (OpenJDK 21). Connected via MyJDownloader as `mediastack-jd2`.

---

This file provides guidance to WARP (warp.dev) when working with code in
this repository.

## Project Overview

NexusOS is a **from-scratch Rust microkernel** — the world's first AI-native
operating system.  No Linux.  No glibc.  No distro assumptions.

**Current state: v0.6.2 — Phases 1–5 and Kernel Completion Roadmap K1–K5 verified under QEMU (`-smp 4`). Ring-3 interactive shell boots; ACPI/SMP (multi-core scheduling with per-AP LAPIC timers), xHCI/USB-HID, fd-based VFS, and process spawn/wait/kill are all live.**  
**Next: Phase K6 — real-hardware graphics validation (Limine framebuffer on the i9-13900HX's Intel iGPU, bare metal). See the "NexusOS Kernel Completion Roadmap" plan for full detail on every K1–K6 increment.**

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
| `arch/x86_64/{gdt,idt,interrupts,timer_isr}` | 1 | CPU structures, naked timer ISR — `gdt`/`TSS` are now per-core arrays (K5.4) |
| `arch/aarch64/exceptions` | 1 | AArch64 vector table, VBAR_EL1 |
| `memory/{physical,paging,heap}` | 1 | Bitmap allocator (huge-page aware), 4-level paging, heap |
| `io/{serial,uart,framebuffer}` | 1 | Serial/UART, 2x-scaled framebuffer console |
| `timer/{pic,pit}` | 2 | 8259A PIC remap, 8253 PIT 100 Hz — PIC now fully masked once the I/O APIC takes over (K5.3) |
| `process` | 2 | PCB, ring-0/ring-3 spawn, per-process fd table + heap (K3/K1); every id-keyed lookup rejects pid 0 (K5.6 fix) |
| `scheduler` | 2 | Preemptive round-robin — per-core `CURRENT`/runqueue pick with `PICK_LOCK` for genuine multi-core correctness (K5.6) |
| `ipc/{mod,ports}` | 3 | Message queues (depth=8), blocking send/recv, named ports |
| `syscall` | 4 | STAR/LSTAR/FMASK/EFER, GS-relative naked entry, per-core PERCPU array, 36 syscalls (see `kernel/src/syscall/mod.rs`) |
| `userspace` | 4 | Ring-3 page mapping in PML4[1], NASM shell binary (shell_init.asm) |
| `build.rs`  | 6 | Assembles userspace/shell_init.asm → OUT_DIR/shell_init.bin via NASM |
| `acpi` | K5.1 | RSDP → RSDT/XSDT → FADT + MADT parsing, incl. Interrupt Source Override |
| `arch/x86_64/{lapic,ioapic}` | K5.2–K5.6 | Local APIC + I/O APIC bring-up, AP registration, calibrated per-AP LAPIC periodic timer |
| `drivers/xhci` + `drivers/usb_hid` | K4 | xHCI controller, USB HID boot-protocol keyboard/mouse |
| `fs/{fat,vfs}` | K3 | FAT32 + per-process fd table (SYS_OPEN/CLOSE/READ/LSEEK) |

Full per-increment writeups (with verification methodology) live in the "NexusOS Kernel Completion Roadmap" plan, not duplicated here.

## Phase 5: AI Core — DONE

Real Ollama HTTP client (`net::tcp_request`) is live, not a mock — the `nexus-ai` daemon serves `nexus.ai` IPC requests by making a real blocking TCP call to Ollama. [PHASE5_ARCHITECTURE.md](PHASE5_ARCHITECTURE.md) is the original **design doc** for this phase (syscall numbers/snippets there are illustrative pre-implementation sketches, superseded by the real numbering in `kernel/src/syscall/mod.rs` — e.g. `SYS_IPC_QUERY`/`SYS_IPC_TIMEOUT`/`SYS_GPU_MMAP` actually landed as 10/11/12, not 7/8/9). Kept for historical design context only.

## Kernel Completion Roadmap: K1–K6

Phases 1–5 above built the original AI-native foundation; **K1–K6** (a separate, later roadmap — see the "NexusOS Kernel Completion Roadmap" plan for full per-increment detail) closed out everything a daily-driver kernel still needed:

| Phase | Scope | Status |
|-------|-------|--------|
| K1 | User-space heap (`SYS_BRK`) | DONE |
| K2 | Decoupled process spawn/wait/kill, `Zombie` state | DONE |
| K3 | Per-process fd table + offset-aware VFS (`SYS_OPEN/CLOSE/READ/LSEEK`) | DONE |
| K4 | xHCI + USB HID (keyboard/mouse boot protocol) | DONE |
| K5 | ACPI (RSDP/FADT/MADT), LAPIC/IOAPIC, per-core TSS/PERCPU, AP bring-up (Limine `SmpRequest`), per-CPU scheduler + calibrated AP LAPIC timers | DONE |
| K6 | Real-hardware graphics validation (Limine framebuffer on real Intel iGPU) + VirtIO-GPU for the QEMU test path | Next |

K5 in particular required a real-hardware-correctness mindset throughout: every increment was verified against real QEMU-reported architectural constants (not just "it boots"), and K5's final increment caught and fixed a genuine, previously-latent triple-fault bug (a `pid == 0` table-lookup ambiguity that only manifests once a second CPU core is actively scheduling) — see the plan's increment 6 writeup for the full root-cause analysis.

## Key Gotchas

1. **Limine huge pages** — `map_page` must detect 1 GB/2 MB huge-page entries
   (HUGE flag) and refuse to walk through them.  User addresses must be in
   PML4[1] (512 GB+) where Limine has no identity-map entries.

2. **Limine v12 config syntax** — `key: value` (colon-space), NOT `key=value`.
   Entry headers are `/Name`, NOT `:Name`.

3. **TSS.RSP0** — must be updated to each process's kernel stack top on every
   context switch so timer interrupts from ring 3 land on the right stack.

4. **Syscall PERCPU** — use `gs:[0]` / `gs:[8]` (GS-relative addressing) in
   the naked syscall entry stub, not `[PERCPU+N]` absolute symbol references.

5. **Syscall calling convention** — syscalls use x86-64 System V:
   - rax = syscall number
   - rdi, rsi, rdx, r10, r8, r9 = arguments
   - Return value in rax (may be negative for errors)

6. **VM** — Test VM at `/media/loufogle/Data/vms/nexusos/`, scripts in
   `scripts/vm/`.  RTX 4080 is in IOMMU Group 16 (isolated), passthrough
   ready via `scripts/vm/vfio-bind.sh`.

7. **Phase 5 reserved ports** — nexus.ai, nexus.fs, nexus.gpu, nexus.net are
   registered at boot. Do not use these names for non-system services.

## Host Environment

Pop!_OS 22.04 on Intel i9-13900HX + RTX 4080 + 64 GB DDR5.
Preferred package manager: `nala` (not raw `apt`).

### Laptop host policy (2026-08-03)

- **NO Docker.** All Docker/containerd packages are purged from the laptop and
  must not be reintroduced. Container workloads belong on **CT-300**
  (`192.168.12.30`). Use libvirt/QEMU locally instead.
- **NVIDIA:** `nvidia-driver-610-open` (610.43.02) is the only driver; DKMS
  builds `nvidia/610.43.02`. The stale `nvidia-dkms-580` package was removed.
- **Intentionally masked units:** `pop-upgrade.service`, `acpid.service`,
  `nvidia-powerd.service`, plus user-level `pop-upgrade-notify.{timer,service}`.
  Do not unmask these — release upgrades are managed manually.
- **Ubuntu Pro:** attached with `esm-infra` + `esm-apps` enabled. **Kernel
  Livepatch must stay off** — Canonical only livepatches its own signed
  kernels, and Pop kernels are built by `jenkins@warp.pop-os.org`. Enabling it
  crash-loops the snap and leaves systemd `degraded`.
  Upstream report: pop-os/pop#4062.
- `fluidsynth` is masked for the **gdm** user only (`/var/lib/gdm3/.config/`),
  which fixes a shutdown timeout on `user@111.service`. It stays enabled for
  the `loufogle` session.

## Code Quality Standards

Every function must be complete and working.  No stubs, no TODOs, no zombie code.

- **NO** `TODO`, `FIXME`, `XXX`, `HACK`, `stub`, or `unimplemented` markers
- **NO** incomplete functions or dead code paths
- Code compiles clean and runs correctly before committing
- Phase 5: Syscall stubs (SYS_GPU_MMAP) are acceptable **only** with inline comments explaining Phase 5.1+ scope

## Testing Phase 5

```bash
# Run integration test
./scripts/phase5-integration-test.sh

# Expected output:
# ✓ Phase 5 boot sequence
# ✓ nexus-ai daemon spawned
# ✓ Reserved port: nexus.ai
# ✓ Reserved port: nexus.fs
# ✓ nexus-ai IPC bind
# ✓ nexus-ai daemon loop
```

---

## Future Phases

- **Phase K6** — Real-hardware graphics validation (bare-metal Limine framebuffer on the laptop's Intel iGPU) + VirtIO-GPU driver for QEMU
- **Beyond K6 (explicitly deferred, see the Kernel Completion Roadmap plan)** — compositor/window manager, widget toolkit, desktop shell, file manager, GPU acceleration beyond mode-setting, filesystems beyond FAT32
- **Long-term** — Linux/BSD/macOS/Win32 personality servers (see "The Mission" below)
---

## Session Continuity

**IMPORTANT: At the start of EVERY conversation, before doing anything else:**
1. Read `docs/NEXUS-OS-SOURCE-OF-TRUTH.md` — it wins over every other doc.
2. Read `/media/loufogle/Data/Repos/nexus-os/packages/nexus-terminal/PROGRESS.md`
   to get current state.
3. Check `git --no-pager log --oneline -5` in `/media/loufogle/Data/Repos/nexus-os`
   to see recent commits.
4. You are already caught up. Do not ask the user to explain what we were doing.

Before proposing a new package or tool, **check `packages/` first**. Ideas here
get designed, parked, forgotten and then re-invented — that is the specific
failure `packages/nexus-brain` exists to stop. Search the catalog rather than
trusting recall of a name.

This eliminates the need to re-explain context every session.

---

## The Mission

NexusOS is the **world's first universal AI-native operating system**.

No OS has ever natively bridged all four major platform ABIs in a single microkernel.
NexusOS will be the first.

### Universal Compatibility via Personality Servers

```
NexusOS Microkernel (Rust, from scratch)
├── nexus.linux  — ELF binaries, Linux syscalls     → run any Linux app
├── nexus.bsd    — ELF binaries, BSD/POSIX syscalls → run any BSD app
├── nexus.macos  — Mach-O binaries, Darwin APIs     → run any macOS app
├── nexus.win    — PE/COFF binaries, Win32 API      → run any Windows app
└── nexus.native — NexusOS native (Rust, our ABI)
```

Each personality server is a userspace process that:
1. Registers itself with the kernel for a binary format
2. Translates platform syscalls → NexusOS IPC
3. Returns results in the expected ABI format

### NexusStore
One store. All platforms. AI-curated.
Install anything from Linux, macOS, BSD, or Windows — no emulation, no VM.
Just native personality servers translating the ABI.

### Milestones
- Phase 5 (now):      Foundation — kernel, disk, FAT32, AI Core, installer
- Phase 6 (Month 2):  Linux personality server — run any Linux ELF
- Phase 7 (Month 3):  BSD personality server + NexusStore v0.1
- Phase 8 (Month 4):  Windows personality server (Wine-level API coverage)
- Phase 9 (Month 6):  macOS personality server (Darling-level Mach-O support)
- Phase 10 (Month 8): NexusStore v1.0 — install anything from any source
- Phase 11 (Month 12): Bare metal on i9-13900HX + RTX 4080 daily driver
