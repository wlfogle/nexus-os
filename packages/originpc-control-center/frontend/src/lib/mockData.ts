// Canned fixtures used by `api.ts` only when the bundle is running outside
// the actual Tauri webview (e.g. `vite dev` opened in a plain browser tab,
// or before backend-agent's commands land). Never used inside the real app,
// since `isTauri()` is true there and every call hits the real backend.

import type { ConnectionStatus, PowerInfo, SensorSnapshot, SystemUsage } from "../types";

export const MOCK_CONNECTION_STATUS: ConnectionStatus = {
  connected: true,
  device_path: "/dev/hidraw0 (mock)",
};

function jitter(spread: number): number {
  return (Math.random() * 2 - 1) * spread;
}

export function mockSensorSnapshot(): SensorSnapshot {
  return {
    cpu: [
      { label: "coretemp Package id 0", celsius: 58 + jitter(3) },
      { label: "coretemp Core 0", celsius: 55 + jitter(3) },
    ],
    gpu: [{ label: "nvidia GPU Core", celsius: 52 + jitter(4) }],
    nvme: [{ label: "nvme Composite", celsius: 41 + jitter(2) }],
    fans_rpm: [
      { label: "clevo fan1", celsius: 3200 + jitter(150) },
      { label: "clevo fan2", celsius: 3050 + jitter(150) },
    ],
  };
}

export function mockPowerInfo(): PowerInfo {
  return {
    battery_percent: 87,
    ac_connected: true,
    status: "Charging",
    tlp_active: true,
  };
}

export function mockSystemUsage(): SystemUsage {
  return {
    cpu_percent: Math.max(0, Math.min(100, 12 + jitter(8))),
    memory_percent: 54 + jitter(2),
    memory_used_gb: 26.1,
    memory_total_gb: 62.5,
    disk_percent: 95.5,
    disk_used_gb: 821.8,
    disk_total_gb: 907.0,
    load_avg: [2.1 + jitter(0.5), 2.4 + jitter(0.3), 2.3 + jitter(0.2)],
    uptime_secs: 8 * 86400 + 18 * 3600 + 41 * 60,
  };
}
