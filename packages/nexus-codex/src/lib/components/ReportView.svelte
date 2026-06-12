<script lang="ts">
  import { save } from "@tauri-apps/plugin-dialog";
  import { exportReport } from "$lib/api";
  import { reportStore } from "$lib/stores.svelte";
  import type { DocResult, DocSource, DocStatus, DocType } from "$lib/types";
  import { STATUS_COLORS, STATUS_LABELS } from "$lib/types";

  const ALL_STATUSES: DocStatus[] = [
    "current",
    "stale",
    "outdated",
    "orphaned",
    "needs_review",
  ];
  const ALL_TYPES: DocType[] = ["markdown", "text", "pdf", "rst", "adoc"];

  // Filter state — all enabled by default.
  let statusFilter = $state<Record<DocStatus, boolean>>({
    current: true,
    stale: true,
    outdated: true,
    orphaned: true,
    needs_review: true,
  });
  let typeFilter = $state<Record<DocType, boolean>>({
    markdown: true,
    text: true,
    pdf: true,
    rst: true,
    adoc: true,
  });
  let sourceFilter = $state<"all" | DocSource>("all");
  let repoQuery = $state("");

  let expandedPath = $state<string | null>(null);
  let exporting = $state(false);
  let exportMsg = $state<string | null>(null);
  let exportError = $state<string | null>(null);

  const results = $derived(reportStore.value?.results ?? []);

  const filtered = $derived(
    results.filter((d) => {
      if (!statusFilter[d.status]) return false;
      if (!typeFilter[d.doc_type]) return false;
      if (sourceFilter !== "all" && d.source !== sourceFilter) return false;
      if (repoQuery.trim()) {
        const q = repoQuery.trim().toLowerCase();
        const repo = (d.repo ?? "").toLowerCase();
        if (!repo.includes(q)) return false;
      }
      return true;
    })
  );

  function toggleStatus(s: DocStatus) {
    statusFilter = { ...statusFilter, [s]: !statusFilter[s] };
  }

  function toggleType(t: DocType) {
    typeFilter = { ...typeFilter, [t]: !typeFilter[t] };
  }

  function toggleRow(path: string) {
    expandedPath = expandedPath === path ? null : path;
  }

  function formatDate(iso: string): string {
    const d = new Date(iso);
    return Number.isNaN(d.getTime()) ? iso : d.toLocaleString();
  }

  function formatDuration(secs: number): string {
    if (secs < 60) return `${secs.toFixed(1)}s`;
    const m = Math.floor(secs / 60);
    const s = Math.round(secs % 60);
    return `${m}m ${s}s`;
  }

  function pct(v: number): string {
    return `${Math.round(v * 100)}%`;
  }

  async function doExport(format: "markdown" | "json") {
    const report = reportStore.value;
    if (!report || exporting) return;
    exporting = true;
    exportMsg = null;
    exportError = null;
    try {
      const ext = format === "markdown" ? "md" : "json";
      const path = await save({
        defaultPath: `nexus-codex-report.${ext}`,
        filters: [
          {
            name: format === "markdown" ? "Markdown" : "JSON",
            extensions: [ext],
          },
        ],
      });
      if (!path) {
        exporting = false;
        return;
      }
      const written = await exportReport(report.scan_id, format, path);
      exportMsg = `Exported to ${written}`;
      setTimeout(() => (exportMsg = null), 4000);
    } catch (e) {
      exportError = e instanceof Error ? e.message : String(e);
    } finally {
      exporting = false;
    }
  }
</script>

<section class="view">
  <header class="view-head">
    <h1>Report</h1>
    <p class="sub">Review, filter, and export the documentation analysis.</p>
  </header>

  {#if !reportStore.value}
    <div class="panel empty-state">No report yet — run a scan first.</div>
  {:else}
    {@const report = reportStore.value}
    <!-- Summary bar -->
    <div class="panel">
      <div class="summary-bar">
        <div class="summary-card">
          <span class="num">{report.summary.total}</span>
          <span class="lbl">Total</span>
        </div>
        {#each ALL_STATUSES as s (s)}
          <div class="summary-card">
            <span class="badge" style="background: {STATUS_COLORS[s]}">{STATUS_LABELS[s]}</span>
            <span class="num small">{report.summary[s]}</span>
          </div>
        {/each}
      </div>
    </div>

    <!-- Metadata -->
    <div class="panel meta">
      <div class="meta-item">
        <span class="meta-label">Generated</span>
        <span class="mono">{formatDate(report.generated_at)}</span>
      </div>
      <div class="meta-item">
        <span class="meta-label">Model</span>
        <span class="mono">{report.model_used}</span>
      </div>
      <div class="meta-item">
        <span class="meta-label">Duration</span>
        <span class="mono">{formatDuration(report.summary.scan_duration_secs)}</span>
      </div>
      <div class="meta-item">
        <span class="meta-label">Local repos</span>
        <span class="mono">{report.summary.local_repos_scanned}</span>
      </div>
      <div class="meta-item">
        <span class="meta-label">GitHub repos</span>
        <span class="mono">{report.summary.github_repos_scanned}</span>
      </div>
      <div class="meta-item">
        <span class="meta-label">PDFs</span>
        <span class="mono">{report.summary.pdfs_scanned}</span>
      </div>
    </div>

    <!-- Filter toolbar -->
    <div class="panel filters">
      <div class="filter-group">
        <span class="filter-label">Status</span>
        <div class="badge-row">
          {#each ALL_STATUSES as s (s)}
            <button
              class="badge toggle"
              class:off={!statusFilter[s]}
              style="background: {statusFilter[s] ? STATUS_COLORS[s] : 'transparent'}; border-color: {STATUS_COLORS[s]}; color: {statusFilter[s] ? '#fff' : STATUS_COLORS[s]}"
              onclick={() => toggleStatus(s)}
            >
              {STATUS_LABELS[s]}
            </button>
          {/each}
        </div>
      </div>

      <div class="filter-group">
        <span class="filter-label">Source</span>
        <div class="seg">
          <button class="seg-btn" class:active={sourceFilter === "all"} onclick={() => (sourceFilter = "all")}>All</button>
          <button class="seg-btn" class:active={sourceFilter === "local"} onclick={() => (sourceFilter = "local")}>Local</button>
          <button class="seg-btn" class:active={sourceFilter === "github"} onclick={() => (sourceFilter = "github")}>GitHub</button>
        </div>
      </div>

      <div class="filter-group">
        <span class="filter-label">Repo</span>
        <input class="text-input" placeholder="Search repo name…" bind:value={repoQuery} />
      </div>

      <div class="filter-group">
        <span class="filter-label">Type</span>
        <div class="check-row">
          {#each ALL_TYPES as t (t)}
            <label class="check">
              <input type="checkbox" checked={typeFilter[t]} onchange={() => toggleType(t)} />
              <span>{t}</span>
            </label>
          {/each}
        </div>
      </div>
    </div>

    <!-- Export -->
    <div class="panel export-bar">
      <button class="btn accent" onclick={() => doExport("markdown")} disabled={exporting}>Export Markdown</button>
      <button class="btn accent" onclick={() => doExport("json")} disabled={exporting}>Export JSON</button>
      {#if exportMsg}<span class="flash success">{exportMsg}</span>{/if}
      {#if exportError}<span class="flash error">{exportError}</span>{/if}
    </div>

    <!-- Results table -->
    <div class="panel">
      <div class="results-head">
        <h2>Results</h2>
        <span class="count mono">{filtered.length} of {results.length}</span>
      </div>
      {#if filtered.length === 0}
        <p class="empty">No documents match the current filters.</p>
      {:else}
        <div class="table-wrap">
          <table class="results">
            <thead>
              <tr>
                <th>Status</th>
                <th>Path</th>
                <th>Repo</th>
                <th>Type</th>
                <th>Confidence</th>
                <th>Staleness</th>
                <th>Reason</th>
              </tr>
            </thead>
            <tbody>
              {#each filtered as doc (doc.path)}
                <tr class="row" class:expanded={expandedPath === doc.path} onclick={() => toggleRow(doc.path)}>
                  <td>
                    <span class="badge" style="background: {STATUS_COLORS[doc.status]}">{STATUS_LABELS[doc.status]}</span>
                  </td>
                  <td class="mono path-cell" title={doc.path}>{doc.path}</td>
                  <td class="mono">{doc.repo ?? "—"}</td>
                  <td>{doc.doc_type}</td>
                  <td class="mono">{pct(doc.confidence)}</td>
                  <td class="mono">{pct(doc.staleness_score)}</td>
                  <td class="reason-cell">{doc.reason}</td>
                </tr>
                {#if expandedPath === doc.path}
                  <tr class="detail-row">
                    <td colspan="7">
                      {@render details(doc)}
                    </td>
                  </tr>
                {/if}
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  {/if}
</section>

{#snippet details(doc: DocResult)}
  <div class="detail">
    <div class="detail-grid">
      <div><span class="detail-label">Source</span> {doc.source}</div>
      <div><span class="detail-label">File size</span> {(doc.file_size_bytes / 1024).toFixed(1)} KB</div>
      <div><span class="detail-label">Last modified</span> {doc.last_modified ?? "—"}</div>
      <div><span class="detail-label">Last commit</span> {doc.last_commit_date ?? "—"}</div>
      <div>
        <span class="detail-label">Related code age</span>
        {doc.related_code_age_days != null ? `${doc.related_code_age_days} days` : "—"}
      </div>
      {#if doc.repo_url}
        <div><span class="detail-label">Repo URL</span> <span class="mono">{doc.repo_url}</span></div>
      {/if}
    </div>

    <div class="detail-block">
      <span class="detail-label">Evidence</span>
      <pre class="evidence">{doc.evidence}</pre>
    </div>

    {#if doc.suggested_rewrite}
      <div class="detail-block">
        <span class="detail-label">Suggested rewrite</span>
        <pre class="rewrite">{doc.suggested_rewrite}</pre>
      </div>
    {/if}
  </div>
{/snippet}

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
    margin: 0;
    font-size: 1.05rem;
  }

  .empty-state {
    text-align: center;
    color: #b8c0d8;
    font-size: 1rem;
    padding: 2.5rem;
  }

  .summary-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: center;
  }

  .summary-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.3rem;
    background: #0f3460;
    border-radius: 8px;
    padding: 0.6rem 1rem;
    min-width: 90px;
  }

  .summary-card .num {
    font-size: 1.5rem;
    font-weight: 700;
  }

  .summary-card .num.small {
    font-size: 1.15rem;
  }

  .summary-card .lbl {
    font-size: 0.75rem;
    color: #b8c0d8;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 1.5rem;
  }

  .meta-item {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .meta-label {
    font-size: 0.75rem;
    color: #8b94b3;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .filters {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .filter-group {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .filter-label {
    font-size: 0.8rem;
    font-weight: 700;
    color: #b8c0d8;
    min-width: 60px;
  }

  .badge-row,
  .check-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .badge {
    color: #ffffff;
    font-size: 0.72rem;
    font-weight: 700;
    padding: 0.18rem 0.55rem;
    border-radius: 10px;
    white-space: nowrap;
    text-align: center;
  }

  .badge.toggle {
    cursor: pointer;
    border: 1px solid transparent;
  }

  .badge.toggle.off {
    opacity: 0.85;
  }

  .seg {
    display: flex;
    border: 1px solid #1f4e8c;
    border-radius: 6px;
    overflow: hidden;
  }

  .seg-btn {
    background: #0f3460;
    border: none;
    color: #b8c0d8;
    padding: 0.35rem 0.8rem;
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .seg-btn.active {
    background: #e94560;
    color: #ffffff;
  }

  .text-input {
    background: #0f3460;
    border: 1px solid #1f4e8c;
    color: #ffffff;
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    font-size: 0.85rem;
    min-width: 220px;
  }

  .text-input:focus {
    outline: none;
    border-color: #e94560;
  }

  .check {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.82rem;
    cursor: pointer;
  }

  .export-bar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
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

  .flash {
    font-weight: 600;
    font-size: 0.85rem;
  }

  .flash.success {
    color: #22c55e;
  }

  .flash.error {
    color: #e94560;
  }

  .results-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .count {
    color: #8b94b3;
  }

  .table-wrap {
    overflow-x: auto;
  }

  .results {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }

  .results th {
    text-align: left;
    padding: 0.5rem 0.65rem;
    color: #8b94b3;
    border-bottom: 1px solid #0f3460;
    font-weight: 700;
    white-space: nowrap;
  }

  .results td {
    padding: 0.5rem 0.65rem;
    border-bottom: 1px solid #0f3460;
    vertical-align: top;
  }

  .row {
    cursor: pointer;
  }

  .row:hover {
    background: #0f3460;
  }

  .row.expanded {
    background: #1a2c52;
  }

  .mono {
    font-family: "JetBrains Mono", "Fira Code", ui-monospace, monospace;
  }

  .path-cell {
    max-width: 280px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .reason-cell {
    max-width: 280px;
    color: #d8def0;
  }

  .detail-row td {
    background: #12203c;
  }

  .detail {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    padding: 0.5rem 0.25rem;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 0.5rem 1.5rem;
    font-size: 0.85rem;
  }

  .detail-label {
    color: #8b94b3;
    font-weight: 700;
    margin-right: 0.35rem;
  }

  .detail-block {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  pre.evidence,
  pre.rewrite {
    margin: 0;
    background: #0b1428;
    border: 1px solid #0f3460;
    border-radius: 6px;
    padding: 0.75rem;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: "JetBrains Mono", "Fira Code", ui-monospace, monospace;
    font-size: 0.8rem;
    max-height: 320px;
    overflow-y: auto;
  }

  pre.rewrite {
    border-color: #22c55e;
  }

  .empty {
    color: #6b7280;
    font-style: italic;
    margin: 0;
  }
</style>
