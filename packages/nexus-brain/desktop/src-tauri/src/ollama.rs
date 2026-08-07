//! Ollama client.
//!
//! Talks to the local server over plain HTTP. Three things are needed:
//!
//!   * `/api/embed`    batched 768-dim vectors from `nomic-embed-text`
//!   * `/api/generate` interpretation, constrained to JSON via `format`
//!   * `/api/tags`     what is actually installed, so routing can fall back
//!
//! `keep_alive` is passed explicitly on every call. The engine drains one model
//! at a time, so the resident model should stay loaded across a whole batch
//! rather than being evicted between files.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct Ollama {
    base: String,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    #[serde(default)]
    embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Serialize)]
struct EmbedRequest<'a> {
    model: &'a str,
    input: &'a [String],
    keep_alive: &'a str,
}

#[derive(Debug, Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    keep_alive: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    /// `"json"` forces syntactically valid JSON out of the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
    options: GenOptions,
}

#[derive(Debug, Serialize)]
struct GenOptions {
    temperature: f32,
    num_predict: i32,
    num_ctx: i32,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    #[serde(default)]
    response: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagModel>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TagModel {
    pub name: String,
    #[serde(default)]
    pub size: u64,
}

impl Ollama {
    pub fn new(base: &str) -> Self {
        Self {
            base: base.trim_end_matches('/').to_string(),
            // Large models on first load can take minutes; embeddings are fast.
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(600))
                .connect_timeout(Duration::from_secs(5))
                .build()
                .expect("reqwest client"),
        }
    }

    pub async fn reachable(&self) -> bool {
        self.http
            .get(format!("{}/api/tags", self.base))
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Every model installed locally.
    pub async fn models(&self) -> Result<Vec<TagModel>> {
        let r: TagsResponse = self
            .http
            .get(format!("{}/api/tags", self.base))
            .send()
            .await
            .context("GET /api/tags")?
            .json()
            .await
            .context("decoding /api/tags")?;
        Ok(r.models)
    }

    /// Embed a batch of strings. Returns one vector per input, in order.
    pub async fn embed(&self, model: &str, inputs: &[String]) -> Result<Vec<Vec<f32>>> {
        if inputs.is_empty() {
            return Ok(Vec::new());
        }
        let req = EmbedRequest {
            model,
            input: inputs,
            keep_alive: "24h",
        };
        let resp = self
            .http
            .post(format!("{}/api/embed", self.base))
            .json(&req)
            .send()
            .await
            .context("POST /api/embed")?;

        if !resp.status().is_success() {
            let code = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("embed failed ({code}): {body}"));
        }

        let parsed: EmbedResponse = resp.json().await.context("decoding /api/embed")?;
        if parsed.embeddings.len() != inputs.len() {
            return Err(anyhow!(
                "embed returned {} vectors for {} inputs",
                parsed.embeddings.len(),
                inputs.len()
            ));
        }
        Ok(parsed.embeddings)
    }

    /// Generate free text.
    pub async fn generate(
        &self,
        model: &str,
        system: Option<&str>,
        prompt: &str,
        num_ctx: i32,
        num_predict: i32,
    ) -> Result<String> {
        self.generate_inner(model, system, prompt, None, false, num_ctx, num_predict)
            .await
    }

    /// Generate, forcing the reply to be a single JSON object.
    pub async fn generate_json(
        &self,
        model: &str,
        system: Option<&str>,
        prompt: &str,
        num_ctx: i32,
    ) -> Result<serde_json::Value> {
        let raw = self
            .generate_inner(model, system, prompt, None, true, num_ctx, 700)
            .await?;
        parse_json_loose(&raw)
    }

    /// Generate against an image (base64, no data: prefix), forcing JSON out.
    pub async fn generate_json_image(
        &self,
        model: &str,
        system: Option<&str>,
        prompt: &str,
        image_b64: String,
        num_ctx: i32,
    ) -> Result<serde_json::Value> {
        let raw = self
            .generate_inner(
                model,
                system,
                prompt,
                Some(vec![image_b64]),
                true,
                num_ctx,
                700,
            )
            .await?;
        parse_json_loose(&raw)
    }

    #[allow(clippy::too_many_arguments)]
    async fn generate_inner(
        &self,
        model: &str,
        system: Option<&str>,
        prompt: &str,
        images: Option<Vec<String>>,
        json: bool,
        num_ctx: i32,
        num_predict: i32,
    ) -> Result<String> {
        let req = GenerateRequest {
            model,
            prompt,
            stream: false,
            keep_alive: "24h",
            system,
            format: if json { Some("json") } else { None },
            images,
            options: GenOptions {
                // Deterministic: this is classification, not creative writing.
                temperature: 0.0,
                num_predict,
                num_ctx,
            },
        };

        let resp = self
            .http
            .post(format!("{}/api/generate", self.base))
            .json(&req)
            .send()
            .await
            .with_context(|| format!("POST /api/generate ({model})"))?;

        if !resp.status().is_success() {
            let code = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("generate failed ({code}) on {model}: {body}"));
        }

        let parsed: GenerateResponse = resp.json().await.context("decoding /api/generate")?;
        Ok(parsed.response)
    }
}

/// Models sometimes wrap JSON in prose or a fenced block even when asked not
/// to. Recover the first balanced object rather than failing the whole file.
pub fn parse_json_loose(raw: &str) -> Result<serde_json::Value> {
    let trimmed = raw.trim();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return Ok(v);
    }

    let bytes = trimmed.as_bytes();
    let start = match bytes.iter().position(|&b| b == b'{') {
        Some(i) => i,
        None => return Err(anyhow!("no JSON object in model reply: {trimmed:.200}")),
    };

    let mut depth = 0usize;
    let mut in_str = false;
    let mut escaped = false;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_str {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    let slice = &trimmed[start..=i];
                    return serde_json::from_str(slice)
                        .with_context(|| format!("parsing recovered JSON: {slice:.200}"));
                }
            }
            _ => {}
        }
    }
    Err(anyhow!("unbalanced JSON in model reply: {trimmed:.200}"))
}

#[cfg(test)]
mod tests {
    use super::parse_json_loose;

    #[test]
    fn plain_object() {
        let v = parse_json_loose(r#"{"a":1}"#).unwrap();
        assert_eq!(v["a"], 1);
    }

    #[test]
    fn fenced_object() {
        let v = parse_json_loose("```json\n{\"a\": \"x\"}\n```").unwrap();
        assert_eq!(v["a"], "x");
    }

    #[test]
    fn object_with_prose_around_it() {
        let v = parse_json_loose("Sure!\n{\"k\": {\"n\": 2}}\nHope that helps").unwrap();
        assert_eq!(v["k"]["n"], 2);
    }

    #[test]
    fn braces_inside_strings_do_not_confuse_it() {
        let v = parse_json_loose(r#"{"s":"a } b","t":1}"#).unwrap();
        assert_eq!(v["t"], 1);
        assert_eq!(v["s"], "a } b");
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_json_loose("no json here").is_err());
    }
}
