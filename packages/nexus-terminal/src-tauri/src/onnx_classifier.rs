/// ML fallback stage for the input classifier.
///
/// Ported from Warp's `crates/input_classifier/src/onnx/candle.rs` (AGPL-3.0,
/// https://github.com/warpdotdev/warp). Runs the same `bert_tiny_v1` ONNX model Warp's own
/// OSS build uses (selected via their `nld_classifier_v1` feature, which maps to the
/// `onnx_candle` backend — not `onnx_ort` — so this uses `candle_onnx` rather than the `ort`
/// crate, matching Warp's actual default and avoiding a system libonnxruntime dependency).
///
/// This is the ML half of Warp's two-stage classifier design: `commandRouting.ts`'s heuristic
/// tiers are the fast pre-filter (Tiers 1-4); when those are inconclusive (Tier 5, "default to
/// AI"), the frontend calls into this instead of blindly defaulting.
use anyhow::{ensure, Context, Result};
use candle_core::{Device, IndexOp, Tensor};
use candle_onnx::onnx::ModelProto;
use once_cell::sync::Lazy;
use prost::Message;
use std::collections::HashMap;
use tokenizers::Tokenizer;

const MODEL_BYTES: &[u8] = include_bytes!("../models/bert_tiny_v1.onnx");
const TOKENIZER_BYTES: &[u8] = include_bytes!("../models/bert_tiny_tokenizer.json");

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct ClassificationResult {
    /// Probability the input is a shell command.
    pub p_shell: f32,
    /// Probability the input is a natural-language query.
    pub p_ai: f32,
}

impl ClassificationResult {
    pub fn is_shell(&self) -> bool {
        self.p_shell > self.p_ai
    }
}

struct OnnxClassifier {
    model: ModelProto,
    tokenizer: Tokenizer,
}

impl OnnxClassifier {
    fn load() -> Result<Self> {
        let model = ModelProto::decode(MODEL_BYTES).context("failed to decode bert_tiny_v1.onnx")?;
        let tokenizer =
            Tokenizer::from_bytes(TOKENIZER_BYTES).map_err(|e| anyhow::anyhow!("failed to load tokenizer: {e}"))?;
        Ok(Self { model, tokenizer })
    }

    fn classify(&self, text: &str) -> Result<ClassificationResult> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("tokenizer encode failed: {e}"))?;

        let device = Device::Cpu;
        let input_ids = Tensor::new(
            encoding.get_ids().iter().map(|&x| x as i64).collect::<Vec<_>>().as_slice(),
            &device,
        )
        .context("failed to build input_ids tensor")?;
        let attention_mask = Tensor::new(
            encoding
                .get_attention_mask()
                .iter()
                .map(|&x| x as i64)
                .collect::<Vec<_>>()
                .as_slice(),
            &device,
        )
        .context("failed to build attention_mask tensor")?;

        let outputs = candle_onnx::simple_eval(
            &self.model,
            HashMap::from([
                ("input_ids".to_string(), input_ids.unsqueeze(0)?),
                ("attention_mask".to_string(), attention_mask.unsqueeze(0)?),
            ]),
        )
        .context("error evaluating bert_tiny_v1 ONNX model")?;

        let logits = outputs.get("logits").context("model output missing 'logits'")?;
        let probabilities = candle_nn::ops::softmax_last_dim(logits)
            .context("failed to compute softmax")?
            .i(0)
            .context("failed to index first dimension")?
            .to_vec1::<f32>()
            .context("failed to convert softmax output to Vec<f32>")?;

        ensure!(
            probabilities.len() == 2,
            "expected 2 output probabilities from bert_tiny_v1, got {}",
            probabilities.len()
        );

        // Same output ordering as Warp's onnx/candle.rs: index 0 = p_ai, index 1 = p_shell.
        Ok(ClassificationResult {
            p_ai: probabilities[0],
            p_shell: probabilities[1],
        })
    }
}

static CLASSIFIER: Lazy<Result<OnnxClassifier, String>> =
    Lazy::new(|| OnnxClassifier::load().map_err(|e| e.to_string()));

/// Classify input text as shell command vs AI query using Warp's bert_tiny ONNX model.
/// Runs on a blocking thread since `candle_onnx::simple_eval` is synchronous CPU-bound work.
pub async fn classify(text: &str) -> Result<ClassificationResult, String> {
    let text = text.to_string();
    tokio::task::spawn_blocking(move || match &*CLASSIFIER {
        Ok(classifier) => classifier.classify(&text).map_err(|e| e.to_string()),
        Err(e) => Err(format!("ONNX classifier failed to load: {e}")),
    })
    .await
    .map_err(|e| format!("classification task panicked: {e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn loads_and_returns_normalized_probabilities() {
        let result = classify("ls -la").await.expect("classification should succeed");
        let sum = result.p_shell + result.p_ai;
        assert!(
            (sum - 1.0).abs() < 0.01,
            "probabilities should sum to ~1.0 (softmax output), got p_shell={} p_ai={} sum={}",
            result.p_shell,
            result.p_ai,
            sum
        );
    }

    #[tokio::test]
    async fn classifies_unambiguous_shell_command_as_shell() {
        let result = classify("git status --short --branch").await.expect("classification should succeed");
        assert!(
            result.is_shell(),
            "expected shell for 'git status --short --branch', got p_shell={} p_ai={}",
            result.p_shell,
            result.p_ai
        );
    }

    #[tokio::test]
    async fn classifies_unambiguous_question_as_ai() {
        let result = classify("how do I revert my last commit?").await.expect("classification should succeed");
        assert!(
            !result.is_shell(),
            "expected AI for 'how do I revert my last commit?', got p_shell={} p_ai={}",
            result.p_shell,
            result.p_ai
        );
    }
}
