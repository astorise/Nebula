use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const INPUT_TOPIC: &str = "pulsar.telemetry.inference_results";
pub const DRIFT_TOPIC: &str = "nebula.drift.detected";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InferenceMetric {
    pub topic: String,
    pub timestamp_ms: u64,
    #[serde(default)]
    pub fallback_tool_called: bool,
    #[serde(default)]
    pub high_variance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopicWindow {
    pub topic: String,
    pub total: usize,
    pub uncertain: usize,
    pub confidence_score: f32,
    pub threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriftDetectedEvent {
    pub topic: String,
    pub confidence_score: f32,
    pub threshold: f32,
    pub sample_count: usize,
    pub uncertain_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriftConfig {
    pub window_ms: u64,
    pub min_samples: usize,
    pub confidence_threshold: f32,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            window_ms: 60 * 60 * 1000,
            min_samples: 1000,
            confidence_threshold: 0.90,
        }
    }
}

pub trait MetricStore {
    fn load(&mut self, topic: &str) -> Result<Vec<InferenceMetric>>;
    fn save(&mut self, topic: &str, metrics: &[InferenceMetric]) -> Result<()>;
}

pub trait EventBus {
    fn publish(&mut self, topic: &str, event: &DriftDetectedEvent) -> Result<()>;
}

pub fn handle_metric(
    store: &mut impl MetricStore,
    bus: &mut impl EventBus,
    topic: &str,
    raw_payload: &[u8],
    config: DriftConfig,
) -> Result<Option<DriftDetectedEvent>> {
    if topic != INPUT_TOPIC {
        return Err(anyhow!("unsupported topic: {topic}"));
    }

    let metric: InferenceMetric = serde_json::from_slice(raw_payload)?;
    record_metric(store, bus, metric, config)
}

pub fn record_metric(
    store: &mut impl MetricStore,
    bus: &mut impl EventBus,
    metric: InferenceMetric,
    config: DriftConfig,
) -> Result<Option<DriftDetectedEvent>> {
    let mut metrics = store.load(&metric.topic)?;
    metrics.push(metric.clone());
    let min_timestamp = metric.timestamp_ms.saturating_sub(config.window_ms);
    metrics.retain(|item| item.timestamp_ms >= min_timestamp);
    store.save(&metric.topic, &metrics)?;

    let window = summarize_topic(&metric.topic, &metrics, config);
    if window.total < config.min_samples || window.confidence_score >= config.confidence_threshold {
        return Ok(None);
    }

    let event = DriftDetectedEvent {
        topic: window.topic,
        confidence_score: window.confidence_score,
        threshold: window.threshold,
        sample_count: window.total,
        uncertain_count: window.uncertain,
    };
    bus.publish(DRIFT_TOPIC, &event)?;
    Ok(Some(event))
}

pub fn summarize_topics(
    metrics: &[InferenceMetric],
    config: DriftConfig,
) -> BTreeMap<String, TopicWindow> {
    let mut grouped: BTreeMap<String, Vec<InferenceMetric>> = BTreeMap::new();
    for metric in metrics {
        grouped
            .entry(metric.topic.clone())
            .or_default()
            .push(metric.clone());
    }

    grouped
        .into_iter()
        .map(|(topic, metrics)| {
            let window = summarize_topic(&topic, &metrics, config);
            (topic, window)
        })
        .collect()
}

fn summarize_topic(topic: &str, metrics: &[InferenceMetric], config: DriftConfig) -> TopicWindow {
    let uncertain = metrics
        .iter()
        .filter(|metric| metric.is_uncertain())
        .count();
    let total = metrics.len();
    let uncertain_rate = if total == 0 {
        0.0
    } else {
        uncertain as f32 / total as f32
    };

    TopicWindow {
        topic: topic.to_string(),
        total,
        uncertain,
        confidence_score: 1.0 - uncertain_rate,
        threshold: config.confidence_threshold,
    }
}

impl InferenceMetric {
    fn is_uncertain(&self) -> bool {
        self.fallback_tool_called || self.high_variance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Store(BTreeMap<String, Vec<InferenceMetric>>);
    struct Bus(Vec<DriftDetectedEvent>);

    impl MetricStore for Store {
        fn load(&mut self, topic: &str) -> Result<Vec<InferenceMetric>> {
            Ok(self.0.get(topic).cloned().unwrap_or_default())
        }

        fn save(&mut self, topic: &str, metrics: &[InferenceMetric]) -> Result<()> {
            self.0.insert(topic.into(), metrics.to_vec());
            Ok(())
        }
    }

    impl EventBus for Bus {
        fn publish(&mut self, _topic: &str, event: &DriftDetectedEvent) -> Result<()> {
            self.0.push(event.clone());
            Ok(())
        }
    }

    #[test]
    // spec: drift-monitor
    fn emits_drift_when_confidence_drops_below_threshold() {
        let mut store = Store::default();
        let mut bus = Bus(Vec::new());
        let config = DriftConfig {
            window_ms: 1000,
            min_samples: 3,
            confidence_threshold: 0.90,
        };

        for idx in 0..3 {
            let _ = record_metric(
                &mut store,
                &mut bus,
                InferenceMetric {
                    topic: "React 19 hooks".into(),
                    timestamp_ms: idx,
                    fallback_tool_called: idx < 2,
                    high_variance: false,
                },
                config,
            )
            .unwrap();
        }

        assert_eq!(bus.0.len(), 1);
        assert_eq!(bus.0[0].topic, "React 19 hooks");
        assert_eq!(bus.0[0].uncertain_count, 2);
    }

    #[test]
    // spec: drift-monitor
    fn ignores_topics_with_too_few_samples() {
        let mut store = Store::default();
        let mut bus = Bus(Vec::new());

        let event = record_metric(
            &mut store,
            &mut bus,
            InferenceMetric {
                topic: "Rust async".into(),
                timestamp_ms: 1,
                fallback_tool_called: true,
                high_variance: false,
            },
            DriftConfig {
                window_ms: 1000,
                min_samples: 2,
                confidence_threshold: 0.90,
            },
        )
        .unwrap();

        assert!(event.is_none());
        assert!(bus.0.is_empty());
    }
}
