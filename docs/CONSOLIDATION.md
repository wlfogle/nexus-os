# NexusOS Repository Consolidation

Consolidated on 2026-04-07. All NexusOS-related repos, scripts, docs, and config from the laptop are now in this monorepo.

## Monorepo Structure

```
nexus-os/
├── boot/, bootloader/, kernel/, lib/, include/   (OS core)
├── core/                                          (14 subsystems)
├── userland/, userspace/                          (user-facing apps)
├── drivers/                                       (display, input, network, storage)
├── installer/                                     (Calamares installer)
├── build/, tests/
│
├── packages/                  ← All consolidated repos (22 packages)
│   ├── ai-powerhouse-setup/        AI/ML dev environment setup
│   ├── ai-sysadmin-supreme/        Autonomous AI sysadmin
│   ├── awesome-stack/              Self-hosting infrastructure (65+ services)
│   ├── awesome-stack-optimization-suite/  Infrastructure optimization
│   ├── eartrumpet-linux/           Audio per-app volume control
│   ├── firestick-mediacontrol-app/ Android Fire TV remote
│   ├── garuda-hello/               Biometric authentication
│   ├── homelab-media-stack/        Media server + clients
│   ├── Hyperion/                   Linux power utilities
│   ├── i9-13900hx-optimizations/   Hardware-specific tuning
│   ├── kvm-manager/                KVM/QEMU VM manager (Rust/Tauri)
│   ├── linux-gaming-vm-toolkit/    GPU passthrough + VFIO
│   ├── media-stack-admin-scripts/  Production media scripts
│   ├── mediastack-control/         Docker container control panel
│   ├── mediastack-control-popos/   Pop!_OS media control variant
│   ├── mobalivecd-linux/           Portable Linux LiveCD
│   ├── nexus-terminal/             AI-powered terminal
│   ├── ollama-code-checker/        AI code analysis
│   ├── ollama-manager-gui/         Ollama model manager
│   ├── OmnioSearch/                AI file search
│   ├── PortProton-Enhanced/        Gaming/Proton launcher
│   └── universal-zfs-installer/    ZFS root installer
│
├── scripts/                   ← Consolidated from ~/
│   ├── hardware/                   gpu-vfio-check.sh
│   ├── optimization/               optimize-system.sh
│   ├── networking/                 wireguard-killswitch.sh + wireguard-tools/
│   ├── vm/                         spice-setup.sh + streaming/ (Sunshine)
│   └── gaming/                     11 scripts (vortex, qemu-seamless, clevo-kbd-rgb, game-*)
│
├── docs/                      ← Consolidated from ~/
│   ├── security-audit.md           Lynis audit results
│   ├── proxmox-wifi-setup.md       Proxmox Wi-Fi CLI guide
│   ├── media-center-plan.md        Media architecture plan
│   ├── mount-qcow.md               QEMU qcow2 mount guide
│   ├── notes/                      controller-question.txt
│   ├── setup/                      Pop!_OS config, starship, bluetooth, bashrc aliases
│   ├── AI_NATIVE_ARCHITECTURE.md
│   └── CONSOLIDATION.md           (this file)
```

## Source Mapping

### From ~/  (loose files)
- `~/gpu.sh` → `scripts/hardware/gpu-vfio-check.sh`
- `~/optimize-system.sh` → `scripts/optimization/optimize-system.sh`
- `~/wireguard-killswitch.sh` → `scripts/networking/wireguard-killswitch.sh`
- `~/wireguard-tools/` → `scripts/networking/wireguard-tools/`
- `~/do this.txt` → `scripts/vm/spice-setup.sh`
- `~/vm-streaming/` → `scripts/vm/streaming/`
- `~/bin/` → `scripts/gaming/`
- `~/security audit.md` → `docs/security-audit.md`
- `~/Connecting Proxmox to Wi-Fi via Command Line.md` → `docs/proxmox-wifi-setup.md`
- `~/Media center plan` → `docs/media-center-plan.md`
- `~/controller uestion` → `docs/notes/controller-question.txt`
- `~/Documents/Mount Qcow file.md` → `docs/mount-qcow.md`
- `~/Documents/Pop OS scripts/` → `docs/setup/`
- `~/.bashrc` homelab aliases → `docs/setup/bashrc-homelab-aliases.sh`

### From GitHub repos → `packages/`
All 22 repos copied in with build artifacts stripped (.git, node_modules, target, dist, legacy, .gradle).
Original repos remain on GitHub for independent history.

## Not Consolidated (intentionally excluded)

- `~/HomeDockOS/` — Docker container UI, unrelated to NexusOS
- `~/w3se/` — Wasteland 3 save editor (third-party, stolinator/w3se)
- `~/WinBoat/` — Windows boot tool (third-party, TibixDev/WinBoat)
- `~/quickshare/` — File sharing tool (third-party, ihexxa/quickshare)
- `~/ubuntu2204-setup/` — Third-party Ubuntu setup (edueo/ubuntu2204-setup)
- `~/qemu-build/`, `~/qemu-src/` — QEMU build artifacts (too large)
- `~/popos-setup/` — Pop!_OS setup (minimal, separate concern)
- `~/arr_passwords` — Sensitive credentials, never commit
- Personal files: `Burgers and Fries.txt`, gaming launchers, `remap.ps1`
