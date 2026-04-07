# NexusOS Repository Consolidation

Consolidated on 2026-04-07. Loose scripts, docs, and project dirs from `~/` were organized into the nexus-os monorepo.

## Scripts (`scripts/`)

| Destination | Source | Description |
|---|---|---|
| `scripts/hardware/gpu-vfio-check.sh` | `~/gpu.sh` | VFIO/IOMMU GPU passthrough sanity check |
| `scripts/optimization/optimize-system.sh` | `~/optimize-system.sh` | System performance tuning (BBR, scheduler, NVIDIA) |
| `scripts/networking/wireguard-killswitch.sh` | `~/wireguard-killswitch.sh` | VPN killswitch script |
| `scripts/networking/wireguard-tools/` | `~/wireguard-tools/` | Full WireGuard toolkit: client manager, server rotation, tray widget, web dashboard, API masking proxy |
| `scripts/vm/spice-setup.sh` | `~/do this.txt` | Win11 VM SPICE display + clipboard setup |
| `scripts/vm/streaming/` | `~/vm-streaming/` | Sunshine VM streaming (PS1 installer + Windows exe) |
| `scripts/gaming/` | `~/bin/` | Gaming scripts: clevo-kbd-rgb, game-add-vm, game-add-native, game-list, game-run, qemu-seamless, vortex-moonlight, vortex-on/off/status/stream-status |

## Docs (`docs/`)

| Destination | Source | Description |
|---|---|---|
| `docs/security-audit.md` | `~/security audit.md` | Lynis 3.1.4 security audit results |
| `docs/proxmox-wifi-setup.md` | `~/Connecting Proxmox to Wi-Fi via Command Line.md` | Proxmox Wi-Fi CLI configuration guide |
| `docs/media-center-plan.md` | `~/Media center plan` | Media center architecture using desktop as server |
| `docs/notes/controller-question.txt` | `~/controller uestion` | Gaming controller + FireTV streaming question |

## Reference Repos (`reference/` — gitignored, local only)

Local copies for cross-referencing. Not committed to git.

### Already present
ai-powerhouse-setup, ai-sysadmin-supreme, awesome-stack, eartrumpet-linux, garuda-hello, Hyperion, i9-13900hx-optimizations, kvm-manager, linux-gaming-vm-toolkit, mobalivecd-linux, nexus-terminal, ollama-code-checker, ollama-manager-gui, OmnioSearch, PortProton-Enhanced, universal-zfs-installer

### Added this consolidation
- `reference/firestick-mediacontrol-app/` — Android Fire TV media remote app
- `reference/mediastack-control-popos/` — Pop!_OS variant of mediastack-control

## Related GitHub Repos
- https://github.com/wlfogle/nexus-terminal — Terminal/CLI component
- https://github.com/wlfogle/eartrumpet-linux — Audio management
- https://github.com/wlfogle/kvm-manager — KVM virtualization manager
- https://github.com/wlfogle/mobalivecd-linux — Bootable LiveCD support
- https://github.com/wlfogle/Hyperion — Linux power utilities
- https://github.com/wlfogle/OmnioSearch — AI-enhanced file search
- https://github.com/wlfogle/homelab-media-stack — Media stack
- https://github.com/wlfogle/mediastack-control — Media control panel
- https://github.com/wlfogle/media-stack-admin-scripts — Media admin scripts
- https://github.com/wlfogle/awesome-stack-optimization-suite — Optimization utilities

## Not Consolidated (intentionally excluded)

- `~/HomeDockOS/` — Docker container UI, unrelated to NexusOS
- `~/w3se/` — Wasteland 3 save editor (third-party)
- `~/WinBoat/` — Windows boot tool (third-party)
- `~/quickshare/` — File sharing tool (third-party)
- `~/ubuntu2204-setup/` — Third-party Ubuntu setup scripts
- `~/qemu-build/`, `~/qemu-src/` — QEMU build artifacts (too large)
- `~/popos-setup/` — Pop!_OS setup, separate concern
- `~/arr_passwords` — Sensitive credentials, never commit
- Personal files: `Burgers and Fries.txt`, gaming launchers (`diablo4_launch.sh`, `xcom2_run`, etc.), `remap.ps1`
