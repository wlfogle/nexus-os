<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getOllamaModels, saveConfig } from "$lib/api";
  import { configStore, modelsStore } from "$lib/stores.svelte";
  import type { Config } from "$lib/types";

  // Local editable draft, seeded from the loaded config.
  let draft = $state<Config | null>(
    configStore.value
      ? structuredClone($state.snapshot(configStore.value))
      : null
  );

  let newRoot = $state("");
  let newExcluded = $state("");
  let saving = $state(false);
  let saved = $state(false);
  let errorMsg = $state<string | null>(null);
  let modelsLoading = $state(false);
  let modelsError = $state<string | null>(null);

  // Keep the draft in sync if the global config arrives after mount.
  $effect(() => {
    if (!draft && configStore.value) {
      draft = structuredClone($state.snapshot(configStore.value));
    }
  });

  onMount(async () => {
    await loadModels();
  });

  async function loadModels() {
    modelsLoading = true;
    modelsError = null;
    try {
      modelsStore.value = await getOllamaModels();
    } catch (e) {
      modelsError = e instanceof Error ? e.message : String(e);
    } finally {
      modelsLoading = false;
    }
  }

  async function pickRoot() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string" && draft) {
      if (!draft.scan_roots.includes(selected)) {
        draft.scan_roots = [...draft.scan_roots, selected];
      }
    }
  }

  function addRootManual() {
    const value = newRoot.trim();
    if (value && draft && !draft.scan_roots.includes(value)) {
      draft.scan_roots = [...draft.scan_roots, value];
      newRoot = "";
    }
  }

  function removeRoot(index: number) {
    if (draft) {
      draft.scan_roots = draft.scan_roots.filter((_, i) => i !== index);
    }
  }

  function addExcluded() {
    const value = newExcluded.trim();
    if (value && draft && !draft.excluded_paths.includes(value)) {
      draft.excluded_paths = [...draft.excluded_paths, value];
      newExcluded = "";
    }
  }

  function removeExcluded(index: number) {
    if (draft) {
      draft.excluded_paths = draft.excluded_paths.filter((_, i) => i !== index);
    }
  }

  async function pickReportDir() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string" && draft) {
      draft.report_output_dir = selected;
    }
  }

  function selectModel(name: string) {
    if (draft) {
      // Empty override means "use recommended / default".
      draft.ollama_model_override =
        draft.ollama_model_override === name ? null : name;
    }
  }

  async function handleSave() {
    if (!draft) return;
    saving = true;
    saved = false;
    errorMsg = null;
    try {
      const snapshot = structuredClone($state.snapshot(draft));
      await saveConfig(snapshot);
      configStore.value = snapshot;
      saved = true;
      setTimeout(() => (saved = false), 2500);
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }
</script>

<section class="view">
  <header class="view-head">
    <h1>Configuration</h1>
    <p class="sub">Configure what Nexus Codex scans and how it analyses docs.</p>
  </header>

  {#if !draft}
    <div class="panel">Configuration not loaded yet.</div>
  {:else}
    <!-- Scan roots -->
    <div class="panel">
      <h2>Scan Roots</h2>
      <p class="hint">Local directories to scan for documentation.</p>
      {#if draft.scan_roots.length === 0}
        <p class="empty">No scan roots added.</p>
      {:else}
        <ul class="list">
          {#each draft.scan_roots as root, i (root)}
            <li class="list-row">
              <span class="mono">{root}</span>
              <button class="btn-icon" onclick={() => removeRoot(i)} title="Remove">✕</button>
            </li>
          {/each}
        </ul>
      {/if}
      <div class="row-controls">
        <input
          class="text-input mono"
          placeholder="/path/to/directory"
          bind:value={newRoot}
          onkeydown={(e) => e.key === "Enter" && addRootManual()}
        />
        <button class="btn" onclick={addRootManual}>Add Path</button>
        <button class="btn accent" onclick={pickRoot}>Browse…</button>
      </div>
    </div>

    <!-- Excluded paths -->
    <div class="panel">
      <h2>Excluded Paths</h2>
      <p class="hint">Path patterns to skip (e.g. <span class="mono">node_modules</span>, <span class="mono">.git</span>).</p>
      {#if draft.excluded_paths.length === 0}
        <p class="empty">No exclusions added.</p>
      {:else}
        <ul class="list">
          {#each draft.excluded_paths as pattern, i (pattern)}
            <li class="list-row">
              <span class="mono">{pattern}</span>
              <button class="btn-icon" onclick={() => removeExcluded(i)} title="Remove">✕</button>
            </li>
          {/each}
        </ul>
      {/if}
      <div class="row-controls">
        <input
          class="text-input mono"
          placeholder="pattern or path fragment"
          bind:value={newExcluded}
          onkeydown={(e) => e.key === "Enter" && addExcluded()}
        />
        <button class="btn" onclick={addExcluded}>Add Pattern</button>
      </div>
    </div>

    <!-- GitHub -->
    <div class="panel">
      <h2>GitHub</h2>
      <label class="toggle">
        <input type="checkbox" bind:checked={draft.github_enabled} />
        <span>Scan remote GitHub repositories</span>
      </label>
      <div class="field">
        <span class="field-label">GitHub username</span>
        <input
          class="text-input"
          placeholder="octocat"
          bind:value={draft.github_username}
          disabled={!draft.github_enabled}
        />
      </div>
    </div>

    <!-- Ollama -->
    <div class="panel">
      <h2>Ollama</h2>
      <div class="field">
        <span class="field-label">Ollama URL</span>
        <input
          class="text-input mono"
          placeholder="http://localhost:11434"
          bind:value={draft.ollama_url}
        />
      </div>

      <div class="field">
        <div class="models-head">
          <span class="field-label">Model</span>
          <button class="btn small" onclick={loadModels} disabled={modelsLoading}>
            {modelsLoading ? "Refreshing…" : "Refresh"}
          </button>
        </div>
        {#if modelsError}
          <p class="error-text">Could not load models: {modelsError}</p>
        {/if}
        {#if modelsStore.value.length === 0}
          <p class="empty">No models discovered. Default model will be used.</p>
        {:else}
          <div class="models">
            {#each modelsStore.value as model (model.name)}
              <button
                class="model"
                class:selected={draft.ollama_model_override === model.name}
                onclick={() => selectModel(model.name)}
              >
                <span class="model-name mono">
                  {model.name}
                  {#if model.recommended}
                    <span class="star" title="Recommended">★</span>
                  {/if}
                </span>
                <span class="model-meta">
                  {model.family} · {model.size_gb.toFixed(1)} GB · score {model.score.toFixed(0)}
                </span>
              </button>
            {/each}
          </div>
          <p class="hint">
            {#if draft.ollama_model_override}
              Override: <span class="mono">{draft.ollama_model_override}</span> (click again to clear)
            {:else}
              Using recommended / default model.
            {/if}
          </p>
        {/if}
      </div>
    </div>

    <!-- Report output -->
    <div class="panel">
      <h2>Report Output</h2>
      <div class="field">
        <span class="field-label">Output directory</span>
        <div class="row-controls">
          <input
            class="text-input mono"
            placeholder="/path/to/reports"
            bind:value={draft.report_output_dir}
          />
          <button class="btn accent" onclick={pickReportDir}>Browse…</button>
        </div>
      </div>
    </div>

    <!-- Max file size -->
    <div class="panel">
      <h2>Max File Size</h2>
      <p class="hint">Documents larger than this are skipped.</p>
      <div class="slider-row">
        <input
          type="range"
          min="64"
          max="10240"
          step="64"
          bind:value={draft.max_file_size_kb}
          class="slider"
        />
        <span class="slider-value mono">{draft.max_file_size_kb} KB</span>
      </div>
    </div>

    <!-- Save -->
    <div class="save-bar">
      <button class="btn accent large" onclick={handleSave} disabled={saving}>
        {saving ? "Saving…" : "Save Configuration"}
      </button>
      {#if saved}
        <span class="flash success">✓ Saved</span>
      {/if}
      {#if errorMsg}
        <span class="flash error">{errorMsg}</span>
      {/if}
    </div>
  {/if}
</section>

<style>
  .view {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .view-head h1 {
    margin: 0;
    font-size: 1.5rem;
  }

  .sub {
    margin: 0.25rem 0 0;
    color: #b8c0d8;
  }

  .panel {
    background: #16213e;
    border: 1px solid #0f3460;
    border-radius: 10px;
    padding: 1.25rem;
  }

  .panel h2 {
    margin: 0 0 0.5rem;
    font-size: 1.05rem;
  }

  .hint {
    margin: 0 0 0.75rem;
    color: #8b94b3;
    font-size: 0.85rem;
  }

  .empty {
    color: #6b7280;
    font-style: italic;
    margin: 0 0 0.75rem;
  }

  .list {
    list-style: none;
    margin: 0 0 0.75rem;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .list-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    background: #0f3460;
    padding: 0.45rem 0.65rem;
    border-radius: 6px;
  }

  .mono {
    font-family: "JetBrains Mono", "Fira Code", ui-monospace, monospace;
    font-size: 0.85rem;
    word-break: break-all;
  }

  .row-controls {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .text-input {
    flex: 1;
    background: #0f3460;
    border: 1px solid #1f4e8c;
    color: #ffffff;
    padding: 0.5rem 0.65rem;
    border-radius: 6px;
    font-size: 0.9rem;
  }

  .text-input:focus {
    outline: none;
    border-color: #e94560;
  }

  .text-input:disabled {
    opacity: 0.5;
  }

  .btn {
    background: #0f3460;
    border: 1px solid #1f4e8c;
    color: #ffffff;
    padding: 0.5rem 0.9rem;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 600;
    font-size: 0.85rem;
    white-space: nowrap;
  }

  .btn:hover:not(:disabled) {
    border-color: #e94560;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .btn.accent {
    background: #e94560;
    border-color: #e94560;
  }

  .btn.accent:hover:not(:disabled) {
    background: #ff5c77;
  }

  .btn.small {
    padding: 0.3rem 0.6rem;
    font-size: 0.75rem;
  }

  .btn.large {
    padding: 0.7rem 1.4rem;
    font-size: 0.95rem;
  }

  .btn-icon {
    background: transparent;
    border: none;
    color: #e94560;
    cursor: pointer;
    font-size: 0.9rem;
    padding: 0.1rem 0.4rem;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    margin-bottom: 0.75rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-top: 0.75rem;
  }

  .field-label {
    font-size: 0.85rem;
    color: #b8c0d8;
    font-weight: 600;
  }

  .models-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .models {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .model {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    text-align: left;
    background: #0f3460;
    border: 1px solid #1f4e8c;
    border-radius: 8px;
    padding: 0.6rem 0.75rem;
    cursor: pointer;
    color: #ffffff;
  }

  .model:hover {
    border-color: #e94560;
  }

  .model.selected {
    border-color: #e94560;
    background: #1a2c52;
    box-shadow: 0 0 0 1px #e94560 inset;
  }

  .model-name {
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .star {
    color: #f59e0b;
  }

  .model-meta {
    font-size: 0.75rem;
    color: #8b94b3;
  }

  .slider-row {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .slider {
    flex: 1;
    accent-color: #e94560;
  }

  .slider-value {
    min-width: 90px;
    text-align: right;
  }

  .save-bar {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .flash {
    font-weight: 600;
  }

  .flash.success {
    color: #22c55e;
  }

  .flash.error,
  .error-text {
    color: #e94560;
  }

  .error-text {
    font-size: 0.8rem;
    margin: 0 0 0.5rem;
  }
</style>
