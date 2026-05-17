use anyhow::Result;
use nebula_tenant_core::TenantId;
use serde::{Deserialize, Serialize};

pub const INPUT_TOPIC: &str = "nebula.eval.divergence_detected";
pub const FORWARD_TOPIC: &str = "nebula.finops.governor.request";
pub const DEDUPLICATED_TOPIC: &str = "nebula.finops.deduplicated";
pub const SIMILARITY_THRESHOLD: f32 = 0.95;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DivergenceEvent {
    pub tenant_id: TenantId,
    pub prompt: String,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CachedEmbedding {
    pub namespace: String,
    pub prompt: String,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeduplicationDecision {
    Duplicate {
        topic: &'static str,
        matched_prompt: String,
        similarity: f32,
    },
    Unique {
        topic: &'static str,
    },
}

pub trait EmbeddingModel {
    fn embed(&self, prompt: &str) -> Result<Vec<f32>>;
}

pub trait VectorStore {
    fn query_recent(&self, tenant_id: TenantId) -> Result<Vec<CachedEmbedding>>;
}

pub fn classify_divergence(
    embeddings: &impl EmbeddingModel,
    store: &impl VectorStore,
    event: &DivergenceEvent,
) -> Result<DeduplicationDecision> {
    let embedding = embeddings.embed(&event.prompt)?;
    let best = store
        .query_recent(event.tenant_id)?
        .into_iter()
        .map(|cached| {
            let similarity = cosine_similarity(&embedding, &cached.embedding)?;
            Ok((cached.prompt, similarity))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max_by(|left, right| left.1.total_cmp(&right.1));

    if let Some((matched_prompt, similarity)) = best {
        if similarity > SIMILARITY_THRESHOLD {
            return Ok(DeduplicationDecision::Duplicate {
                topic: DEDUPLICATED_TOPIC,
                matched_prompt,
                similarity,
            });
        }
    }

    Ok(DeduplicationDecision::Unique {
        topic: FORWARD_TOPIC,
    })
}

pub fn cosine_similarity(left: &[f32], right: &[f32]) -> Result<f32> {
    if left.is_empty() || left.len() != right.len() {
        anyhow::bail!("embedding dimensions must match and be non-empty");
    }

    let dot = left.iter().zip(right).map(|(a, b)| a * b).sum::<f32>();
    let left_norm = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f32>().sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        anyhow::bail!("embedding norm must be non-zero");
    }
    Ok(dot / (left_norm * right_norm))
}

#[cfg(test)]
mod tests {
    use super::*;

    use nebula_tenant_core::{deterministic_test_tenant, TenantId};

    struct Model;
    struct Store(Vec<CachedEmbedding>);

    impl EmbeddingModel for Model {
        fn embed(&self, prompt: &str) -> Result<Vec<f32>> {
            if prompt.contains("different") {
                Ok(vec![0.0, 1.0])
            } else {
                Ok(vec![1.0, 0.0])
            }
        }
    }

    impl VectorStore for Store {
        fn query_recent(&self, _tenant_id: TenantId) -> Result<Vec<CachedEmbedding>> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn marks_near_identical_prompt_as_duplicate() {
        let tenant_id = deterministic_test_tenant("acme");
        let decision = classify_divergence(
            &Model,
            &Store(vec![CachedEmbedding {
                namespace: "pending_arbitrations".into(),
                prompt: "same".into(),
                embedding: vec![1.0, 0.0],
            }]),
            &DivergenceEvent {
                tenant_id,
                prompt: "same".into(),
                context: serde_json::json!({}),
            },
        )
        .unwrap();

        assert!(matches!(decision, DeduplicationDecision::Duplicate { .. }));
    }

    #[test]
    fn computes_cosine_similarity() {
        assert_eq!(cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]).unwrap(), 1.0);
        assert_eq!(cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]).unwrap(), 0.0);
        assert!(cosine_similarity(&[1.0], &[1.0, 0.0]).is_err());
    }

    #[test]
    fn flood_of_identical_queries_forwards_only_first_request() {
        let tenant_id = deterministic_test_tenant("acme");
        let events = (0..10)
            .map(|_| DivergenceEvent {
                tenant_id,
                prompt: "same failing query".into(),
                context: serde_json::json!({}),
            })
            .collect::<Vec<_>>();

        let mut forwarded = 0;
        let mut cache = Vec::new();
        for event in events {
            let decision = classify_divergence(&Model, &Store(cache.clone()), &event).unwrap();
            if matches!(decision, DeduplicationDecision::Unique { .. }) {
                forwarded += 1;
                cache.push(CachedEmbedding {
                    namespace: "pending_arbitrations".into(),
                    prompt: event.prompt,
                    embedding: vec![1.0, 0.0],
                });
            }
        }

        assert_eq!(forwarded, 1);
    }

    #[test]
    fn flood_test_keeps_distinct_prompts_unique() {
        let tenant_id = deterministic_test_tenant("acme");
        let mut cache = vec![CachedEmbedding {
            namespace: "pending_arbitrations".into(),
            prompt: "same failing query".into(),
            embedding: vec![1.0, 0.0],
        }];

        let decision = classify_divergence(
            &Model,
            &Store(cache.clone()),
            &DivergenceEvent {
                tenant_id,
                prompt: "different failing query".into(),
                context: serde_json::json!({}),
            },
        )
        .unwrap();

        assert!(matches!(decision, DeduplicationDecision::Unique { .. }));
        cache.push(CachedEmbedding {
            namespace: "pending_arbitrations".into(),
            prompt: "different failing query".into(),
            embedding: vec![0.0, 1.0],
        });
        assert_eq!(cache.len(), 2);
    }
}
