# NexusOS Vision Archive

The documents in this directory describe the **original long-term vision**
for NexusOS — a standalone, AI-native operating system built from scratch:
our own kernel (`boot/`, `kernel/`), AI-aware scheduler and syscall layer,
integrated GPU/tensor subsystems, and native container orchestration.

That path is still the preferred destination for the project. It is not the
*current* implementation, because:

- The kernel work underneath these designs is not yet stable enough to boot
  a usable userspace on commodity hardware.
- The AI-native kernel subsystems (`ai/`, `virt/`, `net/`, `fs/` in
  `docs/vision/AI_NATIVE_ARCHITECTURE.md`) have no working reference
  implementation.
- Shipping a daily-drivable distro with real users and real hardware
  (NVIDIA Optimus + RTX 4080, 64 GB DDR5, ZFS root) requires a working
  Linux kernel today, not eventually.

## What is actually shipped (at repo root)

The active implementation is a **standalone Linux distribution
bootstrapped from Ubuntu Jammy (22.04) via `debootstrap`** — see the
project `README.md`, `CHANGELOG.md`, `scripts/build-iso.sh`, and
`installer/` for the real build. KDE Plasma X11 + SDDM, NVIDIA PRIME
render offload, `nexuspkg` universal package manager, Stella/Max Jr. AI
companions as FastAPI services, 65+ self-hosted media services, Calamares
installer.

Everything in this directory should be read as **"where we want to
eventually arrive"**, not "what builds today".

## Contents

| File                               | Original purpose                                               |
|------------------------------------|----------------------------------------------------------------|
| `AI_NATIVE_ARCHITECTURE.md`        | Kernel subsystems for AI (tensor mm, GPU scheduler, AI syscalls) |
| `DEVELOPMENT-KERNEL.md`            | Dev guide for the from-scratch kernel (`make run`, QEMU, GDB)  |
| `DEVELOPMENT_ROADMAP.md`           | Earlier Pop!_OS-overlay phase plan (pre-debootstrap pivot)     |
| `ROADMAP-KERNEL.md`                | Phased kernel roadmap (Phase 0 baseline → Phase N)             |
| `WARP-KERNEL.md`                   | WARP.md for the kernel-build workflow                          |
| `README-public.md`                 | Earlier public-facing README for the vision                    |
| `social-media-preview.md`          | Launch messaging written for the vision                        |
| `website-preview-content.md`       | Landing-page copy written for the vision                       |

## When (if) the vision path becomes feasible

Bring these documents back to the top level and update the active
`README.md`, `WARP.md`, and `ROADMAP.md` to cite them as the implemented
design. Until then they live here so the work is not lost and so the
direction stays visible to anyone reading the repo.
