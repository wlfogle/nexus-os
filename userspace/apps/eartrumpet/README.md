# EarTrumpet (Linux) — Tray‑first per‑app audio mixer

A cross‑desktop, tray‑centric mixer for PipeWire with per‑application volume/mute, quick device switching, and live meters — inspired by EarTrumpet on Windows.

## Goals
- Per‑app controls (volume, mute, move stream between devices)
- Fast device switching and default device selection
- Compact tray popover UI with live meters and search
- Cross‑DE support (KDE/GNOME/others) on Wayland

## Non‑goals (initially)
- Full routing graph editor (use qpwgraph/helvum)
- DSP/effects processing (use EasyEffects)

## Architecture
- Backend: Rust + libpipewire via pipewire‑rs
- UI: GTK4/libadwaita (tray via StatusNotifierItem/AppIndicator)
- Persistence: Rules/preferences in a simple local store (TOML/SQLite)

## MVP Milestones
1. Enumerate devices and streams; show per‑app list with icons and live meters
2. Adjust per‑app volume/mute; set default output; move streams between devices
3. Tray icon with popover UI and quick device switch
4. Persist per‑app rules (preferred device/volume)

## Build
On Arch/Garuda, install GTK4 dev files, then:

- Ensure Rust toolchain installed
- From this directory:
  - `cargo run`

## Roadmap (post‑MVP)
- Mic mute indicator + quick toggle
- Per‑app rules UI and profiles
- Global hotkeys via portal/per‑DE integration
- Flatpak packaging
