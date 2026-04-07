# Android (Tablet / Phone) Setup Guide

## Apps to Install

| App | Purpose | Source |
|-----|---------|--------|
| Jellyfin | Media playback | Play Store |
| TiamatsStack | All-in-one dashboard (custom APK) | Sideload — see android-app/README.md |
| nzb360 | Unified arr control (Sonarr, Radarr, qBit) | Play Store ($5) |
| Moonlight | Game streaming from laptop | Play Store (free) |
| Bitwarden | Password manager (Vaultwarden) | Play Store |
| WireGuard | Remote VPN access | Play Store |

## Media Playback
1. Install **Jellyfin** from Play Store
2. Add server: `http://192.168.12.231:8096`
3. Sign in and enjoy your media library

## Requesting & Managing Content

### nzb360 — Best option (native app, full control)
nzb360 is a unified controller for the entire arr stack — add shows, movies, manage downloads, search indexers, all from one native Android app.

1. Install **nzb360** from Play Store (~$5 one-time)
2. Open → tap the menu → add each service:
   - **Sonarr**: `http://192.168.12.214:8989` + API key
   - **Radarr**: `http://192.168.12.215:7878` + API key
   - **qBittorrent**: `http://192.168.12.212:8080`
   - **Prowlarr**: `http://192.168.12.210:9696` + API key

> Get API keys from each service: Settings → General → Security → API Key

### Jellyseerr — Simpler request-only option (free, browser)
1. Open Chrome → `http://192.168.12.151:5055`
2. Sign in with Jellyfin account
3. Search → click **Request** — Sonarr/Radarr handles the rest

## Admin Dashboards (browser bookmarks)
| Service | URL |
|---------|-----|
| Jellyfin | http://192.168.12.231:8096 |
| Plex | http://192.168.12.230:32400/web |
| Sonarr | http://192.168.12.214:8989 |
| Radarr | http://192.168.12.215:7878 |
| Prowlarr | http://192.168.12.210:9696 |
| qBittorrent | http://192.168.12.212:8080 |
| Bazarr | http://192.168.12.188:6767 |
| Jellyseerr | http://192.168.12.151:5055 |
| Open WebUI | http://192.168.12.223:3000 |
| Authentik | http://192.168.12.107:9000 |
| AdGuard Home | http://192.168.12.20:3000 |
| wg-easy | http://192.168.12.20:51821 |
| Vaultwarden | https://192.168.12.20 |

## Bitwarden / Vaultwarden
1. Install **Bitwarden** from Play Store
2. Settings → Self-hosted → Server URL: `https://192.168.12.20`
3. Create account and start saving passwords

## Game Streaming (Moonlight)
Stream games from the laptop's RTX 4080 to your phone or tablet.

1. Install **Moonlight Game Streaming** from Play Store (free)
2. Open → tap **+** → enter laptop IP:
   - Home (LAN): `192.168.12.172`
   - Away (Tailscale): `100.66.87.38`
3. Enter the PIN shown → go to `https://localhost:47990` on the laptop → **PIN** → type it
4. Available apps: Desktop, Switch (Ryubing), Steam Big Picture

### Controller
- **Bluetooth**: pair Xbox / PS4 / PS5 / 8BitDo via Android Settings → Bluetooth
- **USB-C OTG**: USB-C to USB-A adapter → wired controller
- **Touch**: Moonlight Settings → **On-Screen Controls** → enable overlay

> Full guide: `docs/SUNSHINE-MOONLIGHT.md`

## Remote Access (WireGuard)
1. Install **WireGuard** from Play Store
2. Open wg-easy: `http://192.168.12.20:51821` → create client → scan QR code
3. Toggle VPN on when away from home — all services accessible as if on LAN
