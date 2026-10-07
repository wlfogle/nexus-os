//! Locating executables on `$PATH`.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The directories on `$PATH`, in order, deduplicated.
pub fn path_dirs() -> Vec<PathBuf> {
    let raw = match env::var_os("PATH") {
        Some(v) => v,
        None => return Vec::new(),
    };
    let mut seen = Vec::new();
    for dir in env::split_paths(&raw) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        if !seen.contains(&dir) {
            seen.push(dir);
        }
    }
    seen
}

/// True if `path` resolves to a regular file with any execute bit set.
pub fn is_executable(path: &Path) -> bool {
    // `metadata` follows symlinks, which is what we want: a dangling symlink
    // on $PATH is not a runnable program.
    match fs::metadata(path) {
        Ok(meta) => meta.is_file() && meta.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

/// Resolve a bare program name to its first match on `$PATH`.
pub fn lookup(name: &str) -> Option<PathBuf> {
    if name.contains('/') {
        let p = PathBuf::from(name);
        return is_executable(&p).then_some(p);
    }
    for dir in path_dirs() {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// Every executable reachable on `$PATH`, mapped name -> first winning path.
/// Earlier `$PATH` entries shadow later ones, matching shell behaviour.
pub fn all_executables() -> BTreeMap<String, PathBuf> {
    let mut found: BTreeMap<String, PathBuf> = BTreeMap::new();
    for dir in path_dirs() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if found.contains_key(name) {
                continue; // shadowed by an earlier $PATH directory
            }
            if is_executable(&path) {
                found.insert(name.to_string(), path);
            }
        }
    }
    found
}

/// Exact names that are never menu material.
const NOISE_NAMES: &[&str] = &[
    // coreutils and friends
    "[", "arch", "awk", "b2sum", "base32", "base64", "basename", "basenc", "cat", "chcon", "chgrp",
    "chmod", "chown", "chroot", "cksum", "comm", "cp", "csplit", "cut", "date", "dd", "df", "dir",
    "dircolors", "dirname", "du", "echo", "egrep", "env", "expand", "expr", "factor", "false",
    "fgrep", "fmt", "fold", "grep", "groups", "head", "hostid", "id", "install", "join", "kill",
    "link", "ln", "logname", "ls", "md5sum", "mkdir", "mkfifo", "mknod", "mktemp", "mv", "nice",
    "nl", "nohup", "nproc", "numfmt", "od", "paste", "pathchk", "pinky", "pr", "printenv",
    "printf", "ptx", "pwd", "readlink", "realpath", "rm", "rmdir", "runcon", "sed", "seq",
    "sha1sum", "sha224sum", "sha256sum", "sha384sum", "sha512sum", "shred", "shuf", "sleep",
    "sort", "split", "stat", "stdbuf", "stty", "sum", "sync", "tac", "tail", "tee", "test",
    "timeout", "touch", "tr", "true", "truncate", "tsort", "tty", "uname", "unexpand", "uniq",
    "unlink", "users", "vdir", "wc", "who", "whoami", "yes",
    // shells and interpreters
    "bash", "dash", "fish", "ksh", "sh", "tcsh", "zsh", "csh", "rbash", "busybox",
    "perl", "php", "python2", "python3", "ruby", "lua", "tclsh", "wish", "node", "deno",
    // build/dev plumbing
    "ar", "as", "cc", "c++", "cpp", "g++", "gcc", "ld", "make", "nm", "objcopy", "objdump",
    "ranlib", "readelf", "size", "strings", "strip", "ldd", "ldconfig", "pkg-config", "patch",
    "diff", "diff3", "cmp", "m4", "flex", "bison", "yacc", "autoconf", "automake", "libtool",
    "cargo", "rustc", "go", "javac", "java",
    // archive/compression
    "bzip2", "bunzip2", "gzip", "gunzip", "xz", "unxz", "zstd", "tar", "cpio", "zip", "unzip",
    "7z", "7za", "lz4", "lzma",
    // networking / misc CLI
    "curl", "wget", "ssh", "scp", "sftp", "rsync", "ping", "dig", "host", "nslookup", "ip",
    "ifconfig", "netstat", "ss", "traceroute", "nc", "openssl", "gpg", "gpg2", "git", "svn", "hg",
    // session / system plumbing
    "systemctl", "journalctl", "loginctl", "udevadm", "modprobe", "insmod", "rmmod", "lsmod",
    "dmesg", "mount", "umount", "swapon", "swapoff", "fsck", "mkfs", "sysctl", "dbus-send",
    "dbus-launch", "xdg-open", "xdg-mime", "xdg-settings", "update-desktop-database",
    "gtk-launch", "gio", "gsettings", "dconf", "pkexec", "sudo", "su", "doas",
];

/// Substrings and affixes that reliably indicate a non-interactive helper.
const NOISE_PREFIXES: &[&str] = &[
    "lib", "x86_64-", "aarch64-", "i686-", "arm-", "systemd-", "dpkg-", "apt-", "deb-", "update-",
    "gdbus-", "gvfs", "gresource", "glib-", "gtk-", "gdk-", "qt5", "qt6", "kf5", "kf6",
];

const NOISE_SUFFIXES: &[&str] = &[
    "-config", "config", "-devel", "-dev", ".sh", ".py", ".pl", ".rb", "ctl", "d", "stat",
    "-cli", "-server", "-daemon", "-agent", "-helper", "-wrapper", "-shim",
];

/// Heuristic: would this binary almost certainly be junk in an application menu?
///
/// This is best-effort and only gates the opt-in `orphans` report; it never
/// affects the catalog-driven path. Pass `--no-filter` to see everything.
pub fn looks_like_noise(name: &str) -> bool {
    if NOISE_NAMES.contains(&name) {
        return true;
    }
    // Versioned binaries: python3.11, gcc-12, llvm-ar-15, foo.so.1
    if name.contains(".so") {
        return true;
    }
    if name
        .rsplit(['-', '.'])
        .next()
        .is_some_and(|tail| !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()))
    {
        return true;
    }
    if NOISE_PREFIXES.iter().any(|p| name.starts_with(p)) {
        return true;
    }
    // `d` and `ctl` suffixes only count for names long enough to be meaningful.
    if NOISE_SUFFIXES
        .iter()
        .any(|s| name.len() > s.len() + 2 && name.ends_with(s))
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_obvious_noise() {
        for name in ["ls", "awk", "gcc", "python3.11", "libfoo", "libexec-thing"] {
            assert!(looks_like_noise(name), "{name} should be filtered");
        }
    }

    #[test]
    fn keeps_real_applications() {
        for name in ["htop", "gkrellm", "ncdu", "xclock", "ranger", "cmus"] {
            assert!(!looks_like_noise(name), "{name} should NOT be filtered");
        }
    }

    #[test]
    fn versioned_names_are_noise() {
        assert!(looks_like_noise("gcc-12"));
        assert!(looks_like_noise("llvm-ar-15"));
    }

    #[test]
    fn path_lookup_finds_sh() {
        // /bin/sh exists on every system this tool targets.
        assert!(lookup("sh").is_some(), "expected to resolve `sh` on $PATH");
    }

    #[test]
    fn absolute_paths_are_checked_directly() {
        assert!(lookup("/definitely/not/here/xyz").is_none());
    }
}
