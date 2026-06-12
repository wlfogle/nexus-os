# NexusTerminal – Progress Report
Last updated: 2026-06-07 (Phase 1 complete)

---

## Repository State

- **Canonical codebase:** `/home/loufogle/nexus-os/packages/nexus-terminal` (nexus-os monorepo)
- **Archived standalone:** `https://github.com/wlfogle/nexus-terminal` (read-only since 2026-03-26)
- **Standalone local copy:** `/home/loufogle/nexus-terminal` — 3 commits ahead of archived origin, never pushed

### Standalone commits NOT yet synced to nexus-os:
| Commit | Description |
|--------|-------------|
| `11a4457` | fix(routing): rewrite command classifier with correct shell-first priority order |
| `629168a` | fix(backend): eliminate all .unwrap() crash risks in production code paths |
| `2d220c3` | fixes (misc) |

### Frontend sync status:
`src/` (TypeScript/React) — **already identical** between standalone and nexus-os. No action needed.

---

## Crash Risks Found in nexus-os (Not Yet Fixed)

### 1. `src-tauri/src/analytics.rs` — Line 402
**Crash:** `.unwrap()` on `partial_cmp` panics on NaN floats.

```rust
// BROKEN (nexus-os)
values.sort_by(|a, b| a.partial_cmp(b).unwrap());

// FIX
values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
```

### 2. `src-tauri/src/ai.rs` — Line 759
**Crash:** `.try_clone().unwrap()` on log file handle panics if file clone fails.

```rust
// BROKEN (nexus-os)
command.stdout(log_file.try_clone().unwrap());
command.stderr(log_file);

// FIX
let log_file_stderr = log_file.try_clone()
    .map_err(|e| anyhow::anyhow!("Failed to clone log file handle: {}", e))?;
command.stdout(log_file_stderr);
command.stderr(log_file);
```

### 3. `src-tauri/src/config.rs` — Line 65
**Crash:** `.current_dir().unwrap()` panics if working directory is unavailable.

```rust
// BROKEN (nexus-os)
let app_dir = dirs::data_dir()
    .unwrap_or_else(|| std::env::current_dir().unwrap().join(".data"))
    .join("nexus-terminal");

// FIX
let app_dir = dirs::data_dir()
    .unwrap_or_else(|| {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(".data")
    })
    .join("nexus-terminal");
```

### 4. `src-tauri/src/command_flow.rs` — Lines 977–978
**Crash:** Two `.unwrap()` calls on `HashMap::get_mut()` in `topological_sort()`. If an edge references a node ID not in the flow map, this panics.

```rust
// BROKEN (nexus-os)
adj_list.get_mut(&edge.from).unwrap().push(edge.to.clone());
*in_degree.get_mut(&edge.to).unwrap() += 1;

// FIX
if let Some(neighbors) = adj_list.get_mut(&edge.from) {
    neighbors.push(edge.to.clone());
}
if let Some(degree) = in_degree.get_mut(&edge.to) {
    *degree += 1;
}
```

### 5–8. Remaining differing files (deeper scan needed)
These files differ between standalone and nexus-os but crash sites were not found in first 200 lines.
Requires deeper read or targeted grep:
- `src-tauri/src/ecosystem_awareness.rs`
- `src-tauri/src/ollama_config.rs`
- `src-tauri/src/collaboration.rs`
- `src-tauri/src/plugin_system.rs`

---

## Missing Tauri Backend Commands (Never Implemented Anywhere)

The following commands are invoked by frontend services but have **no Rust implementation** in `main.rs`:

| Command | Called by | Purpose |
|---------|-----------|---------|
| `load_aliases` | `aliasService.ts` | Load persisted alias list |
| `save_aliases` | `aliasService.ts` | Persist alias list |
| `load_command_frequency` | `aliasService.ts` | Load command frequency map |
| `save_command_frequency` | `aliasService.ts` | Persist command frequency map |
| `preview_command` | `commandPreview.ts` | Dry-run preview of a command |
| `get_health_metrics` | `terminalHealth.ts` | System + terminal health metrics |
| `generate_health_report` | `terminalHealth.ts` | Export health report (html/json) |

All these will throw at runtime, causing the frontend services to log errors on startup.

---

## Shell Integration (Not Yet Implemented)

Warp-style OSC 133 / FTCS escape sequence injection into PTY sessions is **not implemented**.

- `terminal.rs` — no bootstrap script injection when spawning shells
- `TerminalWithAI.tsx` — uses naive keyboard-input line buffering instead of OSC parser

This means command block boundaries, prompt detection, and accurate command history are all heuristic/unreliable.

**Required work:**
1. In `terminal.rs` `create_terminal_with_config()`: inject a shell bootstrap that emits `\x1b]133;A\x07` (prompt start), `\x1b]133;B\x07` (command start), `\x1b]133;C\x07` (output start), `\x1b]133;D\x07` (command end) sequences.
2. In `TerminalWithAI.tsx`: parse OSC 133 markers to create accurate command blocks.

---

## Frontend Placeholder Issues (Not Yet Fixed)

From prior Copilot analysis:

| File | Issue |
|------|-------|
| `src/components/ai/EnhancedAIAssistant.tsx` | Malformed/incomplete file — needs recreation |
| `src/components/terminal/AIAssistantPanel.tsx` | Placeholder returns, stub UI |
| `src/components/terminal/NewTabModal.tsx` | Placeholder UI code |
| `src/components/terminal/TerminalTabManager.tsx` | Placeholder component logic |
| `src/services/ragService.ts` | Missing method implementations |
| `src/services/visionService.ts` | Missing method implementations / stubs |

---

## Work Completed (Prior Sessions)

- Rewrote `src/services/commandRouting.ts` — strict 5-tier priority: known shell cmds → shell patterns → PATH lookup → AI triggers → NL heuristics
- Fixed `.unwrap()` crash risks in standalone repo (committed as `629168a`) across: `ai.rs`, `analytics.rs`, `config.rs`, `command_flow.rs`, `ecosystem_awareness.rs`, `ollama_config.rs`, `collaboration.rs`, `plugin_system.rs`
- NexusOS Phase 5.3 verified: kernel boots from installed disk, IPC tests pass
- Limine UEFI boot fully fixed: GPT ESP detection, limine.conf, ELF writing, startup.nsh

---

## Completed This Session

1. ✅ All 4 crash fixes already applied (verified)
2. ✅ Scanned 4 remaining files — 1 new crash fix in `ecosystem_awareness.rs:1914`
3. ✅ 7 missing Tauri commands implemented: `load_aliases`, `save_aliases`, `load_command_frequency`, `save_command_frequency`, `preview_command`, `get_health_metrics`, `generate_health_report`
4. ✅ OSC 133 shell integration — `terminal.rs` now injects bash/zsh/fish hooks on spawn
5. ✅ **Agent engine** — `src-tauri/src/agent.rs`: autonomous AI with tool use (read_file, write_file, run_cmd, list_dir, grep, git_status), 20-step loop, works with any Ollama model
6. ✅ `EnhancedAIAssistant.tsx` — wired to `agent_chat`, shows tool calls inline
7. ✅ **Fixed agent model** — `AGENT_MODEL=llama3.1:8b` (supports tool calling). `deepseek-coder-v2:16b` does NOT support Ollama tool calls — kept only for completions.
8. ✅ **Warp input classifier ported** — `src-tauri/src/input_classifier.rs`: exact port of Warp’s HeuristicClassifier (AGPL-3.0). Uses Warp’s `words.txt` + `stack_overflow.txt` word lists, shell syntax detection, PATH lookup. Exposed as `classify_input` Tauri command.
9. ✅ **commandRouting.ts rewritten** — primary path now calls Rust `classify_input` instead of TypeScript regex. Regex kept as fallback only.
10. ✅ **Tool results persisted** (Warp-style blocks) — `EnhancedAIAssistant.tsx` + `useInputRouting.ts`: tool call outputs are now baked into the committed Redux message, not discarded on `agent-done`.
11. ✅ `cargo check` + `npm run build` both pass clean.

---

## 🏁 Phase 1 Complete — 2026-06-07

### Core Terminal
| Feature | Status |
|---------|--------|
| Fish shell PTY | ✅ Full-width, fish colors, correct cwd |
| Shell command routing | ✅ `ls -la` → green **SHELL** badge → PTY → output in xterm |
| NL routing | ✅ `list files` → purple **AGENT** badge → llama3.1:8b → overlay |
| Warp-style UDI | ✅ Rounded pill, border color by mode, cwd chip, model name, mode buttons |
| Auto-detect | ✅ Warp HeuristicClassifier ported (AGPL-3.0), 120ms debounce |

### AI Agent
| Feature | Status |
|---------|--------|
| Tool-calling model | ✅ `llama3.1:8b` (deepseek-coder-v2 doesn’t support tools) |
| 30 agent tools | ✅ read/edit/write/run/grep/git/ssh/docker/proxmox/http/systemctl/scan |
| `edit_file` | ✅ Exact search/replace — same as Warp/Oz; no full-file rewrites |
| `fix it` context | ✅ Auto git-status + cargo check/tsc injected before every request |
| Warp BlockContext | ✅ OSC 133 captures command+output+exitCode into recentBlocks |
| Tool results persist | ✅ Tool outputs baked into final Redux message (not discarded) |
| AI overlay | ✅ Floats over terminal, translucent, × dismiss |

### Screen Vision (Phase 1.5)
| Feature | Status |
|---------|--------|
| Screen capture | ✅ `scrap` + 200ms compositor flush fix for GNOME X11 |
| Vision model | ✅ `llama3.2-vision:11b` via Ollama `/api/chat` |
| OCR | ✅ Via vision model (no Tesseract dependency) |
| 📸 camera button | ✅ In UDI — captures screen + describes in 2-3 sentences |
| Agent `screenshot` tool | ✅ Agent can autonomously call screenshot to see the screen |
| Warp classifier port | ✅ `crates/input_classifier` + `natural_language_detection` word lists |

---

## 🚀 Phase 2 — Next Development

### Priority 1: Predictive Command Engine
- **Next-command suggestions**: after each command completes, `llama3.1:8b` predicts likely next command (ghost text in input)
- **History-aware**: uses recent commands + cwd + git status as context
- **Inline ghost text**: render suggestion in gray in the input field, Tab to accept
- **Files**: `agent.rs` new tool `predict_next_command`, `TerminalWithAI.tsx` ghost text renderer

### Priority 2: Advanced RAG
- **Codebase indexing**: `nomic-embed-text:latest` (already installed) embeds files into vector store
- **Context injection**: relevant code snippets automatically added to agent context
- **Status**: `ragService.ts` and `local_recall.rs` already scaffolded, need real embedding pipeline
- **Files**: `src/services/ragService.ts`, `src-tauri/src/local_recall.rs`

### Priority 3: Autonomous Debugging
- **Error intercept**: when a shell command exits non-zero, offer `Fix this?` prompt
- **Self-healing loop**: agent reads error → edits file → runs build → iterates until pass
- **Already partially works** via `fix it` + `edit_file` + `run_cmd`
- **Needs**: error block UI trigger, progress indication, iteration limit display

### Priority 4: Multi-Agent Coordination
- **Parallel agents**: spawn N agents on different tasks simultaneously
- **Agent-to-agent messaging**: shared context, result aggregation
- **Architecture**: Tauri event bus already supports multiple session IDs

### Priority 5: Warp Drive equivalent
- **Snippets**: save frequently used commands/prompts
- **Notebooks**: persistent AI conversation sessions
- **Rules**: persistent context injected into every agent request

---

## 🚀 Phase 2 Complete — 2026-06-09

### Model & Capability Upgrades
| Feature | Status |
|---------|--------|
| `codestral:22b` agent | ✅ 12GB Mistral code model, fits RTX 4080 VRAM exactly |
| `llama3.3:70b` deep mode | ✅ `**` prefix routes to 42GB model |
| `**` deep mode prefix | ✅ Complex tasks use the most capable local model |
| `@file` injection | ✅ `@src/agent.rs` injects file content into query |
| `>` Open WebUI routing | ✅ Routes through Open WebUI OpenAI-compatible API |
| Session memory | ✅ AI conversation persists in localStorage across restarts |
| MAX_STEPS 50 | ✅ More thorough multi-step fixes |
| num_predict 8192 | ✅ Longer, more complete responses |

### Agent Reliability
| Feature | Status |
|---------|--------|
| Self-correction | ✅ Step 0 text + 0 tools → inject 'call the tools now' |
| Pre-flight scan | ✅ `scan/fix/check` requests run cargo check + npm build first |
| System optimize | ✅ Executes in Rust (sysctl, swapoff, drop_caches), model summarizes real results |
| Routing fix | ✅ isShellCommand() fallback from Warp input.rs (no more blind shell default) |
| No sudo password | ✅ Removed destructive Ollama restart on startup |

### New Tools (32 total)
| Tool | Description |
|------|-------------|
| `web_search` | DuckDuckGo + SearXNG if `SEARXNG_URL` set |
| `mcp_call` | JSON-RPC to any MCP server (HA, Nextcloud, Obsidian) |
| `screenshot` | Screen capture + `llama3.2-vision:11b` analysis |
| `predict_command` | History + `llama3.2:3b` ghost text prediction |

### Phase 3 — Next
- RAG: `nomic-embed-text:latest` embeddings, codebase indexing
- Voice input: whisper/STT integration
- Multi-agent: parallel agents on separate tasks
- SearXNG: install natively in CT-300 for web search
- Open WebUI: finish native install in CT-300
- MCP: wire Home Assistant, n8n, Nextcloud MCP servers

## Desktop Launchers
| Entry | Path | Behavior |
|-------|------|----------|
| NexusTerminal | `~/.local/share/applications/nexus-terminal.desktop` | Launches Tauri app directly (clean) |
| NexusTerminal (Debug) | `~/.local/share/applications/nexus-terminal-debug.desktop` | Runs inside Konsole — shows all Rust/Tauri/Ollama logs |

Icon: `src-tauri/icons/128x128.png` — used by both entries.

## Phase 3 — Production Hardening (2026-06-09)

### CTD Fixes (5 crash causes eliminated)
| Bug | Root Cause | Fix |
|-----|-----------|-----|
| CTD on any agent request | `gather_project_context` ran `cargo check`+`npm build` on every request — OOM'd by compiling running app in-process | Removed build checks from context function entirely |
| CTD on scan/optimize | `pkexec` fallback spawned polkit GUI dialogs that crashed GNOME desktop session | Replaced with graceful no-sudo message; sudoers file created |
| Panic in @file handling | `regex::Regex::new().unwrap()` | Fixed to `match` with early return |
| Panic if cwd=/ | `.parent().unwrap()` | Fixed to `.map().unwrap_or_default()` |
| Image @ref crashed app | 4 MB PNG base64 → VRAM OOM (codestral 12 GB + vision 8 GB > 16 GB RTX 4080) | Resize to ≤1280 px JPEG (~120 KB) before encoding; unload text model before vision; `keep_alive:0` on vision to free VRAM immediately |

### Agent CWD — Now Tracks Real Shell Directory
- `terminal.rs`: parses OSC 7 (`\x1b]7;file://host/path\x07`) from PTY stream on every fish prompt
- `TerminalWithAI.tsx`: `terminal-cwd` listener dispatches `updateTabWorkingDirectory` — cwd chip and agent always show real location

### System Optimization — Now Actually Works
- `/etc/sudoers.d/nexus-terminal`: NOPASSWD for `sysctl`, `swapoff`, `swapon`, `tee`
- Uses `echo 1 | sudo tee /proc/sys/vm/drop_caches` (matches sudoers, no shell escaping)
- No more pkexec, no more desktop crashes

## Test Suite (2026-06-09)

### Rust — 50 tests, 0 failures (`cargo test`)
| Module | Tests | What is covered |
|--------|-------|-----------------|
| `terminal::tests` | 7 | `extract_osc7_cwd`: fish format, embedded output, root path, percent-encoded spaces, absent, malformed, deep path |
| `vision_commands::tests` | 7 | `resize_for_vision`: 1920×1080 → 1280×720, portrait, small unchanged, boundary, invalid data, JPEG output, size reduction |
| `tests` (main.rs) | 6 | `is_scan_fix_request` / `is_system_optimize_request`: positive/negative cases, no-double-trigger guard |
| `collaboration::tests` | 3 | session creation, join session, manager creation |
| `workflow_automation::tests` | 3 | engine creation, workflow, execution order |
| `analytics`, `cloud`, `command_flow`, etc. | 24 | existing module tests |

### TypeScript — 49 tests, 0 failures (`npm test`)
| Suite | Tests | What is covered |
|-------|-------|-----------------|
| `translateNLToShell` | 15 | NL phrases → exact shell commands, unrecognised returns null |
| `isShellCommand — shell` | 16 | Known commands, flags, pipes, env vars, paths |
| `isShellCommand — NL` | 11 | Questions, help requests, NL phrases correctly rejected |
| `isShellCommand — edge` | 3 | Empty, whitespace, bare `?` |
| `CommandRoutingService` | 3 | Instance method, systemctl, ping, question mark |

### Bugs fixed during test writing
| Bug | Fix |
|-----|-----|
| `broadcast_event` panicked with no subscribers | Changed `send().map_err()?` to `let _ = send()` |
| `get_execution_order` returned reversed topological order | Removed incorrect `order.reverse()` call |
| `isShellCommand("?")` returned `true` (shell) | Strip trailing `?,!.,` before pattern checks — mirrors Warp parser.rs line 32 (`WordDelimiter::Separator`) |

## Phase 3 — ollama-code-checker Integration (2026-06-12)

### Fixes & Improvements from standalone ollama-code-checker study
| Fix | Root Cause | Result |
|-----|-----------|--------|
| Question card never appeared on dir scan | `"\u274c"` is invalid Rust escape syntax; app compiled with old binary | Fixed: use literal `'❌'` char — card now fires unconditionally |
| Single-file AI analysis model refusals | Soft prompt let models say "please provide code" | Fixed: forceful prefix + 13-phrase unhelpful-response detection + automatic retry |
| `autofix_code` only handled single files | Directory path returned error | Fixed: directory mode finds all code files recursively, large-codebase guard (>20 files → ask user) |
| Autofix prompt too vague | Old: "Fix all issues" | New: explicit list (syntax, unused imports, commented-out code, style, performance) |

## Remaining Known Issues
- OSC 133 hooks disabled (removed to fix display noise)
- `NewTabModal.tsx` / `TerminalTabManager.tsx` placeholder components still unused
- Open WebUI pip install pending in CT-300 (large deps)

---

## Key File Locations

| Purpose | Path |
|---------|------|
| Rust backend | `src-tauri/src/` |
| Tauri commands entry | `src-tauri/src/main.rs` |
| PTY/terminal management | `src-tauri/src/terminal.rs` |
| AI service | `src-tauri/src/ai.rs` |
| Frontend services | `src/services/` |
| Terminal components | `src/components/terminal/` |
| AI components | `src/components/ai/` |
| This file | `packages/nexus-terminal/PROGRESS.md` |
