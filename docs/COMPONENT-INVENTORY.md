# NexusOS Component Inventory
NexusOS is the target operating system. The `wlfogle` repositories are component source, platform research, reference history, or separate specialty apps.

## Source of truth
`bulletproof-mediastack` is the first major NexusOS component. It prototypes the media/network module with:
- CT-300 native Riven/RivenVFS/Jellyfin
- VM-100 OpenWrt routing/DHCP/DNS control plane
- Bahamut native AdGuard Home, Caddy + DuckDNS, and Vaultwarden
- Real-Debrid-first acquisition that removes torrent VPN requirements
- fish as the interactive Linux shell standard

## Core NexusOS component sources
These repos contain code that should be preserved and mapped into NexusOS:
- `bulletproof-mediastack` → `core/media-stack` and homelab provisioning
- `mobalivecd-linux` → portable environment and Windows-app-style compatibility layer
- `nexus-terminal` → terminal UX and AI command layer
- `proxmox-infrastructure-admin` → Proxmox/LXC/VM management UI
- `kvm-manager` → KVM/VM management UI
- `OmnioSearch` → AI-enhanced local search
- `Hyperion` → desktop power utilities
- `eartrumpet-linux` → Linux audio UX component
- `PortProton-Enhanced` → gaming/Wine/PortProton integration
- `ollama-manager-gui` → local model management UI
- `Ai-Coding-Assistant` → AI coding/userspace assistant
- `ai-sysadmin-supreme` → AI sysadmin/Stella ancestry
- `media-stack-admin-scripts` → admin/provisioning runbook source

## Platform and installer research
These repos feed installer, recovery, and platform-support modules:
- `universal-zfs-installer`
- `ultimate-zfs-installer`
- `zfs-installer-enhanced`
- `zfs-garuda-setup`
- `garuda-media-stack`
- `ultimate-garuda-powerhouse`
- `garuda-ultimate-restore-system`
- `ai-powerhouse-setup`
- `calamares-zfs-integration`
- `ultimate-rescue-usb`
- `blendos-dragonized`
- `blendos-dragonized-iso`
- `blendos-awesome-stack`
- `image-builder`
- `kinoite-awesome-stack`
- `i9-13900hx-optimizations`
- `linux-gaming-vm-toolkit`
- `awesome-stack-optimization-suite`

## Historical/reference media stack repos
These explain the path to the current design and should be clearly labeled as superseded unless specific code is promoted:
- `homelab-media-stack`
- `awesome-stack`
- `proxmox-awesome-media-stack`
- `proxmox-ultimate-media-server`
- `grandma-media-stack`
- `mediastack-control`

Historical content may mention WireGuard, wg-easy, Traefik, Authentik, Docker-first Bahamut, zurg, rclone, qBittorrent, or multi-CT *arr services. Those are not current NexusOS media/network architecture unless explicitly reintroduced.

## Separate specialty apps/tools
Keep these separate unless explicitly promoted:
- `disability-app-tauri`
- `calibre-library-fixer`
- `garuda-hello`
- `note9-n960u1-android12`
- `ADB-Toolbox`

## Hardware TODOs
- RGB keyboard lighting support is unresolved. Track it as a NexusOS hardware-support component, not as random cleanup debris.
- Windows gaming VM stays gaming-only. Do not add WSL/dev toolchains there unless explicitly required.

## Preservation rule
Do not delete working source code during cleanup. Delete only generated/vendor output, stale duplicate artifacts, or dead operational configs with no unique source logic. Otherwise label, move, or map it into this inventory.
