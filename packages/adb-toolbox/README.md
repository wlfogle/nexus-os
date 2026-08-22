# ADB Toolbox

Desktop GUI for managing Android devices via ADB, built with [Tauri v2](https://tauri.app/) + TypeScript.

## Features

- **Native ADB protocol backend** — Talks directly to the local `adb-server` over its TCP protocol via the [`adb_client`](https://docs.rs/adb_client) Rust crate, instead of shelling out to the `adb` CLI for every operation. This avoids blocking process-spawn overhead and lets every device command run off the async runtime via `tokio::task::spawn_blocking`.
- **Multi-device support** — A `list_devices` command enumerates all attached/authorized devices (serial, connection state, model, transport id). Every device-targeted command accepts an optional device identifier so operations can be aimed at a specific device when more than one is connected.
- **App & Payload Control** — Push/pull files, install APKs (single or batch), purge app data
- **Google Play Store** — Search and download/stream APKs via gplaycli
- **Diagnostics** — Logcat viewer, screenshots, screen recording, text injection
- **Device Control** — Copy to mounted SD images, restart framework, reboot to bootloader/recovery

## Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (stable)
- ADB — `sudo nala install adb` (needed to run the local `adb-server` daemon that `adb_client` connects to; individual operations no longer shell out to the `adb` binary itself)
- [gplaycli](https://github.com/matlink/gplaycli) (optional) — `pip install gplaycli`

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```
