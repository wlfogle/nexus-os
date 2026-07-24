// prediction.rs — Autosuggest and next-command prediction for NexusTerminal.
//
// Ported from warpdotdev/warp (AGPL-3.0): https://github.com/warpdotdev/warp
//
// Two-tier API:
//
//   autosuggest()      — synchronous, instant (history + PATH executables).
//                         Call this on every keystroke for inline ghost-text.
//
//   predict_command()  — async, AI-enhanced (history → PATH → Ollama LLM).
//                         Call this when the user pauses or presses Tab.
//
// Pipeline inside predict_command():
//   1. History match  — O(n) scan of recent commands by prefix (instant)
//   2. PATH match     — scan PATH executables for prefix match (fast sync IO)
//   3. AI prediction  — llama3.2:3b completes or suggests the next command
//                       (only if history + PATH have no strong match)

use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

const PREDICT_MODEL: &str = "llama3.2:3b";
const MAX_PREDICT_TOKENS: u32 = 40; // Short prediction — just a command, not an essay

/// Predict the next / completed command.
///
/// `partial_input` — what the user has typed so far (may be empty)
/// `history`       — recent commands, newest first
/// `cwd`           — current working directory
/// `ollama_url`    — Ollama base URL
///
/// Returns a full command string, or None if no confident prediction.
pub async fn predict_command(
    partial_input: &str,
    history: &[String],
    cwd: &str,
    ollama_url: &str,
) -> Option<String> {
    let partial = partial_input.trim();

    // ── Stage 1: history match (instant, no LLM) ──────────────────────────────
    if !partial.is_empty() {
        // Count frequency of matching history entries
        let mut freq: HashMap<&str, usize> = HashMap::new();
        for cmd in history {
            let cmd = cmd.trim();
            if cmd.starts_with(partial) && cmd.len() > partial.len() {
                *freq.entry(cmd).or_insert(0) += 1;
            }
        }
        if !freq.is_empty() {
            // Return most frequent match
            let best = freq.into_iter().max_by_key(|(_, count)| *count);
            if let Some((cmd, _)) = best {
                return Some(cmd.to_string());
            }
        }
    }

    // ── Stage 2: AI prediction via llama3.2:3b ────────────────────────────────
    let client = match Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
    {
        Ok(c) => c,
        Err(_) => return None,
    };

    let recent = history.iter().take(10).cloned().collect::<Vec<_>>().join("\n");

    let prompt = if partial.is_empty() {
        // Predict the NEXT command based on context
        format!(
            "You are a shell command predictor. Based on the recent commands and current directory, predict the single most likely NEXT command the user will run.\n\nDirectory: {}\nRecent commands (newest first):\n{}\n\nRespond with ONLY the exact shell command, nothing else. No explanation. No markdown.",
            cwd, recent
        )
    } else {
        // Complete the partial command
        format!(
            "You are a shell command predictor. Complete this partial command: `{}`\n\nDirectory: {}\nRecent commands: {}\n\nRespond with ONLY the complete shell command. No explanation. No markdown.",
            partial, cwd, recent
        )
    };

    let body = serde_json::json!({
        "model": PREDICT_MODEL,
        "messages": [{ "role": "user", "content": prompt }],
        "stream": false,
        "options": {
            "temperature": 0.1,   // Low temp = deterministic, likely correct
            "num_predict": MAX_PREDICT_TOKENS,
            "stop": ["\n", ";", "&&", "||"]  // Stop at command boundary
        }
    });

    let url = format!("{}/api/chat", ollama_url);
    let resp = match client.post(&url).json(&body).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return None,
    };

    let data: serde_json::Value = match resp.json().await {
        Ok(d) => d,
        Err(_) => return None,
    };

    let prediction = data["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .filter(|s| {
            // Sanity check: must look like a shell command, not prose
            let first_word = s.split_whitespace().next().unwrap_or("");
            !first_word.is_empty() && first_word.len() < 30 && !s.contains('\n')
        })?;

    // If predicting completion, ensure it starts with the partial input
    if !partial.is_empty() && !prediction.starts_with(partial) {
        return None;
    }

    Some(prediction)
}

// ─── PATH-based executable completions ────────────────────────────────────────

/// Scan every directory in `$PATH` for executables whose names start with
/// `partial`.  Results are sorted alphabetically and deduplicated.
///
/// Only considers the first word of `partial`; returns an empty list when
/// `partial` is empty, contains a `/` (absolute path), or contains a space
/// (already past the command word).
///
/// This is a synchronous filesystem scan; it is fast enough for keystroke
/// latency on a typical PATH (< 1 ms) and does not need `spawn_blocking`.
#[cfg(unix)]
pub fn get_path_completions(partial: &str) -> Vec<String> {
    use std::os::unix::fs::PermissionsExt;

    if partial.is_empty() || partial.contains('/') || partial.contains(' ') {
        return Vec::new();
    }

    let path_env = std::env::var("PATH").unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    let mut matches: Vec<String> = Vec::new();

    for dir in path_env.split(':') {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = match entry.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };
            if !name.starts_with(partial) || !seen.insert(name.clone()) {
                continue;
            }
            // Only include files with at least one executable bit set.
            if let Ok(meta) = entry.metadata() {
                if meta.permissions().mode() & 0o111 != 0 {
                    matches.push(name);
                }
            }
        }
    }

    matches.sort();
    matches
}

/// Stub for non-Unix targets (Windows is not a target platform for NexusOS).
#[cfg(not(unix))]
pub fn get_path_completions(_partial: &str) -> Vec<String> {
    Vec::new()
}

// ─── Synchronous autosuggest (history + PATH) ─────────────────────────────

/// Inline ghost-text autosuggest: returns up to `limit` candidate completions
/// for `partial`, combining recent shell history (prefix-matched, deduplicated)
/// with PATH executables.
///
/// Results are ordered: most-recently-used history matches first, then
/// alphabetically-sorted PATH matches.
///
/// This function is **synchronous** and safe to call on every keystroke
/// without spawning a task.  For AI-enhanced prediction call
/// [`predict_command`] instead.
pub fn autosuggest(partial: &str, history: &[String], limit: usize) -> Vec<String> {
    let partial = partial.trim();
    let limit = limit.max(1);
    let mut seen = std::collections::HashSet::new();
    let mut results: Vec<String> = Vec::with_capacity(limit);

    // Stage 1: history prefix-match (most recent first, as supplied by caller).
    for cmd in history.iter() {
        if results.len() >= limit {
            break;
        }
        let cmd = cmd.trim();
        if cmd.starts_with(partial) && !cmd.is_empty() && seen.insert(cmd.to_string()) {
            results.push(cmd.to_string());
        }
    }

    // Stage 2: PATH executable completions (single-word partial only).
    if results.len() < limit {
        for exe in get_path_completions(partial) {
            if results.len() >= limit {
                break;
            }
            if seen.insert(exe.clone()) {
                results.push(exe);
            }
        }
    }

    results
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autosuggest_history_prefix_match() {
        let history = vec![
            "git status".to_string(),
            "git diff".to_string(),
            "cargo build".to_string(),
        ];
        let suggestions = autosuggest("git", &history, 10);
        // Both git commands should appear (most recent first).
        assert!(suggestions.contains(&"git status".to_string()));
        assert!(suggestions.contains(&"git diff".to_string()));
        // Non-matching command must not appear.
        assert!(!suggestions.contains(&"cargo build".to_string()));
    }

    #[test]
    fn autosuggest_deduplicates_history() {
        let history = vec![
            "ls -la".to_string(),
            "ls -la".to_string(),
            "ls -la".to_string(),
        ];
        let suggestions = autosuggest("ls", &history, 10);
        assert_eq!(suggestions.len(), 1);
    }

    #[test]
    fn autosuggest_respects_limit() {
        let history: Vec<String> = (0..50).map(|i| format!("cmd_{}", i)).collect();
        let suggestions = autosuggest("cmd", &history, 5);
        assert_eq!(suggestions.len(), 5);
    }

    #[test]
    fn autosuggest_empty_partial_returns_history_prefix() {
        let history = vec!["echo hello".to_string()];
        // An empty partial matches every history entry.
        let suggestions = autosuggest("", &history, 10);
        assert!(suggestions.contains(&"echo hello".to_string()));
    }

    #[cfg(unix)]
    #[test]
    fn path_completions_finds_ls() {
        // `ls` must exist on any sane Linux system.
        let completions = get_path_completions("ls");
        assert!(
            completions.iter().any(|e| e == "ls"),
            "expected 'ls' in PATH completions, got: {:?}",
            &completions[..completions.len().min(10)],
        );
    }

    #[cfg(unix)]
    #[test]
    fn path_completions_rejects_empty_and_slashes() {
        assert!(get_path_completions("").is_empty());
        assert!(get_path_completions("/usr/bin/ls").is_empty());
        assert!(get_path_completions("ls -la").is_empty());
    }
}
