# OriginPC Control Center (Rust/Tauri)

Linux control center for the OriginPC EON17-X (Clevo-based, ITE 8910 RGB
keyboard controller, USB `048d:8910`): RGB lighting, fan/power/temperature
monitoring, lighting effects, Flexikey key remapping/macros, and Fn-hotkey
capture with an on-screen display.

This is a Rust/Tauri rewrite of the previous PyQt5 implementation, migrated
to fix a real UI-freeze bug (RGB writes ran synchronously on the Qt GUI
thread) and to match the stack already used elsewhere in this repo
(`packages/nexus-brain/desktop`, `packages/nexus-terminal`). The prior
Python version is preserved on the `archive/legacy` branch
(`git checkout archive/legacy -- packages/originpc-control-center`) per
this repo's convention for superseded code.

## Structure

- `hw/` - shared, GUI-independent Rust crate (`clevo-hw`): RGB protocol,
  keymap/key-groups, hidraw discovery with a persistent device handle,
  sensor/power readers with TTL caching, and the Flexikey evdev/uinput
  engine. Used by both `src-tauri/` and the standalone `lid-monitor/`
  binary, so the protocol is implemented exactly once.
- `src-tauri/` - Tauri 2 backend. Every hardware-touching command is
  `async fn` with the actual device I/O inside `tokio::task::spawn_blocking`
  - this is what makes the UI-freeze bug structurally impossible here,
  not just less likely.
- `frontend/` - React + TypeScript + Vite UI: RGB Control, System, Effects,
  and Key Bindings (Flexikey) tabs, plus a second transparent `osd` window
  for Fn-hotkey popups.
- `lid-monitor/` - standalone binary (no GUI dependency) that clears the
  keyboard on lid close; runs as its own `systemd --user` service.
- `kernel/clevo-hotkeys/` - small GPL DKMS kernel module that captures
  Fn-hotkey events via this hardware's Clevo ACPI-WMI interface. Verified
  bound and working on the reference machine.
- `packaging/` - the udev rule (`99-originpc-rgb.rules`) and the
  lid-monitor systemd unit.
- `CONTRACT.md` - the frozen Tauri command/event contract between the
  frontend and backend; read this before changing any command signature.

## Build / run

```bash
# One-time: install the udev rule (RGB device access) and the
# clevo-hotkeys kernel module via DKMS - see kernel/clevo-hotkeys/ and
# packaging/99-originpc-rgb.rules for the exact rule/module.

cd frontend && npm install && npm run build && cd ..
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

# Dev loop (hot reload):
cargo tauri dev

# Production build (produces .deb/AppImage per src-tauri/tauri.conf.json):
cargo tauri build
```

A dedicated `install.sh` (udev rule + DKMS module + desktop entry +
systemd unit installation, mirroring the archived Python package's
installer) has not been written yet for this package - tracked as a
follow-up, not a blocker for the app itself running from a built binary.

## Protocol reference

RGB: raw 16-byte writes to `/dev/hidraw0` (with hidraw1-3 fallback
discovery): `[0xCC, 0x01, key_index, r, g, b, 0x00 * 10]`. See
`hw/src/rgb.rs` and `hw/src/keymap.rs` for the full key index table -
already validated against real hardware, do not change without
re-testing on a device.

Fn hotkeys: the `clevo-hotkeys` kernel module binds to this laptop's
Clevo ACPI-WMI GUID (`ABBC0F6B-8EA1-11D1-...`) and reports events through
a standard Linux input device named "Clevo WMI Hotkeys" - see
`src-tauri/src/hotkey_osd.rs`.

## Known follow-ups

- No `install.sh` yet (see above).
- GPU/CPU overclocking is deliberately not implemented - no safe, vetted
  Linux interface exists for this hardware (see the archived Python
  package's README on `archive/legacy` for the research behind that
  decision, which still applies).
- Several effect/fan/power code paths are compile-verified and
  unit-tested but need real-hardware confirmation (visual effect quality,
  `nbfc`/`tlp` presence on the target machine, a narrow lazy-abort race in
  `stop_effect` where an in-flight write can land just after a stop is
  issued) - see the migration PR description for the full list.
