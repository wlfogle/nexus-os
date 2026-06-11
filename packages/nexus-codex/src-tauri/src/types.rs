use serde::{Deserialize, Serialize};

// ─── Configuration ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Local filesystem roots to scan (defaults to $HOME)
    pub scan_roots: Vec<String>,
    /// Directory/path segments to skip during scanning
    pub excluded_paths: Vec<String>,
    /// Whether to also scan remote GitHub repos
    pub github_enabled: bool,
    /// GitHub username to list repos for
    pub github_username: String,
    /// Ollama base URL
    pub ollama_url: String,
    /// Override auto-selected model (None = auto-select)
    pub ollama_model_override: Option<String>,
    /// Directory to write report files into
    pub report_output_dir: String,
    /// Maximum file size to analyse (larger files are skipped)
    pub max_file_size_kb: u64,
}

impl Default for Config {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
        Config {
            scan_roots: vec![home.clone()],
            excluded_paths: vec![
                ".git".into(),
                "node_modules".into(),
                "target".into(),
                "dist".into(),
                "build".into(),
                ".cache".into(),
                ".local/share/Steam".into(),
                ".local/share/containers".into(),
                "snap".into(),
                ".var".into(),
                ".mozilla".into(),
                ".config/google-chrome".into(),
                "Media".into(),
                "media".into(),
            ],
            github_enabled: true,
            github_username: "wlfogle".into(),
            ollama_url: "http://localhost:11434".into(),
            ollama_model_override: None,
            report_output_dir: format!("{}/Documents/nexus-codex-reports", home),
            max_file_size_kb: 5120,
        }
    }
}

// ─── Document Classification ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DocStatus {
    /// Document accurately reflects current code/project state
    Current,
    /// Document is partially outdated but salvageable
    Stale,
    /// Document is significantly behind and misleading
    Outdated,
    /// Document has no related code or project to correspond with
    Orphaned,
    /// Confidence too low to classify reliably — human review needed
    NeedsReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DocSource {
    Local,
    Github,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DocType {
    Markdown,
    Text,
    Pdf,
    Rst,
    Adoc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocResult {
    /// Absolute local path or "github:<owner>/<repo>/<path>"
    pub path: String,
    /// Repository name if detected
    pub repo: Option<String>,
    /// Remote URL of the repository
    pub repo_url: Option<String>,
    pub source: DocSource,
    pub doc_type: DocType,
    pub status: DocStatus,
    /// 0.0–1.0: how confident the model is in the classification
    pub confidence: f32,
    /// 0.0–1.0: how stale the document appears
    pub staleness_score: f32,
    /// Short human-readable reason for the classification
    pub reason: String,
    /// Specific evidence extracted from the document or codebase
    pub evidence: String,
    /// LLM-proposed rewrite (read-only suggestion, never applied automatically)
    pub suggested_rewrite: Option<String>,
    /// ISO-8601 timestamp of last filesystem modification
    pub last_modified: Option<String>,
    /// ISO-8601 timestamp of last git commit touching this file
    pub last_commit_date: Option<String>,
    /// How many days since the nearest code in the same repo was last changed
    pub related_code_age_days: Option<i64>,
    pub file_size_bytes: u64,
}

// ─── Ollama Models ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size_gb: f32,
    /// Internal scoring used for auto-selection (higher = preferred)
    pub score: f32,
    /// True for the single model the app will use by default
    pub recommended: bool,
    /// Model family tag (qwen, llama, mistral, …)
    pub family: String,
}

// ─── Report ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total: usize,
    pub current: usize,
    pub stale: usize,
    pub outdated: usize,
    pub orphaned: usize,
    pub needs_review: usize,
    pub local_repos_scanned: usize,
    pub github_repos_scanned: usize,
    pub pdfs_scanned: usize,
    pub scan_duration_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub scan_id: String,
    pub generated_at: String,
    pub model_used: String,
    pub config: Config,
    pub results: Vec<DocResult>,
    pub summary: ReportSummary,
}

// ─── Scan Status ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScanState {
    Idle,
    Running,
    Complete,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStatus {
    pub scan_id: String,
    pub state: ScanState,
    pub total: usize,
    pub processed: usize,
    pub current_file: Option<String>,
    pub stage: String,
    pub error: Option<String>,
    /// Live results as they arrive (used by the frontend progress view)
    pub results_so_far: Vec<DocResult>,
}

// ─── Tauri Events (emitted by backend → received by frontend) ────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgressEvent {
    pub scan_id: String,
    pub total: usize,
    pub processed: usize,
    pub current_file: String,
    pub stage: String,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCompleteEvent {
    pub scan_id: String,
    pub report: Report,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanErrorEvent {
    pub scan_id: String,
    pub error: String,
}

/// Emitted each time a single document finishes analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanDocEvent {
    pub scan_id: String,
    pub doc: DocResult,
}

// ─── Event name constants ─────────────────────────────────────────────────────
pub const EVENT_SCAN_PROGRESS: &str = "scan-progress";
pub const EVENT_SCAN_COMPLETE: &str = "scan-complete";
pub const EVENT_SCAN_ERROR: &str = "scan-error";
pub const EVENT_SCAN_DOC: &str = "scan-doc";
