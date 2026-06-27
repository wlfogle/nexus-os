#!/usr/bin/env python3
"""
nexus-consolidate.py — scan the system for NexusOS code/docs, separate the
good-but-scattered material from the stale/obsolete, check it against the
nexus-os git repo, and either stage it for the repo or send it to a
recoverable trash.

SAFETY MODEL (because: "DO NOT miss anything")
  * REPORT-ONLY by default. No file is touched unless you pass --apply.
  * "Trash" is never `rm`: obsolete files are MOVED to a timestamped quarantine
    dir with a manifest, fully restorable.
  * Third-party clones (git origin not github.com/wlfogle) are skipped wholesale
    so none of their code is pulled into your repo or trashed.
  * Anything unique and uncertain is classed REVIEW (kept), never trashed.
  * Secret-looking files are classed SECURE: reported, never moved, never trashed.

CLASSIFICATION (per file)
  SECURE             secret-looking (bitwarden export, *.pem, id_rsa, .env, ...)
  DUPLICATE_IN_REPO  byte-identical content already tracked in nexus-os  -> TRASH
  STALE_OLDER        same name in nexus-os, different content, this copy older -> TRASH
  NEWER_THAN_REPO    same name in nexus-os, different content, this copy newer -> REVIEW
  RELEVANT_MISSING   nexus-related, not in repo                          -> MOVE
  PERSONAL           recipes / clearly personal text                     -> MOVE (~/personal)
  JUNK               logs / scratch / temp                               -> TRASH
  REVIEW             unique, unclear relevance                           -> keep, list it

USAGE
  python3 nexus-consolidate.py                 # dry-run, writes a report
  python3 nexus-consolidate.py --apply         # execute the proposed actions
  python3 nexus-consolidate.py --roots ~/a ~/b --nexus ~/nexus-os
"""
from __future__ import annotations

import argparse
import csv
import datetime as dt
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
from collections import defaultdict

# ----------------------------------------------------------------------------- config
HOME = os.path.expanduser("~")

INTERESTING_EXT = {
    ".md", ".markdown", ".rst", ".txt", ".sh", ".bash", ".fish", ".zsh",
    ".py", ".rs", ".ts", ".tsx", ".js", ".jsx", ".go", ".c", ".h", ".cpp",
    ".hpp", ".kt", ".java", ".rb", ".pl", ".lua", ".toml", ".yaml", ".yml",
    ".json", ".conf", ".cfg", ".ini", ".service", ".timer", ".ld", ".s",
    ".asm", ".sql", ".env",
}
INTERESTING_NAMES = {"Dockerfile", "Makefile", "Caddyfile", ".env.example", "WARP.md", "AGENTS.md"}

# directories never worth scanning
EXCLUDE_DIRS = {
    ".git", "node_modules", "target", "build", "dist", ".cache", ".local",
    ".steam", ".var", ".venv", "venv", "__pycache__", "vendor", ".npm",
    ".cargo", ".rustup", "snap", ".mozilla", ".config", "Steam", "Games",
    ".gradle", ".android", "third-party", ".nexus-consolidate-trash",
    ".m2", ".pub-cache", ".nuget", "site-packages", ".pytest_cache",
    "Android", "go", "Faugus", "redroid-data", ".wine", "drive_c",
    "OSX-KVM", ".svelte-kit", ".next", "out", "coverage", "iso_root",
    "buildiso-chroots", "_consolidate",
}

SECRET_PATTERNS = [
    re.compile(r"bitwarden.*export", re.I),
    re.compile(r"\bid_rsa\b|\bid_ed25519\b"),
    re.compile(r"\.(pem|key|p12|pfx|keystore|jks)$", re.I),
    re.compile(r"(^|/)\.env$"),
    re.compile(r"(^|/)(secrets?|credentials?)\.(ya?ml|json|env|txt|conf)$", re.I),
]
SECRET_CONTENT = [
    re.compile(r"BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY"),
    re.compile(r"(api[_-]?key|secret|password|bearer)\s*[:=]\s*[A-Za-z0-9/+_\-]{16,}", re.I),
]

PERSONAL_PATTERNS = [
    re.compile(r"recipe|lasagna|burgers|fries|dinner|grocery", re.I),
]
JUNK_PATTERNS = [
    re.compile(r"\.log$", re.I),
    re.compile(r"(^|/)(continue|do this|net_changed|todo|scratch|tmp|temp)\.txt$", re.I),
    re.compile(r"-log\.txt$", re.I),
]

RELEVANCE = [
    "nexus", "nexusos", "tiamat", "bahamut", "jellyfin", "riven", "real-debrid",
    "realdebrid", "mediastack", "proxmox", "cockpit", "ollama", "jdownloader",
    "aria2", "homelab", "vaultwarden", "adguard", "wireguard", "habridge",
    "threadfin", "homarr", "sonarr", "radarr", "prowlarr", "qbittorrent",
    "firetv", "fire tv", "waydroid", "redroid", "vfio", "passthrough", "lxc",
    "ct-300", "ct300", "haos", "home assistant", "n8n", "caddy", "crowdsec",
    "rivenvfs", "torrentio", "metatube", "unified-guide", "epg", "hdhomerun",
    "kvm", "qemu", "pop!_os", "pop-os", "dietpi", "raspberry",
]

MAX_BYTES = 10 * 1024 * 1024  # skip hashing/reading files larger than 10 MB

# ----------------------------------------------------------------------------- helpers
def sha256(path: str) -> str | None:
    try:
        h = hashlib.sha256()
        with open(path, "rb") as f:
            for chunk in iter(lambda: f.read(1 << 16), b""):
                h.update(chunk)
        return h.hexdigest()
    except OSError:
        return None


def head_text(path: str, limit: int = 8192) -> str:
    try:
        with open(path, "r", errors="ignore") as f:
            return f.read(limit)
    except OSError:
        return ""


def git_origin(repo_root: str) -> str:
    try:
        out = subprocess.run(
            ["git", "-C", repo_root, "remote", "get-url", "origin"],
            capture_output=True, text=True, timeout=10,
        )
        return out.stdout.strip()
    except Exception:
        return ""


def is_interesting(name: str) -> bool:
    if name in INTERESTING_NAMES:
        return True
    _, ext = os.path.splitext(name)
    return ext.lower() in INTERESTING_EXT


def matches(patterns, text: str) -> bool:
    return any(p.search(text) for p in patterns)


def is_secret(path: str, name: str) -> bool:
    if matches(SECRET_PATTERNS, name):
        return True
    if name.endswith((".md", ".txt", ".env", ".conf", ".yaml", ".yml", ".json")):
        if matches(SECRET_CONTENT, head_text(path)):
            return True
    return False


def relevant(path: str, sample: str) -> bool:
    blob = (path + "\n" + sample).lower()
    return any(k in blob for k in RELEVANCE)


def suggest_dest(name: str) -> str:
    _, ext = os.path.splitext(name)
    ext = ext.lower()
    if ext in {".md", ".markdown", ".rst", ".txt"}:
        return "docs"
    if ext in {".sh", ".bash", ".fish", ".zsh", ".py", ".pl"}:
        return "scripts"
    if ext in {".toml", ".yaml", ".yml", ".json", ".conf", ".cfg", ".ini",
               ".service", ".timer", ".env"}:
        return "config"
    if ext in {".rs", ".ts", ".tsx", ".js", ".jsx", ".go", ".c", ".h",
               ".cpp", ".hpp", ".kt", ".java"}:
        return "code"
    return "other"


# ----------------------------------------------------------------------------- nexus-os index
def build_nexus_index(nexus: str):
    """Hash every tracked file in nexus-os; return (hashes, basename->[(path,mtime)])."""
    hashes: set[str] = set()
    by_name: dict[str, list[tuple[str, float]]] = defaultdict(list)
    try:
        tracked = subprocess.run(
            ["git", "-C", nexus, "ls-files"],
            capture_output=True, text=True, timeout=60,
        ).stdout.splitlines()
    except Exception:
        tracked = []
    for rel in tracked:
        full = os.path.join(nexus, rel)
        try:
            if not os.path.isfile(full) or os.path.getsize(full) > MAX_BYTES:
                continue
            mt = os.path.getmtime(full)
        except OSError:
            continue
        h = sha256(full)
        if h:
            hashes.add(h)
        by_name[os.path.basename(rel)].append((full, mt))
    return hashes, by_name


# ----------------------------------------------------------------------------- scan
def discover_repo_kind(dirpath: str, nexus_abs: str) -> str:
    """Return 'nexus', 'wlfogle', or 'thirdparty' for a dir that contains .git."""
    if os.path.abspath(dirpath) == nexus_abs:
        return "nexus"
    origin = git_origin(dirpath)
    if "github.com/wlfogle" in origin or "wlfogle" in origin:
        return "wlfogle"
    if origin == "":
        return "wlfogle"  # local-only repo: treat as yours, be safe
    return "thirdparty"


def _collect_repo(repo, files):
    for dirpath, dirnames, filenames in os.walk(repo, topdown=True):
        dirnames[:] = [d for d in dirnames if d not in EXCLUDE_DIRS and not d.startswith(".")]
        for fn in filenames:
            if not is_interesting(fn):
                continue
            full = os.path.join(dirpath, fn)
            if os.path.islink(full):
                continue
            try:
                if os.path.getsize(full) > MAX_BYTES:
                    continue
            except OSError:
                continue
            files.append(full)


def scan(roots, nexus, include_repos=False):
    """Default scope = LOOSE files sitting directly in each root (the actually
    scattered snippets). Your git repos are NOT harvested unless include_repos
    is set, because moving files out of a healthy repo is destructive. The
    nexus-os destination and third-party clones are always skipped, and non-repo
    toolchain/data trees (Android SDK, go cache, Wine/game prefixes, ...) are
    ignored."""
    nexus_abs = os.path.abspath(nexus)
    files = []
    skipped_thirdparty = []
    skipped_repos = []
    for root in roots:
        root = os.path.abspath(os.path.expanduser(root))
        if not os.path.isdir(root):
            continue
        for entry in sorted(os.listdir(root)):
            p = os.path.join(root, entry)
            if os.path.islink(p):
                continue
            if os.path.isfile(p):
                # loose file directly under the root
                if is_interesting(entry):
                    try:
                        if os.path.getsize(p) <= MAX_BYTES:
                            files.append(p)
                    except OSError:
                        pass
                continue
            if not os.path.isdir(p) or not os.path.isdir(os.path.join(p, ".git")):
                continue  # non-repo dir: ignore (avoids toolchain/data trees)
            kind = discover_repo_kind(p, nexus_abs)
            if kind == "nexus":
                continue
            if kind == "thirdparty":
                skipped_thirdparty.append(p)
                continue
            if include_repos:
                _collect_repo(p, files)
            else:
                skipped_repos.append(p)
    return files, skipped_thirdparty, skipped_repos


# ----------------------------------------------------------------------------- classify
def classify(path, nexus_hashes, nexus_by_name):
    name = os.path.basename(path)
    try:
        mt = os.path.getmtime(path)
        size = os.path.getsize(path)
    except OSError:
        return None
    if is_secret(path, name):
        return dict(action="SECURE", cls="SECURE", reason="secret-looking; handle manually",
                    dest="", hash="", size=size, mtime=mt)
    h = sha256(path) or ""
    sample = head_text(path)
    if h and h in nexus_hashes:
        return dict(action="TRASH", cls="DUPLICATE_IN_REPO",
                    reason="byte-identical copy already tracked in nexus-os",
                    dest="", hash=h, size=size, mtime=mt)
    twins = nexus_by_name.get(name, [])
    if twins:
        newest_repo = max(m for _, m in twins)
        if mt > newest_repo + 1:
            return dict(action="REVIEW", cls="NEWER_THAN_REPO",
                        reason="same name in nexus-os but THIS copy is newer; verify before replacing",
                        dest=suggest_dest(name), hash=h, size=size, mtime=mt)
        return dict(action="TRASH", cls="STALE_OLDER",
                    reason="older variant of a file already in nexus-os",
                    dest="", hash=h, size=size, mtime=mt)
    if matches(PERSONAL_PATTERNS, name) or matches(PERSONAL_PATTERNS, sample):
        return dict(action="MOVE_PERSONAL", cls="PERSONAL", reason="personal/non-code",
                    dest="personal", hash=h, size=size, mtime=mt)
    if matches(JUNK_PATTERNS, path):
        return dict(action="TRASH", cls="JUNK", reason="log/scratch/temp",
                    dest="", hash=h, size=size, mtime=mt)
    if relevant(path, sample):
        return dict(action="MOVE", cls="RELEVANT_MISSING",
                    reason="nexus-related and not yet in the repo",
                    dest=suggest_dest(name), hash=h, size=size, mtime=mt)
    return dict(action="REVIEW", cls="REVIEW", reason="unique; relevance unclear (kept)",
                dest=suggest_dest(name), hash=h, size=size, mtime=mt)


# ----------------------------------------------------------------------------- apply
def do_apply(results, nexus, trash_dir, personal_dir, manifest):
    staging = os.path.join(nexus, "_consolidate")
    moved = trashed = 0
    for path, c in results:
        try:
            if c["action"] == "TRASH":
                dest = os.path.join(trash_dir, path.lstrip("/"))
                os.makedirs(os.path.dirname(dest), exist_ok=True)
                shutil.move(path, dest)
                manifest.append(dict(op="trash", src=path, dst=dest, **{k: c[k] for k in ("cls", "reason", "hash")}))
                trashed += 1
            elif c["action"] == "MOVE":
                d = os.path.join(staging, c["dest"])
                os.makedirs(d, exist_ok=True)
                shutil.copy2(path, os.path.join(d, os.path.basename(path)))
                tdest = os.path.join(trash_dir, path.lstrip("/"))
                os.makedirs(os.path.dirname(tdest), exist_ok=True)
                shutil.move(path, tdest)
                manifest.append(dict(op="move", src=path, dst=os.path.join(d, os.path.basename(path)),
                                     trashed_original=tdest, cls=c["cls"], reason=c["reason"]))
                moved += 1
            elif c["action"] == "MOVE_PERSONAL":
                os.makedirs(personal_dir, exist_ok=True)
                shutil.move(path, os.path.join(personal_dir, os.path.basename(path)))
                manifest.append(dict(op="personal", src=path, dst=personal_dir, cls=c["cls"]))
                moved += 1
            # SECURE / REVIEW / NEWER_THAN_REPO: left in place by design
        except OSError as e:
            manifest.append(dict(op="error", src=path, error=str(e)))
    return moved, trashed


# ----------------------------------------------------------------------------- report
def write_report(report_path, results, skipped_tp, dup_groups, counts, args):
    with open(report_path, "w") as r:
        r.write("# NexusOS Consolidation Report\n\n")
        r.write(f"- generated: {dt.datetime.now().isoformat(timespec='seconds')}\n")
        r.write(f"- roots: {', '.join(args.roots)}\n")
        r.write(f"- nexus repo: {args.nexus}\n")
        r.write(f"- mode: {'APPLY' if args.apply else 'DRY-RUN (no files changed)'}\n")
        r.write(f"- files examined: {len(results)}\n\n")
        r.write("## Action summary\n\n")
        for k in sorted(counts):
            r.write(f"- **{k}**: {counts[k]}\n")
        r.write(f"\n- third-party clones skipped: {len(skipped_tp)}\n\n")
        if skipped_tp:
            r.write("### Third-party clones skipped (not yours; untouched)\n\n")
            for d in sorted(skipped_tp):
                r.write(f"- `{d}`\n")
            r.write("\n")
        dgroups = {h: ps for h, ps in dup_groups.items() if len(ps) > 1}
        if dgroups:
            r.write("## Duplicate clusters (identical content)\n\n")
            for h, ps in sorted(dgroups.items(), key=lambda x: -len(x[1])):
                r.write(f"- `{h[:12]}` x{len(ps)}\n")
                for p in sorted(ps):
                    r.write(f"    - `{p}`\n")
            r.write("\n")
        order = ["SECURE", "NEWER_THAN_REPO", "RELEVANT_MISSING", "REVIEW",
                 "PERSONAL", "STALE_OLDER", "DUPLICATE_IN_REPO", "JUNK"]
        bycls = defaultdict(list)
        for p, c in results:
            bycls[c["cls"]].append((p, c))
        for cls in order:
            items = bycls.get(cls, [])
            if not items:
                continue
            r.write(f"## {cls} ({len(items)})\n\n")
            for p, c in sorted(items):
                tag = f" -> {c['dest']}" if c.get("dest") else ""
                r.write(f"- `{p}` — {c['action']}{tag} — {c['reason']}\n")
            r.write("\n")


# ----------------------------------------------------------------------------- main
def main() -> int:
    ap = argparse.ArgumentParser(description="Consolidate NexusOS code/docs; report-only unless --apply.")
    ap.add_argument("--roots", nargs="+", default=[HOME], help="dirs to scan (default: ~)")
    ap.add_argument("--nexus", default=os.path.join(HOME, "nexus-os"), help="canonical repo")
    ap.add_argument("--report", default=os.path.join(HOME, "nexus-consolidate-report.md"))
    ap.add_argument("--apply", action="store_true", help="execute actions (default: dry-run)")
    ap.add_argument("--include-repos", action="store_true",
                    help="ALSO harvest files from inside your git repos (destructive; OFF by default)")
    args = ap.parse_args()

    if not os.path.isdir(args.nexus):
        print(f"FATAL: nexus repo not found: {args.nexus}", file=sys.stderr)
        return 2

    ts = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    trash_dir = os.path.join(HOME, ".nexus-consolidate-trash", ts)
    personal_dir = os.path.join(HOME, "personal", "imported")

    print("indexing nexus-os ...", file=sys.stderr)
    nexus_hashes, nexus_by_name = build_nexus_index(args.nexus)
    print(f"  {len(nexus_hashes)} tracked files indexed", file=sys.stderr)

    print("scanning ...", file=sys.stderr)
    files, skipped_tp, skipped_repos = scan(args.roots, args.nexus, include_repos=args.include_repos)
    print(f"  {len(files)} candidate files; skipped {len(skipped_tp)} third-party + "
          f"{len(skipped_repos)} of your repos (pass --include-repos to harvest them)", file=sys.stderr)

    results = []
    dup_groups = defaultdict(list)
    counts = defaultdict(int)
    for p in files:
        c = classify(p, nexus_hashes, nexus_by_name)
        if c is None:
            continue
        results.append((p, c))
        counts[c["cls"]] += 1
        if c.get("hash"):
            dup_groups[c["hash"]].append(p)

    write_report(args.report, results, skipped_tp, dup_groups, counts, args)

    # machine-readable sidecar
    with open(args.report + ".json", "w") as jf:
        json.dump([{"path": p, **c} for p, c in results], jf, indent=2)
    with open(args.report + ".csv", "w", newline="") as cf:
        w = csv.writer(cf)
        w.writerow(["path", "class", "action", "dest", "reason", "size", "sha256"])
        for p, c in results:
            w.writerow([p, c["cls"], c["action"], c.get("dest", ""), c["reason"], c["size"], c.get("hash", "")])

    manifest = []
    if args.apply:
        os.makedirs(trash_dir, exist_ok=True)
        moved, trashed = do_apply(results, args.nexus, trash_dir, personal_dir, manifest)
        with open(os.path.join(trash_dir, "manifest.json"), "w") as mf:
            json.dump(manifest, mf, indent=2)
        print(f"APPLIED: moved/staged {moved}, trashed {trashed} (recoverable in {trash_dir})", file=sys.stderr)

    # console summary
    print("\n==== SUMMARY ====")
    for k in sorted(counts):
        print(f"  {k:18} {counts[k]}")
    print(f"  third-party skipped: {len(skipped_tp)}")
    print(f"\nReport: {args.report}")
    print(f"  (+ {os.path.basename(args.report)}.json / .csv)")
    if not args.apply:
        print("\nDRY-RUN only — nothing changed. Review the report, then re-run with --apply.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
