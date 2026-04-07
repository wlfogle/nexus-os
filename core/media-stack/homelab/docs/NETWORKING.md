# Networking

## LAN Layout
```
Router (192.168.12.1)
├── Tiamat (Proxmox VE)       → 192.168.12.242 (static)
├── Ziggy (Raspberry Pi 3B+)  → 192.168.12.20 (static)
├── Laptop                    → DHCP
├── Fire TV                   → DHCP
└── Tablet / phones           → DHCP
```

## Proxmox Bridge
Proxmox host networking is bridged through `vmbr0` so all CTs/VMs get LAN access.

`/etc/network/interfaces`:
```
auto lo
iface lo inet loopback

auto enp3s0
iface enp3s0 inet manual

auto vmbr0
iface vmbr0 inet static
    address 192.168.12.242/24
    gateway 192.168.12.1
    bridge-ports enp3s0
    bridge-stp off
    bridge-fd 0
    dns-nameservers 192.168.12.1
```

## Core Service Map

### Proxmox host
| Service | URL |
|---|---|
| Proxmox UI | `https://192.168.12.242:8006` |

### Infrastructure CTs
| CT | Purpose | URL / Port |
|---|---|---|
| CT-100 `wireguard` | WireGuard server | `:51820/udp` |
| CT-101 `wg-proxy` | WireGuard client + TinyProxy | `http://192.168.12.101:8888` |
| CT-102 `flaresolverr` | Cloudflare bypass | `http://192.168.12.102:8191` |
| CT-103 `traefik` | Reverse proxy | `:80`, `:443`, `:8080` |
| CT-104 `vaultwarden` | Password manager backend | `http://192.168.12.104` |
| CT-105 `valkey` | Redis-compatible cache | `:6379` |
| CT-106 `postgresql` | Database | `:5432` |
| CT-107 `authentik` | SSO | `http://192.168.12.107:9000` |

### Download stack (static)
| Service | URL |
|---|---|
| Prowlarr (CT-210) | `http://192.168.12.210:9696` |
| Jackett (CT-211) | `http://192.168.12.211:9117` — fallback indexer (native, hdd-ct storage) |
| qBittorrent (CT-212) | `http://192.168.12.212:8080` |
| Deluge-fallback (CT-213) | `http://192.168.12.213:8112` — fallback download client (native, hdd-ct storage) |
| Sonarr (CT-214) | `http://192.168.12.214:8989` |
| Radarr (CT-215) | `http://192.168.12.225:7878` |
| Readarr (CT-217) | `http://192.168.12.217:8787` |
| Whisparr (CT-219) | `http://192.168.12.219:6969` |
| Sonarr Extended (CT-220) | `http://192.168.12.220:8989` |
| Autobrr (CT-223) | `http://192.168.12.223:7474` |
| Deluge (CT-224) | `http://192.168.12.224:8112` |

### Media servers
| Service | URL |
|---|---|
| Plex (CT-230) | `http://192.168.12.230:32400/web` |
| Jellyfin (CT-231) | `http://192.168.12.231:8096` |

### VMs
| VM | Purpose | IP | Notes |
|---|---|---|---|
| VM-901 `windows-gaming` | Windows 11 gaming + PlayOn | `192.168.12.201` | RX 580 GPU passthrough, sdb disk passthrough |
| VM-500 `haos` | Home Assistant OS | `192.168.12.250` | Smart home hub |
| VM-200 `alexa-bridge` | Alexa media bridge | `192.168.12.200` | Ubuntu |

### Laptop NFS Exports (192.168.12.172)
| Export path | Tiamat mount | Consumer |
|---|---|---|
| `/media/loufogle/Data/Calibre Library` | `/mnt/laptop/calibre` | CT-233 Calibre-Web |
| `/media/loufogle/Data/Cookbooks` | `/mnt/laptop/cookbooks` | CT-233 second library |
| `/media/loufogle/SystemBackup/Videos` | `/mnt/laptop/videos` | Plex / Jellyfin Home Videos |
| `/media/loufogle/ISOs1` | `/mnt/laptop/isos` | Proxmox ISO uploads |
| `/media/loufogle/Games/roms` | `/mnt/laptop/roms` | Future emulation frontend |
| `/media/loufogle/Data/Downloads/lou.m3u` | `/mnt/laptop/iptv/lou.m3u` | Jellyfin Live TV / TVHeadend CT-235 |

See `docs/NFS.md` for setup.

## VPN Architecture (kill-switch path)

CT-101 is not Gluetun software. It runs:
- `wireguard-tools` client to CT-100
- `tinyproxy` on port `8888`

Traffic flow:
```
qBittorrent/Prowlarr -> 192.168.12.101:8888 (TinyProxy)
                     -> WireGuard tunnel (CT-101 -> CT-100)
                     -> NAT on CT-100
                     -> Internet
```

If WG tunnel drops, proxy path breaks and downloads fail closed.

## Traefik Reverse Proxy Routes (CT-103)
All services are accessible via `*.tiamat.local` through Traefik at `192.168.12.103`.
Dynamic route files live in `infrastructure/traefik/dynamic/`.

| Hostname | Service | Backend |
|---|---|---|
| `traefik.tiamat.local` | Traefik dashboard | `192.168.12.103:8080` |
| `jellyfin.tiamat.local` | Jellyfin (CT-231) | `192.168.12.231:8096` |
| `plex.tiamat.local` | Plex (CT-230) | `192.168.12.230:32400` |
| `sonarr.tiamat.local` | Sonarr (CT-214) | `192.168.12.214:8989` |
| `radarr.tiamat.local` | Radarr (CT-215) | `192.168.12.225:7878` |
| `prowlarr.tiamat.local` | Prowlarr (CT-210) | `192.168.12.210:9696` |
| `qbittorrent.tiamat.local` | qBittorrent (CT-212) | `192.168.12.212:8080` |
| `jackett.tiamat.local` | Jackett (CT-211) | `192.168.12.211:9117` |
| `deluge.tiamat.local` | Deluge fallback (CT-213) | `192.168.12.213:8112` |
| `bazarr.tiamat.local` | Bazarr (CT-240) | `192.168.12.188:6767` \* |
| `jellyseerr.tiamat.local` | Seerr/Jellyseerr (CT-242) | `192.168.12.151:5055` \* |
| `vault.tiamat.local` | Vaultwarden (CT-104) | `192.168.12.104:80` |
| `auth.tiamat.local` | Authentik (CT-107) | `192.168.12.107:9000` |

\* CT-242 has static IP 192.168.12.151. CT-240 is DHCP — set a static reservation on the router.
CT-242 runs Seerr natively (no Docker). Proxmox vmbr0 has static ARP for Radarr: `ip neigh replace 192.168.12.225 dev vmbr0 lladdr BC:24:11:2A:83:BB nud permanent`
then update `infrastructure/traefik/dynamic/media-management.yml` if IPs change.

To add a new route, drop a YAML file in `/etc/traefik/dynamic/` on CT-103.
Traefik hot-reloads it immediately (no restart needed).

## DNS / Remote Access on Ziggy

Ziggy (`192.168.12.20`) runs:
- AdGuard Home (primary DNS): `:53`, setup UI `:3000`
- wg-easy (remote VPN mgmt): `http://192.168.12.20:51821`
- WireGuard tunnel endpoint: `:51820/udp`
- Vaultwarden behind Caddy: `https://192.168.12.20`

Router DNS recommendation:
1. `192.168.12.20`
2. `1.1.1.1`
