/// NexusTerminal Predictive Command Engine
/// Warp-style next-command prediction: history-first (instant) + AI-enhanced (async).
///
/// Pipeline:
///   1. History match  — O(n) scan of recent commands starting with partial_input (instant)
///   2. Frequency rank — prefer commands run more often
///   3. AI prediction  — llama3.2:3b completes the command or suggests the next one
///                       (only if history has no strong match, or input is empty)

use anyhow::Result;
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
