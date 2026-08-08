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
    cfg.ensure_dirs().map_err(err)?;
    cfg.save().map_err(err)?;
    *state.cfg.lock().map_err(err)? = cfg;
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
    actions::list_actions(&conn, Some("applied"), limit).map_err(err)
}

#[tauri::command]
pub fn decide_action(
    state: State<'_, Arc<AppState>>,
    action_id: i64,
    approve: bool,
) -> R<()> {
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
