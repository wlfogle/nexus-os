//! Commands callable from the window.
//!
//! Every command converts errors to `String` because Tauri needs a serialisable
//! error type; the message is shown verbatim in the UI rather than swallowed.

use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::actions::{self, Action};
use crate::config::Config;
use crate::db::{self, Stats};
use crate::engine::{self, AppState};
use crate::ollama::TagModel;
use crate::repos::{self, RepoInfo};
use crate::search::{self, Hit};

type R<T> = Result<T, String>;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[tauri::command]
pub fn export_action_backup(state: State<'_, Arc<AppState>>, action_id: i64) -> R<String> {
    let conn = state.db.lock().map_err(err)?;
    let snapshot = crate::safety::load(&conn, action_id).map_err(err)?;
    let bytes: Vec<u8> = conn
        .query_row(
            "SELECT backup FROM action_snapshots WHERE action_id=?1",
            rusqlite::params![action_id],
            |r| r.get(0),
        )
        .map_err(err)?;
    if crate::compare::hash(&bytes) != snapshot.sha256 {
        return Err("Backup checksum mismatch".into());
    }
    let path = crate::config::state_dir().join(format!(
        "safety-backup-{action_id}-{}.bin",
        chrono::Utc::now().format("%Y%m%dT%H%M%S%.fZ")
    ));
    crate::compare::checked_path(&path, true).map_err(err)?;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .map_err(err)?;
    file.write_all(&bytes).map_err(err)?;
    file.sync_all().map_err(err)?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn saved_comparison_reports() -> R<Vec<String>> {
    let root = crate::config::state_dir();
    let mut paths: Vec<_> = std::fs::read_dir(&root)
        .map_err(err)?
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.starts_with("comparison-") && n.ends_with(".json"))
                && e.file_type().is_ok_and(|t| t.is_file())
        })
        .collect();
    paths.sort_by_key(|e| std::cmp::Reverse(e.metadata().and_then(|m| m.modified()).ok()));
    Ok(paths
        .into_iter()
        .take(100)
        .map(|e| e.path().display().to_string())
        .collect())
}

#[tauri::command]
pub fn save_comparison_report(state: State<'_, Arc<AppState>>) -> R<String> {
    let session = state.comparison.lock().map_err(err)?;
    if session.running {
        return Err("Wait for comparison completion".into());
    }
    let report = session.report.as_ref().ok_or("No report to save")?;
    let path = crate::config::state_dir().join(format!(
        "comparison-{}.json",
        chrono::Utc::now().format("%Y%m%dT%H%M%S%.fZ")
    ));
    crate::compare::checked_path(&path, true).map_err(err)?;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .map_err(err)?;
    serde_json::to_writer_pretty(&mut file, report).map_err(err)?;
    file.sync_all().map_err(err)?;
    Ok(path.display().to_string())
}

#[tauri::command]
pub fn start_comparison(
    state: State<'_, Arc<AppState>>,
    options: crate::compare::Options,
) -> R<()> {
    let st = state.inner().clone();
    let cancel = {
        let mut session = st.comparison.lock().map_err(err)?;
        if session.running {
            return Err("A comparison is already running".into());
        }
        *session = crate::compare::Session::default();
        session.running = true;
        session.cancel.clone()
    };
    let cfg = st.config();
    tauri::async_runtime::spawn_blocking(move || {
        let result = crate::compare::run(&cfg, &options, &cancel, |progress| {
            if let Ok(mut session) = st.comparison.lock() {
                session.progress = progress;
            }
        });
        if let Ok(mut session) = st.comparison.lock() {
            session.running = false;
            match result {
                Ok(report) => session.report = Some(report),
                Err(e) => session.error = Some(e.to_string()),
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub fn load_comparison_report(state: State<'_, Arc<AppState>>, path: String) -> R<()> {
    let path = std::path::PathBuf::from(path);
    let parent = crate::config::state_dir();
    if path.parent() != Some(parent.as_path()) {
        return Err("Choose a report directly in Librarian's private state directory".into());
    }
    crate::compare::checked_path(&path, false).map_err(err)?;
    if std::fs::metadata(&path).map_err(err)?.len() > 64 * 1024 * 1024 {
        return Err("Report exceeds 64 MiB limit".into());
    }
    let mut report: crate::compare::Report =
        serde_json::from_slice(&std::fs::read(&path).map_err(err)?).map_err(err)?;
    for assessment in report.assessments.values_mut() {
        *assessment = crate::recovery::validate(assessment.clone(), assessment.evidence.clone());
    }
    if report
        .findings
        .iter()
        .any(|f| !report.roots.contains(&f.root) || !f.path.starts_with(&f.root))
    {
        return Err("Invalid report source boundaries".into());
    }
    let mut session = state.comparison.lock().map_err(err)?;
    if session.running {
        return Err("Wait for the current comparison".into());
    }
    session.report = Some(report);
    session.error = None;
    Ok(())
}

#[tauri::command]
pub fn comparison_status(state: State<'_, Arc<AppState>>) -> R<crate::compare::Status> {
    let session = state.comparison.lock().map_err(err)?;
    Ok(crate::compare::Status {
        running: session.running,
        progress: session.progress.clone(),
        report: session.report.clone(),
        error: session.error.clone(),
    })
}

#[tauri::command]
pub fn stop_comparison(state: State<'_, Arc<AppState>>) -> R<()> {
    state
        .comparison
        .lock()
        .map_err(err)?
        .cancel
        .store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn comparison_preview(
    state: State<'_, Arc<AppState>>,
    index: usize,
    counterpart: Option<usize>,
) -> R<crate::compare::Preview> {
    let finding = state
        .comparison
        .lock()
        .map_err(err)?
        .finding(index)
        .map_err(err)?;
    tauri::async_runtime::spawn_blocking(move || crate::compare::preview(&finding, counterpart))
        .await
        .map_err(err)?
        .map_err(err)
}

#[tauri::command]
pub async fn assess_ideas(
    state: State<'_, Arc<AppState>>,
    index: usize,
) -> R<crate::recovery::Assessment> {
    let finding = state
        .comparison
        .lock()
        .map_err(err)?
        .finding(index)
        .map_err(err)?;
    let path = finding.path.clone();
    let sha = finding.sha256.clone();
    let assessment = crate::recovery::assess(finding, state.config())
        .await
        .map_err(err)?;
    let mut session = state.comparison.lock().map_err(err)?;
    if let Some(report) = &mut session.report {
        if report
            .findings
            .iter()
            .any(|f| f.path == path && f.sha256 == sha)
        {
            report
                .assessments
                .insert(path.display().to_string(), assessment.clone());
        }
    }
    Ok(assessment)
}

#[tauri::command]
pub fn queue_reference(
    state: State<'_, Arc<AppState>>,
    index: usize,
    repo: String,
    relative: String,
    reason: String,
) -> R<i64> {
    let session = state.comparison.lock().map_err(err)?;
    let finding = session.finding(index).map_err(err)?;
    let report = session.report.as_ref().ok_or("No comparison report")?;
    let repo = std::path::PathBuf::from(repo);
    if !report.repositories.contains(&repo) {
        return Err("Choose a repository from this comparison".into());
    }
    let dest =
        crate::recovery::archive_destination(&finding, &repo, std::path::Path::new(&relative))
            .map_err(err)?;
    crate::compare::preview(&finding, None).map_err(err)?;
    let mut conn = state.db.lock().map_err(err)?;
    actions::queue_reference(&mut conn, &finding, &repo, &dest, &reason).map_err(err)
}

#[derive(Debug, Serialize)]
pub struct CatalogRow {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub class: String,
    pub size: i64,
    pub mtime: f64,
    pub repo: Option<String>,
    pub stage: i64,
    pub title: String,
    pub kind: String,
    pub purpose: String,
    pub summary: String,
    pub topics: Vec<String>,
    pub status: String,
    pub action: String,
    pub reason: String,
    pub confidence: f32,
    pub model: String,
}

#[derive(Debug, Serialize)]
pub struct DupGroup {
    pub sha256: String,
    pub count: i64,
    pub size: i64,
    pub paths: Vec<String>,
    pub reclaimable: i64,
}

#[derive(Debug, Serialize)]
pub struct Health {
    pub ollama_up: bool,
    pub ollama_url: String,
    pub models_installed: usize,
    pub db_path: String,
    pub running: bool,
}

#[tauri::command]
pub fn get_stats(state: State<'_, Arc<AppState>>) -> R<Stats> {
    let conn = state.db.lock().map_err(err)?;
    db::stats(&conn).map_err(err)
}

#[tauri::command]
pub fn get_config(state: State<'_, Arc<AppState>>) -> R<Config> {
    Ok(state.config())
}

#[tauri::command]
pub fn save_config(state: State<'_, Arc<AppState>>, cfg: Config) -> R<()> {
    cfg.validate().map_err(err)?;
    cfg.ensure_dirs().map_err(err)?;
    cfg.save().map_err(err)?;
    *state.cfg.lock().map_err(err)? = cfg;
    *state.ollama.lock().map_err(err)? = crate::ollama::Ollama::new(&state.config().ollama_url);
    Ok(())
}

#[tauri::command]
pub async fn health(state: State<'_, Arc<AppState>>) -> R<Health> {
    let cfg = state.config();
    let client = state.client();
    let up = client.reachable().await;
    let models = if up {
        client.models().await.map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    Ok(Health {
        ollama_up: up,
        ollama_url: cfg.ollama_url.clone(),
        models_installed: models,
        db_path: crate::config::state_dir()
            .join("catalog.db")
            .to_string_lossy()
            .to_string(),
        running: state.running.load(std::sync::atomic::Ordering::Relaxed),
    })
}

#[tauri::command]
pub async fn list_models(state: State<'_, Arc<AppState>>) -> R<Vec<TagModel>> {
    state.client().models().await.map_err(err)
}

#[tauri::command]
pub fn start_pipeline(app: AppHandle, state: State<'_, Arc<AppState>>) -> R<()> {
    let st = state.inner().clone();
    let sink: Arc<dyn engine::ProgressSink> = Arc::new(engine::WindowSink(app));
    tauri::async_runtime::spawn(async move {
        if let Err(e) = engine::run_pipeline(sink, st).await {
            eprintln!("librarian: pipeline error: {e}");
        }
    });
    Ok(())
}

#[tauri::command]
pub fn stop_pipeline(state: State<'_, Arc<AppState>>) -> R<()> {
    engine::stop(state.inner());
    Ok(())
}

#[tauri::command]
pub fn list_catalog(
    state: State<'_, Arc<AppState>>,
    class: Option<String>,
    status: Option<String>,
    loose_only: bool,
    limit: i64,
    offset: i64,
) -> R<Vec<CatalogRow>> {
    let conn = state.db.lock().map_err(err)?;
    let mut sql = String::from(
        "SELECT f.id, f.path, f.name, f.class, f.size, f.mtime, r.name, f.stage,
                COALESCE(i.title,''), COALESCE(i.kind,''), COALESCE(i.purpose,''),
                COALESCE(i.summary,''), COALESCE(i.topics,'[]'), COALESCE(i.status,''),
                COALESCE(i.action,''), COALESCE(i.reason,''), COALESCE(i.confidence,0),
                COALESCE(i.model,'')
           FROM files f
           LEFT JOIN repos r           ON r.id = f.repo_id
           LEFT JOIN interpretations i ON i.file_id = f.id
          WHERE f.present = 1",
    );
    if class.is_some() {
        sql.push_str(" AND f.class = :class");
    }
    if status.is_some() {
        sql.push_str(" AND i.status = :status");
    }
    if loose_only {
        sql.push_str(" AND f.repo_id IS NULL");
    }
    sql.push_str(" ORDER BY f.mtime DESC LIMIT :limit OFFSET :offset");

    let mut stmt = conn.prepare(&sql).map_err(err)?;
    let mut named: Vec<(&str, &dyn rusqlite::ToSql)> = Vec::new();
    if let Some(c) = &class {
        named.push((":class", c));
    }
    if let Some(s) = &status {
        named.push((":status", s));
    }
    named.push((":limit", &limit));
    named.push((":offset", &offset));

    let rows = stmt
        .query_map(named.as_slice(), |r| {
            let topics: String = r.get(12)?;
            Ok(CatalogRow {
                id: r.get(0)?,
                path: r.get(1)?,
                name: r.get(2)?,
                class: r.get(3)?,
                size: r.get(4)?,
                mtime: r.get(5)?,
                repo: r.get(6)?,
                stage: r.get(7)?,
                title: r.get(8)?,
                kind: r.get(9)?,
                purpose: r.get(10)?,
                summary: r.get(11)?,
                topics: serde_json::from_str(&topics).unwrap_or_default(),
                status: r.get(13)?,
                action: r.get(14)?,
                reason: r.get(15)?,
                confidence: r.get(16)?,
                model: r.get(17)?,
            })
        })
        .map_err(err)?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(err)?);
    }
    Ok(out)
}

#[tauri::command]
pub async fn search_catalog(
    state: State<'_, Arc<AppState>>,
    query: String,
    limit: usize,
) -> R<Vec<Hit>> {
    let cfg = state.config();
    let client = state.client();
    let dbh = state.db.clone();
    search::query(&dbh, &client, &cfg.models.embed, &query, limit)
        .await
        .map_err(err)
}

#[tauri::command]
pub fn list_stale(state: State<'_, Arc<AppState>>, limit: i64) -> R<Vec<Hit>> {
    let conn = state.db.lock().map_err(err)?;
    search::stale(&conn, limit).map_err(err)
}

#[tauri::command]
pub fn list_repos(state: State<'_, Arc<AppState>>) -> R<Vec<RepoInfo>> {
    let conn = state.db.lock().map_err(err)?;
    repos::list(&conn).map_err(err)
}

#[tauri::command]
pub fn list_duplicates(state: State<'_, Arc<AppState>>, limit: i64) -> R<Vec<DupGroup>> {
    let conn = state.db.lock().map_err(err)?;
    let groups = crate::extract::duplicate_groups(&conn, limit).map_err(err)?;
    Ok(groups
        .into_iter()
        .map(|(sha256, count, size, paths)| DupGroup {
            reclaimable: (count - 1) * size,
            sha256,
            count,
            size,
            paths,
        })
        .collect())
}

#[tauri::command]
pub fn list_review(state: State<'_, Arc<AppState>>, limit: i64) -> R<Vec<Action>> {
    let conn = state.db.lock().map_err(err)?;
    actions::list_actions(&conn, Some("pending"), limit).map_err(err)
}

#[tauri::command]
pub fn list_history(state: State<'_, Arc<AppState>>, limit: i64) -> R<Vec<Action>> {
    let conn = state.db.lock().map_err(err)?;
    actions::list_actions(&conn, None, limit).map_err(err)
}

#[tauri::command]
pub fn decide_action(state: State<'_, Arc<AppState>>, action_id: i64, approve: bool) -> R<()> {
    let conn = state.db.lock().map_err(err)?;
    actions::decide(&conn, action_id, approve).map_err(err)
}

#[tauri::command]
pub fn apply_approved(state: State<'_, Arc<AppState>>) -> R<actions::ApplyReport> {
    let mut conn = state.db.lock().map_err(err)?;
    let plans: Vec<i64> = {
        let mut q = conn
            .prepare("SELECT DISTINCT plan_id FROM actions WHERE state = 'approved'")
            .map_err(err)?;
        let rows = q.query_map([], |r| r.get::<_, i64>(0)).map_err(err)?;
        rows.filter_map(|r| r.ok()).collect()
    };
    let mut total = actions::ApplyReport::default();
    for plan_id in plans {
        let r = actions::apply(&mut conn, plan_id).map_err(err)?;
        total.applied += r.applied;
        total.failed += r.failed;
        total.skipped += r.skipped;
    }
    Ok(total)
}

#[tauri::command]
pub fn undo_plan(state: State<'_, Arc<AppState>>, plan_id: i64) -> R<actions::ApplyReport> {
    let mut conn = state.db.lock().map_err(err)?;
    actions::undo(&mut conn, plan_id).map_err(err)
}

#[tauri::command]
pub fn similar_files(
    state: State<'_, Arc<AppState>>,
    file_id: i64,
    limit: usize,
) -> R<Vec<(i64, String, f32)>> {
    let conn = state.db.lock().map_err(err)?;
    search::similar_to(&conn, file_id, limit).map_err(err)
}

#[tauri::command]
pub fn read_file_text(state: State<'_, Arc<AppState>>, file_id: i64) -> R<String> {
    let conn = state.db.lock().map_err(err)?;
    conn.query_row(
        "SELECT COALESCE(body,'') FROM file_text WHERE file_id = ?1",
        rusqlite::params![file_id],
        |r| r.get::<_, String>(0),
    )
    .map_err(err)
}
