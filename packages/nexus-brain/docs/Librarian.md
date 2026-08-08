# Librarian — architecture and defect history

> Phase 2 of `nexus-brain`. Where Phase 1 (`nexus-brain.py`) captures *new*
> thoughts, Librarian recovers the ones already scattered across the disk: it
> reads every file, has a local model decide what it actually is, and files it.
>
> Origin: `docs/origin-transcript.md` (the design conversation) and
> `docs/Librarian_Enhancements.md` (the enhancement list that fed this build).
> Built 2026-08-07.

## Why interpretation, not search

`nexus-brain` Phase 1 already did capture plus full-text search, was flagged
"promising", and was parked. It was then **re-invented from scratch** by someone
who had forgotten the package existed. Keyword search could not have prevented
that — you cannot grep for a name you no longer remember. Only something that
reads the content and tells you *what it is* can surface a forgotten idea.

That is the entire justification for the cost of tier 3.

## Pipeline

Four tiers. Progress lives in `files.stage`, not in memory, so a run is
resumable and killing the app loses nothing.

| tier | module | work | cost |
|------|--------|------|------|
| 0 | `scan.rs` | walk allowed roots, `stat` only | seconds |
| 1 | `extract.rs` | SHA-256, text extraction, `pdftotext` / Tesseract OCR | minutes |
| 2 | `embed.rs` | chunk + embed via `nomic-embed-text` (768-dim) | minutes |
| 3 | `interpret.rs` | a local LLM reads each file and judges it | hours |

Tier 0 is incremental on size + mtime, so unchanged files keep their stage and
are never reprocessed. Vanished files are marked `present = 0` rather than
deleted, which preserves their interpretation in case they reappear — and a file
that reappears elsewhere with the same hash has its interpretation re-attached
instead of recomputed.

### Tier 3 output

Per file, as structured JSON: `title`, `kind`, `purpose`, `summary`, `topics`,
`entities`, `related_repo`, `status` (current / stale / superseded / reference /
junk), `action` (keep / file / archive / quarantine / review), `reason`,
`confidence`.

### Model routing

Ollama here serves 42 models from an external drive with
`OLLAMA_KEEP_ALIVE=24h`, against a 12 GB GPU. Two consequences shaped the
design:

- **The queue is grouped by model, not by file.** Each model drains its whole
  bucket before the next loads. Interleaving files across models would evict and
  reload multi-gigabyte weights continuously.
- **Escalation, not brute force.** Cheap models take the bulk; only results
  below the confidence threshold are retried one tier up.

```
image        moondream ──────────────┐
                                     ├─► llama3.2-vision:11b
code/script/config  qwen2.5-coder:7b ─┐
doc/prose (>2 KB)   qwen2.5:7b       ─┼─► codestral:22b ──► phi4
everything else     llama3.2:3b ─────┘
```

Confirmed in practice on a live run: `qwen2.5-coder:7b` took the config and
source files, `qwen2.5:7b` the prose, `llama3.2:3b` the short odds and ends, and
a screenshot escalated from `moondream` to `llama3.2-vision:11b`, which
correctly described it as a recording of a Konsole session.

### Search

FTS5/BM25 fused with vector cosine via Reciprocal Rank Fusion (k=60), then
grouped by content hash so the canonical copy leads and identical copies sit
beneath it. Degrades to keyword-only when Ollama is unreachable.

## Safety model

- **Nothing is deleted.** "Remove" means a move into `Quarantine/`.
- **Repo-owned files are never touched.** Git is the authority inside a
  working tree; the planner only considers files with `repo_id IS NULL`.
- **Secrets are never auto-filed.** Anything the model labels `secret` is forced
  to `review`. Credential directories (`.ssh`, `.gnupg`, `.aws`, `.kube`,
  `.docker`, `.password-store`) are pruned from the walk entirely.
- **Cross-filesystem moves verify before unlinking.** Same-filesystem moves use
  `rename`; otherwise copy, compare hashes, and only then remove the source.
- **Every operation is journalled**, so any plan reverses from History.
- **Auto-apply only above the confidence threshold** (default 0.85). Everything
  else waits in Review.

## Layout

```
desktop/src-tauri/src/
  config.rs      roots, prune rules, model routing, versioned migration
  db.rs          schema, vector helpers, honest stat counters
  scan.rs        tier 0
  extract.rs     tier 1
  embed.rs       tier 2
  interpret.rs   tier 3
  repos.rs       git graph: worktrees, submodules, recoverability
  search.rs      hybrid retrieval
  actions.rs     planner, verified mover, journal, undo
  engine.rs      pipeline driver + ProgressSink
  commands.rs    window command surface
desktop/frontend/src/   React UI (9 views)
desktop/tools/          launchers, debug wrapper, migration scripts
```

State lives outside the repo in `~/.local/state/librarian/` (`catalog.db`,
`config.json`, `run.log`). The managed library is `~/Library/Librarian/`.

## Running

```bash
# window
./desktop/src-tauri/target/release/librarian

# pipeline only, no display — cron-able, and how the engine is verified
./desktop/src-tauri/target/release/librarian --headless

# debug build with backtraces, inspector, and output teed to run.log
./desktop/tools/run-debug.sh
```

Launchers **Librarian** and **Librarian (Debug)** are installed in the
application menu, with right-click actions for headless runs and tailing the log.

---

# Resolved defects

Kept because several of these are mistakes that look like working code, and the
notes are cheaper than rediscovering them.

## 1. Pipeline deadlock — the app froze on "discovering git repositories"

`run_pipeline` held the database guard while calling `emit()`, and `emit()` locks
the same `std::sync::Mutex` to read stats. `std::sync::Mutex` is **not
reentrant**, so the second acquisition blocked forever. The window appeared to
freeze because the frontend's `get_stats` poll queued behind the same lock.

**Fix.** Release the guard before reporting, and run every blocking phase inside
`spawn_blocking` so the async runtime stays free to serve the window. The rule is
documented inline at each phase because it is easy to reintroduce.

## 2. Metrics overstated progress by six figures

The dashboard reported **229,307 files interpreted while the `interpretations`
table held zero rows.** `stats.interpreted` counted `stage >= 3`, but
`skip_unreadable` promotes binaries and oversized files straight to stage 3
without reading them.

**Fix.** Each counter now measures what its name claims — `interpreted` counts
rows a model produced, `skipped` is reported separately, and the progress bar is
a fraction of *eligible* files rather than of everything on disk. `Stats` derives
`Default` so the fallback literal cannot drift out of sync again.

## 3. Prune-list changes silently had no effect

Adding `.local` and `.config` to the deny list changed nothing on a machine that
had already run once. `Config::load` deserialises the existing `config.json`, so
the defaults compiled into the binary were overridden by whatever the file was
first written with. `.local/share` alone contributed icon themes, shell
completions, flatpak appstream data and shader binaries.

**Fix.** `CONFIG_VERSION` plus an additive migration that merges newly shipped
prune entries while preserving user-added ones. Corpus went from **332,506 files
to 4,487**, and extraction from **0 of 200** files yielding text to **199 of
200**.

This is the most dangerous class of bug here: the code was correct, the test
would have passed, and the behaviour was unchanged.

## 4. "Connection refused" on launch

A plain `cargo build` left the `custom-protocol` feature off, so the binary tried
to load the frontend from `devUrl` (`localhost:5173`) instead of the embedded
bundle. **Fix:** `default = ["custom-protocol"]`.

## 5. Bundle build failed on a missing `package.json`

Tauri runs `beforeBuildCommand` from the *app* directory, not from `src-tauri/`,
so `npm --prefix ../frontend` resolved one level too high. **Fix:** drop the
`../`.

## 6. Dropdown text was unreadable

WebKitGTK draws native form controls with the *platform* light theme, washing out
selected text on a dark background. **Fix:** `appearance: none`, paint the
controls and their popup options explicitly, draw the arrow ourselves, and
declare `color-scheme: dark`.

## 7. UTF-8 panic risk in the HTML stripper

`html_to_text` indexed a lowercased copy of the input using byte offsets from the
original. On any non-ASCII page the lengths diverge and slicing can land
mid-codepoint and panic. It also pushed raw bytes as `char`, mangling multi-byte
sequences. **Fix:** operate on bytes with ASCII-case-insensitive matching, decode
once at the end.

## 8. Two unit tests asserted folklore over behaviour

- FTS5 escapes an embedded quote by **doubling** it, so `a"b` becomes `"a""b"`.
  The test expected the quote to be stripped.
- With RRF at k=60, ranking first-and-last (1/61 + 1/63) narrowly **beats**
  second-and-second (2 × 1/62). The test asserted the opposite.

Both were the tests being wrong, not the code. Fixed with the arithmetic in a
comment so they are not "corrected" back.

## 9. Orphaned rows from non-cascading deletes

`DELETE FROM files` in the `sqlite3` CLI left `file_text` rows behind, because
**`PRAGMA foreign_keys` is per-connection and off by default in the CLI** — the
schema declares `ON DELETE CASCADE`, but the app enables the pragma and the CLI
does not. This produced impossible readings (`with_text` exceeding `hashed`)
during diagnosis. Any manual surgery on this database must set
`PRAGMA foreign_keys=ON` first.

## Verification method

The bugs above were found by running the pipeline, not by reading it — the
deadlock compiled cleanly and passed every test. `--headless` behind the
`ProgressSink` trait exists so the engine can be exercised without a display,
and is the reason items 1–3 became observable at all.

Two traps encountered while measuring, worth avoiding next time:

- `tail -n30` on a live pipe prints nothing until EOF, so a `timeout`-killed run
  looks silent. Poll the database or use `stdbuf -oL`.
- `pkill -f librarian` matches its own command line and kills the shell running
  it. Use `pkill -x`.

## Known limitation

Tiers run to completion in sequence: extraction fully drains before embedding,
which fully drains before interpretation. On a cold run the dashboard therefore
sits at zero interpreted for several minutes while doing real work, which reads
as broken. Interleaving the tiers so judgements appear immediately is the obvious
next improvement.
