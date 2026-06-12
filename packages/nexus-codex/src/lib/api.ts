import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  Config,
  DocResult,
  OllamaModel,
  Report,
  ScanCompleteEvent,
  ScanDocEvent,
  ScanErrorEvent,
  ScanProgressEvent,
  ScanStatus,
} from "./types";
import {
  EVENT_SCAN_COMPLETE,
  EVENT_SCAN_DOC,
  EVENT_SCAN_ERROR,
  EVENT_SCAN_PROGRESS,
} from "./types";

// ─── Config ───────────────────────────────────────────────────────────────────

export function getConfig(): Promise<Config> {
  return invoke<Config>("get_config");
}

export function saveConfig(config: Config): Promise<void> {
  return invoke<void>("save_config", { newConfig: config });
}

// ─── Ollama models ────────────────────────────────────────────────────────────

export function getOllamaModels(): Promise<OllamaModel[]> {
  return invoke<OllamaModel[]>("get_ollama_models");
}

// ─── Scan lifecycle ───────────────────────────────────────────────────────────

export function startScan(): Promise<string> {
  return invoke<string>("start_scan");
}

export function cancelScan(scanId: string): Promise<void> {
  return invoke<void>("cancel_scan", { scanId });
}

export function getScanStatus(scanId: string): Promise<ScanStatus> {
  return invoke<ScanStatus>("get_scan_status", { scanId });
}

// ─── Report ───────────────────────────────────────────────────────────────────

export function getReport(scanId: string): Promise<Report> {
  return invoke<Report>("get_report", { scanId });
}

export function exportReport(
  scanId: string,
  format: "markdown" | "json",
  outputPath: string
): Promise<string> {
  return invoke<string>("export_report", { scanId, format, outputPath });
}

// ─── Event listeners ─────────────────────────────────────────────────────────

export function onScanProgress(
  handler: (e: ScanProgressEvent) => void
): Promise<UnlistenFn> {
  return listen<ScanProgressEvent>(EVENT_SCAN_PROGRESS, (event) =>
    handler(event.payload)
  );
}

export function onScanDoc(
  handler: (e: ScanDocEvent) => void
): Promise<UnlistenFn> {
  return listen<ScanDocEvent>(EVENT_SCAN_DOC, (event) =>
    handler(event.payload)
  );
}

export function onScanComplete(
  handler: (e: ScanCompleteEvent) => void
): Promise<UnlistenFn> {
  return listen<ScanCompleteEvent>(EVENT_SCAN_COMPLETE, (event) =>
    handler(event.payload)
  );
}

export function onScanError(
  handler: (e: ScanErrorEvent) => void
): Promise<UnlistenFn> {
  return listen<ScanErrorEvent>(EVENT_SCAN_ERROR, (event) =>
    handler(event.payload)
  );
}
