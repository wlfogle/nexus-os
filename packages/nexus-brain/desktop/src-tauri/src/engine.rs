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
use crate::classify;
use crate::config::{self, Config};
use crate::db::{self, Db};
use crate::embed;
use crate::extract;
use crate::interpret;
use crate::notes;
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

/// Anything that can receive progress updates.
///
/// The window implementation emits a Tauri event; the headless implementation
/// prints a line. Keeping this generic is what makes the pipeline runnable --
/// and therefore testable -- without a GUI attached.
pub trait ProgressSink: Send + Sync + 'static {
    fn report(&self, progress: Progress);
}

/// Reports into the app window.
pub struct WindowSink(pub AppHandle);

impl ProgressSink for WindowSink {
    fn report(&self, progress: Progress) {
        let _ = self.0.emit("librarian://progress", progress);
    }
}

/// Reports to stdout, one line per phase.
pub struct StdoutSink;

impl ProgressSink for StdoutSink {
    fn report(&self, p: Progress) {
        let s = &p.stats;
        println!(
            "[{:<10}] {:<58} scanned {:>6} extracted {:>6} embedded {:>6} interpreted {:>6}",
            p.phase, p.detail, s.scanned, s.extracted, s.embedded, s.interpreted
        );
    }
}

fn emit(sink: &dyn ProgressSink, state: &AppState, phase: &str, detail: String) {
    // Scoped so the guard is released before `sink.report` below: `emit` is
    // called from phases that may already be holding this lock, and
    // std::sync::Mutex is not reentrant.
    let stats = {
        let conn = state.db.lock().unwrap();
        db::stats(&conn).unwrap_or_default()
    };
    let payload = Progress {
        phase: phase.to_string(),
        detail,
        stats,
        running: state.running.load(Ordering::Relaxed),
    };
    sink.report(payload);
}

/// Run the whole pipeline until every file reaches stage 3.
pub async fn run_pipeline(sink: Arc<dyn ProgressSink>, state: Arc<AppState>) -> Result<()> {
    if state.running.swap(true, Ordering::SeqCst) {
        return Ok(()); // already running
    }
    state.cancel.store(false, Ordering::SeqCst);
    let cfg = state.config();
    let client = state.client();

    let finish = |sink: &dyn ProgressSink, state: &AppState, msg: String| {
        state.running.store(false, Ordering::SeqCst);
        emit(sink, state, "idle", msg);
    };

    // Every phase below follows the same two rules, and breaking either one
    // deadlocks the whole app:
    //
    //   1. The database guard is released *before* `emit`, because `emit`
    //      locks the same non-reentrant std::sync::Mutex to read stats.
    //   2. Blocking work (filesystem walks, git subprocesses, SQLite) runs
    //      inside `spawn_blocking`, so the async runtime stays free to serve
    //      commands from the window while a long pass is in flight.

    // --- repos first: file ownership feeds every later decision -------------
    emit(&*sink, &state, "repos", "discovering git repositories".into());
    {
        let dbh = state.db.clone();
        let c = cfg.clone();
        let res = tokio::task::spawn_blocking(move || {
            let mut conn = dbh.lock().unwrap();
            repos::refresh(&mut conn, &c)
        })
        .await?;
        match res {
            Ok(n) => emit(&*sink, &state, "repos", format!("{n} repositories")),
            Err(e) => emit(&*sink, &state, "repos", format!("repo scan failed: {e}")),
        }
    }

    // --- tier 0 -------------------------------------------------------------
    emit(&*sink, &state, "scan", "walking the filesystem".into());
    {
        let dbh = state.db.clone();
        let c = cfg.clone();
        let res = tokio::task::spawn_blocking(move || -> Result<scan::ScanReport> {
            let mut conn = dbh.lock().unwrap();
            let report = scan::run(&mut conn, &c)?;
            repos::assign_files(&mut conn)?;
            extract::skip_unreadable(&conn, c.max_read_bytes as i64)?;
            extract::reattach_moved(&conn)?;
            Ok(report)
        })
        .await?;
        match res {
            Ok(r) => emit(
                &*sink,
                &state,
                "scan",
                format!(
                    "{} files ({} new, {} changed, {} gone)",
                    r.seen, r.added, r.updated, r.vanished
                ),
            ),
            Err(e) => {
                finish(&*sink, &state, format!("scan failed: {e}"));
                return Err(e);
            }
        }
    }

    if !client.reachable().await {
        finish(
            &*sink,
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
            finish(&*sink, &state, "paused".into());
            return Ok(());
        }
        state.pass.fetch_add(1, Ordering::Relaxed);

        // tier 1
        let batch = {
            let conn = state.db.lock().unwrap();
            scan::pending_extract(&conn, &cfg, EXTRACT_BATCH)?
        };
        if !batch.is_empty() {
            let dbh = state.db.clone();
            let r = tokio::task::spawn_blocking(move || {
                let mut conn = dbh.lock().unwrap();
                extract::run(&mut conn, &batch)
            })
            .await??;
            emit(
                &*sink,
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
                &*sink,
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
                &*sink,
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
        let dbh = state.db.clone();
        let n = tokio::task::spawn_blocking(move || {
            let mut conn = dbh.lock().unwrap();
            embed::rebuild_centroids(&mut conn)
        })
        .await??;
        emit(&*sink, &state, "centroids", format!("{n} repo centroids"));
    }

    // Labels, repo fingerprints and the supersession graph. Runs after
    // centroids because it consumes them.
    {
        let dbh = state.db.clone();
        let c = cfg.clone();
        let r = tokio::task::spawn_blocking(move || {
            let mut conn = dbh.lock().unwrap();
            classify::run(&mut conn, &c)
        })
        .await??;
        emit(
            &*sink,
            &state,
            "classify",
            format!(
                "{} labelled, {} repo topics, {} superseded",
                r.classified, r.topics, r.supersedes
            ),
        );
    }

    // Index any markdown notes written since the last pass.
    {
        let dbh = state.db.clone();
        let c = cfg.clone();
        let n = tokio::task::spawn_blocking(move || {
            let mut conn = dbh.lock().unwrap();
            notes::reindex(&mut conn, &c)
        })
        .await??;
        emit(&*sink, &state, "notes", format!("{n} note(s) indexed"));
    }
    {
        let dbh = state.db.clone();
        let c = cfg.clone();
        let p = tokio::task::spawn_blocking(move || {
            let mut conn = dbh.lock().unwrap();
            actions::plan(&mut conn, &c)
        })
        .await??;
        emit(
            &*sink,
            &state,
            "plan",
            format!(
                "{} proposed ({} automatic, {} need review)",
                p.proposed, p.auto, p.pending
            ),
        );

        if p.auto > 0 {
            let dbh = state.db.clone();
            let plan_id = p.plan_id;
            let a = tokio::task::spawn_blocking(move || {
                let mut conn = dbh.lock().unwrap();
                actions::apply(&mut conn, plan_id)
            })
            .await??;
            emit(
                &*sink,
                &state,
                "apply",
                format!("{} applied, {} failed", a.applied, a.failed),
            );
        }
    }

    finish(&*sink, &state, "complete".into());
    Ok(())
}

pub fn stop(state: &AppState) {
    state.cancel.store(true, Ordering::SeqCst);
}
