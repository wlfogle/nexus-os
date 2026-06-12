use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::types::{DocStatus, OllamaModel};

/// Result of analysing a single document with the LLM.
#[derive(Debug, Clone)]
pub struct DocAnalysis {
    pub status: DocStatus,
    pub confidence: f32,
    pub staleness_score: f32,
    pub reason: String,
    pub evidence: String,
    pub suggested_rewrite: Option<String>,
}

// ─── /api/tags response shapes ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagModel>,
}

#[derive(Debug, Deserialize)]
struct TagModel {
    name: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    details: Option<TagDetails>,
}

#[derive(Debug, Deserialize)]
struct TagDetails {
    #[serde(default)]
    family: Option<String>,
}

/// Score a model by name (case-insensitive) plus a small size bonus.
fn score_model(name: &str, size_bytes: u64) -> f32 {
    let lower = name.to_ascii_lowercase();
    let base: f32 = if lower.contains("qwen") {
        10.0
    } else if lower.contains("codestral") {
        9.0
    } else if lower.contains("deepseek-coder") {
        8.0
    } else if lower.contains("codellama") {
        7.0
    } else if lower.contains("llama3.1") {
        6.0
    } else if lower.contains("llama3") {
        5.0
    } else if lower.contains("mistral") {
        4.0
    } else {
        3.0
    };
    let size_bonus = (size_bytes as f32 / 1e9) * 0.1;
    base + size_bonus
}

/// List all locally available Ollama models, scored for auto-selection.
///
/// The highest-scoring model is flagged `recommended = true`.
pub async fn list_models(ollama_url: &str) -> Result<Vec<OllamaModel>> {
    let url = format!("{}/api/tags", ollama_url.trim_end_matches('/'));
    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| anyhow!("Ollama /api/tags request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(anyhow!("Ollama /api/tags returned {}", resp.status()));
    }

    let tags: TagsResponse = resp
        .json()
        .await
        .map_err(|e| anyhow!("failed to parse /api/tags response: {e}"))?;

    let mut models: Vec<OllamaModel> = tags
        .models
        .into_iter()
        .map(|m| {
            let family = m
                .details
                .and_then(|d| d.family)
                .unwrap_or_else(|| "unknown".to_string());
            OllamaModel {
                score: score_model(&m.name, m.size),
                size_gb: m.size as f32 / 1e9,
                recommended: false,
                family,
                name: m.name,
            }
        })
        .collect();

    // Flag the single highest-scoring model as recommended.
    if let Some(best_idx) = models
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
    {
        models[best_idx].recommended = true;
    }

    Ok(models)
}

/// Choose which model to use.
///
/// Returns `override_name` if it is set and present in `models`; otherwise the
/// recommended model; otherwise the first model; `None` if there are no models.
pub fn select_model(models: &[OllamaModel], override_name: Option<&str>) -> Option<String> {
    if let Some(name) = override_name {
        if !name.is_empty() && models.iter().any(|m| m.name == name) {
            return Some(name.to_string());
        }
    }

    models
        .iter()
        .find(|m| m.recommended)
        .or_else(|| models.first())
        .map(|m| m.name.clone())
}

// ─── /api/generate response shapes ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    #[serde(default)]
    response: String,
}

#[derive(Debug, Deserialize)]
struct AnalysisJson {
    #[serde(default)]
    status: String,
    #[serde(default)]
    confidence: f32,
    #[serde(default)]
    staleness_score: f32,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    evidence: String,
    #[serde(default)]
    suggested_rewrite: Option<String>,
}

/// Map a status string from the model into a [`DocStatus`].
fn parse_status(raw: &str) -> DocStatus {
    match raw.trim().to_ascii_lowercase().as_str() {
        "current" => DocStatus::Current,
        "stale" => DocStatus::Stale,
        "outdated" => DocStatus::Outdated,
        "orphaned" => DocStatus::Orphaned,
        _ => DocStatus::NeedsReview,
    }
}

/// Analyse a single document against nearby code and classify its freshness.
///
/// Sends a focused prompt to Ollama's `/api/generate` endpoint with
/// `format: "json"` so the model returns a structured classification. If the
/// model's confidence is below 0.5, the status is forced to `NeedsReview`.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OllamaModel;

    fn make_model(name: &str, size_gb: f32, score: f32) -> OllamaModel {
        OllamaModel {
            name: name.to_string(),
            size_gb,
            score,
            recommended: false,
            family: "test".to_string(),
        }
    }

    #[test]
    fn score_ordering() {
        assert!(score_model("qwen2.5-coder", 0) > score_model("codestral", 0));
        assert!(score_model("codestral", 0) > score_model("deepseek-coder", 0));
        assert!(score_model("deepseek-coder", 0) > score_model("codellama", 0));
        assert!(score_model("llama3.1", 0) > score_model("llama3", 0));
        assert!(score_model("mistral", 0) > score_model("gemma", 0));
    }

    #[test]
    fn size_bonus_applied() {
        // A 10 GB model gets +1.0 bonus (10 * 0.1)
        let score_0gb = score_model("mistral", 0);
        let score_10gb = score_model("mistral", 10_000_000_000);
        assert!((score_10gb - score_0gb - 1.0).abs() < 0.01);
    }

    #[test]
    fn parse_status_known_values() {
        assert_eq!(parse_status("current"), DocStatus::Current);
        assert_eq!(parse_status("STALE"), DocStatus::Stale);
        assert_eq!(parse_status("outdated"), DocStatus::Outdated);
        assert_eq!(parse_status("orphaned"), DocStatus::Orphaned);
        assert_eq!(parse_status("needs_review"), DocStatus::NeedsReview);
        assert_eq!(parse_status("garbage"), DocStatus::NeedsReview);
    }

    #[test]
    fn select_model_override() {
        let models = vec![
            make_model("llama3", 4.0, 5.0),
            make_model("qwen2.5", 8.0, 10.5),
        ];
        // Override selects the named model even if it is not recommended.
        assert_eq!(
            select_model(&models, Some("llama3")),
            Some("llama3".to_string())
        );
    }

    #[test]
    fn select_model_recommended() {
        let mut models = vec![
            make_model("llama3", 4.0, 5.0),
            make_model("qwen2.5", 8.0, 10.5),
        ];
        models[1].recommended = true;
        assert_eq!(
            select_model(&models, None),
            Some("qwen2.5".to_string())
        );
    }

    #[test]
    fn select_model_no_models() {
        assert_eq!(select_model(&[], None), None);
    }

    #[test]
    fn select_model_invalid_override_falls_back() {
        let mut models = vec![make_model("llama3", 4.0, 5.0)];
        models[0].recommended = true;
        // Override names a model that is not in the list.
        assert_eq!(
            select_model(&models, Some("nonexistent")),
            Some("llama3".to_string())
        );
    }
}

pub async fn analyze_doc(
    ollama_url: &str,
    model: &str,
    doc_content: &str,
    nearby_code_snippet: &str,
    repo_name: &str,
    file_path: &str,
) -> Result<DocAnalysis> {
    // Keep the prompt focused and bounded so very large docs do not blow context.
    let doc_excerpt: String = doc_content.chars().take(12_000).collect();
    let code_excerpt: String = nearby_code_snippet.chars().take(4_000).collect();

    let prompt = format!(
        "You are a documentation auditor. Classify the freshness of the documentation \
file below relative to its codebase.\n\n\
Repository: {repo_name}\n\
File: {file_path}\n\n\
Classify the document into exactly one status:\n\
- \"current\": accurately reflects the current code/project state\n\
- \"stale\": partially outdated but salvageable\n\
- \"outdated\": significantly behind and misleading\n\
- \"orphaned\": has no related code or project to correspond with\n\
- \"needs_review\": cannot be classified reliably\n\n\
Respond ONLY with a JSON object using exactly these keys:\n\
{{\n  \"status\": \"current|stale|outdated|orphaned|needs_review\",\n  \
\"confidence\": 0.0-1.0,\n  \"staleness_score\": 0.0-1.0,\n  \
\"reason\": \"short explanation\",\n  \"evidence\": \"specific evidence from the text or code\",\n  \
\"suggested_rewrite\": \"optional improved version or null\"\n}}\n\n\
=== DOCUMENT CONTENT ===\n{doc_excerpt}\n\n\
=== NEARBY CODE ===\n{code_excerpt}\n"
    );

    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "format": "json",
        "stream": false,
    });

    let url = format!("{}/api/generate", ollama_url.trim_end_matches('/'));
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow!("Ollama /api/generate request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(anyhow!("Ollama /api/generate returned {}", resp.status()));
    }

    let generate: GenerateResponse = resp
        .json()
        .await
        .map_err(|e| anyhow!("failed to parse /api/generate response: {e}"))?;

    let parsed: AnalysisJson = serde_json::from_str(generate.response.trim())
        .map_err(|e| anyhow!("failed to parse model JSON output: {e}"))?;

    let confidence = parsed.confidence.clamp(0.0, 1.0);
    let staleness_score = parsed.staleness_score.clamp(0.0, 1.0);

    let mut status = parse_status(&parsed.status);
    if confidence < 0.5 {
        status = DocStatus::NeedsReview;
    }

    let suggested_rewrite = parsed
        .suggested_rewrite
        .filter(|s| !s.trim().is_empty() && s.trim().to_ascii_lowercase() != "null");

    Ok(DocAnalysis {
        status,
        confidence,
        staleness_score,
        reason: parsed.reason,
        evidence: parsed.evidence,
        suggested_rewrite,
    })
}
