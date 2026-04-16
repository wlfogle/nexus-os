# NexusOS Roadmap

Two tracks live side by side in this repository:

- **Active track — Ubuntu Jammy distro.** Ships today as a bootable ISO
  built by `scripts/build-iso.sh`. Everything below is part of this
  track.
- **Vision track — AI-native kernel from scratch.** Preferred long-term
  direction, not currently feasible. Archived under
  `docs/vision/` (see `docs/vision/README.md` and
  `docs/vision/ROADMAP-KERNEL.md` for its own phases).

The two tracks share a name and a repo. They do **not** share a build
pipeline. When anything on this page says "NexusOS", it means the active
Ubuntu-based distro.

## Where we are (1.0.1-dev, 2026-04-07)

Per `CHANGELOG.md`:

- 1.0.0-dev (2026-03-09) — first bootable 4.8 GB ISO from debootstrap.
  KDE Plasma X11 + SDDM, NVIDIA PRIME, `nexuspkg` universal package
  manager with 15+ backends, Stella/Max Jr. AI companions as FastAPI
  services, 65+ media services.
- 1.0.1-dev (2026-04-07) — full sync of `homelab-media-stack` into
  `core/media-stack/homelab/`. Seerr native, Jackett/Deluge fallbacks,
  Sunshine+Moonlight streaming, Phase 10 Home Assistant (HAOS VM-500),
  stack-wide credentials doc, 10-tier container boot order, pipeline
  watchdog, Radarr IP fix.

Known limitations (carried forward):

- GUI package manager not yet implemented (CLI-only).
- No ARM64 build target.
- No community repo hosting.
- Enterprise/signing features not yet shipped.

## Near-term (next one to two releases)

Milestone **1.0.2-dev — Hardware validation and installer polish**.

1. Fresh-install (`INSTALL_MODE=fresh`) tested on at least two distinct
   machines (the reference i9-13900HX + RTX 4080 laptop, plus one
   NVIDIA-less box). Document edge cases in `installer/INSTALL.md`.
2. Calamares modules smoke-tested from the live ISO (language,
   locale/keyboard, users, partition, summary, zfs if present). File
   any required fixes against `core/installer/`.
3. `patch-iso.sh` hardened for failure recovery (idempotent rerun after
   partial squashfs mount failure).
4. NVIDIA driver selection auditable from the live session —
   `nexus-control gpu` reports the driver series and confirms PRIME is
   wired.

Milestone **1.0.3-dev — `nexuspkg` quality**.

1. Each of the 15+ backends verified end-to-end on a fresh install:
   search → install → upgrade → remove. Record results in
   `UNIVERSAL_PACKAGE_MANAGER_SPEC.md`.
2. Structured failure modes: every backend returns well-defined error
   codes (no silent fallthrough). Zero stubs per repo policy.
3. Package conversion (e.g. `rpm → deb` via `alien`) exits cleanly on
   unconvertible payloads rather than partial installs.

Milestone **1.0.4-dev — AI companion hardening**.

1. Orchestrator/Stella/Max Jr. services expose authenticated endpoints
   only (no open `:8600-8602` on LAN without API key).
2. systemd timers verified to self-recover after service crashes.
3. `stella --digital-fortress` + `maxjr --gaming-mode` are reversible
   via the same CLIs — document + test rollback.

## Mid-term (next several releases)

- **Repository + signing infrastructure.** APT repo with signed
  `Release`/`Packages`, published via the existing Caddy instance on
  Bahamut or a dedicated CT. Includes `nexuspkg`-native repository for
  cross-backend meta packages.
- **ARM64 build target.** Second ISO flavor for Raspberry Pi 5 / Orange
  Pi class boards. Likely headless; media-stack-only profile initially.
- **Calamares fresh-install-without-ZFS.** LVM or plain ext4 path for
  users who do not want ZFS root. Keep ZFS-on-root as the recommended
  option.
- **In-place upgrade**. `sudo nexus-control update` promoted from
  package-manager wrapper to a proper release-channel upgrade
  (ISO-metadata-aware, rollback via ZFS snapshot where ZFS root is
  used).

## Long-term (aspirational, active track)

- **Community repo hosting** for third-party `nexuspkg` packages.
- **GUI package manager** wrapping `nexuspkg` (likely Tauri, reusing
  the `nexus-terminal` stack).
- **Mobile companion app** reading Stella/Max Jr. metrics.
- **Distribution-agnostic installer ISO** that can recover or transplant
  an existing install to a new disk.

## Long-term (vision track)

Kernel-from-scratch, AI-native syscall layer, tensor memory manager,
integrated GPU scheduler, native container orchestration. See
`docs/vision/ROADMAP-KERNEL.md` and
`docs/vision/AI_NATIVE_ARCHITECTURE.md`. Unblocking this track requires
a usable minimal kernel + userspace pair running on real hardware; none
of that is in scope for the next few releases on the active track.

## How to read a milestone entry

- Each numbered item is a deliverable, not a wish. Each should close
  with a working artifact, a passing test, or a committed change.
- "Done" means merged to `main` **and** the behavior is demonstrable
  from the live ISO or an overlay install — not "code exists but
  untested".
- Every commit path obeys the Zero Stub Code policy in `WARP.md`.
