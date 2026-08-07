//! The background pipeline.
//!
//! One task drives all four tiers in a loop. Each pass does a bounded amount of
//! work and then reports progress, so the window stays responsive and the run
//! can be paused or resumed at any point without losing position -- progress
//! lives in the `files.stage` column, not in memory.

use anyhow::Result;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use crate::actions;
use crate::config::{self, Config};
use crate::db::{self, Db};
use crate::embed;
use crate::extract;
use crate::interpret;
use crate::ollama::Ollama;
use crate::repos;
use crate::scan;

/// Files processed per tier per pass. Small enough that pausing feels instant.
const EXTRACT_BATCH: i64 = 200;
const EMBED_BATCH: i64 = 40;
const INTERPRET_BATCH: i64 = 20;

pub struct AppState {
    pub db: Db,
    pub cfg: std::sync::Mutex<Config>,
    pub ollama: std::sync::Mutex<Ollama>,
    pub running: Arc<AtomicBool>,
    pub cancel: Arc<AtomicBool>,
    pub pass: Arc<AtomicU64>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let cfg = Config::load()?;
        cfg.ensure_dirs()?;
        let db = db::open(&config::state_dir().join("catalog.db"))?;
        let ollama = Ollama::new(&cfg.ollama_url);
        Ok(Self {
            db,
            cfg: std::sync::Mutex::new(cfg),
            ollama: std::sync::Mutex::new(ollama),
            running: Arc::new(AtomicBool::new(false)),
            cancel: Arc::new(AtomicBool::new(false)),
            pass: Arc::new(AtomicU64::new(0)),
        })
    }

    pub fn config(&self) -> Config {
        self.cfg.lock().unwrap().clone()
    }

    pub fn client(&self) -> Ollama {
        self.ollama.lock().unwrap().clone()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub phase: String,
    pub detail: String,
    pub stats: db::Stats,
    pub running: bool,
}

fn emit(app: &AppHandle, state: &AppState, phase: &str, detail: String) {
    let stats = {
        let conn = state.db.lock().unwrap();
        db::stats(&conn).unwrap_or(db::Stats {
            files_total: 0,
            files_present: 0,
            scanned: 0,
            extracted: 0,
            embedded: 0,
            interpreted: 0,
            repos: 0,
            notes: 0,
            pending_actions: 0,
            duplicate_groups: 0,
            loose_files: 0,
            bytes_loose: 0,
        })
    };
    let payload = Progress {
        phase: phase.to_string(),
        detail,
        stats,
        running: state.running.load(Ordering::Relaxed),
    };
    let _ = app.emit("librarian://progress", payload);
}

/// Run the whole pipeline until every file reaches stage 3.
pub async fn run_pipeline(app: AppHandle, state: Arc<AppState>) -> Result<()> {
    if state.running.swap(true, Ordering::SeqCst) {
        return Ok(()); // already running
    }
    state.cancel.store(false, Ordering::SeqCst);
    let cfg = state.config();
    let client = state.client();

    let finish = |app: &AppHandle, state: &AppState, msg: String| {
        state.running.store(false, Ordering::SeqCst);
        emit(app, state, "idle", msg);
    };

    // --- repos first: file ownership feeds every later decision -------------
    emit(&app, &state, "repos", "discovering git repositories".into());
    {
        let mut conn = state.db.lock().unwrap();
        match repos::refresh(&mut conn, &cfg) {
            Ok(n) => emit(&app, &state, "repos", format!("{n} repositories")),
            Err(e) => emit(&app, &state, "repos", format!("repo scan failed: {e}")),
        }
    }

    // --- tier 0 -------------------------------------------------------------
    emit(&app, &state, "scan", "walking the filesystem".into());
    {
        let mut conn = state.db.lock().unwrap();
        match scan::run(&mut conn, &cfg) {
            Ok(r) => emit(
                &app,
                &state,
                "scan",
                format!(
                    "{} files ({} new, {} changed, {} gone)",
                    r.seen, r.added, r.updated, r.vanished
                ),
            ),
            Err(e) => {
                finish(&app, &state, format!("scan failed: {e}"));
                return Err(e);
            }
        }
        repos::assign_files(&mut conn)?;
        extract::skip_unreadable(&conn, cfg.max_read_bytes as i64)?;
        extract::reattach_moved(&conn)?;
    }

    if !client.reachable().await {
        finish(
            &app,
            &state,
            "Ollama is not reachable - inventory is complete but nothing can be \
             interpreted until it is running."
                .into(),
        );
        return Ok(());
    }

    // --- tiers 1..3 ---------------------------------------------------------
    loop {
        if state.cancel.load(Ordering::Relaxed) {
            finish(&app, &state, "paused".into());
            return Ok(());
        }
        state.pass.fetch_add(1, Ordering::Relaxed);

        // tier 1
        let batch = {
            let conn = state.db.lock().unwrap();
            scan::pending_extract(&conn, &cfg, EXTRACT_BATCH)?
        };
        if !batch.is_empty() {
            let mut conn = state.db.lock().unwrap();
            let r = extract::run(&mut conn, &batch)?;
            drop(conn);
            emit(
                &app,
                &state,
                "extract",
                format!("read {} files ({} with text)", r.processed, r.with_text),
            );
            continue;
        }

        // tier 2
        let r = embed::run(&state.db, &cfg, &client, EMBED_BATCH).await?;
        if r.files > 0 {
            emit(
                &app,
                &state,
                "embed",
                format!("embedded {} files / {} chunks", r.files, r.chunks),
            );
            continue;
        }

        // tier 3
        let r = interpret::run(&state.db, &cfg, &client, INTERPRET_BATCH).await?;
        if r.done > 0 {
            let models: Vec<String> = r
                .by_model
                .iter()
                .map(|(m, n)| format!("{m} x{n}"))
                .collect();
            emit(
                &app,
                &state,
                "interpret",
                format!(
                    "interpreted {} ({} escalated) [{}]",
                    r.done,
                    r.escalated,
                    models.join(", ")
                ),
            );
            continue;
        }

        break;
    }

    // --- derived artefacts --------------------------------------------------
    {
        let mut conn = state.db.lock().unwrap();
        let n = embed::rebuild_centroids(&mut conn)?;
        drop(conn);
        emit(&app, &state, "centroids", format!("{n} repo centroids"));
    }
    {
        let mut conn = state.db.lock().unwrap();
        let p = actions::plan(&mut conn, &cfg)?;
        drop(conn);
        emit(
            &app,
            &state,
            "plan",
            format!(
                "{} proposed ({} automatic, {} need review)",
                p.proposed, p.auto, p.pending
            ),
        );

        if p.auto > 0 {
            let mut conn = state.db.lock().unwrap();
            let a = actions::apply(&mut conn, p.plan_id)?;
            drop(conn);
            emit(
                &app,
                &state,
                "apply",
                format!("{} applied, {} failed", a.applied, a.failed),
            );
        }
    }

    finish(&app, &state, "complete".into());
    Ok(())
}

pub fn stop(state: &AppState) {
    state.cancel.store(true, Ordering::SeqCst);
}
