# Tauri command/event contract

This is the frozen interface between the frontend (`frontend/`) and the
backend (`src-tauri/`). Changing a signature here requires updating both
sides, so treat it as an API boundary. All types are defined once in Rust
(`hw/src/{rgb,sensors,power}.rs`, serde-derived) and mirrored by hand in
`frontend/src/types.ts` - keep them in sync.

## Already implemented (Phase 1 scaffold, verified on real hardware)

- `get_connection_status() -> ConnectionStatus { connected: bool, device_path: string | null }`
- `set_key_color(key: string, r: u8, g: u8, b: u8) -> void`
- `set_group_color(group: string, r: u8, g: u8, b: u8) -> void`
  - Valid `group` values: see `hw::keymap::KEY_GROUPS` (`wasd`, `arrow_keys`, `function_keys`, `numbers`, `letters`, `keypad`, `modifiers`, `spacebar`, `navigation`, `all_keys`).
- `clear_all_keys() -> void`
- `get_sensor_snapshot() -> SensorSnapshot { cpu, gpu, nvme, fans_rpm: TemperatureReading[] }` where `TemperatureReading = { label: string, celsius: number }`
- `get_power_info() -> PowerInfo { battery_percent: number, ac_connected: bool, status: string, tlp_active: bool }`

All commands return `Result<T, String>` on the Rust side (a plain error
message string on failure) and are `async fn` with the actual device I/O
inside `tokio::task::spawn_blocking` - **do not** call blocking I/O
directly in a command body; wrap it, per the existing commands in
`src-tauri/src/lib.rs`.

## backend-agent: commands to add

- `start_effect(effect: "wave" | "breathing" | "rainbow", speed: number) -> void` - spawns a cancellable Tokio task that calls `RgbController::apply_frame` at an animation cadence (diff-based, per the optimization notes in the plan). Track the running task's cancellation handle in `AppState`.
- `stop_effect() -> void` - cancels the running effect task, if any.
- `set_power_profile(profile: "performance" | "balanced" | "power_save") -> void` - wraps `hw::power::PowerReader::set_profile`.
- `set_fan_mode(mode: "auto" | "silent") -> void` - fan control was not reverse-engineered in the Python app beyond shelling out to vendor tools; check `packages/originpc-control-center/src/hardware_optimizations.py` for what it actually did before assuming a protocol.
- Replace frontend polling with a push model: a background Tokio task emits a `system-stats` event (payload: `{ sensors: SensorSnapshot, power: PowerInfo }`) every ~2s via `app_handle.emit(...)`. Keep the existing `get_sensor_snapshot`/`get_power_info` commands too (useful for on-demand refresh), but the frontend should switch its polling loop to `listen("system-stats", ...)`.

## frontend-agent: UI to add

Build against the commands above (implemented and to-be-added). Four tabs
matching the Python app's feature set (`packages/originpc-control-center/src/enhanced-professional-control-center.py`
is the feature-reference, not a UI reference - don't aim for pixel parity):
- **RGB Control**: color picker, group buttons (`set_group_color`), clear button, per-key picker (`set_key_color`) if time allows.
- **System**: sensor/power display, subscribing to the `system-stats` event once backend-agent adds it (fall back to polling `get_sensor_snapshot`/`get_power_info` if that lands later).
- **Effects**: buttons for `start_effect`/`stop_effect`.
- **Key Bindings**: hosts the Flexikey profile UI (see flexikey-agent's contract below).
`frontend/src/App.tsx` currently has a minimal working shell (connection status + preset colors + raw JSON dump) establishing the `invoke` pattern - replace its content, keep the pattern.

## flexikey-agent: crate + commands + UI to add

New module `hw/src/flexikey.rs` (evdev grab + uinput virtual device + JSON
profiles), reusing the **same profile file format and location** as the
Python version for continuity: `~/.config/originpc-control-center/flexikey/`
(index file `profiles.json` with `{ active_profile, profiles: string[] }`,
one `<name>.json` per profile with `{ name, mappings: { [key: string]: Action } }`).
See `packages/originpc-control-center/src/flexikey.py` for the exact JSON
shapes (`Action` = `{type: "remap", target} | {type: "combo", keys} | {type: "text", text} | {type: "launch", command} | {type: "disabled"}`).

Commands to add in `src-tauri/src/lib.rs` (thin wrappers over the new module, same `spawn_blocking` pattern):
- `list_flexikey_profiles() -> { active_profile: string | null, profiles: string[] }`
- `get_flexikey_profile(name: string) -> { name: string, mappings: Record<string, Action> }`
- `save_flexikey_profile(profile: {...}) -> void`
- `delete_flexikey_profile(name: string) -> void`
- `set_active_flexikey_profile(name: string) -> void`
- `capture_next_key() -> string` - blocks (in `spawn_blocking`) until one key event is read from the keyboard device, returns its evdev key name (e.g. `"KEY_F13"`).
- `start_flexikey_engine() -> void` / `stop_flexikey_engine() -> void` - grabs/releases the keyboard device.

## osd-lidmonitor-agent: window + binary to add

- Background Tokio task (spawned at app startup in `src-tauri/src/lib.rs`) that opens the `clevo-hotkeys` kernel module's input device (look it up by name, `"Clevo WMI Hotkeys"`, via evdev device enumeration - don't hardcode `/dev/input/eventN`, it can shift) and on each key-down event emits:
  - Tauri event `hotkey-event` with payload `HotkeyEventPayload = { key: string, label: string, icon: string }` (see `frontend/src/types.ts`; label/icon mapping mirrors `packages/originpc-control-center/src/hotkey_osd.py`'s `OSD_MESSAGES` table) to the `osd` window specifically.
  - Show the `osd` window (`window.show()`), then hide it (`window.hide()`) after ~1.5s via a timer, matching the Python OSD's auto-hide behavior.
- Extend `lid-monitor/src/main.rs` from its current `--clear-once` tool into the actual persistent daemon: multi-method lid-state detection mirroring `packages/originpc-control-center/src/lid-monitor-daemon.py::_check_lid_state()` (ACPI `/proc/acpi/button/lid/*/state` first - confirmed present and reliable on this hardware - then the other fallback methods), calling `RgbController::clear_all_keys()` on detected closure.
- Update `packaging/originpc-lid-monitor.service` (copy the Python package's fixed version - already a correct `systemd --user` unit - and just repoint `ExecStart` at the new Rust binary path).
