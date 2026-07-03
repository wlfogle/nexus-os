# NexusOS Component Inventory
NexusOS is a **from-scratch, AI-native Rust microkernel** (this repo). This file maps what
**currently** lives in `/home/loufogle/nexus-os`. Anything superseded/old has been moved off
`main` to the **`archive/legacy`** branch (recover via `git checkout archive/legacy`).

## The OS (kernel + boot)
- `kernel/` — from-scratch Rust microkernel. v0.6.x, Phases 1–6: boot, preemptive scheduler,
  IPC, syscalls, ring-3 shell (`nexus>`), FAT32, ELF64 loader, self-installer.
- `drivers/`, `lib/`, `include/`, `limine/` — driver/support code and the Limine boot path.
- `installer/` — NexusOS installer; `distro/os-release` — distro identity.
- `scripts/`, `tests/`, `Makefile`, linker scripts — build, VM helpers, tests.

## Core userland (`core/`)
- `core/services/` — **Stella 🐕 (operations)** + **Max Jr. 🐱 (security)** AI companions,
  coordinated by `nexus-orchestrator` (`stella.py`, `maxjr.py`, `nexus-orchestrator.py` +
  systemd units). **NexusOS is dedicated to them.**
- `core/desktop/` — NexusDE desktop (QML) + branding/mascot art.
- `core/branding/`, `core/security/`, `core/shell/`, `core/package-management/`,
  `core/config/`, `core/bin/`, `core/ai/` (Ollama integration), `core/installer/`,
  `core/media-stack/`.

## userspace/
- `userspace/system/` — `nexuspkg` (native package manager + foreign-format handlers for
  `.deb`/`.rpm`/`.pkg.tar.zst`/flatpak/snap/appimage/pip/npm/cargo), `nexus-ai-assistant`,
  `nexus-setup-assistant`, system services.
- `userspace/apps/` — desktop app components: `nexus-terminal`, `nexusbrowser`, `kvm-manager`,
  `omniosearch`, `hyperion`, `eartrumpet`, `ollama-manager`, `portproton`, `proxmox-admin`,
  `ai-coding-assistant`.

## packages/ (component apps)
`nexus-terminal`, `nexus-codex`, `nexus-brain`, `kvm-manager`, `OmnioSearch`, `Hyperion`,
`eartrumpet-linux`, `ollama-manager-gui`, `ollama-code-checker`, `PortProton-Enhanced`,
`mobalivecd-linux`, `i9-13900hx-optimizations`, `universal-zfs-installer`,
`firestick-mediacontrol-app`, `mediastack-control-popos`.

## Current media stack (external repo)
`nexus-mediastack` — the current media stack, all services consolidated into one LXC
(**CT-300**). See `docs/NEXUS-OS-SOURCE-OF-TRUTH.md`.

## Moved to `archive/legacy` (off `main`, 2026-06-29)
Old/superseded repos and docs, kept for reference only: `ai-powerhouse-setup`, `awesome-stack`,
`awesome-stack-optimization-suite`, `homelab-media-stack`, `mediastack-control`,
`media-stack-admin-scripts`, `ai-sysadmin-supreme` (Stella's ancestry), `garuda-hello`,
`linux-gaming-vm-toolkit`, the Garuda/Calamares ZFS installer, the old `legacy/` tree, and
outdated docs (`DISTROWATCH_SUBMISSION.md`, `RELEASE_CHECKLIST.md`,
`UNIVERSAL_PACKAGE_MANAGER_SPEC.md`, `Analysis.md`).

## Preservation rule
Do not delete working source during cleanup — it lives on `archive/legacy`. Keep `main` to
current NexusOS only.
