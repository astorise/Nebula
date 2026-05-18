#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const RESULTS_TOPIC: &str = "nebula.eval.results";
pub const DEFAULT_MODEL: &str = "all-MiniLM-L6-v2";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticEvaluationRequest {
    pub prompt: String,
    pub responses: [String; 3],
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationResult {
    pub diverged: bool,
    pub evaluator: String,
    pub reason: String,
    pub average_similarity: f32,
    pub prompt: String,
    pub responses: [String; 3],
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticDecision {
    Ignore { average_similarity: f32 },
    Diverged(EvaluationResult),
    Ambiguous { average_similarity: f32 },
}

pub trait InferenceHost {
    fn embed(&self, model: &str, input: &str) -> Result<Vec<f32>>;
}

pub trait EventBus {
    fn publish(&mut self, topic: &str, result: &EvaluationResult) -> Result<()>;
}

pub fn evaluate_and_publish(
    bus: &mut impl EventBus,
    inference: &impl InferenceHost,
    request: SemanticEvaluationRequest,
) -> Result<SemanticDecision> {
    let decision = evaluate(inference, request)?;
    if let SemanticDecision::Diverged(result) = &decision {
        bus.publish(RESULTS_TOPIC, result)?;
    }

    Ok(decision)
}

pub fn evaluate(
    inference: &impl InferenceHost,
    request: SemanticEvaluationRequest,
) -> Result<SemanticDecision> {
    let baseline = inference.embed(DEFAULT_MODEL, &request.responses[0])?;
    let first_hot = inference.embed(DEFAULT_MODEL, &request.responses[1])?;
    let second_hot = inference.embed(DEFAULT_MODEL, &request.responses[2])?;
    let average_similarity = (cosine_similarity(&baseline, &first_hot)?
        + cosine_similarity(&baseline, &second_hot)?)
        / 2.0;

    if average_similarity >= 0.95 {
        return Ok(SemanticDecision::Ignore { average_similarity });
    }

    if average_similarity < 0.85 {
        return Ok(SemanticDecision::Diverged(EvaluationResult {
            diverged: true,
            evaluator: "nebula-eval-semantic".into(),
            reason: "semantic similarity below divergence threshold".into(),
            average_similarity,
            prompt: request.prompt,
            responses: request.responses,
            context: request.context,
        }));
    }

    Ok(SemanticDecision::Ambiguous { average_similarity })
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> Result<f32> {
    if left.len() != right.len() || left.is_empty() {
        return Err(anyhow!("vectors must have the same non-zero dimension"));
    }

    let mut dot = 0.0;
    let mut left_norm = 0.0;
    let mut right_norm = 0.0;

    for (left_value, right_value) in left.iter().zip(right.iter()) {
        dot += left_value * right_value;
        left_norm += left_value * left_value;
        right_norm += right_value * right_value;
    }

    if left_norm == 0.0 || right_norm == 0.0 {
        return Err(anyhow!("vectors must not be zero vectors"));
    }

    Ok(dot / (left_norm.sqrt() * right_norm.sqrt()))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Host;

    impl InferenceHost for Host {
        fn embed(&self, _model: &str, input: &str) -> Result<Vec<f32>> {
            Ok(match input {
                "a" => vec![1.0, 0.0],
                "b" => vec![0.0, 1.0],
                _ => vec![1.0, 0.0],
            })
        }
    }

    #[test]
    // spec: eval-semantic
    fn low_similarity_diverges() {
        let request = SemanticEvaluationRequest {
            prompt: "p".into(),
            responses: ["a".into(), "b".into(), "b".into()],
            context: serde_json::json!({}),
        };

        assert!(matches!(
            evaluate(&Host, request).unwrap(),
            SemanticDecision::Diverged(_)
        ));
    }
}
