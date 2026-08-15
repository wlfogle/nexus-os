// Mirrors the serde-serialized shapes of the Rust types in
// `../../hw/src/{rgb,sensors,power}.rs` and the backend commands in
// `../../src-tauri/src/lib.rs`. Keep in sync with CONTRACT.md.

export interface ConnectionStatus {
  connected: boolean;
  device_path: string | null;
}

export interface TemperatureReading {
  label: string;
  celsius: number;
}

export interface SensorSnapshot {
  cpu: TemperatureReading[];
  gpu: TemperatureReading[];
  nvme: TemperatureReading[];
  fans_rpm: TemperatureReading[];
}

export interface PowerInfo {
  battery_percent: number;
  ac_connected: boolean;
  status: string;
  tlp_active: boolean;
}

// Emitted by the backend's evdev hotkey reader (owned by
// osd-lidmonitor-agent) on the "hotkey-event" Tauri event.
export interface HotkeyEventPayload {
  key: string;
  label: string;
  icon: string;
}

// Mirrors `../../hw/src/flexikey.rs`'s `ProfilesIndex`/`Profile`/`Action`
// (owned by flexikey-agent). Key names throughout (map keys, `target`,
// `keys`) are raw evdev key names such as "KEY_F13", exactly what
// `capture_next_key` returns.
export interface ProfilesIndex {
  active_profile: string | null;
  profiles: string[];
}

export type Action =
  | { type: "remap"; target: string }
  | { type: "combo"; keys: string[] }
  | { type: "text"; text: string }
  | { type: "launch"; command: string }
  | { type: "disabled" };

export interface Profile {
  name: string;
  mappings: Record<string, Action>;
}
