// Svelte 5 runes-based stores.
//
// This file uses the `.svelte.ts` extension so the Svelte compiler processes
// the `$state` runes within it. Each store is a small object exposing a
// reactive `value` getter/setter; reading `store.value` inside any component
// template or `$derived`/`$effect` will track it reactively.

import type { Config, OllamaModel, Report, ScanStatus } from "./types";

export type ActiveView = "config" | "scan" | "report";

interface RuneStore<T> {
  get value(): T;
  set value(v: T);
}

function createStore<T>(initial: T): RuneStore<T> {
  let state = $state(initial);
  return {
    get value(): T {
      return state;
    },
    set value(v: T) {
      state = v;
    },
  };
}

/** Current loaded configuration (null until `getConfig()` resolves). */
export const configStore = createStore<Config | null>(null);

/** Which top-level view is active in the nav bar. */
export const activeView = createStore<ActiveView>("config");

/** The scan ID for the most recently started scan, or null. */
export const scanIdStore = createStore<string | null>(null);

/** The completed report, or null when no scan has finished. */
export const reportStore = createStore<Report | null>(null);

/** Latest scan status snapshot, or null. */
export const scanStatusStore = createStore<ScanStatus | null>(null);

/** Available Ollama models discovered on the host. */
export const modelsStore = createStore<OllamaModel[]>([]);
