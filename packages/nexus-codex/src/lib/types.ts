// ─── Configuration ────────────────────────────────────────────────────────────

export interface Config {
  scan_roots: string[];
  excluded_paths: string[];
  github_enabled: boolean;
  github_username: string;
  ollama_url: string;
  ollama_model_override: string | null;
  report_output_dir: string;
  max_file_size_kb: number;
}

// ─── Document Classification ──────────────────────────────────────────────────

export type DocStatus = "current" | "stale" | "outdated" | "orphaned" | "needs_review";
export type DocSource = "local" | "github";
export type DocType = "markdown" | "text" | "pdf" | "rst" | "adoc";

export interface DocResult {
  path: string;
  repo: string | null;
  repo_url: string | null;
  source: DocSource;
  doc_type: DocType;
  status: DocStatus;
  /** 0.0–1.0 */
  confidence: number;
  /** 0.0–1.0 */
  staleness_score: number;
  reason: string;
  evidence: string;
  suggested_rewrite: string | null;
  last_modified: string | null;
  last_commit_date: string | null;
  related_code_age_days: number | null;
  file_size_bytes: number;
}

// ─── Ollama Models ────────────────────────────────────────────────────────────

export interface OllamaModel {
  name: string;
  size_gb: number;
  score: number;
  recommended: boolean;
  family: string;
}

// ─── Report ───────────────────────────────────────────────────────────────────

export interface ReportSummary {
  total: number;
  current: number;
  stale: number;
  outdated: number;
  orphaned: number;
  needs_review: number;
  local_repos_scanned: number;
  github_repos_scanned: number;
  pdfs_scanned: number;
  scan_duration_secs: number;
}

export interface Report {
  scan_id: string;
  generated_at: string;
  model_used: string;
  config: Config;
  results: DocResult[];
  summary: ReportSummary;
}

// ─── Scan Status ──────────────────────────────────────────────────────────────

export type ScanState = "idle" | "running" | "complete" | "cancelled" | "error";

export interface ScanStatus {
  scan_id: string;
  state: ScanState;
  total: number;
  processed: number;
  current_file: string | null;
  stage: string;
  error: string | null;
  results_so_far: DocResult[];
}

// ─── Events ───────────────────────────────────────────────────────────────────

export interface ScanProgressEvent {
  scan_id: string;
  total: number;
  processed: number;
  current_file: string;
  stage: string;
  percentage: number;
}

export interface ScanCompleteEvent {
  scan_id: string;
  report: Report;
}

export interface ScanErrorEvent {
  scan_id: string;
  error: string;
}

export interface ScanDocEvent {
  scan_id: string;
  doc: DocResult;
}

// ─── Event name constants ─────────────────────────────────────────────────────
export const EVENT_SCAN_PROGRESS = "scan-progress";
export const EVENT_SCAN_COMPLETE = "scan-complete";
export const EVENT_SCAN_ERROR   = "scan-error";
export const EVENT_SCAN_DOC     = "scan-doc";

// ─── UI helpers ───────────────────────────────────────────────────────────────

export const STATUS_LABELS: Record<DocStatus, string> = {
  current:      "Current",
  stale:        "Stale",
  outdated:     "Outdated",
  orphaned:     "Orphaned",
  needs_review: "Needs Review",
};

export const STATUS_COLORS: Record<DocStatus, string> = {
  current:      "#22c55e",
  stale:        "#f59e0b",
  outdated:     "#ef4444",
  orphaned:     "#6b7280",
  needs_review: "#8b5cf6",
};
