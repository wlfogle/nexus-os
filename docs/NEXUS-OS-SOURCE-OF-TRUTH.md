# NexusOS — Source of Truth

> The single **current** reference for NexusOS + the media stack. If anything
> elsewhere disagrees with this file, **this file wins**. Last updated 2026-08-28.
> Legacy/forerunner material is labeled as such so it is never mistaken for live.

## 0. Where the code lives (changed 2026-08-07)
**All repos now live in `/media/loufogle/Data/Repos/<name>`.** Home contains no
repositories. Clone new work into the vault, never into `~`.
- Reclaimed 37 GB from `/` (was 96% full, now 92%) by deleting every repo whose
  content was verifiably recoverable from its remote, then re-cloning `nexus-os`
  into the vault.
- Local-only state that could not be pushed (stashes, uncommitted work, repos
  with no remote) is preserved as git bundles and patches in
  `Repos/_bundles/<repo>/`, each with a `MANIFEST.txt` restore recipe.
- Deleted outright as re-clonable or unwanted: the third-party clones listed in
  §1, plus `w3se`, `quickshare`, `rootAVD`, `docling-ui`, and the superseded
  standalone `nexus-terminal` (canonical copy is `packages/nexus-terminal`).
- All 10 `nexus-os` worktrees were removed; their branches survive in the repo.

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
  Owns the 2 TB HDD at `/mnt/hdd/media`; the **file-share hub**. Gateway: `192.168.12.222` (OpenWrt).
- **CT-300** `192.168.12.30` — consolidated media-stack LXC (Debian 12). Gateway `192.168.12.222` (OpenWrt). wg0 split-tunnel (10.92.29.5, VPN subnet only; aria2 inbound).
- **Bahamut** `192.168.12.244` — Pi 4 (DietPi): AdGuard DNS, Caddy+DuckDNS,
  Vaultwarden. Gateway `192.168.12.222` (OpenWrt). **PiVPN kept as client-config management tool only** (no longer runs WireGuard server). Edge node — keep light.
- **OpenWrt VM-100** `192.168.12.222` — **Primary router/gateway for all homelab hosts** (router-on-a-stick on 192.168.12.0/24 via vmbr0). **WireGuard SERVER** `10.92.29.1`, port 51820, uses Bahamut's former server keys (clients unchanged). LuCI packages: ttyd, watchcat, pbr, statistics, nlbwmon, filemanager, vnstat2, crowdsec-firewall-bouncer. DNS upstream: Bahamut AdGuard + 8.8.8.8 fallback. **Full routing migration COMPLETE** (2026-07-28).
- **VM-990** `192.168.12.123` — Home Assistant OS. HA web UI `:8123` → `ha.tiamat.local`. Credentials: `haos/haos`. **Critical — stop only when reducing I/O pressure.**
- **Archer AX55 Pro** `192.168.12.254` — upstream NAT/WiFi. **DMZ → 192.168.12.222** (all inbound to OpenWrt). Built-in WireGuard VPN Server must remain **disabled** (conflicts with OpenWrt on port 51820). Still primary DHCP server for 192.168.12.0/24.
- **Laptop** `192.168.1.188` (wired, Spectrum direct) / `192.168.12.172` (WiFi, Archer) — Pop!_OS, i9-13900HX, RTX 4080, **control center**.
- **ISP:** Spectrum gigabit `74.134.128.100`. Archer AX55 Pro "Stella" **Router mode** `192.168.12.254` (WAN: `192.168.1.61` via Spectrum SAX1V1K at `192.168.1.1`).
- **WireGuard server:** OpenWrt `10.92.29.1`. **Clients:** laptop `10.92.29.2`, Tiamat `10.92.29.3`, CT-300 `10.92.29.5`. **ALL clients split-tunnel** (`AllowedIPs = 10.92.29.0/24` only, no DNS override — full-tunnel was killing internet/apt). All LAN clients endpoint: `192.168.12.222:51820`. External clients: `74.134.128.100:51820` → DMZ → OpenWrt.
- **Cockpit** (laptop localhost:9090): remote hosts configured — Tiamat `192.168.12.242` (user: cockpit), Bahamut `192.168.12.244` (user: root), CT-300 `192.168.12.30` (user: root).
- **Tailscale** tailnet `tail9d8b73.ts.net`; CT-300 node `100.115.82.71`.
- Full device inventory: `bulletproof-mediastack/docs/NETWORKING.md` → "Device Inventory".

## 3. Media stack (CT-300, consolidated)
All services on `192.168.12.30`: Riven frontend `:3000`, Riven backend `:8080`
(+ RivenVFS FUSE at `/mount`), Jellyfin `:8096`, Caddy `:80/443`, PostgreSQL
`:5432`, Redis `:6379`, n8n `:5678`, Threadfin `:34400`, Homarr `:7575`,
Uptime Kuma `:3001`, Cockpit `:9090`, CrowdSec, JDownloader2 (headless Xvfb),
aria2, MetaTube `:32217`, unified-guide `:7700`, Immich `:2283`
(native build — server/web via Node+pnpm, machine-learning via Python venv,
core plugin via extism-js built from source since Debian 12's glibc predates
the upstream prebuilt binary; own Postgres DB with VectorChord+pgvector,
Redis DB index 1), flexget `:5050` (pipx venv), autobrr `:7474` (Go binary).
Immich/flexget/autobrr were migrated 2026-08-02 from Docker containers on the
laptop's HomeDock stack to native CT-300 services.

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
- **`nexus-os/packages/nexus-brain/`** — the second brain, now two halves:
  - **Phase 1** (`nexus-brain.py`) — capture + FTS search for *new* thoughts.
    Pure stdlib, no pip deps.
  - **Phase 2** (`desktop/`) — **Librarian**, implemented 2026-08-07. Tauri 2
    desktop app that crawls the ecosystem, **interprets every file with a local
    Ollama model**, scores it against the repo set, labels current vs stale, and
    files it (auto above 0.85 confidence, otherwise queued for review). Never
    deletes: "remove" means move to `Quarantine/`; every move is journalled and
    reversible; repo-owned files are never touched. 50 unit tests.
    Routes across the 42 local models by content class with confidence-based
    escalation, draining the queue one model at a time to avoid VRAM thrashing.
  - **Phase 3** (`nexus-brain.py`) — implemented 2026-08-28. `POST
    /api/note/<id>/promote` marks the idea actionable, fires a configurable
    n8n webhook, and asks a local Ollama model to draft an importable n8n
    workflow JSON, stored on the note. Both n8n and Ollama stay optional
    bolt-ons: unset, promote just flips status with no network calls.
  - Built because Phase 1 was designed, parked, forgotten, and then re-invented
    from scratch. **Check `packages/` before proposing anything new.**

## 7. Maintenance
- Re-run consolidation any time: `python3 nexus-os/scripts/nexus-consolidate.py`
  (dry-run) → review `~/nexus-consolidate-report.md` → `--apply`.
- Keep **this file** current when topology changes; it is the canonical reference
  to hand to anyone (or any AI) for accurate context.
