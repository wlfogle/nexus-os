/// NexusTerminal Input Classifier
/// Ported from Warp's open-source heuristic classifier (AGPL-3.0).
/// Source: https://github.com/warpdotdev/warp (crates/input_classifier, crates/natural_language_detection)
///
/// Classifies terminal input as either a shell command or natural language (AI query).
/// Uses the same two-stage approach as Warp:
///   1. One-off keyword allowlists (instant decision)
///   2. Token scoring: English word dict + shell syntax detection + PATH lookup

use std::collections::HashSet;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

// ── Word lists (from Warp's natural_language_detection crate) ────────────────

static ENGLISH_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    include_str!("../words.txt")
        .lines()
        .filter(|l| !l.is_empty())
        .collect()
});

static STACKOVERFLOW_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    include_str!("../stack_overflow.txt")
        .lines()
        .filter(|l| !l.is_empty())
        .collect()
});

// ── One-off allowlists (from Warp's input_classifier/src/util.rs) ──────────────

static ONE_OFF_NL_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "hello", "hi", "hey", "hola", "thanks", "explain", "yes", "no",
        "what", "nice", "please", "help", "show", "list", "find", "search",
        "create", "build", "write", "fix", "debug", "analyze", "check",
        "install", "remove", "update", "describe", "summarize", "generate",
    ])
});

/// NL verb prefixes — if the first word is one of these, the whole phrase is natural language.
/// "list files", "show processes", "check disk" etc. are all AI queries.
static NL_VERB_PREFIXES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "list", "show", "display", "check", "get", "find", "search",
        "describe", "explain", "what", "how", "why", "when", "where",
        "fix", "debug", "help", "tell", "give", "make", "create", "write",
        "analyze", "analyze", "review", "summarize", "generate", "run",
        "start", "stop", "restart", "install", "update", "remove",
    ])
});

static ONE_OFF_SHELL_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "#", "echo", "man", "sudo", "claude", "codex", "gemini",
        "ls", "cd", "pwd", "mkdir", "rm", "cp", "mv", "cat", "grep",
        "chmod", "chown", "ps", "kill", "top", "df", "du",
        "git", "docker", "kubectl", "npm", "yarn", "cargo", "pip",
        "ssh", "curl", "wget", "tar", "zip", "unzip", "apt", "nala",
        "systemctl", "journalctl", "bash", "sh", "fish", "zsh",
    ])
});

// ── Shell syntax characters (from Warp's natural_language_detection/src/lib.rs) ─

fn has_shell_syntax(token: &str) -> bool {
    !token.contains(' ')
        && token.contains(['$', '=', '{', '}', '[', ']', '>', '<', '*', '~', '&', '(', ')', '|', '/', '-'])
}

fn wrapped_in_quotes(token: &str) -> bool {
    (token.starts_with('"') && token.ends_with('"'))
        || (token.starts_with('\'') && token.ends_with('\''))
}

// ── Thresholds (from Warp's heuristic_classifier/mod.rs) ────────────────────

const DETECT_AS_NL_THRESHOLD: f32 = 0.6;
const DETECT_AS_NL_LOW_TOKEN_THRESHOLD: f32 = 0.8;
const DETECT_AS_SHELL_THRESHOLD: f32 = 0.5;

// ── Public API ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InputType {
    Shell,
    NaturalLanguage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyResult {
    pub input_type: InputType,
    pub confidence: f32,
    pub reason: String,
}

/// Classify a terminal input string as shell command or natural language.
/// Ported from Warp's HeuristicClassifier pipeline.
pub async fn classify(input: &str) -> ClassifyResult {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return ClassifyResult {
            input_type: InputType::Shell,
            confidence: 1.0,
            reason: "empty input".to_string(),
        };
    }

    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let first = tokens[0].to_lowercase();

    // ── Stage 1: one-off shell keyword ─────────────────────────────────────
    if ONE_OFF_SHELL_KEYWORDS.contains(first.as_str()) {
        return ClassifyResult {
            input_type: InputType::Shell,
            confidence: 0.95,
            reason: format!("one-off shell keyword: {}", first),
        };
    }

    // ── Stage 1b: shell structural patterns ────────────────────────────────
    if trimmed.starts_with("./") || trimmed.starts_with('/') || trimmed.starts_with('~') {
        return ClassifyResult {
            input_type: InputType::Shell,
            confidence: 0.95,
            reason: "starts with path prefix".to_string(),
        };
    }
    if trimmed.contains(" | ") || trimmed.contains(" > ") || trimmed.contains(" < ") || trimmed.contains(" && ") || trimmed.contains(" || ") {
        return ClassifyResult {
            input_type: InputType::Shell,
            confidence: 0.95,
            reason: "contains shell operators".to_string(),
        };
    }
    if trimmed.starts_with("VAR=") || (trimmed.contains('=') && !trimmed.contains(' ')) {
        return ClassifyResult {
            input_type: InputType::Shell,
            confidence: 0.9,
            reason: "variable assignment".to_string(),
        };
    }

    // ── Stage 1c: NL verb prefix — "list files", "show processes", "check disk" etc.
    // If first word is a known NL verb AND input has no shell syntax, it's AI.
    if NL_VERB_PREFIXES.contains(first.as_str()) {
        // Only override if there are no flags/paths in the rest
        let has_shell_args = tokens.iter().skip(1).any(|t|
            t.starts_with('-') || t.starts_with('/') || t.starts_with('~') || t.contains('.')
        );
        if !has_shell_args {
            return ClassifyResult {
                input_type: InputType::NaturalLanguage,
                confidence: 0.88,
                reason: format!("NL verb prefix '{}' with no shell flags", first),
            };
        }
    }
    // One-off single-word NL
    if tokens.len() == 1 && ONE_OFF_NL_WORDS.contains(first.as_str()) {
        return ClassifyResult {
            input_type: InputType::NaturalLanguage,
            confidence: 0.9,
            reason: format!("one-off natural language word: {}", first),
        };
    }

    // ── Stage 1d: question mark = always AI ────────────────────────────────
    if trimmed.ends_with('?') {
        return ClassifyResult {
            input_type: InputType::NaturalLanguage,
            confidence: 0.98,
            reason: "ends with question mark".to_string(),
        };
    }

    // ── Stage 2: PATH lookup for first token ────────────────────────────────
    let first_token_is_command = is_installed_command(&first).await;
    if first_token_is_command && tokens.len() < 3 {
        return ClassifyResult {
            input_type: InputType::Shell,
            confidence: 0.88,
            reason: format!("executable found in PATH: {}", first),
        };
    }

    // ── Stage 3: token scoring (Warp's natural_language_words_score) ────────
    let score = natural_language_score(&tokens, first_token_is_command);
    let total = tokens.len();

    // For short inputs (2-3 tokens) lower the bar — 50%+ NL words is enough.
    // Warp uses 1.0 only when there are <=2 tokens AND they include a known command.
    // We're more liberal: any multi-word phrase that looks English is AI.
    let threshold = if total <= 2 {
        0.5   // "list files" → 1/2 = 0.5 → AI ✔
    } else if total <= 4 {
        DETECT_AS_NL_LOW_TOKEN_THRESHOLD  // 0.8
    } else {
        DETECT_AS_NL_THRESHOLD  // 0.6
    };

    let nl_ratio = score.nl_count as f32 / total as f32;
    let shell_ratio = score.shell_count as f32 / total as f32;

    if nl_ratio >= threshold {
        return ClassifyResult {
            input_type: InputType::NaturalLanguage,
            confidence: nl_ratio.min(0.95),
            reason: format!("{}/{} tokens are natural language words", score.nl_count, total),
        };
    }

    if first_token_is_command || shell_ratio >= DETECT_AS_SHELL_THRESHOLD {
        return ClassifyResult {
            input_type: InputType::Shell,
            confidence: (0.5 + shell_ratio * 0.4).min(0.9),
            reason: format!("shell command tokens detected ({}/{})", score.shell_count, total),
        };
    }

    // ── Stage 4: long input with NL structure = AI ──────────────────────────
    if total >= 4 {
        return ClassifyResult {
            input_type: InputType::NaturalLanguage,
            confidence: 0.75,
            reason: format!("4+ word input without clear shell structure"),
        };
    }

    // Default: shell (safer for short unrecognized input)
    ClassifyResult {
        input_type: InputType::Shell,
        confidence: 0.6,
        reason: "default: short unclassified input treated as shell".to_string(),
    }
}

struct TokenScore {
    nl_count: usize,
    shell_count: usize,
}

/// Port of Warp's natural_language_words_score.
fn natural_language_score(tokens: &[&str], first_is_command: bool) -> TokenScore {
    let mut nl_count: usize = 0;
    let mut shell_count: usize = 0;

    for (i, token) in tokens.iter().enumerate() {
        let lower = token.to_lowercase();
        let t = lower.as_str();

        // Skip first token if it's a command (same logic as Warp)
        if i == 0 && first_is_command {
            continue;
        }

        if STACKOVERFLOW_WORDS.contains(t) || ONE_OFF_SHELL_KEYWORDS.contains(t) {
            nl_count += 1;
        } else if ENGLISH_WORDS.contains(t) {
            nl_count += 1;
        } else if !wrapped_in_quotes(t) && has_shell_syntax(t) {
            shell_count += 1;
            if nl_count > 0 {
                nl_count -= 1;
            }
        }
    }

    TokenScore { nl_count, shell_count }
}

/// Check if a word is an executable in PATH.
async fn is_installed_command(cmd: &str) -> bool {
    if cmd.is_empty() || cmd.contains('/') {
        return false;
    }
    match tokio::time::timeout(
        std::time::Duration::from_millis(200),
        tokio::process::Command::new("sh")
            .arg("-c")
            .arg(format!("command -v '{}' 2>/dev/null", cmd))
            .output(),
    )
    .await
    {
        Ok(Ok(out)) => !out.stdout.is_empty(),
        _ => false,
    }
}
