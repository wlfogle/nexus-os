// Canned fixtures used by `api.ts` only when the bundle is running outside
// the actual Tauri webview (e.g. `vite dev` opened in a plain browser tab,
// or before backend-agent's commands land). Never used inside the real app,
// since `isTauri()` is true there and every call hits the real backend.

import type { ConnectionStatus, PowerInfo, SensorSnapshot } from "../types";

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
