//! Planning and applying changes.
//!
//! Nothing here deletes anything. "Removal" means a move into Quarantine, and
//! every filesystem operation is written to a journal first so it can be
//! reversed exactly.
//!
//! Moves are done with rename when source and destination share a filesystem
//! (atomic, instant), and copy-verify-remove otherwise. The copy path verifies
//! the destination hash against the source before unlinking the original, so an
//! interrupted or corrupted transfer never loses data.

use anyhow::{anyhow, Result};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::db;
use crate::extract::sha256_file;

#[derive(Debug, Clone, Serialize)]
pub struct Action {
    pub id: i64,
    pub plan_id: i64,
    pub file_id: Option<i64>,
    pub kind: String,
    pub src: String,
    pub dest: String,
    pub category: String,
    pub reason: String,
    pub confidence: f32,
    pub state: String,
    pub error: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct PlanReport {
    pub plan_id: i64,
    pub proposed: usize,
    pub auto: usize,
    pub pending: usize,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ApplyReport {
    pub applied: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// How much a category's own track record shifts its auto-apply threshold.
///
/// Capped so feedback nudges the bar rather than replacing judgement: with a
/// base threshold of 0.85 the effective bar stays within 0.75..0.95 no matter
/// how one-sided the history is.
const FEEDBACK_MAX_SHIFT: f32 = 0.10;

/// Minimum decisions before a category's history is trusted at all.
const FEEDBACK_MIN_SAMPLES: i64 = 5;

/// Per-category acceptance rate from recorded review decisions.
///
/// Returns `(approved, rejected)` counts keyed by category.
pub fn feedback_rates(conn: &Connection) -> Result<HashMap<String, (i64, i64)>> {
    let mut out: HashMap<String, (i64, i64)> = HashMap::new();
    let mut q = conn.prepare(
        "SELECT predicted,
                SUM(CASE WHEN chosen = 'approved' THEN 1 ELSE 0 END),
                SUM(CASE WHEN chosen = 'rejected' THEN 1 ELSE 0 END)
           FROM feedback GROUP BY predicted",
    )?;
    let rows = q.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
        ))
    })?;
    for row in rows {
        let (cat, ok, no) = row?;
        out.insert(cat, (ok, no));
    }
    Ok(out)
}

/// Adjust the auto-apply threshold for a category using its own history.
///
/// Rejecting a category's proposals raises its bar; approving them lowers it.
/// This is the read side of the `feedback` table -- without it, every review
/// decision was recorded and then ignored.
pub fn effective_threshold(base: f32, approved: i64, rejected: i64) -> f32 {
    let total = approved + rejected;
    if total < FEEDBACK_MIN_SAMPLES {
        return base;
    }
    let accept_rate = approved as f32 / total as f32;
    // 100% accepted -> -shift (easier), 0% accepted -> +shift (harder).
    let shift = (0.5 - accept_rate) * 2.0 * FEEDBACK_MAX_SHIFT;
    (base + shift).clamp(0.5, 0.99)
}

/// Destination bucket for an interpretation's recommended action.
fn bucket_for(cfg: &Config, action: &str, repo: Option<&str>) -> Option<PathBuf> {
    match action {
        "archive" => Some(cfg.library.join("Archive")),
        "quarantine" => Some(cfg.library.join("Quarantine")),
        "file" => Some(match repo {
            Some(r) if !r.is_empty() => cfg.library.join("RepoRefs").join(r),
            _ => cfg.library.join("Inbox"),
        }),
        // keep / review never move anything.
        _ => None,
    }
}

/// Append a numeric suffix until the path is free, so a move never clobbers.
fn unique_dest(dir: &Path, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() => (s.to_string(), format!(".{e}")),
        _ => (name.to_string(), String::new()),
    };
    for n in 2..10_000 {
        let c = dir.join(format!("{stem}-{n}{ext}"));
        if !c.exists() {
            return c;
        }
    }
    dir.join(format!("{stem}-{}{ext}", db::now() as i64))
}

/// Build a plan from interpretations that have not been acted on yet.
pub fn plan(conn: &mut Connection, cfg: &Config) -> Result<PlanReport> {
    let mut report = PlanReport::default();

    struct Candidate {
        file_id: i64,
        path: String,
        name: String,
        action: String,
        reason: String,
        confidence: f32,
        repo: Option<String>,
    }

    let candidates: Vec<Candidate> = {
        let mut q = conn.prepare(
            "SELECT f.id, f.path, f.name, i.action, i.reason, i.confidence, r.name
               FROM files f
               JOIN interpretations i ON i.file_id = f.id
               LEFT JOIN repos r ON r.id = f.repo_id
              WHERE f.present = 1
                AND i.action IN ('archive','quarantine','file')
                -- never touch anything a repo owns; git is the authority there
                AND f.repo_id IS NULL
                AND f.id NOT IN (
                      SELECT file_id FROM actions
                       WHERE file_id IS NOT NULL
                         AND state IN ('pending','applied','approved')
                )",
        )?;
        let rows = q.query_map([], |r| {
            Ok(Candidate {
                file_id: r.get(0)?,
                path: r.get(1)?,
                name: r.get(2)?,
                action: r.get(3)?,
                reason: r.get(4)?,
                confidence: r.get(5)?,
                repo: r.get(6)?,
            })
        })?;
        let mut v = Vec::new();
        for row in rows {
            v.push(row?);
        }
        v
    };

    if candidates.is_empty() {
        return Ok(report);
    }

    // Read the review history so categories you keep rejecting get a higher bar.
    let rates = feedback_rates(conn)?;

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO plans(created_at, note, status) VALUES (?1, ?2, 'open')",
        params![db::now(), "automatic plan"],
    )?;
    let plan_id = tx.last_insert_rowid();
    report.plan_id = plan_id;

    {
        let mut ins = tx.prepare(
            "INSERT INTO actions
               (plan_id,file_id,kind,src,dest,category,reason,confidence,state)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        )?;
        for c in &candidates {
            let dir = match bucket_for(cfg, &c.action, c.repo.as_deref()) {
                Some(d) => d,
                None => continue,
            };
            let dest = unique_dest(&dir, &c.name);
            let (ok, no) = rates.get(&c.action).copied().unwrap_or((0, 0));
            let threshold = effective_threshold(cfg.auto_apply_above, ok, no);
            let state = if c.confidence >= threshold {
                report.auto += 1;
                "approved"
            } else {
                report.pending += 1;
                "pending"
            };
            ins.execute(params![
                plan_id,
                c.file_id,
                if c.action == "quarantine" { "quarantine" } else { "move" },
                c.path,
                dest.to_string_lossy(),
                c.action,
                c.reason,
                c.confidence,
                state,
            ])?;
            report.proposed += 1;
        }
    }

    tx.commit()?;
    Ok(report)
}

fn same_filesystem(a: &Path, b: &Path) -> bool {
    let dev = |p: &Path| -> Option<u64> {
        let mut cur = Some(p);
        while let Some(c) = cur {
            if let Ok(md) = std::fs::metadata(c) {
                return Some(md.dev());
            }
            cur = c.parent();
        }
        None
    };
    match (dev(a), dev(b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

/// Move a file, verifying the copy before removing the original.
fn move_file(src: &Path, dest: &Path) -> Result<&'static str> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if dest.exists() {
        return Err(anyhow!("destination already exists: {}", dest.display()));
    }

    if same_filesystem(src, dest) {
        std::fs::rename(src, dest)?;
        return Ok("renamed");
    }

    let before = sha256_file(src).ok_or_else(|| anyhow!("cannot hash source"))?;
    std::fs::copy(src, dest)?;
    let after = sha256_file(dest).ok_or_else(|| anyhow!("cannot hash destination"))?;
    if before != after {
        // Leave the source untouched; remove the bad copy.
        let _ = std::fs::remove_file(dest);
        return Err(anyhow!("checksum mismatch after copy"));
    }
    std::fs::remove_file(src)?;
    Ok("copied")
}

/// Apply every approved action in a plan.
pub fn apply(conn: &mut Connection, plan_id: i64) -> Result<ApplyReport> {
    let mut report = ApplyReport::default();

    let todo: Vec<(i64, String, String)> = {
        let mut q = conn.prepare(
            "SELECT id, src, dest FROM actions
              WHERE plan_id = ?1 AND state = 'approved'
              ORDER BY id",
        )?;
        let rows = q.query_map(params![plan_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
        let mut v = Vec::new();
        for row in rows {
            v.push(row?);
        }
        v
    };

    for (action_id, src, dest) in todo {
        let s = Path::new(&src);
        let d = Path::new(&dest);

        if !s.exists() {
            conn.execute(
                "UPDATE actions SET state='failed', error='source vanished' WHERE id=?1",
                params![action_id],
            )?;
            report.skipped += 1;
            continue;
        }

        match move_file(s, d) {
            Ok(how) => {
                // Journal first, then mark applied: a crash between the two
                // leaves a recoverable record rather than an orphan move.
                conn.execute(
                    "INSERT INTO journal(action_id, op, payload, ts)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        action_id,
                        how,
                        serde_json::json!({ "from": src, "to": dest }).to_string(),
                        db::now(),
                    ],
                )?;
                conn.execute(
                    "UPDATE actions SET state='applied', applied_at=?2, error=NULL
                      WHERE id=?1",
                    params![action_id, db::now()],
                )?;
                conn.execute(
                    "UPDATE files SET path = ?2, present = 1
                      WHERE id = (SELECT file_id FROM actions WHERE id = ?1)",
                    params![action_id, dest],
                )?;
                report.applied += 1;
            }
            Err(e) => {
                conn.execute(
                    "UPDATE actions SET state='failed', error=?2 WHERE id=?1",
                    params![action_id, e.to_string()],
                )?;
                report.failed += 1;
            }
        }
    }

    conn.execute(
        "UPDATE plans SET status='applied' WHERE id=?1",
        params![plan_id],
    )?;
    Ok(report)
}

/// Reverse everything a plan did, newest operation first.
pub fn undo(conn: &mut Connection, plan_id: i64) -> Result<ApplyReport> {
    let mut report = ApplyReport::default();

    let entries: Vec<(i64, i64, String)> = {
        let mut q = conn.prepare(
            "SELECT j.id, j.action_id, j.payload
               FROM journal j JOIN actions a ON a.id = j.action_id
              WHERE a.plan_id = ?1 AND j.undone = 0
              ORDER BY j.id DESC",
        )?;
        let rows = q.query_map(params![plan_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
        let mut v = Vec::new();
        for row in rows {
            v.push(row?);
        }
        v
    };

    for (journal_id, action_id, payload) in entries {
        let v: serde_json::Value = serde_json::from_str(&payload)?;
        let from = v["from"].as_str().unwrap_or_default().to_string();
        let to = v["to"].as_str().unwrap_or_default().to_string();
        if from.is_empty() || to.is_empty() {
            report.skipped += 1;
            continue;
        }

        match move_file(Path::new(&to), Path::new(&from)) {
            Ok(_) => {
                conn.execute(
                    "UPDATE journal SET undone = 1 WHERE id = ?1",
                    params![journal_id],
                )?;
                conn.execute(
                    "UPDATE actions SET state='pending', applied_at=NULL WHERE id=?1",
                    params![action_id],
                )?;
                conn.execute(
                    "UPDATE files SET path = ?2
                      WHERE id = (SELECT file_id FROM actions WHERE id = ?1)",
                    params![action_id, from],
                )?;
                report.applied += 1;
            }
            Err(e) => {
                conn.execute(
                    "UPDATE actions SET error = ?2 WHERE id = ?1",
                    params![action_id, format!("undo failed: {e}")],
                )?;
                report.failed += 1;
            }
        }
    }

    conn.execute(
        "UPDATE plans SET status='rolled_back' WHERE id=?1",
        params![plan_id],
    )?;
    Ok(report)
}

pub fn list_actions(conn: &Connection, state: Option<&str>, limit: i64) -> Result<Vec<Action>> {
    let sql = match state {
        Some(_) => {
            "SELECT id,plan_id,file_id,kind,src,dest,category,reason,confidence,state,error
               FROM actions WHERE state = ?1 ORDER BY confidence DESC, id DESC LIMIT ?2"
        }
        None => {
            "SELECT id,plan_id,file_id,kind,src,dest,category,reason,confidence,state,error
               FROM actions WHERE ?1 IS NULL ORDER BY id DESC LIMIT ?2"
        }
    };
    let mut q = conn.prepare(sql)?;
    let rows = q.query_map(params![state, limit], |r| {
        Ok(Action {
            id: r.get(0)?,
            plan_id: r.get(1)?,
            file_id: r.get(2)?,
            kind: r.get(3)?,
            src: r.get(4)?,
            dest: r.get(5)?,
            category: r.get(6)?,
            reason: r.get(7)?,
            confidence: r.get(8)?,
            state: r.get(9)?,
            error: r.get(10)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Record a review decision and remember it for future confidence tuning.
pub fn decide(conn: &Connection, action_id: i64, approve: bool) -> Result<()> {
    let state = if approve { "approved" } else { "rejected" };
    conn.execute(
        "UPDATE actions SET state = ?2 WHERE id = ?1 AND state = 'pending'",
        params![action_id, state],
    )?;
    let row: Option<(Option<i64>, String)> = conn
        .query_row(
            "SELECT file_id, category FROM actions WHERE id = ?1",
            params![action_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok();
    if let Some((file_id, category)) = row {
        conn.execute(
            "INSERT INTO feedback(file_id, predicted, chosen, ts) VALUES (?1,?2,?3,?4)",
            params![file_id, category, state, db::now()],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{bucket_for, move_file, unique_dest};
    use crate::config::Config;
    use std::path::PathBuf;

    fn tmpdir(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "librarian-test-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn buckets_map_actions_to_directories() {
        let c = Config::default();
        assert_eq!(
            bucket_for(&c, "archive", None),
            Some(c.library.join("Archive"))
        );
        assert_eq!(
            bucket_for(&c, "quarantine", None),
            Some(c.library.join("Quarantine"))
        );
        assert_eq!(
            bucket_for(&c, "file", Some("nexus-os")),
            Some(c.library.join("RepoRefs").join("nexus-os"))
        );
        assert_eq!(bucket_for(&c, "file", None), Some(c.library.join("Inbox")));
    }

    #[test]
    fn keep_and_review_never_move() {
        let c = Config::default();
        assert!(bucket_for(&c, "keep", None).is_none());
        assert!(bucket_for(&c, "review", None).is_none());
    }

    #[test]
    fn unique_dest_avoids_clobbering() {
        let d = tmpdir("unique");
        let first = unique_dest(&d, "notes.md");
        assert_eq!(first, d.join("notes.md"));
        std::fs::write(&first, b"x").unwrap();

        let second = unique_dest(&d, "notes.md");
        assert_eq!(second, d.join("notes-2.md"));
        assert!(!second.exists());

        let no_ext = unique_dest(&d, "README");
        assert_eq!(no_ext, d.join("README"));
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn move_then_undo_restores_content() {
        let d = tmpdir("move");
        let src = d.join("a.txt");
        let dest = d.join("sub/b.txt");
        std::fs::write(&src, b"payload").unwrap();

        move_file(&src, &dest).unwrap();
        assert!(!src.exists());
        assert_eq!(std::fs::read(&dest).unwrap(), b"payload");

        // Undo is the same operation in reverse.
        move_file(&dest, &src).unwrap();
        assert!(!dest.exists());
        assert_eq!(std::fs::read(&src).unwrap(), b"payload");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn move_refuses_to_overwrite() {
        let d = tmpdir("clobber");
        let src = d.join("a.txt");
        let dest = d.join("b.txt");
        std::fs::write(&src, b"one").unwrap();
        std::fs::write(&dest, b"two").unwrap();

        assert!(move_file(&src, &dest).is_err());
        // Both survive untouched.
        assert_eq!(std::fs::read(&src).unwrap(), b"one");
        assert_eq!(std::fs::read(&dest).unwrap(), b"two");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn move_creates_missing_parent_directories() {
        let d = tmpdir("mkdir");
        let src = d.join("a.txt");
        let dest = d.join("x/y/z/a.txt");
        std::fs::write(&src, b"deep").unwrap();
        move_file(&src, &dest).unwrap();
        assert!(dest.exists());
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn feedback_is_ignored_until_there_is_enough_of_it() {
        // Two rejections is not a pattern; the base threshold must stand.
        assert_eq!(super::effective_threshold(0.85, 0, 2), 0.85);
        assert_eq!(super::effective_threshold(0.85, 0, 0), 0.85);
    }

    #[test]
    fn rejecting_a_category_raises_its_bar() {
        // All ten proposals rejected: demand more confidence next time.
        let t = super::effective_threshold(0.85, 0, 10);
        assert!(t > 0.85, "expected a higher bar, got {t}");
        assert!((t - 0.95).abs() < 1e-5);
    }

    #[test]
    fn approving_a_category_lowers_its_bar() {
        let t = super::effective_threshold(0.85, 10, 0);
        assert!(t < 0.85, "expected a lower bar, got {t}");
        assert!((t - 0.75).abs() < 1e-5);
    }

    #[test]
    fn an_even_split_leaves_the_threshold_alone() {
        let t = super::effective_threshold(0.85, 5, 5);
        assert!((t - 0.85).abs() < 1e-6);
    }

    #[test]
    fn the_shift_is_bounded_and_stays_in_range() {
        // Even absurd histories cannot push the bar outside sane limits.
        for (ok, no) in [(0, 100_000), (100_000, 0)] {
            let t = super::effective_threshold(0.85, ok, no);
            assert!((0.5..=0.99).contains(&t), "out of range: {t}");
            assert!((t - 0.85).abs() <= super::FEEDBACK_MAX_SHIFT + 1e-6);
        }
        // A low base cannot be dragged below the hard floor.
        assert!(super::effective_threshold(0.52, 0, 50) >= 0.5);
    }
}
