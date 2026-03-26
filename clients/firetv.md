# Fire TV Setup Guide

## Media Playback

### Jellyfin
1. Open the Fire TV app store and search for **Jellyfin**
2. Install and open it
3. Add server: `http://192.168.12.231:8096`
4. Sign in with your Jellyfin account

### TiamatsStack App (custom — sideloaded)
WebView-based dashboard linking all services. See `android-app/README.md` to build and sideload.

## Requesting & Managing Content

Fire TV runs Android — use **Silk Browser** (pre-installed) to access all arr stack web UIs.

### Option 1 — Jellyseerr (easiest, request only)
Simple Netflix-style interface to request movies and TV shows.
1. Open **Silk Browser**
2. Go to `http://192.168.12.151:5055`
3. Sign in with your Jellyfin account
4. Search for a movie or show → click **Request**
5. Radarr/Sonarr picks it up automatically and downloads it

> Bookmark this — it's the main way to add content from the couch.

### Option 2 — nzb360 (full arr control, sideloaded)
nzb360 is an Android app — it can be sideloaded onto Fire TV for native Sonarr/Radarr/qBittorrent control.

**Enable sideloading:**
1. Settings → My Fire TV → Developer Options → Apps from Unknown Sources → ON

**Install Downloader app:**
1. Search Fire TV app store for **Downloader** (by AFTVnews) → install

**Sideload nzb360:**
1. Open Downloader → enter URL: `https://nzb360.com/apk` (or find latest APK URL from nzb360.com)
2. Install the APK
3. Open nzb360 → Add services:
   - Sonarr: `http://192.168.12.214:8989` + API key
   - Radarr: `http://192.168.12.215:7878` + API key
   - qBittorrent: `http://192.168.12.212:8080`
   - Prowlarr: `http://192.168.12.210:9696` + API key

> API keys are in each service under Settings → General → Security.

## Paid Streaming Services
All paid services work natively — unaffected by the homelab setup:
- Amazon Prime Video — pre-installed
- Netflix, Disney+, etc. — install from Fire TV app store

## Ad Blocking
Handled at the router/DNS level via AdGuard Home — no setup needed on Fire TV.

## Remote Access (WireGuard)
To access your media away from home:
1. Install **WireGuard** from the Fire TV app store
2. Open wg-easy at `http://192.168.12.20:51821` → create a client → download config
3. Import config into WireGuard app
4. Connect to VPN — all services accessible as if on LAN
