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

Config: `--host/--port/--db` or `NEXUS_BRAIN_{HOST,PORT,DB}`. Promoting an
idea into an action (Phase 3) is documented below.

## Phase 2 — Librarian (`desktop/`) — implemented

A Tauri 2 desktop app that reads the ecosystem you already have, **interprets
every file with a local model**, decides what is current versus stale, and files
it — automatically when confident, asking when not.

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

Ollama here serves 42 models from an external drive with `OLLAMA_KEEP_ALIVE=24h`,
against a 12 GB GPU. Two consequences shaped the design:

- **The work queue is grouped by model, not by file.** Each model drains its
  whole bucket before the next loads, so multi-gigabyte weights are not evicted
  and reloaded per file.
- **Escalation instead of brute force.** Cheap models take the bulk
  (`llama3.2:3b` triage, `qwen2.5-coder:7b` for code, `qwen2.5:7b` for prose,
  `moondream` for images). Only files scoring below the confidence threshold are
  retried on `codestral:22b`, then `phi4`. Vision escalates to
  `llama3.2-vision:11b`. All configurable in Settings.

### Safety

- Nothing is deleted. "Remove" means a move into `Quarantine/`.
- Files a git repo owns are never touched — git is the authority there.
- Anything classified as `secret` is forced to `review`, never auto-filed.
- Same-filesystem moves use `rename`; cross-filesystem copies verify the
  destination hash **before** unlinking the source.
- Every operation is journalled, so any plan can be reversed from History.
- Auto-apply only above the confidence threshold (default 0.85); everything
  else waits in Review.

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
cd desktop/src-tauri && cargo test    # 50 tests
```

They cover JSON recovery from imperfect model output, model routing and the
escalation ladder, remote-URL owner parsing, chunking, vector round-trips, FTS5
query escaping, RRF fusion, and move/undo including refusal to overwrite.

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

## Phase 3 — action layer (`nexus-brain.py`) — implemented

`POST /api/note/<id>/promote` marks the idea `actionable`, fires an **n8n**
webhook with the note, then asks a local **Ollama** model to draft an
importable n8n workflow JSON for it. Both steps are optional bolt-ons on top
of Phase 1 — an unset webhook or Ollama URL just skips that step (reported as
`"attempted": false`), so promoting always succeeds even with n8n/Ollama down
or never configured. A successfully drafted workflow is stored on the note
(`workflow_json`, `promoted_at`) and returned inline; fetch it later via
`GET /api/note/<id>`. The embedded UI adds a "promote → n8n" / "workflow"
link per note.

Config: `--n8n-webhook/NEXUS_BRAIN_N8N_WEBHOOK` (default off),
`--ollama-url/NEXUS_BRAIN_OLLAMA_URL` (default `http://127.0.0.1:11434`),
`--ollama-model/NEXUS_BRAIN_OLLAMA_MODEL` (default `qwen2.5-coder:7b`),
`--n8n-timeout`/`--ollama-timeout` (seconds). Point `--n8n-webhook` at a
workflow's webhook URL on CT-300's n8n (`http://192.168.12.30:5678/webhook/...`)
to reuse the existing instance.

## Design rule

Self-contained core. Ollama (AI) and n8n (action) are optional plug-ins, never
hard dependencies — Librarian's inventory, duplicate detection and keyword
search still work with Ollama down; only interpretation and semantic search
degrade. Same rule for Phase 1/3: `nexus-brain.py` capture, search and status
work with no config at all; promoting a note without a webhook or Ollama
configured just flips its status, it never errors.
