use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const INPUT_TOPIC: &str = "pulsar.telemetry.inference_results";
pub const COGNITIVE_DIVERGENCE_METRIC: &str = "nebula.cognitive_divergence";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InferenceOutcome {
    pub model_version: String,
    pub timestamp_ms: u64,
    #[serde(default)]
    pub diverged: bool,
    #[serde(default)]
    pub escalated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricTag {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricType {
    Gauge,
    Counter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub metric_type: MetricType,
    pub tags: Vec<MetricTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RolloutDecision {
    pub model_version: String,
    pub rollout_track: String,
    pub divergence_rate: f64,
    pub threshold: f64,
    pub rollback: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanaryPolicyConfig {
    pub window_ms: u64,
    pub rollback_threshold: f64,
}

impl Default for CanaryPolicyConfig {
    fn default() -> Self {
        Self {
            window_ms: 10_000,
            rollback_threshold: 0.04,
        }
    }
}

pub trait OutcomeStore {
    fn load(&mut self, model_version: &str) -> Result<Vec<InferenceOutcome>>;
    fn save(&mut self, model_version: &str, outcomes: &[InferenceOutcome]) -> Result<()>;
}

pub trait CustomMetrics {
    fn push(&mut self, metric: &Metric) -> Result<()>;
}

pub fn handle_inference_outcome(
    store: &mut impl OutcomeStore,
    metrics: &mut impl CustomMetrics,
    topic: &str,
    raw_payload: &[u8],
    config: CanaryPolicyConfig,
) -> Result<RolloutDecision> {
    if topic != INPUT_TOPIC {
        return Err(anyhow!("unsupported topic: {topic}"));
    }

    let outcome: InferenceOutcome = serde_json::from_slice(raw_payload)?;
    record_outcome(store, metrics, outcome, config)
}

pub fn record_outcome(
    store: &mut impl OutcomeStore,
    metrics: &mut impl CustomMetrics,
    outcome: InferenceOutcome,
    config: CanaryPolicyConfig,
) -> Result<RolloutDecision> {
    let mut outcomes = store.load(&outcome.model_version)?;
    outcomes.push(outcome.clone());
    let min_timestamp = outcome.timestamp_ms.saturating_sub(config.window_ms);
    outcomes.retain(|item| item.timestamp_ms >= min_timestamp);
    store.save(&outcome.model_version, &outcomes)?;

    let decision = evaluate_rollout(&outcome.model_version, &outcomes, config);
    metrics.push(&Metric {
        name: COGNITIVE_DIVERGENCE_METRIC.into(),
        value: decision.divergence_rate,
        metric_type: MetricType::Gauge,
        tags: vec![
            MetricTag {
                key: "model_version".into(),
                value: decision.model_version.clone(),
            },
            MetricTag {
                key: "rollout_track".into(),
                value: decision.rollout_track.clone(),
            },
        ],
    })?;
    Ok(decision)
}

pub fn evaluate_rollout(
    model_version: &str,
    outcomes: &[InferenceOutcome],
    config: CanaryPolicyConfig,
) -> RolloutDecision {
    let failures = outcomes
        .iter()
        .filter(|outcome| outcome.diverged || outcome.escalated)
        .count();
    let divergence_rate = if outcomes.is_empty() {
        0.0
    } else {
        failures as f64 / outcomes.len() as f64
    };
    let rollout_track = rollout_track(model_version);

    RolloutDecision {
        model_version: model_version.into(),
        rollout_track: rollout_track.into(),
        divergence_rate,
        threshold: config.rollback_threshold,
        rollback: rollout_track == "canary" && divergence_rate > config.rollback_threshold,
    }
}

pub fn render_tachyon_canary_template(name: &str, cognitive_threshold: f64) -> String {
    format!(
        "apiVersion: faas.tachyon.mesh/v1alpha1\n\
kind: Deployment\n\
metadata:\n\
  name: {name}\n\
spec:\n\
  replicas: 32\n\
  strategy:\n\
    type: Canary\n\
    canary:\n\
      steps: [10%, 25%, 50%, 100%]\n\
      analysis_interval: 30s\n\
      rules:\n\
        - metric: http_error_rate\n\
          threshold: \"< 0.5%\"\n\
        - metric: nebula.cognitive_divergence\n\
          threshold: \"< {threshold:.1}%\"\n",
        name = name,
        threshold = cognitive_threshold * 100.0
    )
}

pub fn latest_metric_cache(decisions: &[RolloutDecision]) -> BTreeMap<String, RolloutDecision> {
    let mut cache = BTreeMap::new();
    for decision in decisions {
        cache.insert(decision.rollout_track.clone(), decision.clone());
    }
    cache
}

fn rollout_track(model_version: &str) -> &'static str {
    if model_version.contains("canary") {
        "canary"
    } else {
        "stable"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Store(BTreeMap<String, Vec<InferenceOutcome>>);
    struct Metrics(Vec<Metric>);

    impl OutcomeStore for Store {
        fn load(&mut self, model_version: &str) -> Result<Vec<InferenceOutcome>> {
            Ok(self.0.get(model_version).cloned().unwrap_or_default())
        }

        fn save(&mut self, model_version: &str, outcomes: &[InferenceOutcome]) -> Result<()> {
            self.0.insert(model_version.into(), outcomes.to_vec());
            Ok(())
        }
    }

    impl CustomMetrics for Metrics {
        fn push(&mut self, metric: &Metric) -> Result<()> {
            self.0.push(metric.clone());
            Ok(())
        }
    }

    #[test]
    fn pushes_tagged_canary_metric_and_recommends_rollback() {
        let mut store = Store::default();
        let mut metrics = Metrics(Vec::new());
        let config = CanaryPolicyConfig {
            window_ms: 10_000,
            rollback_threshold: 0.04,
        };

        let decision = record_outcome(
            &mut store,
            &mut metrics,
            InferenceOutcome {
                model_version: "pulsar-base:v2-canary".into(),
                timestamp_ms: 1,
                diverged: true,
                escalated: false,
            },
            config,
        )
        .unwrap();

        assert!(decision.rollback);
        assert_eq!(metrics.0[0].name, COGNITIVE_DIVERGENCE_METRIC);
        assert_eq!(
            metrics.0[0].tags,
            vec![
                MetricTag {
                    key: "model_version".into(),
                    value: "pulsar-base:v2-canary".into()
                },
                MetricTag {
                    key: "rollout_track".into(),
                    value: "canary".into()
                }
            ]
        );
    }

    #[test]
    fn stable_track_never_requests_canary_rollback() {
        let decision = evaluate_rollout(
            "pulsar-base:v2",
            &[InferenceOutcome {
                model_version: "pulsar-base:v2".into(),
                timestamp_ms: 1,
                diverged: true,
                escalated: false,
            }],
            CanaryPolicyConfig::default(),
        );

        assert_eq!(decision.rollout_track, "stable");
        assert!(!decision.rollback);
    }

    #[test]
    fn template_contains_cognitive_divergence_rule() {
        let template = render_tachyon_canary_template("pulsar-swarm-rollout", 0.04);

        assert!(template.contains("type: Canary"));
        assert!(template.contains("nebula.cognitive_divergence"));
        assert!(template.contains("< 4.0%"));
    }
}
