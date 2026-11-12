//! Durable source backups and explicit review snapshots for filesystem actions.
use crate::compare;
use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub sha256: String,
    pub source_root: PathBuf,
    pub dest_root: PathBuf,
    pub mode: u32,
    pub reference: bool,
    pub reason: String,
}

pub fn schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS action_snapshots (
            action_id INTEGER PRIMARY KEY REFERENCES actions(id), snapshot TEXT NOT NULL,
            backup BLOB NOT NULL, phase TEXT NOT NULL DEFAULT 'reviewed'
         );",
    )?;
    Ok(())
}

pub fn capture(
    conn: &Connection,
    id: i64,
    src: &Path,
    source_root: &Path,
    dest_root: &Path,
    reference: bool,
    reason: &str,
) -> Result<()> {
    if !src.starts_with(source_root) || source_root == src {
        bail!("Source outside configured root");
    }
    if src
        .components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
    {
        bail!("Hidden application/configuration paths are not cleanup candidates");
    }
    compare::checked_path(source_root, false)?;
    compare::checked_path(dest_root, true)?;
    if compare::protected_tree(src).is_some() {
        bail!("Protected source tree");
    }
    let body = compare::readable(src, 1024 * 1024)?;
    let md = std::fs::metadata(src)?;
    let snapshot = Snapshot {
        sha256: compare::hash(body.as_bytes()),
        source_root: source_root.into(),
        dest_root: dest_root.into(),
        mode: md.permissions().mode(),
        reference,
        reason: reason.into(),
    };
    conn.execute(
        "INSERT INTO action_snapshots(action_id,snapshot,backup) VALUES (?1,?2,?3)",
        params![id, serde_json::to_string(&snapshot)?, body.as_bytes()],
    )?;
    Ok(())
}

pub fn load(conn: &Connection, id: i64) -> Result<Snapshot> {
    let json: String = conn
        .query_row(
            "SELECT snapshot FROM action_snapshots WHERE action_id=?1",
            params![id],
            |r| r.get(0),
        )
        .context("Legacy action has no safe review snapshot; rescan and create a new proposal")?;
    Ok(serde_json::from_str(&json)?)
}

fn manifest_path(dest: &Path) -> PathBuf {
    let mut name = dest.as_os_str().to_os_string();
    name.push(".reference.json");
    name.into()
}

fn validate(src: &Path, dest: &Path, snap: &Snapshot) -> Result<()> {
    if !src.starts_with(&snap.source_root)
        || src == snap.source_root
        || !dest.starts_with(&snap.dest_root)
        || dest == snap.dest_root
        || compare::secret_path(src)
        || compare::secret_path(dest)
    {
        bail!("Source/destination outside reviewed boundaries");
    }
    compare::checked_path(src, false)?;
    compare::checked_path(dest, true)?;
    if compare::protected_tree(src).is_some() {
        bail!("Source is now protected by Git metadata");
    }
    let body = compare::readable(src, 1024 * 1024)?;
    if compare::hash(body.as_bytes()) != snap.sha256
        || std::fs::metadata(src)?.permissions().mode() != snap.mode
    {
        bail!("Source content or permissions changed since review");
    }
    if std::fs::symlink_metadata(dest).is_ok() {
        bail!("Destination collision; re-review");
    }
    if snap.reference && std::fs::symlink_metadata(manifest_path(dest)).is_ok() {
        bail!("Reference manifest already exists");
    }
    Ok(())
}

/// Always exercise the copy path, including across filesystems. Publication
/// uses hard_link in the destination directory: unlike rename it cannot replace.
pub fn copy_new(src: &Path, dest: &Path, expected: &str, mode: u32) -> Result<()> {
    compare::checked_path(src, false)?;
    compare::checked_path(dest, true)?;
    let parent = dest.parent().context("Missing destination parent")?;
    std::fs::create_dir_all(parent)?;
    compare::checked_path(parent, false)?;
    let mut temp_name = dest.as_os_str().to_os_string();
    temp_name.push(format!(
        ".librarian-{}-{}.partial",
        std::process::id(),
        crate::db::now().to_bits()
    ));
    let temp = PathBuf::from(temp_name);
    let result = (|| -> Result<()> {
        let mut out = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)?;
        let mut input = File::open(src)?;
        std::io::copy(&mut input, &mut out)?;
        if crate::extract::sha256_file(&temp).as_deref() != Some(expected) {
            bail!("Copy verification failed");
        }
        out.set_permissions(std::fs::Permissions::from_mode(mode & 0o777))?;
        let md = input.metadata()?;
        if let Ok(modified) = md.modified() {
            out.set_times(std::fs::FileTimes::new().set_modified(modified))?;
        }
        out.sync_all()?;
        compare::checked_path(dest, true)?;
        std::fs::hard_link(&temp, dest)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    let _ = std::fs::remove_file(&temp);
    result
}

pub fn apply(conn: &Connection, id: i64, src: &Path, dest: &Path) -> Result<()> {
    let snap = load(conn, id)?;
    validate(src, dest, &snap)?;
    conn.execute(
        "UPDATE action_snapshots SET phase='prepared' WHERE action_id=?1",
        params![id],
    )?;
    conn.execute(
        "INSERT INTO journal(action_id,op,payload,ts) VALUES (?1,'prepared',?2,?3)",
        params![
            id,
            serde_json::json!({"from":src,"to":dest,"sha256":snap.sha256}).to_string(),
            crate::db::now()
        ],
    )?;
    copy_new(src, dest, &snap.sha256, snap.mode)?;
    conn.execute(
        "UPDATE action_snapshots SET phase='published' WHERE action_id=?1",
        params![id],
    )?;
    if snap.reference {
        let manifest = serde_json::json!({
            "original_path": src, "archive_path": dest, "sha256":snap.sha256, "mode":snap.mode,
            "archived_at": chrono::Utc::now().to_rfc3339(), "reason":snap.reason,
            "replacement": null, "policy":"Reference only; do not execute without modern review",
        });
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(manifest_path(dest))?;
        serde_json::to_writer_pretty(&mut file, &manifest)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
    }
    // Source removal is last and only after both durable copies and provenance.
    compare::checked_path(src, false)?;
    if crate::extract::sha256_file(src).as_deref() != Some(&snap.sha256) {
        bail!("Source changed during copying; source and reference retained for recovery");
    }
    std::fs::remove_file(src)?;
    File::open(src.parent().context("Missing source parent")?)?.sync_all()?;
    conn.execute(
        "UPDATE action_snapshots SET phase='applied' WHERE action_id=?1",
        params![id],
    )?;
    conn.execute(
        "UPDATE journal SET op='completed' WHERE action_id=?1 AND undone=0",
        params![id],
    )?;
    Ok(())
}

/// Recover published or completed operations; never replace later edits.
pub fn undo(conn: &Connection, id: i64, src: &Path, dest: &Path) -> Result<()> {
    let snap = load(conn, id)?;
    if !src.starts_with(&snap.source_root) || !dest.starts_with(&snap.dest_root) {
        bail!("Journal paths outside reviewed boundaries");
    }
    compare::checked_path(src, true)?;
    compare::checked_path(dest, true)?;
    if compare::protected_tree(src).is_some() {
        bail!("Source is now a protected tree");
    }
    let phase: String = conn.query_row(
        "SELECT phase FROM action_snapshots WHERE action_id=?1",
        params![id],
        |r| r.get(0),
    )?;
    if !matches!(phase.as_str(), "published" | "applied") {
        bail!(
            "Interrupted before publication was recorded; inspect both paths and retained backup"
        );
    }
    if crate::extract::sha256_file(dest).as_deref() != Some(&snap.sha256)
        || std::fs::metadata(dest)?.permissions().mode() & 0o777 != snap.mode & 0o777
    {
        bail!("Archive content/permissions changed or missing; original backup retained, no overwrite");
    }
    if std::fs::symlink_metadata(src).is_ok() {
        if phase != "published" || crate::extract::sha256_file(src).as_deref() != Some(&snap.sha256)
        {
            bail!("Original path exists; undo will not overwrite it");
        }
    } else {
        copy_new(dest, src, &snap.sha256, snap.mode)?;
    }
    if snap.reference {
        let path = manifest_path(dest);
        if path.exists() {
            compare::checked_path(&path, false)?;
            let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
            if value["sha256"].as_str() != Some(&snap.sha256)
                || value["original_path"].as_str() != src.to_str()
            {
                bail!("Reference manifest changed; restored source retained, review required");
            }
            std::fs::remove_file(path)?;
        }
    }
    std::fs::remove_file(dest)?;
    File::open(dest.parent().context("Missing archive parent")?)?.sync_all()?;
    conn.execute(
        "UPDATE action_snapshots SET phase='undone' WHERE action_id=?1",
        params![id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{actions, db};
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        dir: PathBuf,
        src: PathBuf,
        dest: PathBuf,
        conn: crate::db::Db,
        id: i64,
        plan: i64,
    }
    impl Fixture {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!(
                "librarian-safety-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let home = dir.join("home");
            let repo = dir.join("repos/project");
            std::fs::create_dir_all(&home).unwrap();
            std::fs::create_dir_all(repo.join(".git")).unwrap();
            let src = home.join("launcher.sh");
            let dest = repo.join("archive/reference/launcher.sh");
            std::fs::write(&src, "# Launch a useful historical service\nprintf safe\n").unwrap();
            std::fs::set_permissions(&src, std::fs::Permissions::from_mode(0o751)).unwrap();
            let finding = compare::Finding {
                path: src.clone(),
                root: home,
                sha256: crate::extract::sha256_file(&src).unwrap(),
                modified: 1.0,
                size: 49,
                protected_tree: None,
                kind: "unmatched".into(),
                disposition: "review".into(),
                ideas: vec![],
                cautions: vec![],
                matches: vec![],
            };
            let conn = db::open(&dir.join("state/catalog.db")).unwrap();
            let (id, plan) = {
                let mut c = conn.lock().unwrap();
                let id = actions::queue_reference(
                    &mut c,
                    &finding,
                    &repo,
                    &dest,
                    "Preserve useful launcher idea",
                )
                .unwrap();
                let plan = c
                    .query_row(
                        "SELECT plan_id FROM actions WHERE id=?1",
                        params![id],
                        |r| r.get(0),
                    )
                    .unwrap();
                (id, plan)
            };
            Self {
                dir,
                src,
                dest,
                conn,
                id,
                plan,
            }
        }
        fn apply(&self) -> crate::actions::ApplyReport {
            let mut c = self.conn.lock().unwrap();
            actions::decide(&c, self.id, true).unwrap();
            actions::apply(&mut c, self.plan).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }
    #[test]
    fn reference_is_pending_until_review_then_restores_exact_bytes_and_mode() {
        let f = Fixture::new();
        let original = std::fs::read(&f.src).unwrap();
        {
            let mut c = f.conn.lock().unwrap();
            assert_eq!(actions::apply(&mut c, f.plan).unwrap().applied, 0);
        }
        assert!(f.src.exists());
        assert!(!f.dest.exists());
        assert_eq!(f.apply().applied, 1);
        assert_eq!(std::fs::read(&f.dest).unwrap(), original);
        assert_eq!(
            std::fs::metadata(&f.dest).unwrap().permissions().mode() & 0o777,
            0o751
        );
        assert!(manifest_path(&f.dest).exists());
        let mut c = f.conn.lock().unwrap();
        let backup: Vec<u8> = c
            .query_row(
                "SELECT backup FROM action_snapshots WHERE action_id=?1",
                params![f.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(backup, original);
        assert_eq!(actions::undo(&mut c, f.plan).unwrap().applied, 1);
        assert_eq!(std::fs::read(&f.src).unwrap(), original);
        assert!(!f.dest.exists());
        assert!(!manifest_path(&f.dest).exists());
    }
    #[test]
    fn changed_source_and_destination_collision_leave_originals_untouched() {
        let f = Fixture::new();
        std::fs::write(&f.src, "a later edit").unwrap();
        assert_eq!(f.apply().failed, 1);
        assert!(!f.dest.exists());
        let g = Fixture::new();
        std::fs::create_dir_all(g.dest.parent().unwrap()).unwrap();
        std::fs::write(&g.dest, "existing").unwrap();
        assert_eq!(g.apply().failed, 1);
        assert!(g.src.exists());
        assert_eq!(std::fs::read_to_string(&g.dest).unwrap(), "existing");
    }
    #[test]
    fn symlink_escape_is_rejected_before_writing() {
        let f = Fixture::new();
        let archive = f.dest.parent().unwrap().parent().unwrap();
        let outside = f.dir.join("outside");
        std::fs::create_dir_all(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, archive).unwrap();
        assert_eq!(f.apply().failed, 1);
        assert!(f.src.exists());
        assert!(std::fs::read_dir(outside).unwrap().next().is_none());
    }
    #[test]
    fn undo_refuses_later_archive_or_source_edits() {
        let f = Fixture::new();
        assert_eq!(f.apply().applied, 1);
        std::fs::write(&f.dest, "archive edited").unwrap();
        assert_eq!(
            actions::undo(&mut f.conn.lock().unwrap(), f.plan)
                .unwrap()
                .failed,
            1
        );
        assert!(!f.src.exists());
        let g = Fixture::new();
        assert_eq!(g.apply().applied, 1);
        std::fs::write(&g.src, "new source").unwrap();
        assert_eq!(
            actions::undo(&mut g.conn.lock().unwrap(), g.plan)
                .unwrap()
                .failed,
            1
        );
        assert_eq!(std::fs::read_to_string(&g.src).unwrap(), "new source");
    }
    #[test]
    fn interrupted_published_copy_can_be_recovered_without_overwriting() {
        let f = Fixture::new();
        let c = f.conn.lock().unwrap();
        let snap = load(&c, f.id).unwrap();
        copy_new(&f.src, &f.dest, &snap.sha256, snap.mode).unwrap();
        c.execute(
            "UPDATE action_snapshots SET phase='published' WHERE action_id=?1",
            params![f.id],
        )
        .unwrap();
        undo(&c, f.id, &f.src, &f.dest).unwrap();
        assert!(f.src.exists());
        assert!(!f.dest.exists());
    }
    #[test]
    fn migration_resets_legacy_auto_approvals_and_is_idempotent() {
        let f = Fixture::new();
        {
            let c = f.conn.lock().unwrap();
            c.execute(
                "UPDATE actions SET state='approved' WHERE id=?1",
                params![f.id],
            )
            .unwrap();
            c.execute("DELETE FROM meta WHERE key='explicit_review_v1'", [])
                .unwrap();
        }
        let reopened = db::open(&f.dir.join("state/catalog.db")).unwrap();
        assert_eq!(
            reopened
                .lock()
                .unwrap()
                .query_row(
                    "SELECT state FROM actions WHERE id=?1",
                    params![f.id],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            "pending"
        );
    }
}
