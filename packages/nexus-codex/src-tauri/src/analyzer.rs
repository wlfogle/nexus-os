use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use tauri::Emitter;

use crate::github::{self, GithubDoc, GithubRepo};
use crate::ollama;
use crate::repo;
use crate::report;
use crate::scanner;
use crate::state::SharedScanRegistry;
use crate::types::{
    Config, DocResult, DocSource, DocStatus, DocType, ScanCompleteEvent, ScanDocEvent,
    ScanErrorEvent, ScanProgressEvent, ScanState, ScanStatus, EVENT_SCAN_COMPLETE,
    EVENT_SCAN_DOC, EVENT_SCAN_ERROR, EVENT_SCAN_PROGRESS,
};

/// A single unit of documentation work to analyse.
enum DocItem {
    Local {
        path: PathBuf,
        doc_type: DocType,
        size: u64,
    },
    Github {
        repo: GithubRepo,
        doc: GithubDoc,
    },
}

/// File extensions treated as "code" when gathering a nearby snippet for context.
const CODE_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "ts", "tsx", "jsx", "go", "c", "h", "cpp", "hpp", "cc", "java",
    "rb", "php", "sh", "bash", "toml", "yaml", "yml", "json", "kt", "swift", "cs",
    "lua", "zig", "svelte", "vue",
];

/// Convert a [`SystemTime`] to an ISO-8601 string.
fn system_time_to_iso(t: SystemTime) -> String {
    let dt: DateTime<Utc> = t.into();
    dt.to_rfc3339()
}

/// Update the live [`ScanStatus`] for a scan, holding the lock only briefly.
fn update_status<F>(registry: &SharedScanRegistry, scan_id: &str, f: F)
where
    F: FnOnce(&mut ScanStatus),
{
    if let Ok(mut guard) = registry.lock() {
        if let Some(entry) = guard.scans.get_mut(scan_id) {
            f(&mut entry.status);
        }
    }
}

/// Read the text content of a local document, extracting PDFs as needed.
async fn read_local_content(path: &Path, doc_type: &DocType) -> String {
    match doc_type {
        DocType::Pdf => {
            let owned = path.to_path_buf();
            match tokio::task::spawn_blocking(move || crate::pdf::extract_text(&owned)).await {
                Ok(Ok(text)) => text,
                _ => String::new(),
            }
        }
        _ => std::fs::read_to_string(path).unwrap_or_default(),
    }
}

/// Find the nearest code file in the same directory as `doc_path` and return a
/// short snippet (first ~150 lines) for analysis context. Empty if none found.
fn read_nearby_code(doc_path: &Path) -> String {
    let dir = match doc_path.parent() {
        Some(d) => d,
        None => return String::new(),
    };

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return String::new(),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_ascii_lowercase(),
            None => continue,
        };
        if !CODE_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            let snippet: String = content
                .lines()
                .take(150)
                .collect::<Vec<_>>()
                .join("\n");
            let header = format!("// {}\n", path.to_string_lossy());
            return format!("{header}{snippet}");
        }
    }

    String::new()
}

/// Map an optional analysis result into the classification fields of a `DocResult`.
fn analysis_fields(
    analysis: Result<ollama::DocAnalysis>,
) -> (DocStatus, f32, f32, String, String, Option<String>) {
    match analysis {
        Ok(a) => (
            a.status,
            a.confidence,
            a.staleness_score,
            a.reason,
            a.evidence,
            a.suggested_rewrite,
        ),
        Err(e) => (
            DocStatus::NeedsReview,
            0.0,
            0.0,
            format!("analysis failed: {e}"),
            String::new(),
            None,
        ),
    }
}

/// Analyse a single local document into a [`DocResult`].
async fn process_local(
    config: &Config,
    model: &str,
    path: PathBuf,
    doc_type: DocType,
    size: u64,
) -> DocResult {
    let path_str = path.to_string_lossy().to_string();

    let (repo_name, repo_url) = match repo::find_repo_for_path(&path) {
        Some((name, _root)) => (Some(name), repo::find_repo_url(&path)),
        None => (None, None),
    };

    let last_modified = std::fs::metadata(&path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(system_time_to_iso);

    let last_commit_date = repo::get_file_git_age(&path);
    let related_code_age_days = path
        .parent()
        .and_then(repo::get_dir_latest_code_age);

    let content = read_local_content(&path, &doc_type).await;
    let nearby = read_nearby_code(&path);
    let repo_for_prompt = repo_name.clone().unwrap_or_else(|| "(unknown)".to_string());

    let analysis = ollama::analyze_doc(
        &config.ollama_url,
        model,
        &content,
        &nearby,
        &repo_for_prompt,
        &path_str,
    )
    .await;

    let (status, confidence, staleness_score, reason, evidence, suggested_rewrite) =
        analysis_fields(analysis);

    DocResult {
        path: path_str,
        repo: repo_name,
        repo_url,
        source: DocSource::Local,
        doc_type,
        status,
        confidence,
        staleness_score,
        reason,
        evidence,
        suggested_rewrite,
        last_modified,
        last_commit_date,
        related_code_age_days,
        file_size_bytes: size,
    }
}

/// Analyse a single remote GitHub document into a [`DocResult`].
async fn process_github(
    config: &Config,
    model: &str,
    token: &str,
    repo_meta: GithubRepo,
    doc: GithubDoc,
) -> DocResult {
    let path = format!("github:{}/{}", repo_meta.full_name, doc.path);
    let content = github::download_content(&doc.download_url, token)
        .await
        .unwrap_or_default();

    let analysis = ollama::analyze_doc(
        &config.ollama_url,
        model,
        &content,
        "",
        &repo_meta.name,
        &doc.path,
    )
    .await;

    let (status, confidence, staleness_score, reason, evidence, suggested_rewrite) =
        analysis_fields(analysis);

    DocResult {
        path,
        repo: Some(repo_meta.name.clone()),
        repo_url: Some(repo_meta.html_url.clone()),
        source: DocSource::Github,
        doc_type: doc.doc_type,
        status,
        confidence,
        staleness_score,
        reason,
        evidence,
        suggested_rewrite,
        last_modified: None,
        last_commit_date: None,
        related_code_age_days: None,
        file_size_bytes: doc.size_bytes,
    }
}

/// The full scan pipeline. Errors here are unrecoverable and surfaced to the caller.
async fn run_scan(
    app: &tauri::AppHandle,
    scan_id: &str,
    config: &Config,
    registry: &SharedScanRegistry,
) -> Result<()> {
    let start = Instant::now();
    let token = github::get_token().unwrap_or_default();

    // 1. Local filesystem scan.
    update_status(registry, scan_id, |s| {
        s.stage = "scanning local files".to_string();
    });
    let local_docs = scanner::scan_local(config);
    let mut items: Vec<DocItem> = local_docs
        .into_iter()
        .map(|(path, doc_type, size)| DocItem::Local {
            path,
            doc_type,
            size,
        })
        .collect();

    // 2. Optional GitHub scan.
    if config.github_enabled && !config.github_username.is_empty() {
        update_status(registry, scan_id, |s| {
            s.stage = "listing GitHub repositories".to_string();
        });
        let repos = github::list_repos(&config.github_username, &token).await?;
        for repo_meta in repos {
            match github::fetch_repo_docs(&repo_meta, &token).await {
                Ok(docs) => {
                    for doc in docs {
                        items.push(DocItem::Github {
                            repo: repo_meta.clone(),
                            doc,
                        });
                    }
                }
                // Empty repos / missing branches are skipped rather than aborting.
                Err(_) => continue,
            }
        }
    }

    let total = items.len();
    update_status(registry, scan_id, |s| {
        s.total = total;
        s.stage = "selecting model".to_string();
    });

    // 3. Model selection.
    let models = ollama::list_models(&config.ollama_url).await?;
    let model = ollama::select_model(&models, config.ollama_model_override.as_deref())
        .ok_or_else(|| anyhow!("no Ollama models available for analysis"))?;

    update_status(registry, scan_id, |s| {
        s.stage = "analyzing documents".to_string();
    });

    // 4. Analyse every document.
    let mut results: Vec<DocResult> = Vec::with_capacity(total);
    for (idx, item) in items.into_iter().enumerate() {
        let processed = idx + 1;

        let doc_result = match item {
            DocItem::Local {
                path,
                doc_type,
                size,
            } => process_local(config, &model, path, doc_type, size).await,
            DocItem::Github { repo, doc } => {
                process_github(config, &model, &token, repo, doc).await
            }
        };

        let current_file = doc_result.path.clone();

        update_status(registry, scan_id, |s| {
            s.processed = processed;
            s.current_file = Some(current_file.clone());
            s.results_so_far.push(doc_result.clone());
        });

        let _ = app.emit(
            EVENT_SCAN_DOC,
            ScanDocEvent {
                scan_id: scan_id.to_string(),
                doc: doc_result.clone(),
            },
        );

        let percentage = if total > 0 {
            (processed as f32 / total as f32) * 100.0
        } else {
            100.0
        };
        let _ = app.emit(
            EVENT_SCAN_PROGRESS,
            ScanProgressEvent {
                scan_id: scan_id.to_string(),
                total,
                processed,
                current_file,
                stage: "analyzing documents".to_string(),
                percentage,
            },
        );

        results.push(doc_result);
    }

    // 5. Build the final report and mark the scan complete.
    let duration_secs = start.elapsed().as_secs_f64();
    let report = report::build_report(scan_id, &model, config, results, duration_secs);

    if let Ok(mut guard) = registry.lock() {
        if let Some(entry) = guard.scans.get_mut(scan_id) {
            entry.status.state = ScanState::Complete;
            entry.status.stage = "complete".to_string();
            entry.report = Some(report.clone());
        }
    }

    let _ = app.emit(
        EVENT_SCAN_COMPLETE,
        ScanCompleteEvent {
            scan_id: scan_id.to_string(),
            report,
        },
    );

    Ok(())
}

/// Register a scan and spawn the background task that performs it.
///
/// Emits `scan-progress`, `scan-doc`, `scan-complete`, and `scan-error` events as
/// it proceeds. The returned future completes once the task has been spawned; the
/// scan itself continues in the background.
pub async fn start_scan(
    app: tauri::AppHandle,
    scan_id: String,
    config: Config,
    registry: SharedScanRegistry,
) -> Result<()> {
    // 1. Register the scan (no handle yet).
    {
        let mut guard = registry
            .lock()
            .map_err(|e| anyhow!("registry lock poisoned: {e}"))?;
        guard.register(scan_id.clone(), None);
    }

    // 2. Spawn the background task.
    let task_registry = Arc::clone(&registry);
    let task_app = app.clone();
    let task_scan_id = scan_id.clone();
    let task_config = config.clone();

    let handle = tokio::spawn(async move {
        if let Err(e) =
            run_scan(&task_app, &task_scan_id, &task_config, &task_registry).await
        {
            let msg = e.to_string();
            update_status(&task_registry, &task_scan_id, |s| {
                s.state = ScanState::Error;
                s.stage = "error".to_string();
                s.error = Some(msg.clone());
            });
            let _ = task_app.emit(
                EVENT_SCAN_ERROR,
                ScanErrorEvent {
                    scan_id: task_scan_id.clone(),
                    error: msg,
                },
            );
        }
    });

    // 3. Store the JoinHandle so the scan can be cancelled.
    {
        let mut guard = registry
            .lock()
            .map_err(|e| anyhow!("registry lock poisoned: {e}"))?;
        if let Some(entry) = guard.scans.get_mut(&scan_id) {
            entry.handle = Some(handle);
        }
    }

    Ok(())
}
