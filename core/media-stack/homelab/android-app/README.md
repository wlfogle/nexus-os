# TiamatsStack Android App

WebView-based dashboard for the Tiamat homelab media stack.
Works on phones, tablets, and Fire TV devices (via LEANBACK_LAUNCHER + D-pad navigation).

## Services

Each service runs in its own Proxmox LXC with a dedicated IP on `192.168.12.x`.
Service URLs are defined in `app/src/main/java/com/tiamat/mediastack/MediaService.kt`.

**Media:** Jellyfin (CT-231), Plex (CT-230)
**Requests:** Jellyseerr (CT-242, DHCP)
**Arr Suite:** Sonarr (CT-214), Radarr (CT-215), Prowlarr (CT-210), Bazarr (CT-240, DHCP)
**Downloads:** qBittorrent (CT-212, VPN proxied), rdt-client (CT-213)
**AI:** Open WebUI (CT-900, DHCP) — Ollama via RTX 4080 on laptop
**Network/Infra:** AdGuard Home (Pi), Authentik (CT-107), Traefik (CT-103), Vaultwarden (CT-104), wg-easy (Pi)

## Prerequisites

```bash
sudo nala install openjdk-17-jdk android-tools-adb
export ANDROID_HOME=~/Android/Sdk
```

## Build

```bash
# From the android-app/ directory:

# Debug builds (both flavors)
./gradlew assembleDebug

# Single flavor
./gradlew assembleMobileDebug
./gradlew assembleFiretvDebug

# Release
./gradlew assembleRelease
```

APKs output to `app/build/outputs/apk/<flavor>/<buildType>/`.

## Product Flavors

- **mobile** (`.mobile`) — Phone/tablet, portrait grid, LAUNCHER intent
- **firetv** (`.firetv`) — Fire TV / Android TV, 4-column grid, LEANBACK_LAUNCHER intent, D-pad navigation

## Install

### Phone / Tablet
```bash
adb install app/build/outputs/apk/mobile/debug/app-mobile-debug.apk
```

### Fire TV (ADB over network)
1. On Fire TV: **Settings → My Fire TV → Developer Options**
   - Enable **ADB Debugging**
   - Enable **Apps from Unknown Sources**
2. Find Fire TV IP: **Settings → My Fire TV → About → Network**
3. Connect and install:
   ```bash
   adb connect <fire-tv-ip>:5555
   adb install app/build/outputs/apk/firetv/debug/app-firetv-debug.apk
   ```

The app appears in Fire TV's **Your Apps & Channels** row.

### Fire TV (Downloader app)
1. Install **Downloader** (by AFTVnews) from Fire TV app store
2. Host APK on LAN: `python3 -m http.server 8000` in APK directory
3. Open Downloader → `http://192.168.12.172:8000/app-firetv-debug.apk`

## Updating Service IPs

If a DHCP container gets a new IP, update `ServiceRepository` in `MediaService.kt` and rebuild.
Current DHCP services:
- Jellyseerr — `192.168.12.151`
- Bazarr — `192.168.12.188`
- Open WebUI — `192.168.12.223`

Static IPs (100–107, 210–215, 230–231) don't change.

## Project Structure

```
app/src/main/
├── java/com/tiamat/mediastack/
│   ├── MainActivity.kt        # Phone/tablet grid launcher
│   ├── TvMainActivity.kt      # Fire TV D-pad launcher
│   ├── WebViewActivity.kt     # Fullscreen WebView for services
│   ├── MediaService.kt        # Service model + ServiceRepository
│   └── ServiceAdapter.kt      # RecyclerView grid adapter
├── res/
│   ├── drawable/               # Service icons (vector XML)
│   ├── layout/                 # Activity + item layouts
│   ├── menu/                   # Toolbar overflow menu
│   ├── values/                 # Colors, strings, themes
│   └── xml/                    # Network security config
└── AndroidManifest.xml
```
