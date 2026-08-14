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
