<script lang="ts">
  import { onDestroy } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import {
    cancelScan,
    onScanComplete,
    onScanDoc,
    onScanError,
    onScanProgress,
    startScan,
  } from "$lib/api";
  import {
    activeView,
    reportStore,
    scanIdStore,
    scanStatusStore,
  } from "$lib/stores.svelte";
  import type { DocResult, ReportSummary } from "$lib/types";
  import { STATUS_COLORS, STATUS_LABELS } from "$lib/types";

  let running = $state(false);
  let starting = $state(false);
  let cancelling = $state(false);
  let completed = $state(false);
  let errorMsg = $state<string | null>(null);

  let total = $state(0);
  let processed = $state(0);
  let percentage = $state(0);
  let currentFile = $state("");
  let stage = $state("");

  let feed = $state<DocResult[]>([]);
  let summary = $state<ReportSummary | null>(null);

  let unlisteners: UnlistenFn[] = [];

  function truncatePath(path: string, max = 64): string {
    if (path.length <= max) return path;
    return "…" + path.slice(path.length - (max - 1));
  }

  async function teardown() {
    const fns = unlisteners;
    unlisteners = [];
    for (const fn of fns) {
      try {
        fn();
      } catch {
        // listener already removed
      }
    }
  }

  function resetState() {
    total = 0;
    processed = 0;
    percentage = 0;
    currentFile = "";
    stage = "";
    feed = [];
    summary = null;
    errorMsg = null;
    completed = false;
  }

  async function handleStart() {
    if (running || starting) return;
    starting = true;
    resetState();
    await teardown();

    try {
      const id = await startScan();
      scanIdStore.value = id;
      running = true;

      const progressUn = await onScanProgress((e) => {
        if (e.scan_id !== scanIdStore.value) return;
        total = e.total;
        processed = e.processed;
        percentage = e.percentage;
        currentFile = e.current_file;
        stage = e.stage;
      });

      const docUn = await onScanDoc((e) => {
        if (e.scan_id !== scanIdStore.value) return;
        feed = [e.doc, ...feed];
      });

      const completeUn = await onScanComplete(async (e) => {
        if (e.scan_id !== scanIdStore.value) return;
        reportStore.value = e.report;
        summary = e.report.summary;
        percentage = 100;
        processed = e.report.summary.total;
        total = e.report.summary.total;
        running = false;
        completed = true;
        currentFile = "";
        stage = "Complete";
        scanStatusStore.value = {
          scan_id: e.scan_id,
          state: "complete",
          total: e.report.summary.total,
          processed: e.report.summary.total,
          current_file: null,
          stage: "Complete",
          error: null,
          results_so_far: e.report.results,
        };
        await teardown();
      });

      const errorUn = await onScanError(async (e) => {
        if (e.scan_id !== scanIdStore.value) return;
        errorMsg = e.error;
        running = false;
        stage = "Error";
        await teardown();
      });

      unlisteners = [progressUn, docUn, completeUn, errorUn];
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
      running = false;
    } finally {
      starting = false;
    }
  }

  async function handleCancel() {
    if (!scanIdStore.value || cancelling) return;
    cancelling = true;
    try {
      await cancelScan(scanIdStore.value);
      running = false;
      stage = "Cancelled";
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
    } finally {
      cancelling = false;
      await teardown();
    }
  }

  function viewReport() {
    activeView.value = "report";
  }

  onDestroy(() => {
    void teardown();
  });
</script>

<section class="view">
  <header class="view-head">
    <h1>Scan</h1>
    <p class="sub">Run a non-destructive documentation scan and watch it live.</p>
  </header>

  <div class="panel controls">
    <button class="btn accent large" onclick={handleStart} disabled={running || starting}>
      {starting ? "Starting…" : running ? "Scanning…" : "Start Scan"}
    </button>
    {#if running}
      <button class="btn large" onclick={handleCancel} disabled={cancelling}>
        {cancelling ? "Cancelling…" : "Cancel"}
      </button>
    {/if}
    {#if completed}
      <button class="btn accent large" onclick={viewReport}>View Report</button>
    {/if}
  </div>

  {#if errorMsg}
    <div class="panel error-panel">
      <strong>Scan error:</strong>
      {errorMsg}
    </div>
  {/if}

  {#if running || completed || processed > 0}
    <div class="panel">
      <div class="progress-head">
        <span class="stage">{stage || "Idle"}</span>
        <span class="counter mono">{processed} / {total} docs</span>
      </div>
      <div class="progress-track">
        <div class="progress-fill" style="width: {Math.min(100, Math.max(0, percentage))}%"></div>
      </div>
      <div class="progress-meta">
        <span class="pct mono">{percentage.toFixed(0)}%</span>
        {#if currentFile}
          <span class="current mono" title={currentFile}>{truncatePath(currentFile)}</span>
        {/if}
      </div>
    </div>
  {/if}

  {#if completed && summary}
    <div class="panel">
      <h2>Summary</h2>
      <div class="summary-grid">
        <div class="summary-card">
          <span class="num">{summary.total}</span>
          <span class="lbl">Total</span>
        </div>
        <div class="summary-card">
          <span class="num" style="color: {STATUS_COLORS.current}">{summary.current}</span>
          <span class="lbl">Current</span>
        </div>
        <div class="summary-card">
          <span class="num" style="color: {STATUS_COLORS.stale}">{summary.stale}</span>
          <span class="lbl">Stale</span>
        </div>
        <div class="summary-card">
          <span class="num" style="color: {STATUS_COLORS.outdated}">{summary.outdated}</span>
          <span class="lbl">Outdated</span>
        </div>
        <div class="summary-card">
          <span class="num" style="color: {STATUS_COLORS.orphaned}">{summary.orphaned}</span>
          <span class="lbl">Orphaned</span>
        </div>
        <div class="summary-card">
          <span class="num" style="color: {STATUS_COLORS.needs_review}">{summary.needs_review}</span>
          <span class="lbl">Needs Review</span>
        </div>
      </div>
    </div>
  {/if}

  <div class="panel feed-panel">
    <h2>Live Feed</h2>
    {#if feed.length === 0}
      <p class="empty">No documents processed yet.</p>
    {:else}
      <ul class="feed">
        {#each feed as doc (doc.path)}
          <li class="feed-row">
            <span
              class="badge"
              style="background: {STATUS_COLORS[doc.status]}"
            >{STATUS_LABELS[doc.status]}</span>
            <span class="feed-path mono" title={doc.path}>{truncatePath(doc.path, 80)}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
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
    margin: 0 0 0.75rem;
    font-size: 1.05rem;
  }

  .controls {
    display: flex;
    gap: 0.75rem;
    align-items: center;
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

  .btn.large {
    padding: 0.7rem 1.4rem;
    font-size: 0.95rem;
  }

  .error-panel {
    border-color: #e94560;
    color: #ffb3c0;
    background: #2a1622;
  }

  .progress-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .stage {
    font-weight: 600;
  }

  .counter {
    color: #b8c0d8;
  }

  .progress-track {
    width: 100%;
    height: 12px;
    background: #0f3460;
    border-radius: 6px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #e94560, #ff7a8c);
    transition: width 0.2s ease;
  }

  .progress-meta {
    display: flex;
    gap: 1rem;
    align-items: center;
    margin-top: 0.5rem;
  }

  .pct {
    font-weight: 700;
    color: #e94560;
  }

  .current {
    color: #8b94b3;
    font-size: 0.8rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mono {
    font-family: "JetBrains Mono", "Fira Code", ui-monospace, monospace;
    font-size: 0.85rem;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(110px, 1fr));
    gap: 0.6rem;
  }

  .summary-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.2rem;
    background: #0f3460;
    border-radius: 8px;
    padding: 0.8rem 0.5rem;
  }

  .summary-card .num {
    font-size: 1.5rem;
    font-weight: 700;
  }

  .summary-card .lbl {
    font-size: 0.75rem;
    color: #b8c0d8;
  }

  .feed-panel {
    max-height: 420px;
    display: flex;
    flex-direction: column;
  }

  .feed {
    list-style: none;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .feed-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    background: #0f3460;
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
  }

  .badge {
    color: #ffffff;
    font-size: 0.7rem;
    font-weight: 700;
    padding: 0.15rem 0.5rem;
    border-radius: 10px;
    white-space: nowrap;
    min-width: 84px;
    text-align: center;
  }

  .feed-path {
    color: #d8def0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty {
    color: #6b7280;
    font-style: italic;
    margin: 0;
  }
</style>
