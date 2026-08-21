# ADB Toolbox

Desktop GUI for managing Android devices via ADB, built with [Tauri v2](https://tauri.app/) + TypeScript.

## Features

- **App & Payload Control** — Push/pull files, install APKs (single or batch), purge app data
- **Google Play Store** — Search and download/stream APKs via gplaycli
- **Diagnostics** — Logcat viewer, screenshots, screen recording, text injection
- **Device Control** — Copy to mounted SD images, restart framework, reboot to bootloader/recovery

## Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (stable)
- ADB — `sudo nala install adb`
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
