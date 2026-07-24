// blocks.rs — Warp-style command-block lifecycle tracking for NexusTerminal.
//
// Ported from warpdotdev/warp (AGPL-3.0): https://github.com/warpdotdev/warp
//
// Each command executed inside a terminal session is wrapped in a "block".
// Three Tauri events implement the BLOCK-EVENTS CONTRACT (field names / types
// must not change — the frontend depends on them):
//
//   "block:start"  { blockId, terminalId, command, cwd, startedAt }
//   "block:output" { blockId, terminalId, chunk, stream }
//   "block:end"    { blockId, terminalId, exitCode, endedAt, durationMs }
//
// terminal.rs calls into this module when it detects OSC 133 sequences in the
// PTY output stream.  blocks.rs has no dependency on terminal.rs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tracing::{debug, error};
use uuid::Uuid;

// ─── Global singleton ──────────────────────────────────────────────────────

static BLOCK_TRACKER: OnceLock<BlockTracker> = OnceLock::new();

/// Returns the process-wide [`BlockTracker`], creating it on first call.
pub fn get_block_tracker() -> &'static BlockTracker {
    BLOCK_TRACKER.get_or_init(BlockTracker::new)
}

// ─── Internal state ────────────────────────────────────────────────────────

/// In-flight block for one terminal session.
#[derive(Debug, Clone)]
struct ActiveBlock {
    block_id: String,
    terminal_id: String,
    command: String,
    cwd: String,
    started_at: i64, // epoch ms
}

// ─── Event payloads (BLOCK-EVENTS CONTRACT) ────────────────────────────────

/// `block:start` payload.
/// startedAt is Unix epoch in milliseconds (JS `number`-safe i64).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockStartPayload {
    pub block_id: String,
    pub terminal_id: String,
    pub command: String,
    pub cwd: String,
    pub started_at: i64,
}

/// `block:output` payload.
/// stream is always `"stdout"` or `"stderr"`.
/// (PTY merges both streams; terminal.rs currently always passes `"stdout"`.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockOutputPayload {
    pub block_id: String,
    pub terminal_id: String,
    pub chunk: String,
    pub stream: String,
}

/// `block:end` payload.
/// exitCode is `null` when the exit status could not be determined.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockEndPayload {
    pub block_id: String,
    pub terminal_id: String,
    pub exit_code: Option<i32>,
    pub ended_at: i64,
    pub duration_ms: i64,
}

// ─── BlockTracker ──────────────────────────────────────────────────────────

/// Tracks at most one active block per terminal.
///
/// `terminal.rs` drives the lifecycle:
/// 1. Call [`start_block`] when OSC `133;B` is detected (preexec hook fires).
/// 2. Call [`emit_output`] for every PTY chunk while the block is active.
/// 3. Call [`end_block`] when OSC `133;D;N` is detected (exit code N).
///
/// All three methods are no-ops when no block is active for the terminal,
/// making them safe to call spuriously (e.g. on shell startup).
#[derive(Debug)]
pub struct BlockTracker {
    /// terminal_id → currently active block
    active: Mutex<HashMap<String, ActiveBlock>>,
}

impl BlockTracker {
    /// Create a new tracker with no active blocks.
    pub fn new() -> Self {
        Self {
            active: Mutex::new(HashMap::new()),
        }
    }

    /// Start a new block for `terminal_id`.
    ///
    /// Emits `block:start` and stores the block as active.  If a block was
    /// already active for this terminal it is silently replaced (the old
    /// block never receives a `block:end`; this can happen if the shell
    /// exits abruptly without emitting an OSC 133;D sequence).
    ///
    /// Returns the new `blockId`.
    pub fn start_block(
        &self,
        terminal_id: &str,
        command: &str,
        cwd: &str,
        app_handle: &AppHandle,
    ) -> String {
        let block_id = Uuid::new_v4().to_string();
        let started_at = now_ms();

        let block = ActiveBlock {
            block_id: block_id.clone(),
            terminal_id: terminal_id.to_string(),
            command: command.to_string(),
            cwd: cwd.to_string(),
            started_at,
        };

        {
            let mut active = self
                .active
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            active.insert(terminal_id.to_string(), block);
        }

        let payload = BlockStartPayload {
            block_id: block_id.clone(),
            terminal_id: terminal_id.to_string(),
            command: command.to_string(),
            cwd: cwd.to_string(),
            started_at,
        };

        if let Err(e) = app_handle.emit("block:start", &payload) {
            error!("Failed to emit block:start for terminal {}: {}", terminal_id, e);
        }
        debug!(
            "block:start  id={}  terminal={}  cmd={:?}",
            block_id, terminal_id, command
        );

        block_id
    }

    /// Emit `block:output` for the active block on `terminal_id`.
    ///
    /// `stream` should be `"stdout"` or `"stderr"`.
    /// This is a no-op if the chunk is empty or no block is active.
    pub fn emit_output(
        &self,
        terminal_id: &str,
        chunk: &str,
        stream: &str,
        app_handle: &AppHandle,
    ) {
        if chunk.is_empty() {
            return;
        }

        let block_id = {
            let active = self.active.lock().unwrap_or_else(|e| e.into_inner());
            active.get(terminal_id).map(|b| b.block_id.clone())
        };

        let Some(block_id) = block_id else {
            return;
        };

        let payload = BlockOutputPayload {
            block_id,
            terminal_id: terminal_id.to_string(),
            chunk: chunk.to_string(),
            stream: stream.to_string(),
        };

        if let Err(e) = app_handle.emit("block:output", &payload) {
            error!("Failed to emit block:output for terminal {}: {}", terminal_id, e);
        }
    }

    /// End the active block for `terminal_id`.
    ///
    /// Emits `block:end` (with duration computed from `started_at`) and removes
    /// the block from the active map.  No-op if no block is active — this makes
    /// it safe to call from the OSC 133;A (prompt-start) backup path.
    pub fn end_block(
        &self,
        terminal_id: &str,
        exit_code: Option<i32>,
        app_handle: &AppHandle,
    ) {
        let block = {
            let mut active = self.active.lock().unwrap_or_else(|e| e.into_inner());
            active.remove(terminal_id)
        };

        let Some(block) = block else {
            return;
        };

        let ended_at = now_ms();
        let duration_ms = ended_at.saturating_sub(block.started_at);

        let payload = BlockEndPayload {
            block_id: block.block_id.clone(),
            terminal_id: terminal_id.to_string(),
            exit_code,
            ended_at,
            duration_ms,
        };

        if let Err(e) = app_handle.emit("block:end", &payload) {
            error!("Failed to emit block:end for terminal {}: {}", terminal_id, e);
        }
        debug!(
            "block:end  id={}  terminal={}  exit={:?}  duration={}ms",
            block.block_id, terminal_id, exit_code, duration_ms
        );
    }

    /// Returns `true` if there is an active block for `terminal_id`.
    pub fn has_active_block(&self, terminal_id: &str) -> bool {
        self.active
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains_key(terminal_id)
    }
}

impl Default for BlockTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Helpers ───────────────────────────────────────────────────────────────

/// Current time as Unix epoch milliseconds (i64, safe for JS `number`).
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ─── Unit tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_ms_is_positive_and_recent() {
        let t = now_ms();
        // Must be after 2024-01-01 00:00:00 UTC (1704067200000 ms)
        assert!(t > 1_704_067_200_000, "now_ms returned unreasonably small value: {}", t);
    }

    #[test]
    fn block_tracker_no_active_block_initially() {
        let tracker = BlockTracker::new();
        assert!(!tracker.has_active_block("term-1"));
    }
}
