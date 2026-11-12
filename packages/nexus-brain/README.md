# nexus-brain

> A self-contained NexusOS package to **capture thoughts/ideas, search them, and
> (later) turn them into actions** (e.g. n8n workflows). Ships in the NexusOS
> install, but is built to **run standalone for testing**.

## Why

Offload "too much info to handle by myself": a personal second brain — capture
from any device, retrieve instantly, and promote the actionable ones into
automations.

There are two halves to that problem, and this package now holds both:

- **Forward** — thoughts you have from now on → `nexus-brain.py` (Phase 1)
- **Backward** — the thoughts already scattered across the disk → `desktop/`,
  the **Librarian** (Phase 2)

The backward half exists because the forward half is not enough on its own.
This README was written to be "the memory" for a parked idea, and it still got
lost: on 2026-08-07 the same idea was designed again from scratch by someone who
had forgotten the package was called `nexus-brain`. Capture and keyword search
cannot fix that — you cannot grep for a name you no longer remember.
Interpretation can.

## Phase 1 — capture service (`nexus-brain.py`)

Pure-stdlib (`http.server` + `sqlite3`/FTS5), zero pip deps, embedded web UI.
Capture + tag + full-text search + status (inbox/idea/actionable/done/archived).
REST API + `/healthz`.

```bash
python3 nexus-brain.py --selftest     # verify
python3 nexus-brain.py                # http://127.0.0.1:8700
```

Config: `--host/--port/--db` or `NEXUS_BRAIN_{HOST,PORT,DB}`.

## Phase 2 — Librarian (`desktop/`) — implemented

A Tauri 2 desktop app that reads the ecosystem you already have, **interprets
candidate files with an optional local model**, compares loose material with
current repositories, and preserves useful ideas separately from their old
implementations. **No date cutoff and no automatic filesystem approval.**

### Compare & recover ideas
The default desktop view runs an independent, read-only comparison. It indexes
tracked files in the configured repository vault and compares allowed source
files by SHA-256, filenames and rarity-weighted lexical evidence. Results retain
multiple candidate repositories, file/line excerpts, scope limits, exclusions
and protected source trees. Similarity is a retrieval lead, never proof that a
feature is implemented or a script is obsolete.

Select **Study** for side-by-side text and a unified difference. Previews recheck
content hashes; changed files require a new comparison. **Assess useful ideas
with local Ollama** is an explicit optional request, not part of every scan.
It retrieves relevant current-file excerpts, canonical project guidance and
local Git history; separates purpose, useful behaviors, outdated assumptions,
adaptation proposals, verification requirements and unknowns. Quotes are checked
against supplied evidence. Documentation alone cannot establish an implementation,
and unclassified/historical code cannot establish a current implementation.
All model conclusions remain advisory, including those with valid quotes.
Adapting an idea is a separate, reviewed and tested coding task; the app does not
silently rewrite production code or execute historical scripts.

Sources with any `.git` marker are protected from individual file moves even
when Git metadata is broken. The home NexusOS tree is one such case: compare
and preserve it as a tree instead of assuming every home file is loose.
Current-module/forerunner/third-party labels come from configured or explicitly
named canonical guidance, not repository age. Missing guidance stays unclassified.

Choose the owning repository and an editable relative destination to queue an
original under `archive/reference/`. Queueing does not approve or move anything.
Review shows the exact source, destination and reason; approve each proposal,
then explicitly **Apply approved**. A `.reference.json` sidecar records the
original path, checksum, permissions, date and reason. Originals are preserved
byte-for-byte as reference, not promoted unchanged into active scripts.
The same UI can save/reopen private JSON comparisons and assessments.

### Read-only CLI
From this package directory (fish-compatible commands):
```fish
./desktop/src-tauri/target/release/librarian --compare --root /home/loufogle --vault /media/loufogle/Data/Repos --output /home/loufogle/.local/state/librarian/comparison-home.json
```
The output parent must already exist. Reports are created mode 0600 and never
overwrite an existing path. Without `--output`, the CLI prints counts, not file
contents. Unlike `--headless`, `--compare` does not initialise/migrate the app
catalog, call models or create cleanup actions. Explicit `--assess-index N`
adds a local advisory assessment for one zero-based result to the report.
`--root` can repeat; `--max-files` and `--max-repo-files` adjust file caps.

Default comparison limits: 10,000 source findings, 30,000 repository files,
128 KiB per text file, 64 MiB repository text, 250,000 visited entries, depth 32.
Only immediate Git repositories in the vault are indexed, using their current
tracked working-tree files (including current edits); untracked repo work is
not assumed absent. Retrieval evaluates up to 100 candidates and presents five
ranked leads. Source trees are not repositories' deletion candidates.
Hidden directories, secrets, browser/application state, dependencies/builds,
media and reference archives are excluded. Binary/non-UTF8/oversized files are
not interpreted. Credential screening is conservative and heuristic, not a
complete DLP guarantee. Reports contain local paths and excerpts: keep them private.
Unreadable paths, cancellation and limits are explicit partial coverage.
No result proves whole-repository absence, runtime compatibility or non-use.

### Pipeline

| tier | module | what it does |
|------|--------|--------------|
| 0 | `scan.rs` | inventory via `stat` only; incremental on size + mtime |
| 1 | `extract.rs` | SHA-256, text extraction, `pdftotext` / Tesseract OCR |
| 2 | `embed.rs` | chunk + embed with `nomic-embed-text` (768-dim) |
| 3 | `interpret.rs` | a local LLM reads each file and returns a judgement |

Tier 3 returns, per file: `title`, `kind`, `purpose`, `summary`, `topics`,
`entities`, `related_repo`, `status`, `action`, `reason`, `confidence`.

### Leveraging the local model library

Available models are queried from the local Ollama server rather than assumed
from this historical inventory. Two consequences shaped the design:

- **The work queue is grouped by model, not by file.** Each model drains its
  whole bucket before the next loads, so multi-gigabyte weights are not evicted
  and reloaded per file.
- **Escalation instead of brute force.** Cheap models take the bulk
  (`qwen3.5:4b` triage, `qwen3.5:9b` for code, `qwen3.5:9b` for prose,
  `moondream` for images). Only files scoring below the confidence threshold are
  retried on the configured escalation model. Vision escalates to
  `qwen3.5:9b`. All configurable in Settings.

### Safety

- Nothing is deleted. "Remove" means a move into `Quarantine/`.
- Files a git repo owns are never touched — git is the authority there.
- Anything classified as `secret` is forced to `review`, never auto-filed.
- Every proposed move stores a full source backup and review fingerprint in the
  private catalog. Apply rechecks boundaries, symlinks, source content and permissions.
- Same- and cross-filesystem moves use staged, checksum-verified copies, fsync and
  no-clobber publication. Filesystems without hard-link support fail safely;
  originals are not removed to work around unsupported publication.
- The journal is written **before** publication. Intermediate phases remain
  inspectable. Reference provenance is written before source removal.
- **All actions require human approval.** Old auto-approved rows migrate back to
  review because old versions did not distinguish model from human approvals.
- Undo refuses later source/archive edits or collisions. Published copies can be
  recovered from History; unknown pre-publication states require manual inspection.
  Backups are retained in `action_snapshots.backup`, even after undo; there is no
  automatic expiry, purge timer or permanent-delete command.
- History's **Export backup** verifies the stored checksum and creates a new
  private mode-0600 `.bin` file in the state directory for manual recovery,
  including when an edited/missing archive prevents automatic undo. It never
  restores over an existing source or archive.
- The tool is not a security sandbox against another process running as the same
  user; avoid concurrent edits/restructuring while applying or undoing moves.
  Reference archives are durable project history, separate from these safety backups.
- Legacy queued actions without review snapshots cannot be approved: reject and
  recreate them from a fresh comparison. History includes pending/failed/undone
  states rather than hiding partial operations.
- Model endpoints must be loopback HTTP; redirects and proxies are disabled.
  Structured JSON requests use `think:false` so thinking does not exhaust the
  final-answer budget. No cloud model or paid API is used.

### Build and run

```bash
cd desktop/frontend && npm install && npm run build
cd ../src-tauri     && cargo build --release
./target/release/librarian
```

Dev mode: `cargo tauri dev` from `desktop/src-tauri`.

State lives outside the repo in `~/.local/state/librarian/` (`catalog.db`,
`config.json`). The managed library is `~/Library/Librarian/` (`Inbox`,
`Archive`, `Quarantine`, `Notes`, `RepoRefs`, `Backups`).

### Tests

```bash
cd desktop/src-tauri && cargo test    # 71 tests
```

They cover parser/model routing, retrieval, exclusions, no date policy, broken-Git
source protection, changed-file previews, explicit approvals, safe copies,
reference provenance, executable bits, interruptions, undo conflicts and migration.
Run `cargo clippy --all-targets -- -D warnings` and the frontend build as well.
Native UI smoke checks should use an isolated HOME and disposable source/repo
fixtures, never the real catalog or home files as mutation fixtures.

### Verification — 2026-11-12
- Frontend TypeScript/production build, all 71 Rust tests, strict Clippy and the
  optimized release build passed. The existing application-menu launcher points
  to the rebuilt canonical release executable.
- Native release startup and the bundled comparison view were observed with
  an isolated HOME and disposable fixtures. Post-click X11 capture returned
  `BadMatch`; a complete automated desktop click-through is not claimed.
- The release read-only home scan produced 8,126 candidates across 54 repos.
  It is explicitly partial because paths under `~/data11` were unreadable;
  no permission changes or privileged scan was attempted.
- Three source trees are protected, including the home NexusOS tree. That tree
  has 640 identical counterparts, 53 divergent-copy candidates and 388 ambiguous
  matches within this scan's coverage. These are comparison leads, not proof
  that differing files are unique features or obsolete.
- Private mode-0600 reports:
  `~/.local/state/librarian/comparison-home-reviewed.json` and
  `~/.local/state/librarian/comparison-openwrt-quoted-evidence.json`.
  Open them from **Compare & recover ideas → Open a saved private comparison report**.
  A real local Qwen assessment completed; its suggestions without verified
  quotes remain `investigate`, not established implementation facts.
- No actual home scripts were executed, moved, archived or rewritten. The real
  catalog was not used as a mutation test fixture. Idea adaptations and home
  cleanup actions still require separate review and approval.

## `tools/`

Standalone scripts from the 2026-08-07 consolidation, kept because they are the
recovery path if a repo has to be reconstructed.

- `audit-local-only.sh` — what would be lost if a repo were deleted and
  re-cloned (unpushed commits on any branch, branches absent from the remote,
  stashes, dirty and untracked files).
- `preserve-local-only.sh` — push what can be pushed, bundle what cannot.
- `migrate-repos.sh` — rsync repos into the vault with verification and
  compatibility symlinks.
- `make_icons.py` — generates the app icons with no image library.

For scanning and de-duplicating `~` against this repo, use the existing
`nexus-os/scripts/nexus-consolidate.py`; it predates these and does that job.

## Phase 3 — action layer (still open)

Promote an idea → fire an **n8n** webhook; then draft an n8n workflow JSON from
the idea via Ollama. Reuse n8n + Ollama already on CT-300.

## Design rule

Self-contained core. Ollama (AI) and n8n (action) are optional plug-ins, never
hard dependencies — Librarian's inventory, duplicate detection and keyword
search still work with Ollama down; only interpretation and semantic search
degrade.
