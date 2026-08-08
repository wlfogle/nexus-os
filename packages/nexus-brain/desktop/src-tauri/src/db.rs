//! Catalog storage.
//!
//! One SQLite file holds everything: the file inventory, extracted text (FTS5),
//! chunk embeddings, the git repo graph, per-file LLM interpretations,
//! classifications, supersession edges, markdown notes, and the action journal
//! that makes every move reversible.
//!
//! Embeddings are stored as raw little-endian f32 blobs on the chunk row.
//! They are L2-normalised at write time so cosine similarity is a plain dot
//! product, which keeps the search path free of any vector-database dependency.

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Connection>>;

const SCHEMA: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA synchronous  = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA temp_store   = MEMORY;

-- ---------------------------------------------------------------- files ----
CREATE TABLE IF NOT EXISTS files (
    id           INTEGER PRIMARY KEY,
    path         TEXT    NOT NULL UNIQUE,
    parent       TEXT    NOT NULL,
    name         TEXT    NOT NULL,
    ext          TEXT    NOT NULL DEFAULT '',
    class        TEXT    NOT NULL DEFAULT 'other',
    size         INTEGER NOT NULL DEFAULT 0,
    mtime        REAL    NOT NULL DEFAULT 0,
    sha256       TEXT,
    repo_id      INTEGER REFERENCES repos(id) ON DELETE SET NULL,
    -- pipeline progress: 0 scanned, 1 extracted, 2 embedded, 3 interpreted
    stage        INTEGER NOT NULL DEFAULT 0,
    text_len     INTEGER NOT NULL DEFAULT 0,
    extract_err  TEXT,
    present      INTEGER NOT NULL DEFAULT 1,
    first_seen   REAL    NOT NULL,
    last_seen    REAL    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_files_stage   ON files(stage) WHERE present = 1;
CREATE INDEX IF NOT EXISTS idx_files_sha     ON files(sha256);
CREATE INDEX IF NOT EXISTS idx_files_repo    ON files(repo_id);
CREATE INDEX IF NOT EXISTS idx_files_parent  ON files(parent);
CREATE INDEX IF NOT EXISTS idx_files_class   ON files(class);

-- Extracted text, searchable. Kept external to `files` so the row stays small.
CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
    path, name, body,
    content='',
    tokenize='porter unicode61'
);
-- maps files_fts rowid -> files.id
CREATE TABLE IF NOT EXISTS fts_map (
    rowid   INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_fts_map_file ON fts_map(file_id);

CREATE TABLE IF NOT EXISTS file_text (
    file_id INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
    body    TEXT NOT NULL
);

-- --------------------------------------------------------------- chunks ----
CREATE TABLE IF NOT EXISTS chunks (
    id        INTEGER PRIMARY KEY,
    file_id   INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    ord       INTEGER NOT NULL,
    text      TEXT    NOT NULL,
    embedding BLOB,               -- little-endian f32[], L2-normalised
    dim       INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_chunks_file ON chunks(file_id);
CREATE INDEX IF NOT EXISTS idx_chunks_emb  ON chunks(id) WHERE embedding IS NOT NULL;

-- ---------------------------------------------------------------- repos ----
CREATE TABLE IF NOT EXISTS repos (
    id            INTEGER PRIMARY KEY,
    path          TEXT    NOT NULL UNIQUE,
    name          TEXT    NOT NULL,
    owner         TEXT    NOT NULL DEFAULT 'local',
    remote        TEXT,
    kind          TEXT    NOT NULL DEFAULT 'repo',  -- repo|worktree|submodule
    parent_id     INTEGER REFERENCES repos(id) ON DELETE CASCADE,
    branch        TEXT,
    last_commit   INTEGER NOT NULL DEFAULT 0,       -- unix seconds
    dirty         INTEGER NOT NULL DEFAULT 0,
    untracked     INTEGER NOT NULL DEFAULT 0,
    unpushed      INTEGER NOT NULL DEFAULT 0,
    stashes       INTEGER NOT NULL DEFAULT 0,
    -- every commit reachable locally also exists on a remote, working tree clean
    recoverable   INTEGER NOT NULL DEFAULT 0,
    size_bytes    INTEGER NOT NULL DEFAULT 0,
    scanned_at    REAL    NOT NULL DEFAULT 0
);

-- Per-repo lexical fingerprint used for relevance scoring.
CREATE TABLE IF NOT EXISTS repo_topics (
    repo_id INTEGER NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    term    TEXT    NOT NULL,
    weight  REAL    NOT NULL,
    PRIMARY KEY (repo_id, term)
);

-- Mean of a repo's chunk embeddings; the cheap first-pass relevance signal.
CREATE TABLE IF NOT EXISTS repo_centroids (
    repo_id   INTEGER PRIMARY KEY REFERENCES repos(id) ON DELETE CASCADE,
    embedding BLOB NOT NULL,
    dim       INTEGER NOT NULL,
    n_chunks  INTEGER NOT NULL
);

-- -------------------------------------------------------- interpretation ----
-- One row per file: what the local LLM decided the file actually is.
CREATE TABLE IF NOT EXISTS interpretations (
    file_id      INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
    title        TEXT NOT NULL DEFAULT '',
    kind         TEXT NOT NULL DEFAULT '',
    purpose      TEXT NOT NULL DEFAULT '',
    summary      TEXT NOT NULL DEFAULT '',
    topics       TEXT NOT NULL DEFAULT '[]',   -- json array
    entities     TEXT NOT NULL DEFAULT '[]',   -- json array
    related_repo TEXT NOT NULL DEFAULT '',
    status       TEXT NOT NULL DEFAULT '',     -- current|stale|superseded|reference|junk
    action       TEXT NOT NULL DEFAULT '',     -- keep|file|archive|quarantine|review
    reason       TEXT NOT NULL DEFAULT '',
    confidence   REAL NOT NULL DEFAULT 0,
    model        TEXT NOT NULL DEFAULT '',
    escalated    INTEGER NOT NULL DEFAULT 0,
    decided_at   REAL NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_interp_status ON interpretations(status);
CREATE INDEX IF NOT EXISTS idx_interp_action ON interpretations(action);

-- Final label after combining interpretation with structural signals.
CREATE TABLE IF NOT EXISTS classifications (
    file_id     INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
    label       TEXT NOT NULL,      -- CURRENT|RELEVANT_STALE|DUPLICATE|UNRELATED|UNKNOWN|SECRET_RISK
    score       REAL NOT NULL DEFAULT 0,
    best_repo   INTEGER REFERENCES repos(id) ON DELETE SET NULL,
    signals     TEXT NOT NULL DEFAULT '{}',   -- json breakdown
    decided_at  REAL NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_class_label ON classifications(label);

-- "This old file is superseded by that newer canonical one."
CREATE TABLE IF NOT EXISTS supersedes (
    old_id     INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    new_id     INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    similarity REAL    NOT NULL,
    reason     TEXT    NOT NULL DEFAULT '',
    PRIMARY KEY (old_id, new_id)
);

-- ---------------------------------------------------------------- notes ----
CREATE TABLE IF NOT EXISTS notes (
    id         INTEGER PRIMARY KEY,
    path       TEXT NOT NULL UNIQUE,
    title      TEXT NOT NULL,
    body       TEXT NOT NULL,
    tags       TEXT NOT NULL DEFAULT '[]',
    created_at REAL NOT NULL,
    updated_at REAL NOT NULL
);
CREATE TABLE IF NOT EXISTS note_links (
    src_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    target TEXT    NOT NULL,          -- wikilink text
    dst_id INTEGER REFERENCES notes(id) ON DELETE SET NULL,
    PRIMARY KEY (src_id, target)
);

-- --------------------------------------------------- plans and journalling --
CREATE TABLE IF NOT EXISTS plans (
    id         INTEGER PRIMARY KEY,
    created_at REAL NOT NULL,
    note       TEXT NOT NULL DEFAULT '',
    status     TEXT NOT NULL DEFAULT 'open'  -- open|applied|rolled_back
);

CREATE TABLE IF NOT EXISTS actions (
    id          INTEGER PRIMARY KEY,
    plan_id     INTEGER NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    file_id     INTEGER REFERENCES files(id) ON DELETE SET NULL,
    kind        TEXT NOT NULL,        -- move|quarantine|link|delete_symlink
    src         TEXT NOT NULL,
    dest        TEXT NOT NULL,
    category    TEXT NOT NULL DEFAULT '',
    reason      TEXT NOT NULL DEFAULT '',
    confidence  REAL NOT NULL DEFAULT 0,
    -- pending: awaiting review. approved: user said yes. auto: high confidence.
    state       TEXT NOT NULL DEFAULT 'pending',
    applied_at  REAL,
    error       TEXT
);
CREATE INDEX IF NOT EXISTS idx_actions_plan  ON actions(plan_id);
CREATE INDEX IF NOT EXISTS idx_actions_state ON actions(state);

-- Append-only record of what actually happened on disk, for exact rollback.
CREATE TABLE IF NOT EXISTS journal (
    id        INTEGER PRIMARY KEY,
    action_id INTEGER NOT NULL REFERENCES actions(id) ON DELETE CASCADE,
    op        TEXT    NOT NULL,   -- moved|symlinked|created_dir
    payload   TEXT    NOT NULL,   -- json: {"from":...,"to":...}
    ts        REAL    NOT NULL,
    undone    INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_journal_action ON journal(action_id);

-- Accept/reject decisions, used to bias future confidence.
CREATE TABLE IF NOT EXISTS feedback (
    id         INTEGER PRIMARY KEY,
    file_id    INTEGER REFERENCES files(id) ON DELETE CASCADE,
    predicted  TEXT NOT NULL,
    chosen     TEXT NOT NULL,
    ts         REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

pub fn open(path: &Path) -> Result<Db> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(SCHEMA)?;
    Ok(Arc::new(Mutex::new(conn)))
}

pub fn now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

// ------------------------------------------------------------- vectors -----

/// Serialise an already-normalised vector to a blob.
pub fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out
}

pub fn blob_to_vec(b: &[u8]) -> Vec<f32> {
    b.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Scale to unit length so cosine similarity reduces to a dot product.
pub fn normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

// --------------------------------------------------------------- meta ------

pub fn meta_get(conn: &Connection, key: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| {
            r.get::<_, String>(0)
        })
        .optional()?)
}

pub fn meta_set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO meta(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

// -------------------------------------------------------------- stats ------

/// Pipeline counters.
///
/// Each field counts what its name says. In particular `interpreted` is the
/// number of rows a model actually produced, not the number of files that
/// reached stage 3 -- `skip_unreadable` promotes binaries and oversized files
/// straight to stage 3 without reading them, and counting those as interpreted
/// overstated progress by six figures.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Stats {
    pub files_total: i64,
    pub files_present: i64,
    pub scanned: i64,
    /// Files whose bytes were actually read and hashed.
    pub extracted: i64,
    /// Files that yielded usable text.
    pub with_text: i64,
    /// Files that have at least one embedded chunk.
    pub embedded: i64,
    /// Files a model has read and judged.
    pub interpreted: i64,
    /// Files deliberately not read (binary, media, oversized).
    pub skipped: i64,
    pub repos: i64,
    pub notes: i64,
    pub pending_actions: i64,
    pub duplicate_groups: i64,
    pub loose_files: i64,
    pub bytes_loose: i64,
}

impl Stats {
    /// Files that are candidates for interpretation at all.
    pub fn eligible(&self) -> i64 {
        (self.files_present - self.skipped).max(0)
    }
}

pub fn stats(conn: &Connection) -> Result<Stats> {
    let one = |sql: &str| -> Result<i64> {
        Ok(conn.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap_or(0))
    };
    Ok(Stats {
        files_total: one("SELECT COUNT(*) FROM files")?,
        files_present: one("SELECT COUNT(*) FROM files WHERE present = 1")?,
        scanned: one("SELECT COUNT(*) FROM files WHERE present = 1")?,
        extracted: one(
            "SELECT COUNT(*) FROM files
              WHERE present = 1 AND sha256 IS NOT NULL",
        )?,
        with_text: one(
            "SELECT COUNT(*) FROM files f JOIN file_text t ON t.file_id = f.id
              WHERE f.present = 1",
        )?,
        embedded: one(
            "SELECT COUNT(DISTINCT c.file_id) FROM chunks c JOIN files f ON f.id = c.file_id
              WHERE f.present = 1 AND c.embedding IS NOT NULL",
        )?,
        interpreted: one(
            "SELECT COUNT(*) FROM interpretations i JOIN files f ON f.id = i.file_id
              WHERE f.present = 1",
        )?,
        skipped: one(
            "SELECT COUNT(*) FROM files
              WHERE present = 1 AND extract_err LIKE 'skipped%'",
        )?,
        repos: one("SELECT COUNT(*) FROM repos")?,
        notes: one("SELECT COUNT(*) FROM notes")?,
        pending_actions: one("SELECT COUNT(*) FROM actions WHERE state = 'pending'")?,
        duplicate_groups: one(
            "SELECT COUNT(*) FROM (SELECT sha256 FROM files
              WHERE present = 1 AND sha256 IS NOT NULL AND size > 0
              GROUP BY sha256 HAVING COUNT(*) > 1)",
        )?,
        loose_files: one("SELECT COUNT(*) FROM files WHERE present = 1 AND repo_id IS NULL")?,
        bytes_loose: one(
            "SELECT COALESCE(SUM(size),0) FROM files WHERE present = 1 AND repo_id IS NULL",
        )?,
    })
}
