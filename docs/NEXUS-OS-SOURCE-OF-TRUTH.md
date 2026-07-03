# NexusOS — Source of Truth

> The single **current** reference for NexusOS + the media stack. If anything
> elsewhere disagrees with this file, **this file wins**. Last updated 2026-06-27.
> Legacy/forerunner material is labeled as such so it is never mistaken for live.

## 1. Component hierarchy (what is current)
- **nexus-os** — umbrella monorepo (THIS repo): AI-native Rust microkernel +
  `packages/` (apps/tools) + distro/installer. The home for your own code.
- **nexus-mediastack** — **CURRENT** media stack: every service consolidated into
  a single LXC, **CT-300**.
- **bulletproof-mediastack** — **FORERUNNER** (the original one-LXC-per-service
  topology). Reference + reusable code only; **not** the live topology.
- **Legacy/superseded** (moved off `main` to the `archive/legacy` branch on 2026-06-29;
  recover via `git checkout archive/legacy`): `homelab-media-stack`, `awesome-stack`,
  `awesome-stack-optimization-suite`, `mediastack-control`, `media-stack-admin-scripts`,
  `ai-powerhouse-setup`, `ai-sysadmin-supreme` (Stella's ancestry), `garuda-hello`,
  `linux-gaming-vm-toolkit`, the Garuda/Calamares ZFS installer, the old `legacy/` tree,
  and outdated docs (`DISTROWATCH_SUBMISSION.md`, `RELEASE_CHECKLIST.md`,
  `UNIVERSAL_PACKAGE_MANAGER_SPEC.md`, `Analysis.md`).
- **Third-party clones** (not ours; kept separate, never merged): `cockpit-file-sharing`,
  `rootAVD*` (3 copies), `waydroid_script`, `redroid-script`, `quickshare`, `w3se`,
  `HomeDockOS`, `hypervisor-launcher`, `win11-dev-proxmox-script`,
  `Self-Healing-Coding-Assistant`.

## 2. Hosts & network
- **Tiamat** `192.168.12.242` — Proxmox VE host (Ryzen 5 3600, 32 GB, RX 580).
  Owns the 2 TB HDD at `/mnt/hdd/media`; the **file-share hub**.
- **CT-300** `192.168.12.30` — consolidated media-stack LXC (Debian 12).
- **Bahamut** `192.168.12.244` — Pi 4 (DietPi): AdGuard DNS, Caddy+DuckDNS,
  Vaultwarden, PiVPN. Edge node (RAM-tight — keep it light).
- **Laptop** `192.168.12.204` (wired) / `.172` (WiFi) — Pop!_OS, RTX 4080,
  **control center** (Cockpit, Ollama).
- Gateway `.1` (T-Mobile KVD21, locked); Archer AX55 Pro `.234` (AP mode).
- **Tailscale** tailnet `tail9d8b73.ts.net`; CT-300 node `100.115.82.71`.
- Full device inventory (Fire TVs, phones, Echos, tablet, printer, HDHomeRun):
  `bulletproof-mediastack/docs/NETWORKING.md` → "Device Inventory".

## 3. Media stack (CT-300, consolidated)
All services on `192.168.12.30`: Riven frontend `:3000`, Riven backend `:8080`
(+ RivenVFS FUSE at `/mount`), Jellyfin `:8096`, Caddy `:80/443`, PostgreSQL
`:5432`, Redis `:6379`, n8n `:5678`, Threadfin `:34400`, Homarr `:7575`,
Uptime Kuma `:3001`, Cockpit `:9090`, CrowdSec, JDownloader2 (headless Xvfb),
aria2, MetaTube `:32217`, unified-guide `:7700`.

**Pipeline:** request in Riven → scrape Torrentio → add to Real-Debrid →
RivenVFS mounts the RD library at `/mount` → Jellyfin plays (seconds, zero local
storage). **Fallbacks:** `riven-jd2-bridge` (RD → JDownloader2 → `/data/media`),
`riven-aria2-bridge` (RD-refused → aria2). Local downloads land in
`/mnt/hdd/media` (host) = `/data/media` (bind mount in CT-300).

## 4. Credentials & secrets
- Convention: **`servicename/servicename`** (jellyfin/jellyfin, riven/riven,
  cockpit/cockpit, adguard/adguard, haos/haos). File share SMB: **`nexus/nexus`**.
- Secret locations: `/etc/bulletproof-mediastack-api-key`, `-auth-secret` (Tiamat);
  CT-300 `/etc/riven-jd2-bridge.env`, `/etc/metatube.env`, etc. Full table:
  `bulletproof-mediastack/docs/CREDENTIALS.md`.
- ⚠ **Rotate — secrets committed in-repo:** DuckDNS token (`bahamut/Caddyfile`),
  Sonarr/Radarr API keys (`docs/TROUBLESHOOTING.md`). A Bitwarden export was also
  found loose in `~` (flagged by the consolidation scan).

## 5. File sharing (deployed 2026-06-27)
- **Hub:** Tiamat `/mnt/hdd/media` — NFS **RW** to LAN (`192.168.12.0/24`) +
  Tailscale (`100.64.0.0/10`) + Samba `[media]` (user `nexus`).
- Laptop `~/Downloads` (NFS+Samba); Bahamut `/srv/share` (NFS-only, to protect
  the 2 GB Pi that also serves DNS).
- Managed via **Cockpit** (laptop = control center; 45Drives `cockpit-file-sharing`).
  Scripts: `nexus-mediastack/scripts/fileshare/{fileshare-server.sh,fileshare-client.sh}`.
- Non-Linux devices (TVs/phones/VMs): SMB to `\\192.168.12.242` (`nexus/nexus`) or
  the Cockpit web file browser. Full plan: nexus-mediastack plan `9602fff9`.

## 6. Repo / code map (nexus-os)
- `kernel/` — Rust microkernel (Phases 1–6 done; ring-3 shell, FAT32, ELF loader).
- `core/services/` — **Stella 🐕 (operations)** + **Max Jr. 🐱 (security)** AI companion
  services, coordinated by `nexus-orchestrator` (`stella.py`, `maxjr.py`,
  `nexus-orchestrator.py` + systemd units). NexusOS is dedicated to them.
- `core/`, `packages/` (nexus-terminal, nexus-codex, kvm-manager, ollama-manager-gui,
  ai-sysadmin-supreme, …), `legacy/`.
- **Consolidation tool:** `nexus-os/scripts/nexus-consolidate.py` — scans `~`,
  dedupes against this repo, recoverably trashes obsolete. Run dry-run, then
  `--apply`. On 2026-06-27 it staged 175 scattered files into `nexus-os/_consolidate/`
  and trashed 462 dups (recoverable in `~/.nexus-consolidate-trash/`).
- **New (parked):** `nexus-os/packages/nexus-brain/` — self-contained idea
  capture/search service (Phase 1). See its README.

## 7. Maintenance
- Re-run consolidation any time: `python3 nexus-os/scripts/nexus-consolidate.py`
  (dry-run) → review `~/nexus-consolidate-report.md` → `--apply`.
- Keep **this file** current when topology changes; it is the canonical reference
  to hand to anyone (or any AI) for accurate context.
