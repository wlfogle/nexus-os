// model_router.rs — NexusTerminal dynamic model router
// Derived from warpdotdev/warp (AGPL-3.0) — model role selection pattern.
// See https://github.com/warpdotdev/warp for the original source.

/// NexusTerminal Model Router
/// Selects the best available local Ollama model for each task type.
/// Priority lists are tuned for the RTX 4080 + 41-model library on this machine.
///
/// Warp does this with named model roles (model, coding_model, cli_agent_model,
/// computer_use_model).  We replicate that pattern locally via env-var overrides
/// and fallback priority chains.
use serde::Deserialize;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tracing::{info, warn};

// ── Task taxonomy ─────────────────────────────────────────────────────────────

/// What the model will be asked to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskKind {
    /// JSON function-calling / tool orchestration.
    ToolUse,
    /// Code generation, rewriting, patching.
    CodeFix,
    /// Code review, audit, security, analysis (read-heavy, high-accuracy).
    CodeAnalysis,
    /// Long-horizon reasoning, planning, complex QA.
    DeepReason,
    /// Vision / screenshot description.
    Vision,
    /// Fast single-turn replies, command explanation, low-latency.
    FastChat,
    /// Vector embedding (no text generation).
    Embed,
}

// ── Available-model cache ──────────────────────────────────────────────────────

struct ModelCache {
    names: Vec<String>,
    refreshed_at: Instant,
}

static CACHE: OnceLock<tokio::sync::RwLock<ModelCache>> = OnceLock::new();

fn cache() -> &'static tokio::sync::RwLock<ModelCache> {
    CACHE.get_or_init(|| {
        tokio::sync::RwLock::new(ModelCache {
            names: Vec::new(),
            refreshed_at: Instant::now() - Duration::from_secs(3_600),
        })
    })
}

/// Refresh the model list from Ollama if stale (>60 s).
pub async fn refresh_if_stale(ollama_url: &str) {
    let stale = {
        let g = cache().read().await;
        g.names.is_empty() || g.refreshed_at.elapsed() > Duration::from_secs(60)
    };
    if !stale {
        return;
    }

    #[derive(Deserialize)]
    struct TagList {
        models: Vec<ModelEntry>,
    }
    #[derive(Deserialize)]
    struct ModelEntry {
        name: String,
    }

    let url = format!("{}/api/tags", ollama_url);
    match reqwest::Client::new()
        .get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(list) = resp.json::<TagList>().await {
                let names: Vec<String> = list.models.into_iter().map(|m| m.name).collect();
                info!("model_router: {} models available", names.len());
                let mut g = cache().write().await;
                g.names = names;
                g.refreshed_at = Instant::now();
            }
        }
        _ => warn!("model_router: could not refresh model list from {}", url),
    }
}

/// Return a copy of all currently known model names.
pub async fn available_models(ollama_url: &str) -> Vec<String> {
    refresh_if_stale(ollama_url).await;
    cache().read().await.names.clone()
}

// ── Name matching ─────────────────────────────────────────────────────────────

fn model_available(names: &[String], candidate: &str) -> bool {
    let base = candidate.split(':').next().unwrap_or(candidate);
    names.iter().any(|n| {
        n == candidate || n == base || n.split(':').next() == Some(base)
    })
}

// ── Model selection ───────────────────────────────────────────────────────────

/// Select the best available model for the given task.
///
/// Order of precedence:
///   1. Matching env-var override (only used if that model is actually installed)
///   2. First installed model from the task's priority list
///   3. `AGENT_MODEL` env var as universal fallback
///   4. First model in the cache
pub async fn select_model(kind: TaskKind, ollama_url: &str) -> String {
    refresh_if_stale(ollama_url).await;
    let names = cache().read().await.names.clone();

    // (env_key, priority_list)
    let (env_key, priority): (&str, &[&str]) = match kind {
        TaskKind::ToolUse => (
            "AGENT_TOOL_MODEL",
            &[
                "hermes3:8b",
                "llama3.1:8b",
                "nous-hermes2:10.7b",
                "llama3.2:3b",
                "phi4:latest",
            ],
        ),
        TaskKind::CodeFix => (
            "AGENT_CODE_MODEL",
            &[
                "codestral:22b",
                "deepseek-coder-v2:16b",
                "qwen2.5-coder:7b",
                "yi-coder:9b",
                "codeqwen:7b",
                "magicoder:7b",
                "codegemma:7b",
                "codellama:7b",
            ],
        ),
        TaskKind::CodeAnalysis => (
            "AGENT_ANALYSIS_MODEL",
            &[
                "deepseek-coder-v2:16b",
                "granite-code:latest",
                "granite-code:8b",
                "deepseek-coder:6.7b",
                "starcoder2:7b",
            ],
        ),
        TaskKind::DeepReason => (
            "AGENT_DEEP_MODEL",
            &[
                "llama3.3:70b",
                "mixtral:8x7b",
                "phi4:latest",
                "gemma2:9b",
                "solar:10.7b",
                "wizard-vicuna-uncensored:30b",
            ],
        ),
        TaskKind::Vision => (
            "VISION_MODEL",
            &[
                "llama3.2-vision:11b",
                "llama3.2-vision:90b",
                "llava:13b",
                "llava:7b",
                "moondream:latest",
            ],
        ),
        TaskKind::FastChat => (
            "AGENT_FAST_MODEL",
            &[
                "llama3.2:3b",
                "phi3.5:3.8b",
                "tinydolphin:1.1b",
                "stablelm2:1.6b",
                "orca-mini:3b",
            ],
        ),
        TaskKind::Embed => (
            "EMBED_MODEL",
            &["nomic-embed-text:latest", "all-minilm:latest"],
        ),
    };

    // 1. Env-var override (must be available)
    if let Ok(ov) = std::env::var(env_key) {
        if !ov.is_empty() && model_available(&names, &ov) {
            info!("model_router: {:?} → {} (env {})", kind, ov, env_key);
            return ov;
        }
    }

    // 2. Priority list
    for &candidate in priority {
        if model_available(&names, candidate) {
            info!("model_router: {:?} → {}", kind, candidate);
            return candidate.to_string();
        }
    }

    // 3. AGENT_MODEL fallback
    if let Ok(fallback) = std::env::var("AGENT_MODEL") {
        if !fallback.is_empty() {
            warn!("model_router: {:?} no priority match, using AGENT_MODEL={}", kind, fallback);
            return fallback;
        }
    }

    // 4. First available model
    let last_resort = names.first().cloned().unwrap_or_else(|| "llama3.1:8b".to_string());
    warn!("model_router: {:?} falling back to {}", kind, last_resort);
    last_resort
}

// ── Startup health check ─────────────────────────────────────────────────────

/// Check Ollama reachability and model availability at startup.
///
/// Returns:
///   Ok(Vec<String>)  — list of available model names (non-empty).
///   Err(String)      — human-readable error with actionable advice.
///
/// Uses a 5-second timeout so it never hangs silently.
pub async fn startup_health_check(ollama_url: &str) -> Result<Vec<String>, String> {
    #[derive(Deserialize)]
    struct TagList {
        models: Vec<ModelEntry>,
    }
    #[derive(Deserialize)]
    struct ModelEntry {
        name: String,
    }

    let url = format!("{}/api/tags", ollama_url);
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        reqwest::Client::new().get(&url).send(),
    )
    .await;

    match result {
        Err(_) => Err(format!(
            "Ollama health check timed out after 5s at {}\n\
             → Is Ollama running?  Run: ollama serve\n\
             → Or set OLLAMA_HOST env var to the correct host:port",
            url
        )),
        Ok(Err(e)) => Err(format!(
            "Ollama not reachable at {}\n\
             → Error: {}\n\
             → Is Ollama running?  Run: ollama serve",
            url, e
        )),
        Ok(Ok(resp)) if !resp.status().is_success() => Err(format!(
            "Ollama returned HTTP {} at {}\n\
             → Unexpected response — check Ollama logs",
            resp.status(),
            url
        )),
        Ok(Ok(resp)) => match resp.json::<TagList>().await {
            Err(e) => Err(format!(
                "Ollama responded but response could not be parsed: {}\n\
                 → Check Ollama version (v0.18.0+ required)",
                e
            )),
            Ok(list) => {
                let names: Vec<String> = list.models.into_iter().map(|m| m.name).collect();
                if names.is_empty() {
                    Err(format!(
                        "Ollama is running at {} but NO models are installed.\n\
                         → Pull a model:  ollama pull llama3.1\n\
                         → Or check OLLAMA_MODELS env var if your models drive is unmounted",
                        ollama_url
                    ))
                } else {
                    // Populate the cache immediately so model selection works right away
                    let mut g = cache().write().await;
                    g.names = names.clone();
                    g.refreshed_at = Instant::now();
                    info!("startup_health_check: {} models found at {}", names.len(), ollama_url);
                    Ok(names)
                }
            }
        },
    }
}

// ── Tauri-command helper ──────────────────────────────────────────────────────

/// Return the selected model name for a named task kind.
/// `kind_str` values: "tool", "code", "analysis", "deep", "vision", "fast", "embed"
pub async fn select_model_by_name(kind_str: &str, ollama_url: &str) -> String {
    let kind = match kind_str {
        "tool"     => TaskKind::ToolUse,
        "code"     => TaskKind::CodeFix,
        "analysis" => TaskKind::CodeAnalysis,
        "deep"     => TaskKind::DeepReason,
        "vision"   => TaskKind::Vision,
        "embed"    => TaskKind::Embed,
        _          => TaskKind::FastChat,
    };
    select_model(kind, ollama_url).await
}
