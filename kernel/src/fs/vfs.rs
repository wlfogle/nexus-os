//! NexusOS Virtual Filesystem (path) layer — Phase 6.1
//!
//! A thin, path-oriented front-end over the mounted FAT32 volume that lets
//! ring-3 callers list and read files in **subdirectories** (e.g. `/EFI/BOOT`,
//! `/boot`).  It normalizes a user-supplied absolute path and delegates the
//! actual directory traversal to the `fat` driver (`fatfs` resolves the
//! `/`-separated components).
//!
//! Paths are absolute and `/`-separated.  A leading `/` is optional and a bare
//! `/` (or the empty string) refers to the volume root.  Parent traversal
//! (`..`) is rejected — there is no escaping the volume root.
//!
//! Backing syscalls: SYS_FS_LIST_PATH(20), SYS_FS_READ_PATH(21).

use crate::fs::fat;

/// Normalize a user-supplied path into a root-relative form.
///
/// Trims surrounding `/` (so `/EFI/BOOT/` → `EFI/BOOT`) and rejects any
/// `..` component.  An empty result denotes the volume root.
fn normalize(path: &str) -> Result<&str, &'static str> {
    let rel = path.trim_matches('/');
    // Reject parent traversal: fatfs exposes ".." dir entries, and we do not
    // allow a caller to walk above the volume root.
    for comp in rel.split('/') {
        if comp == ".." {
            return Err("vfs: '..' not permitted");
        }
    }
    Ok(rel)
}

/// List the directory at `path`, writing newline-separated entry names into
/// `buf`.  Returns the number of bytes written.  A `path` of `/` (or empty)
/// lists the volume root.
pub fn list(path: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let rel = normalize(path)?;
    fat::list_path(rel, buf)
}

/// Read the file at `path` into `buf`.  Returns the number of bytes read.
/// A `path` that resolves to the root (a directory, not a file) is rejected.
pub fn read(path: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let rel = normalize(path)?;
    if rel.is_empty() {
        return Err("vfs: is a directory");
    }
    fat::read_path(rel, buf)
}
