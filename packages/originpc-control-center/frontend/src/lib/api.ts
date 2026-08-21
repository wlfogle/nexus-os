// Thin, typed wrappers around every Tauri command/event in CONTRACT.md.
//
// When this bundle isn't running inside the actual Tauri webview - e.g.
// `vite dev` opened in a plain browser tab during frontend development,
// which is how this tab set was built and exercised before backend-agent's
// effect/power/fan commands and the "system-stats" event landed in a
// parallel branch - `isTauri()` is false and every function here returns
// canned mock data instead of letting `invoke` reject. Inside the real
// Tauri app `isTauri()` is always true, so this fallback never triggers and
// every call goes straight to the real backend command.

import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  ConnectionStatus,
  EffectName,
  FanModeName,
  PowerInfo,
  PowerProfileName,
  SensorSnapshot,
  SystemStatsEvent,
  SystemUsage,
} from "../types";
import {
  MOCK_CONNECTION_STATUS,
  mockPowerInfo,
  mockSensorSnapshot,
  mockSystemUsage,
} from "./mockData";

// --- Already implemented (Phase 1 scaffold) ---

export async function getConnectionStatus(): Promise<ConnectionStatus> {
  if (!isTauri()) return MOCK_CONNECTION_STATUS;
  return invoke<ConnectionStatus>("get_connection_status");
}

export async function setKeyColor(key: string, r: number, g: number, b: number): Promise<void> {
  if (!isTauri()) {
    console.info(`[mock] set_key_color(${key}, ${r}, ${g}, ${b})`);
    return;
  }
  await invoke("set_key_color", { key, r, g, b });
}

export async function setGroupColor(group: string, r: number, g: number, b: number): Promise<void> {
  if (!isTauri()) {
    console.info(`[mock] set_group_color(${group}, ${r}, ${g}, ${b})`);
    return;
  }
  await invoke("set_group_color", { group, r, g, b });
}

export async function clearAllKeys(): Promise<void> {
  if (!isTauri()) {
    console.info("[mock] clear_all_keys()");
    return;
  }
  await invoke("clear_all_keys");
}

export async function getSensorSnapshot(): Promise<SensorSnapshot> {
  if (!isTauri()) return mockSensorSnapshot();
  return invoke<SensorSnapshot>("get_sensor_snapshot");
}

export async function getPowerInfo(): Promise<PowerInfo> {
  if (!isTauri()) return mockPowerInfo();
  return invoke<PowerInfo>("get_power_info");
}

export async function getSystemUsage(): Promise<SystemUsage> {
  if (!isTauri()) return mockSystemUsage();
  return invoke<SystemUsage>("get_system_usage");
}

/** Text output of `tlp-stat -s`, for the "TLP Stats" detail view. */
export async function getTlpStats(): Promise<string> {
  if (!isTauri()) {
    return "[mock] tlp-stat -s\n\nTLP Status\n  State            = enabled\n  Last run         = 12s ago\n  Mode             = battery\n  Power source     = AC\n";
  }
  return invoke<string>("get_tlp_stats");
}

// --- backend-agent's additions (parallel branch; signatures frozen in
// CONTRACT.md, so it's safe to build against them ahead of the merge). ---

export async function startEffect(effect: EffectName, speed: number): Promise<void> {
  if (!isTauri()) {
    console.info(`[mock] start_effect(${effect}, ${speed})`);
    return;
  }
  await invoke("start_effect", { effect, speed });
}

export async function stopEffect(): Promise<void> {
  if (!isTauri()) {
    console.info("[mock] stop_effect()");
    return;
  }
  await invoke("stop_effect");
}

export async function setPowerProfile(profile: PowerProfileName): Promise<void> {
  if (!isTauri()) {
    console.info(`[mock] set_power_profile(${profile})`);
    return;
  }
  await invoke("set_power_profile", { profile });
}

export async function setFanMode(mode: FanModeName): Promise<void> {
  if (!isTauri()) {
    console.info(`[mock] set_fan_mode(${mode})`);
    return;
  }
  await invoke("set_fan_mode", { mode });
}

/**
 * Subscribes to the backend's periodic "system-stats" push event and
 * returns an unlisten function. Outside of Tauri (dev-mode in a plain
 * browser), this instead runs a mock ~2s interval so the System tab and
 * sidebar remain exercisable without a running backend.
 */
export async function listenSystemStats(
  callback: (payload: SystemStatsEvent) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) {
    const interval = setInterval(() => {
      callback({ sensors: mockSensorSnapshot(), power: mockPowerInfo(), usage: mockSystemUsage() });
    }, 2000);
    return () => clearInterval(interval);
  }
  return listen<SystemStatsEvent>("system-stats", (event) => callback(event.payload));
}
