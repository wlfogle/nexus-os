// NexusTerminal — Input Classifier
// Ported from Warp's open-source heuristic classifier (AGPL-3.0).
// Source: https://github.com/warpdotdev/warp
//   crates/input_classifier  (parser.rs, util.rs, heuristic_classifier/mod.rs)
//   crates/natural_language_detection  (lib.rs, word_list.rs)
//
// Classifies terminal input as shell command or natural language (AI query).
// Algorithm matches Warp's HeuristicClassifier:
//   1. One-off allowlists (instant decisions)
//   2. is_likely_shell_command: token-description / PATH + shell-syntax ratio
//   3. Two-pass natural-language word-score heuristic
//
// NOTE: Warp's production code uses rust_stemmers (Porter) before dictionary
// lookup. Since we cannot modify Cargo.toml here, we apply a lightweight suffix
// approximation instead. The integrator should add `rust_stemmers = "1.0"` to
// Cargo.toml and replace `approximate_stem` with a proper Stemmer call.

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

/// Commands that appear in both the StackOverflow tag list and common shell usage.
/// Ported verbatim from Warp's natural_language_detection/stack_overflow_overlap_command.txt.
static COMMAND_LIST: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from([
        "python", "git", "docker", "sqlite", "npm", "curl", "ssh", "time",
        "sed", "vim", "apt", "awk", "diff", "terraform", "zip", "cat",
        "tail", "sftp", "rsync", "chmod", "grep", "gzip", "ls", "zsh",
        "mkdir", "ping", "mount", "openssl", "printf", "scp", "sh", "tar",
        "tcpdump", "unzip", "vi", "xargs", "xcodebuild", "xpath", "yacc",
        "syslog", "alias", "pip", "rspec", "tree", "ftp", "pytest", "conda",
        "ipython", "eslint", "jq", "llvm", "touch", "echo", "screen",
        "kubectl", "psql", "bazel", "vercel", "sudo", "minikube", "nvm",
        "tmux", "rvm", "go", "flask", "nginx", "svn", "cron", "jvm",
        "ffmpeg", "find", "less", "adb", "sleep", "sqlplus", "wget", "glob",
        "windbg", "asterisk", "daemon", "rpm", "bison", "free", "paste",
        "iptables", "yum", "lint", "super", "kill", "watch", "dump", "epoch",
        "scons", "attr", "sqlcmd", "hostname", "autoconf", "automake", "sys",
        "dot", "tslint", "ngrok", "robocopy", "goto",
    ])
});

// ── One-off allowlists (from Warp's input_classifier/src/util.rs) ─────────────
// Copied VERBATIM — do not add custom entries.

static ONE_OFF_SHELL_COMMAND_KEYWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from(["#", "echo", "man", "sudo", "claude", "codex", "gemini", "agy", "omp"])
});

static ONE_OFF_NATURAL_LANGUAGE_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    HashSet::from(["hello", "hi", "hey", "hola", "thanks", "explain", "yes", "no", "what", "nice", "1. "])
});

// ── Reserved keywords that are NOT skipped even when first token is a command ─
const RESERVED_KEYWORDS: &[&str] = &["what"];

// ── Characters that complete an in-progress token (end delimiter) ─────────────
const END_TOKEN_COMPLETE_KEYS: &[char] = &[' ', '?', '!', '.', '"', ','];

// ── NL detection thresholds (from Warp's heuristic_classifier/mod.rs) ────────
/// When token count ≤ 3: require ALL non-command tokens to be NL words.
const NL_THRESHOLD_ALL: f32 = 1.0;
/// When token count is 4: 80% must be NL words.
const DETECT_AS_NATURAL_LANGUAGE_LOW_TOKEN_THRESHOLD: f32 = 0.8;
/// When token count ≥ 5: 60% must be NL words.
const DETECT_AS_NATURAL_LANGUAGE_THRESHOLD: f32 = 0.6;

// ── Shell detection thresholds (from Warp's input_classifier/src/util.rs) ────
/// When token count ≤ 2: require ALL tokens to have shell descriptions.
const SHELL_THRESHOLD_ALL: f32 = 1.0;
/// When token count is 3–4: 70% must have shell descriptions.
const DETECT_AS_COMMAND_LOW_TOKEN_THRESHOLD: f32 = 0.7;
/// When token count ≥ 5: 50% must have shell descriptions.
const DETECT_AS_COMMAND_THRESHOLD: f32 = 0.5;

// ── Minimum token counts to trigger switching ─────────────────────────────────
const MINIMUM_COMMAND_DETECTION_TOKEN_LENGTH: usize = 2;
const MINIMUM_NATURAL_LANGUAGE_DETECTION_TOKEN_LENGTH: usize = 2;

// ── Public API ────────────────────────────────────────────────────────────────

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

/// Classify a terminal input string as Shell or NaturalLanguage.
///
/// Mirrors Warp's `HeuristicClassifier::detect_input_type`:
///   1. Single-token one-off NL word → AI
///   2. `is_likely_shell_command` → Shell
///   3. Two-pass NL word-score heuristic (incl. last token, excl. last token)
///
/// The caller is `classify_input` Tauri command in main.rs — do not change the
/// function signature.
pub async fn classify(input: &str) -> ClassifyResult {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return ClassifyResult {
            input_type: InputType::Shell,
            confidence: 1.0,
            reason: "empty input".to_string(),
        };
    }

    // Parse into tokens using Warp's SentenceParser (handles quotes, contractions).
    let word_tokens = parse_query_into_tokens(trimmed);
    let total_word_token_count = word_tokens.len();

    // Punctuation-only input (e.g. "?") produces no word tokens after parsing.
    if word_tokens.is_empty() {
        return ClassifyResult {
            input_type: InputType::NaturalLanguage,
            confidence: 0.95,
            reason: "punctuation-only input".to_string(),
        };
    }

    // ── Step 1: single-token one-off NL word (or typing prefix) ──────────────
    if total_word_token_count == 1 {
        let lower = word_tokens[0].to_lowercase();
        if is_one_off_natural_language_word_or_prefix(&lower) {
            return ClassifyResult {
                input_type: InputType::NaturalLanguage,
                confidence: 1.0,
                reason: format!("one-off natural language word: {}", lower),
            };
        }
    }

    // ── Step 2: is_likely_shell_command ──────────────────────────────────────
    let first_is_installed = is_installed_command(&word_tokens[0].to_lowercase()).await;
    if is_likely_shell_command(&word_tokens, total_word_token_count, first_is_installed).await {
        return ClassifyResult {
            input_type: InputType::Shell,
            confidence: 0.95,
            reason: "shell heuristic: command tokens exceed threshold or first token is installed binary".to_string(),
        };
    }

    // ── Step 3: two-pass NL heuristic ────────────────────────────────────────
    // Pass A: with last token potentially excluded (in-progress typing)
    let result_a = natural_language_detection_heuristic(
        trimmed,
        &word_tokens,
        false, // try excluding last token
        first_is_installed,
    );

    // If pass A says AI, return AI immediately (mirrors Warp's classify_input).
    if matches!(result_a.input_type, InputType::NaturalLanguage) {
        return result_a;
    }

    // Pass B: always include last token
    natural_language_detection_heuristic(
        trimmed,
        &word_tokens,
        true, // always include last token
        first_is_installed,
    )
}

// ── Parser (from Warp's input_classifier/src/parser.rs) ─────────────────────

#[derive(PartialEq, Eq)]
enum WordDelimiter {
    Separator,
    DoubleQuote,
    SingleQuote,
    Backtick,
    Whitespace,
}

fn convert_char_to_delimiter(c: char) -> Option<WordDelimiter> {
    match c {
        '\'' => Some(WordDelimiter::SingleQuote),
        '"' => Some(WordDelimiter::DoubleQuote),
        '`' => Some(WordDelimiter::Backtick),
        ',' | '.' | '!' | '?' => Some(WordDelimiter::Separator),
        c if c.is_whitespace() => Some(WordDelimiter::Whitespace),
        _ => None,
    }
}

/// Parse a sentence into tokens for natural language classification.
/// Ported verbatim from Warp's SentenceParser.
/// Key difference from `split_whitespace`: quoted substrings are kept as single tokens;
/// sentence-ending punctuation (,.!?) terminates a token but is not itself emitted.
fn parse_query_into_tokens(query: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut active_delimiter: Option<WordDelimiter> = None;
    let mut active_token = String::new();
    let mut chars = query.chars().peekable();

    while let Some(c) = chars.next() {
        let delimiter = convert_char_to_delimiter(c);
        let next_delimiter = chars.peek().map(|&nc| convert_char_to_delimiter(nc));

        match &delimiter {
            Some(WordDelimiter::Whitespace) if active_delimiter.is_none() => {
                if !active_token.is_empty() {
                    tokens.push(std::mem::take(&mut active_token));
                }
            }
            Some(WordDelimiter::Separator) if active_delimiter.is_none() => {
                if active_token.is_empty() {
                    continue;
                }
                // Separator in middle of a word → keep as part of token
                let next_is_end = next_delimiter
                    .map(|nd| nd == Some(WordDelimiter::Whitespace))
                    .unwrap_or(true);
                if next_is_end {
                    tokens.push(std::mem::take(&mut active_token));
                } else {
                    active_token.push(c);
                }
            }
            Some(WordDelimiter::DoubleQuote) => {
                let complete_quote = if active_delimiter == Some(WordDelimiter::DoubleQuote) {
                    active_delimiter = None;
                    true
                } else if !active_token.is_empty() || active_delimiter.is_some() {
                    false
                } else {
                    active_delimiter = Some(WordDelimiter::DoubleQuote);
                    false
                };
                active_token.push(c);
                if complete_quote {
                    let token = std::mem::take(&mut active_token);
                    if token != "\"\"" {
                        tokens.push(token);
                    }
                }
            }
            Some(WordDelimiter::Backtick) => {
                let complete_quote = if active_delimiter == Some(WordDelimiter::Backtick) {
                    active_delimiter = None;
                    true
                } else if !active_token.is_empty() || active_delimiter.is_some() {
                    false
                } else {
                    active_delimiter = Some(WordDelimiter::Backtick);
                    false
                };
                active_token.push(c);
                if complete_quote {
                    tokens.push(std::mem::take(&mut active_token));
                }
            }
            Some(WordDelimiter::SingleQuote) => {
                let complete_quote = if active_delimiter == Some(WordDelimiter::SingleQuote) {
                    active_delimiter = None;
                    true
                } else if !active_token.is_empty() || active_delimiter.is_some() {
                    false
                } else {
                    active_delimiter = Some(WordDelimiter::SingleQuote);
                    false
                };
                active_token.push(c);
                if complete_quote {
                    let token = std::mem::take(&mut active_token);
                    if token != "''" {
                        tokens.push(token);
                    }
                }
            }
            _ => active_token.push(c),
        }
    }
    if !active_token.is_empty() {
        tokens.push(active_token);
    }
    tokens
}

// ── is_likely_shell_command (from Warp's input_classifier/src/util.rs) ────────

/// Mirrors Warp's `is_likely_shell_command`.
/// Uses shell-syntax detection on ALL tokens and PATH lookup on the first token as proxies
/// for Warp's completion-engine `token_description`.
async fn is_likely_shell_command(
    word_tokens: &[String],
    word_tokens_count: usize,
    first_is_installed: bool,
) -> bool {
    let mut likely_command_token_count: usize = 0;
    let mut is_first_token_command = false;

    // Use SentenceParser tokens as a proxy for Warp's `parsed_tokens`.
    // In Warp, token_description.is_some() ↔ completion-engine recognition.
    // We approximate: COMMAND_LIST member OR (first token) installed in PATH.
    let total_token_count = word_tokens.len();

    for (idx, token) in word_tokens.iter().enumerate() {
        let lower = token.to_lowercase();
        let t = lower.as_str();

        // First token: one-off shell keyword → immediate return true
        if idx == 0 && ONE_OFF_SHELL_COMMAND_KEYWORDS.contains(t) {
            return true;
        }

        // Proxy for token_description.is_some():
        // true when token is in COMMAND_LIST OR (first token AND installed in PATH)
        let has_description = COMMAND_LIST.contains(t) ||
            (idx == 0 && first_is_installed);

        let has_shell_syntax = check_if_token_has_shell_syntax(token);

        if has_description || has_shell_syntax {
            likely_command_token_count += 1;
        }

        if idx == 0 {
            is_first_token_command = has_description;
        }
    }

    // Threshold mirrors Warp: ≤2 tokens → 1.0, ≤4 → 0.7, else 0.5
    let command_threshold = if total_token_count <= 2 {
        SHELL_THRESHOLD_ALL  // 1.0
    } else if total_token_count <= 4 {
        DETECT_AS_COMMAND_LOW_TOKEN_THRESHOLD  // 0.7
    } else {
        DETECT_AS_COMMAND_THRESHOLD  // 0.5
    };

    // Classify as shell if:
    // 1) Enough shell-description tokens exceed threshold, OR
    // 2) Short input (< 3 word-tokens) and first token is an installed command
    likely_command_token_count >= (total_token_count as f32 * command_threshold) as usize
        || (word_tokens_count < 3 && is_first_token_command)
}

// ── Natural language word-score heuristic (from Warp's natural_language_detection/lib.rs) ─

/// Single pass of the NL heuristic.
/// `include_last_token=false` mirrors Warp's first pass (last token may be in-progress).
fn natural_language_detection_heuristic(
    buffer_text: &str,
    word_tokens: &[String],
    include_last_token: bool,
    is_first_token_command: bool,
) -> ClassifyResult {
    // Warp checks current_input_type to determine minimum token length;
    // we conservatively use MINIMUM_NATURAL_LANGUAGE_DETECTION_TOKEN_LENGTH.
    let min_token_length = MINIMUM_NATURAL_LANGUAGE_DETECTION_TOKEN_LENGTH;
    let word_tokens_count = word_tokens.len();

    if word_tokens_count < min_token_length {
        return ClassifyResult {
            input_type: InputType::Shell,
            confidence: 0.6,
            reason: "too few tokens for NL detection".to_string(),
        };
    }

    let mut tokens: Vec<&str> = word_tokens.iter().map(|s| s.as_str()).collect();

    // Exclude last token when it is incomplete (buffer doesn't end with separator)
    // and we are in the first pass and there are more than 2 tokens.
    let last_token_is_complete = buffer_text
        .chars()
        .last()
        .map(|c| END_TOKEN_COMPLETE_KEYS.contains(&c))
        .unwrap_or(false);
    if !include_last_token && !last_token_is_complete && tokens.len() > 2 {
        tokens.pop();
    }

    let updated_token_count = tokens.len();
    let nl_count = natural_language_words_score(&tokens, is_first_token_command);

    // Threshold mirrors Warp's heuristic_classifier/mod.rs:
    //   ≤ 3 tokens → 1.0  (ALL must be NL)
    //   ≤ 4 tokens → 0.8
    //     else     → 0.6
    let threshold = if updated_token_count <= 3 {
        NL_THRESHOLD_ALL  // 1.0
    } else if updated_token_count <= 4 {
        DETECT_AS_NATURAL_LANGUAGE_LOW_TOKEN_THRESHOLD  // 0.8
    } else {
        DETECT_AS_NATURAL_LANGUAGE_THRESHOLD  // 0.6
    };

    if nl_count >= (updated_token_count as f32 * threshold) as usize {
        ClassifyResult {
            input_type: InputType::NaturalLanguage,
            confidence: (nl_count as f32 / updated_token_count as f32).min(0.95),
            reason: format!(
                "{}/{} tokens are natural language words (threshold {:.0}%)",
                nl_count,
                updated_token_count,
                threshold * 100.0
            ),
        }
    } else {
        ClassifyResult {
            input_type: InputType::Shell,
            confidence: 0.6,
            reason: format!(
                "only {}/{} NL tokens (need {:.0}%)",
                nl_count,
                updated_token_count,
                threshold * 100.0
            ),
        }
    }
}

// ── natural_language_words_score (from Warp's natural_language_detection/lib.rs) ─

/// Returns the count of tokens that are "natural language" words.
/// Mirrors Warp's `natural_language_words_score` (without rust_stemmers; see module note).
fn natural_language_words_score(tokens: &[&str], is_first_token_command: bool) -> usize {
    let mut nl_count: usize = 0;

    for (i, token) in tokens.iter().enumerate() {
        let preprocessed = token_preprocessing(token);
        let t = preprocessed.as_str();

        // Skip first token when it's a known command (mirrors Warp).
        if i == 0
            && (COMMAND_LIST.contains(t)
                || (is_first_token_command && !RESERVED_KEYWORDS.contains(&t)))
        {
            continue;
        }

        if STACKOVERFLOW_WORDS.contains(t) || COMMAND_LIST.contains(t) {
            nl_count += 1;
        } else if ENGLISH_WORDS.contains(t) {
            nl_count += 1;
        } else {
            // Approximate Porter stemming with common English suffix stripping.
            let stemmed = approximate_stem(t);
            let s = stemmed.as_str();
            if ENGLISH_WORDS.contains(s) || STACKOVERFLOW_WORDS.contains(s) || COMMAND_LIST.contains(s) {
                nl_count += 1;
            } else if !wrapped_in_quotes(t) && check_if_token_has_shell_syntax(t) {
                // Shell-syntax token is a negative NL signal (saturating_sub mirrors Warp).
                nl_count = nl_count.saturating_sub(1);
            }
        }
    }

    nl_count
}

// ── Token preprocessing (from Warp's natural_language_detection/lib.rs) ────────

/// Lowercase + contraction expansion. Mirrors Warp's `token_preprocessing`.
fn token_preprocessing(token: &str) -> String {
    let token = token.to_lowercase();
    if token == "can't" {
        return "can".to_string();
    }
    // Strip common English contractions.
    for suffix in &["'s", "'re", "n't", "'t", "'m", "'ve", "'ll"] {
        if let Some(root) = token.strip_suffix(suffix) {
            return root.to_string();
        }
    }
    token
}

/// Lightweight suffix stripper to approximate Porter stemming without the external crate.
/// Handles the most frequent English inflections that appear in test inputs.
fn approximate_stem(word: &str) -> String {
    let w = word.to_string();
    let len = w.len();

    // -ing: "running" → "run", "building" → "build"
    if len > 5 {
        if let Some(root) = w.strip_suffix("ing") {
            // Double final consonant: "runn" → "run"
            if root.len() >= 2 {
                let b = root.as_bytes();
                let n = b.len();
                if n >= 2 && b[n - 1] == b[n - 2] {
                    return root[..n - 1].to_string();
                }
            }
            return root.to_string();
        }
    }

    // -tion → t: "action" → "act"
    if len > 6 {
        if let Some(root) = w.strip_suffix("tion") {
            return format!("{}t", root);
        }
    }

    // -ed: "failed" → "fail", "walked" → "walk"
    if len > 4 {
        if let Some(root) = w.strip_suffix("ed") {
            return root.to_string();
        }
        // -er: "builder" → "build"
        if let Some(root) = w.strip_suffix("er") {
            return root.to_string();
        }
        // -ly: "quickly" → "quick"
        if let Some(root) = w.strip_suffix("ly") {
            return root.to_string();
        }
    }

    // -es: "processes" → "process"
    if len > 4 {
        if let Some(root) = w.strip_suffix("es") {
            return root.to_string();
        }
    }

    // -s (simple plural): "files" → "file"
    if len > 3 {
        if let Some(root) = w.strip_suffix('s') {
            return root.to_string();
        }
    }

    w
}

// ── One-off NL word helpers (from Warp's input_classifier/src/util.rs) ────────

fn is_one_off_natural_language_word(word: &str) -> bool {
    ONE_OFF_NATURAL_LANGUAGE_WORDS.contains(word)
}

/// Returns true when `word` is one of the one-off NL words OR is a typing prefix
/// of one of them (avoids mode-flipping while the user is still typing).
fn is_one_off_natural_language_word_or_prefix(word: &str) -> bool {
    is_one_off_natural_language_word(word)
        || ONE_OFF_NATURAL_LANGUAGE_WORDS
            .iter()
            .any(|nw| nw.starts_with(word))
}

// ── Shell-syntax detector (from Warp's natural_language_detection/lib.rs) ──────

/// Returns true when `word` contains shell special characters.
/// List sourced from https://mywiki.wooledge.org/BashGuide/SpecialCharacters.
fn check_if_token_has_shell_syntax(word: &str) -> bool {
    !word.contains(' ')
        && word.contains(['$', '=', '{', '}', '[', ']', '>', '<', '*', '~', '&', '(', ')', '|', '/', '-'])
}

fn wrapped_in_quotes(word: &str) -> bool {
    (word.starts_with('"') && word.ends_with('"'))
        || (word.starts_with('\'') && word.ends_with('\''))
}

// ── PATH lookup ───────────────────────────────────────────────────────────────

/// Returns true when `cmd` is found in PATH (within a 200 ms timeout).
/// Used as a proxy for Warp's completion-engine `token_description.is_some()`.
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
