# NexusTerminal — Agent Upgrade Plan

> NexusOS is the world's first AI-driven OS.
> NexusTerminal is its primary interface — the next generation of AI agents.

This document captures every planned upgrade so nothing is lost between sessions.

---

## Status

**Current:** Native Ollama function-calling agent committed (`2389e6b4`). Backend compiles clean. Frontend builds clean.

**Needed:** The items below.

---

## 1. TypeScript Errors (Fix First)

### `src/components/ai/EnhancedAIAssistant.tsx`

**Error 1 — line 56:** `handleInput` imported from `useInputRouting` but never used.
```ts
// REMOVE this line:
const { handleInput } = useInputRouting();
```

**Error 2 — line 312:** `streamingMsgId` ref declared but never used.
```ts
// REMOVE this line:
const streamingMsgId = useRef<string | null>(null);
```

**Error 3 — lines 343 & 380:** `id` passed in `addAIMessage` payload but type is `Omit<AIMessage, 'id'>`.

Fix: use local React state for the streaming buffer instead of dispatching to Redux.
```ts
// Add to component state:
const [streamingContent, setStreamingContent] = useState<string>('');
const [streamingTools, setStreamingTools] = useState<Array<{tool: string; args: string; result?: string}>>([]);

// In agent-token listener: setStreamingContent(prev => prev + payload.token)
// In agent-done listener: dispatch final answer, then setStreamingContent('')
// Remove the `id: sessionId` from addAIMessage calls entirely
```

**Error 4 — `TerminalWithAI.tsx` line 501:** `onSwitchToTerminal` prop passed but not in `EnhancedAIAssistantProps`.
```ts
// Add to interface:
interface EnhancedAIAssistantProps {
  className?: string;
  onSwitchToTerminal?: () => void;  // ADD THIS
}
```

---

## 2. agent.rs — Streaming Bug Fix

**File:** `src-tauri/src/agent.rs` ~line 1094

The `break` inside the `for line in text.lines()` loop only exits the inner for-loop, not the outer `while let Some(chunk_result) = stream.next().await` loop. After `done: true`, the outer loop keeps awaiting new chunks unnecessarily.

**Fix:**
```rust
// Before the while loop:
let mut stream_done = false;

// Inside for line loop, replace `break;` with:
stream_done = true;
break;

// After the for line loop closes (still inside while):
if stream_done { break; }
```

---

## 3. Improved SYSTEM_PROMPT

**File:** `src-tauri/src/agent.rs` — replace `SYSTEM_PROMPT` constant.

```rust
const SYSTEM_PROMPT: &str = r#"You are NexusAI — the autonomous intelligence core of NexusOS, the world's first AI-driven operating system.

You are not a chatbot. You are an agent. You take action.

Capabilities:
- Read, write, search and edit files on the local filesystem
- Execute shell commands and scripts
- Manage Git repositories (status, diff, commit, log, push)
- Control system services via systemctl
- Run commands on remote hosts via SSH
- Query and control Docker containers
- Make HTTP requests to APIs (GET and POST)
- Monitor running processes
- Control Proxmox LXC containers and VMs via SSH
- Edit files with targeted find-and-replace (no full rewrites needed)

Behavior rules:
- Be concise. No filler. No apologies. No preamble.
- Take the shortest path to the correct answer.
- When asked to do something, do it — don't ask permission for obvious steps.
- When you need information, use tools to get it. Don't guess.
- Read files before editing them. Verify changes with run_cmd.
- Never call the same tool twice with the same arguments.
- Give the answer directly. Don't re-summarize your steps.
- Write complete, working code. No stubs. No placeholders.
- You have full access to this machine. Use it."#;
```

---

## 4. New Agent Tools to Add

Add these to `build_tools()` and implement in `exec_tool()`.

### `git_commit`
Stage all and commit.
```json
{
  "path": "string — repo path",
  "message": "string — commit message"
}
```
Implementation: `git -C {path} add -A` then `git -C {path} commit -m {message}`

### `git_log`
Recent commits in oneline format.
```json
{
  "path": "string — repo path",
  "count": "integer — number of commits (default 10)"
}
```
Implementation: `git -C {path} --no-pager log --oneline -{count}`

### `http_post`
POST JSON to an API.
```json
{
  "url": "string",
  "body": "object — JSON body"
}
```
Implementation: `reqwest` POST with `.json(&body)`, returns response body.

### `systemctl_cmd`
Control systemd services.
```json
{
  "action": "string — status/start/stop/restart/enable/disable/list-units",
  "service": "string — service name (optional for list-units)"
}
```
Implementation: `systemctl {action} --no-pager {service}`

### `ssh_exec`
Run a command on a remote host.
```json
{
  "host": "string — SSH host alias or user@host",
  "cmd": "string — command to run"
}
```
Implementation: `ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 {host} {cmd}`

### `edit_file`
Targeted find-and-replace in a file. Use `read_file` first to get exact text.
```json
{
  "path": "string — file path",
  "old": "string — exact text to replace",
  "new": "string — replacement text"
}
```
Implementation: `fs::read_to_string` → `content.replacen(old, new, 1)` → `fs::write`

### `process_list`
List running processes, optionally filtered.
```json
{
  "filter": "string — optional process name filter"
}
```
Implementation: `ps aux | grep {filter}` or `ps aux` if no filter.

### `proxmox_exec`
Run a command in a Proxmox LXC or on the Proxmox host.
```json
{
  "target": "string — 'host' or CT ID like '300'",
  "cmd": "string — command to execute"
}
```
Implementation:
- If target == "host": `ssh tiamat "{cmd}"`
- Else: `ssh tiamat "pct exec {target} -- bash -c '{cmd}'"`

---

## 5. Frontend Streaming UI Improvements

### Live streaming display
Replace the current "AI is thinking..." placeholder with a proper streaming view:
- Show tokens as they arrive in real-time (blinking cursor `█`)
- Show tool calls as expandable cards:
  ```
  🔧 read_file  /path/to/file.rs
  ✓  1204 bytes
  ```
- Show tool results as collapsible code blocks
- Final answer replaces the streaming placeholder

### Keyboard shortcuts
- `Ctrl+K` — focus AI input from terminal
- `Escape` — close AI panel, return to terminal
- `Ctrl+Enter` — send message

### Auto-routing indicator
Show in the input bar which mode is active:
- `>` prefix → shell command
- `?` prefix or natural language → AI agent

### Model indicator
Show currently active model + capability badges (code/vision/fast) in the header.

---

## 6. Dead Code Cleanup (Rust Warnings)

These structs/functions are defined but never used — either wire them up or remove:

| File | Symbol |
|---|---|
| `web_scraper.rs:103` | `ScrapingStats` |
| `web_scraper.rs:115` | `ErrorCount` |
| `vision.rs:72` | `VisionQuery` (already exported in visionService.ts — connect to backend) |
| `security_scanner.rs:74` | `RealtimeSecurityMonitor` |
| `security_scanner.rs:82` | `SecurityAlert` |
| `analytics.rs:314` | `UsagePattern` |
| `analytics.rs:323` | `PatternType` |
| `ollama_config.rs:101` | `install_models_fstab_entry` |

---

## 7. Vision Integration (Phase 2)

The `visionService.ts` already has a full implementation but calls Tauri commands that don't have Rust implementations yet:
- `check_vision_dependencies`
- `capture_screen`
- `perform_ocr`
- `detect_ui_elements`
- `query_vision_ai`

For Phase 2, implement these in a new `src-tauri/src/vision.rs`:
- `capture_screen` → use `scrot` or `spectacle` CLI, return PNG bytes
- `query_vision_ai` → send base64 image to Ollama vision model (llava/moondream)
- `check_vision_dependencies` → verify scrot/tesseract are installed

---

## 8. NexusOS Agent Tool API Contract

When NexusOS is complete, the agent should be callable programmatically:

```typescript
// From any NexusOS module:
const result = await nexusAgent.ask(
  "Why is jellyfin not picking up new files?",
  { context: "tiamat server, CT-300" }
);

// Or with tool execution:
const result = await nexusAgent.execute([
  { tool: "ssh_exec", args: { host: "tiamat", cmd: "pct exec 300 -- systemctl status jellyfin" } },
  { tool: "ssh_exec", args: { host: "tiamat", cmd: "pct exec 300 -- journalctl -u jellyfin -n 50" } }
]);
```

This becomes the IPC bridge between NexusTerminal and all other NexusOS modules.

---

## Execution Order

1. Fix TypeScript errors (unblock clean build)
2. Apply streaming bug fix in agent.rs
3. Replace SYSTEM_PROMPT
4. Add new tools (git_commit, git_log, http_post, systemctl_cmd, ssh_exec, edit_file, process_list, proxmox_exec)
5. Improve streaming UI
6. Clean up dead code warnings
7. Vision integration (Phase 2)
8. NexusOS agent API contract (Phase 3)
