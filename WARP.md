# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in
this repository.

## Project Overview

NexusOS is a **standalone Linux distribution bootstrapped from Ubuntu
Jammy (22.04) via `debootstrap`**. It ships KDE Plasma X11 + SDDM, NVIDIA
PRIME render offload (Intel iGPU + NVIDIA dGPU), every major Linux
package manager compiled from source (via `nexuspkg`), and two AI
companions — Stella (security) and Max Jr. (performance) — implemented
as systemd-managed FastAPI services.

The repository is a monorepo: first-party components (CLIs, services,
branding, installer, media stack), vendored consolidated packages, and
scripts all live in one tree. See `README.md` for the user-facing
overview and `docs/CONSOLIDATION.md` for the monorepo source map.

**Preferred long-term direction (not the current implementation)** is a
from-scratch AI-native kernel. That work is archived in `docs/vision/`
and is still the project's eventual destination — it is simply not
feasible to ship today. The legacy from-scratch kernel source tree
(`kernel/`, `boot/`, `linker.ld`, `Makefile`, `CMakeLists.txt`,
`config.mk`, `create_phase1.sh`, `create_phase2.sh`, `grub.cfg`) is
preserved for the same reason. None of it is invoked by the current
distro build.

**Host environment.** The active build target is Pop!_OS 22.04 LTS on
Intel i9-13900HX + RTX 4080 + 64 GB DDR5. Uses `nala` (not raw `apt`) as
the preferred package manager per project rules.

## Common Commands

### Build the live ISO

```bash
sudo ./scripts/build-iso.sh                        # default (~70 min)
sudo ./scripts/build-iso.sh --no-nvidia            # skip NVIDIA stack
sudo ./scripts/build-iso.sh --output ~/iso         # custom output dir
sudo ./scripts/build-iso.sh --mirror http://...    # alternate APT mirror
```

Output lands in `build/nexusos-1.0-YYYY.MM.DD-x86_64.iso`. The builder
debootstraps a fresh Jammy root, configures APT + NVIDIA PPA, installs
KDE + AI services + the media stack, builds a squashfs, and packs a
hybrid ISO (UEFI + BIOS).

### Patch an existing ISO (faster iteration)

```bash
sudo ./scripts/patch-iso.sh                        # auto-find latest ISO
sudo ./scripts/patch-iso.sh path/to/nexusos.iso    # specific ISO
```

Used to apply fixes without a full rebuild.

### Test in QEMU

```bash
qemu-system-x86_64 -m 4G -enable-kvm \
    -cdrom build/nexusos-1.0-*.iso
```

### Write to USB

```bash
sudo dd if=build/nexusos-1.0-*.iso of=/dev/sdX bs=4M status=progress oflag=sync
```

### Installer (overlay on existing Pop!_OS, or fresh ZFS-on-root)

```bash
sudo ./installer/nexus-install.sh                  # overlay mode
sudo INSTALL_MODE=fresh TARGET_DISK=/dev/sdX \
    ./installer/nexus-install.sh                   # ZFS fresh install
```

See `installer/README.md` and `installer/WARP.md` for installer
internals.

### Daily runtime commands (inside a running NexusOS system)

```bash
nexus-control status | health | gpu | services
sudo nexus-control update

stella --status | --scan
sudo stella --digital-fortress

maxjr --monitor | --temperature
sudo maxjr --optimize | --gaming-mode

nexuspkg install <pkg>               # auto-detect best backend
nexuspkg install --backend flatpak discord
nexuspkg search <term>               # search all backends
nexuspkg repos                       # list backends
```

## Code Architecture

Repo layout (abbreviated — see `README.md` for the full tree):

```
nexus-os/
├── scripts/                # ISO builder, patcher, livecd, rescue-usb
├── core/
│   ├── bin/                # nexus-control, nexuspkg, stella, maxjr, first-run
│   ├── services/           # orchestrator (8600), stella (8601), maxjr (8602)
│   ├── config/             # sysctl, modprobe, udev, limits, hw/, optimization/
│   ├── branding/ desktop/ shell/ security/ gaming/
│   ├── ai/                 # ollama, powerhouse, sysadmin, ollama-checker
│   ├── installer/          # calamares modules, zfs support
│   └── media-stack/        # docker-compose.yml, homelab/, admin-scripts/
├── userspace/
│   └── apps/               # nexus-terminal, kvm-manager, mediastack-control, ...
├── packages/               # 22 consolidated sibling-repo packages
├── installer/              # nexus-install.sh (overlay + ZFS fresh)
├── docs/
│   ├── vision/             # Archived from-scratch-kernel vision (not current)
│   ├── CONSOLIDATION.md    # Monorepo source map
│   ├── setup/ notes/
│   └── security-audit.md media-center-plan.md mount-qcow.md ...
└── build/                  # ISO output
```

Legacy from-scratch source (`boot/`, `kernel/`, `include/`, `lib/`,
`drivers/`, `linker.ld`, `Makefile`, `CMakeLists.txt`, `config.mk`,
`create_phase1.sh`, `create_phase2.sh`, `grub.cfg`) is retained for the
vision direction but is **not** exercised by `scripts/build-iso.sh` or
the installer. Do not modify it unless you are intentionally working on
the vision track — and if you are, update `docs/vision/` in the same
change.

### AI services (runtime)

| Service      | Port | Purpose                                                    |
|--------------|------|------------------------------------------------------------|
| Orchestrator | 8600 | Central coordinator / API gateway                          |
| Stella       | 8601 | Security scanning, firewall, login monitoring             |
| Max Jr.      | 8602 | CPU/GPU/memory metrics, gaming detection                   |

### GPU configuration

PRIME render offload — Intel iGPU drives the desktop, NVIDIA activates
on demand via `prime-run`. Config lives in `core/config/`:

- `10-intel-primary.conf`
- `11-nvidia-prime-offload.conf`
- `nvidia-prime.conf`
- `modprobe-nvidia.conf` (`nvidia-drm modeset=1` required for PRIME)

## Build Dependencies

Installed automatically by `scripts/build-iso.sh` via `nala`:

```
debootstrap squashfs-tools xorriso mtools
grub-efi-amd64-bin grub-efi-amd64-signed shim-signed
grub-pc-bin rsync dosfstools isolinux syslinux-common
```

Install manually if needed:

```bash
sudo nala install debootstrap squashfs-tools xorriso mtools \
    grub-efi-amd64-bin grub-efi-amd64-signed shim-signed \
    grub-pc-bin rsync dosfstools isolinux syslinux-common
```

## File Organization

### Critical files (active distro)

- `scripts/build-iso.sh` — the ISO builder; every step (debootstrap →
  APT → base → KDE → NVIDIA → AI services → media stack → squashfs →
  xorriso) lives here.
- `scripts/patch-iso.sh` — delta patcher for an existing ISO.
- `installer/nexus-install.sh` — overlay install on existing Pop!_OS or
  fresh ZFS-on-root install.
- `core/bin/nexuspkg`, `core/bin/nexus-control`, `core/bin/stella`,
  `core/bin/maxjr` — user-facing CLIs.
- `core/services/*.service` + `*.timer` — systemd units for the AI
  services.

### Build artifacts (`build/`)

- `build/nexusos-1.0-YYYY.MM.DD-x86_64.iso` — hybrid UEFI/BIOS ISO
  (typically ~4.8 GB).

## Code Quality Standards

### ABSOLUTE REQUIREMENT: Zero Stub Code

**This is non-negotiable. Every piece of code committed to this repository must be 100% complete and functional.**

- **NO TODO comments** - No TODO, FIXME, unimplemented, or similar markers
- **NO incomplete functions** - Every function must have a complete, working implementation
- **NO zombie code** - No dead code paths, incomplete logic, or placeholder implementations
- **NO stubs** - Assembly stubs must be complete with proper error handling
- **NO partial features** - Features are either fully working or not included

### Verification Checklist Before Any Commit

1. **No incomplete code patterns**:
   - Search codebase for: `TODO`, `FIXME`, `XXX`, `HACK`, `stub`, `unimplemented`
   - All must return ZERO matches

2. **All functions fully implemented**:
   - Every function body is complete
   - All error paths handled
   - All parameters validated
   - All edge cases covered

3. **Code tested and verified**:
   - Code compiles without errors
   - Code runs and produces correct output
   - No infinite loops or hangs (unless intentional)
   - No memory leaks or buffer overflows

4. **Assembly stubs are production quality**:
   - All exception/interrupt handlers complete
   - Proper register saving/restoring
   - Correct calling conventions
   - No placeholder implementations

### Enforcement

- **Before merging**: All code is audited for stubs/TODOs
- **During development**: Incomplete work stays in branches, never merged to master
- **Commit messages**: Must clearly state what's implemented and verified

### Example of Unacceptable Code

```c
// ❌ REJECTED - Incomplete
int process_file(const char *path) {
    // TODO: implement file reading
    return -1;
}

// ❌ REJECTED - Stub
void handle_interrupt() {
    // stub
}
```

### Example of Acceptable Code

```c
// ✅ ACCEPTED - Complete
int process_file(const char *path) {
    if (!path) return -EINVAL;
    
    int fd = open(path, O_RDONLY);
    if (fd < 0) return fd;
    
    char buffer[4096];
    int bytes = read(fd, buffer, sizeof(buffer));
    if (bytes < 0) {
        close(fd);
        return bytes;
    }
    
    // Process buffer...
    close(fd);
    return bytes;
}

// ✅ ACCEPTED - Production ISR
.globl isr42
isr42:
    pushl $0              /* error code */
    pushl $42             /* exception number */
    jmp isr_common
    /* Complete handler in common code */
```
