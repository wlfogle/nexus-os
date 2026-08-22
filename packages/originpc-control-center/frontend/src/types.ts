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

// Mirrors `../../hw/src/usage.rs`'s `SystemUsage` - CPU/memory/disk/load
// average/uptime, matching the original app's circular CPU/Memory gauges
// and "System Information" text block.
export interface SystemUsage {
  cpu_percent: number;
  memory_percent: number;
  memory_used_gb: number;
  memory_total_gb: number;
  disk_percent: number;
  disk_used_gb: number;
  disk_total_gb: number;
  load_avg: [number, number, number];
  uptime_secs: number;
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

// --- backend-agent's additions (CONTRACT.md "commands to add") ---

export type EffectName = "wave" | "breathing" | "rainbow";

export type PowerProfileName = "performance" | "balanced" | "power_save";

export type FanModeName = "auto" | "silent";

// Payload of the periodic "system-stats" push event.
export interface SystemStatsEvent {
  sensors: SensorSnapshot;
  power: PowerInfo;
  usage: SystemUsage;
}
