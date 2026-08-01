# Nexus Terminal → Exact Warp Clone — Progress & Handoff

**Status:** The `oz/nexus-warp-clone` branch/orchestration below was never merged and is now stale/superseded — the classifier fix landed directly on `main` instead (commit `6aa2db71`, porting the correct 5-tier algorithm from standalone `nexus-terminal` commit `11a4457`), and OSC 133 shell integration also landed on `main` (commit `2051c72c`). On 2026-08-01, found and fixed a separate, unrelated live bug: `write_to_terminal` called `MasterPty::take_writer()` on every write, but `portable-pty` only allows this once per PTY master (subsequent calls fail with "cannot take writer more than once", surfaced to users as "Shell error: Failed to get terminal writer" after the very first write/OSC-133-bootstrap injection). Fixed by taking the writer once at terminal creation and caching it in the `Terminal` struct behind a `Mutex`; `cargo check` passes. The `oz/nexus-warp-clone` remote branch can likely be deleted.
**Plan ID:** `4797e695-63d6-4fa2-a102-c61e50c424b5` (superseded — see Status above)
**Prior integration commit (unmerged, stale):** `c4d2cd2b` on `oz/nexus-warp-clone`

## Goal
Turn `nexus-os/packages/nexus-terminal` into an exact clone of Warp — behavior (command-vs-AI input auto-detection, Oz-style agent, blocks) AND screen layout — using Warp's open-source client (`github.com/warpdotdev/warp`, AGPL-3.0) as reference. Credit Warp; relicense to AGPL-3.0.

## Environment (ground truth)
- Repo: `/home/loufogle/nexus-os/packages/nexus-terminal` — Tauri 2 + React 18 + Vite 7 + Rust.
- Laptop: i9-13900HX, 62.5 GiB RAM, Pop!_OS 22.04, kernel 7.0.11, X11. Node v20.20.2, npm 10.8.2. Laptop Python is 3.10 (irrelevant here).
- Ollama: **laptop `http://127.0.0.1:11434`**, v0.18.0, running (systemd, user `ollama`). Models on external drive `/media/loufogle/73cf9511-0af0-4ac4-9d83-ee21eb17ff5d/models`. **NOT vm-990** (that VM is Home Assistant OS).
- Model selection: auto-pick a tool-calling-capable model via `/api/tags` (nothing hardcoded).

## Disk & offload (IMPORTANT)
- `/` (nvme1n1p3): 907G, **96% used, ~43G free** — tight.
- `/media/loufogle/Data` (nvme0n1p3): 899G, **~137G free** → use this for offload.
- Offload target: **`/media/loufogle/Data/tmp`** (put git worktrees, cargo target, TMPDIR here). `Data/tmp` currently holds ~105G of games/junk.
- **DO NOT DELETE: Diablo IV, Battle.net.** User authorized using whatever space is needed and to fix space constraints (offload, not delete games).

## Base
- Branch all worktrees from current `origin/main` (`40cd407e`).

## Root-cause findings (grounding the fix)
1. `src-tauri/src/input_classifier.rs` is a loose port with an over-aggressive `NL_VERB_PREFIXES` list that forces real commands to AI; missing Warp's parser tokenization + installed-binary/token-description scoring + exact thresholds.
2. `src/hooks/useInputRouting.ts` mixes `invoke('write_to_terminal', { terminalId })` (`:63`) and `{ terminal_id }` (`:80`,`:102`); Tauri v2 wants camelCase → snake_case shell path silently fails.
3. `src/services/commandRouting.ts:222` silently swallows classifier errors → falls back to flawed regex.
4. Layout is one custom terminal view, not Warp's block-based layout.

## Warp reference
`crates/input_classifier`: `util.rs` (one-off allowlists, `is_likely_shell_command`, thresholds 0.5 / 0.7), `heuristic_classifier`, `parser`, optional `onnx` bert_tiny (skip ONNX — use heuristic path). Autodetection wiring: `app/src/ai/blocklist/input_model.rs`; blocks/input/layout: `app/src/terminal/**`.

## Orchestration — 4 local child agents (I integrate)
Each in its own git worktree on the Data disk, branched from `40cd407e`. Shared manifests (`src-tauri/src/main.rs` invoke_handler, `Cargo.toml`, `package.json`, tailwind/vite) are INTEGRATOR-owned; children report needed registrations/deps, never edit manifests.

1. **clone-classifier** — branch `oz/clone-classifier`, worktree `/media/loufogle/Data/tmp/nexus-worktrees/clone-classifier`. Owns `src-tauri/src/input_classifier.rs`, `src-tauri/words.txt`, `src-tauri/stack_overflow.txt`, `src/services/commandRouting.ts`, `src/services/commandRouting.test.ts`, `src/hooks/useInputRouting.ts`. Port Warp heuristic classifier faithfully; drop verb-prefix hack; fix invoke arg casing. Validate: `npm test`.
2. **clone-terminal** — branch `oz/clone-terminal`, worktree `.../clone-terminal`. Owns `src-tauri/src/terminal.rs`, `src-tauri/src/prediction.rs`, new `blocks` event module. Emit per-command block metadata (cmd/cwd/exit/timing); PATH-aware exec; autosuggest. Publishes block event/props contract first. Validate: `cargo check`.
3. **clone-agent** — branch `oz/clone-agent`, worktree `.../clone-agent`. Owns `src-tauri/src/agent.rs`, `ai.rs`, `ai_optimized.rs`, `model_router.rs`. Oz-like tool schema, streaming, planning/todos, ask_user, concise style; Ollama at `127.0.0.1:11434`, model via `/api/tags`. Validate: `cargo check`.
4. **clone-frontend** — branch `oz/clone-frontend`, worktree `.../clone-frontend`. Owns `src/components/**`, `src/App.tsx`, `src/index.css`, UI redux slices, `src/types/terminal.ts`. Warp-exact layout (block list, bottom unified input + mode indicator, top tab bar, right agent panel, command palette, theme). Consumes block contract. Validate: `npm run type-check && npm run build`.

## Sequencing & safety (per user)
- **Sequence the children so they do NOT commit at the same time** (avoid simultaneous git ops / merge races). If risk is high, run them individually (one at a time) rather than fully parallel.
- **Pre-create** all worktrees before launching.
- Every child base prompt: no destructive commands (no `rm -rf`), no edits outside owned files, read-before-edit, fish-safe or `bash -c`, `nala` for OS packages, stop-and-ask when unsure, validate before handoff, do NOT run full `tauri build` (integrator does that).

## Merge strategy
Integrate A→D→C→B onto `oz/nexus-warp-clone`; apply shared-manifest edits (command registration, deps, AGPL relicense + attribution/CREDITS); `cargo check` after each merge; then full validation + `tauri dev` smoke test; one PR. Update docs + commit per user rules.

## Validation
From `packages/nexus-terminal`: `npm install`; `npm run type-check`; `npm run build`; `npm test`; in `src-tauri`: `cargo check` + `cargo clippy`; then `npm run tauri:dev` with Ollama up at `127.0.0.1:11434` and models drive mounted. Smoke: `ls -la`/`git status` → shell; natural language → agent; layout matches Warp.

## Next actions (resume here)
1. Pre-create worktrees under `/media/loufogle/Data/tmp/nexus-worktrees/` from `40cd407e`; point `CARGO_TARGET_DIR`/`TMPDIR` to the Data disk.
2. Launch the 4 child agents (local), sequenced so commits don't collide.
3. Integrate, validate, PR.
