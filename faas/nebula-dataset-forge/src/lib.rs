use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const DATASET_FILE: &str = "dataset_v1.jsonl";
pub const TRAINING_READY_TOPIC: &str = "nebula.training.ready";
pub const ESCALATED_TARGET: f32 = 0.60;
pub const DIRECT_TARGET: f32 = 0.40;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExampleSource {
    Escalated,
    Direct,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingExample {
    pub prompt: String,
    pub answer: String,
    pub source: ExampleSource,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingReadyEvent {
    pub dataset_path: String,
    pub examples: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DatasetCounters {
    pub escalated: usize,
    pub direct: usize,
}

impl DatasetCounters {
    pub fn total(self) -> usize {
        self.escalated + self.direct
    }
}

pub trait CounterStore {
    fn counters(&self) -> Result<DatasetCounters>;
    fn increment(&mut self, source: &ExampleSource) -> Result<DatasetCounters>;
}

pub trait VolumeStore {
    fn append_line(&mut self, path: &str, line: &str) -> Result<()>;
}

pub trait EventBus {
    fn publish_training_ready(&mut self, topic: &str, event: &TrainingReadyEvent) -> Result<()>;
}

pub fn append_example(
    counters: &mut impl CounterStore,
    volume: &mut impl VolumeStore,
    bus: &mut impl EventBus,
    example: TrainingExample,
    threshold: usize,
) -> Result<bool> {
    let current = counters.counters()?;
    if !ratio_allows(current, &example.source) {
        return Ok(false);
    }

    let line = serde_json::to_string(&example)?;
    volume.append_line(DATASET_FILE, &line)?;
    let updated = counters.increment(&example.source)?;

    if updated.total() >= threshold {
        bus.publish_training_ready(
            TRAINING_READY_TOPIC,
            &TrainingReadyEvent {
                dataset_path: DATASET_FILE.into(),
                examples: updated.total(),
            },
        )?;
    }

    Ok(true)
}

fn ratio_allows(counters: DatasetCounters, source: &ExampleSource) -> bool {
    let next = match source {
        ExampleSource::Escalated => DatasetCounters {
            escalated: counters.escalated + 1,
            ..counters
        },
        ExampleSource::Direct => DatasetCounters {
            direct: counters.direct + 1,
            ..counters
        },
    };
    let total = next.total() as f32;

    if total < 10.0 {
        return true;
    }

    let escalated_ratio = next.escalated as f32 / total;
    let direct_ratio = next.direct as f32 / total;
    escalated_ratio <= ESCALATED_TARGET + 0.10 && direct_ratio <= DIRECT_TARGET + 0.10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Counters(DatasetCounters);
    struct Volume(Vec<String>);
    struct Bus(usize);

    impl CounterStore for Counters {
        fn counters(&self) -> Result<DatasetCounters> {
            Ok(self.0)
        }
        fn increment(&mut self, source: &ExampleSource) -> Result<DatasetCounters> {
            match source {
                ExampleSource::Escalated => self.0.escalated += 1,
                ExampleSource::Direct => self.0.direct += 1,
            }
            Ok(self.0)
        }
    }

    impl VolumeStore for Volume {
        fn append_line(&mut self, _path: &str, line: &str) -> Result<()> {
            self.0.push(line.into());
            Ok(())
        }
    }

    impl EventBus for Bus {
        fn publish_training_ready(
            &mut self,
            _topic: &str,
            _event: &TrainingReadyEvent,
        ) -> Result<()> {
            self.0 += 1;
            Ok(())
        }
    }

    #[test]
    fn appends_jsonl_and_emits_threshold() {
        let mut counters = Counters::default();
        let mut volume = Volume(Vec::new());
        let mut bus = Bus(0);
        let example = TrainingExample {
            prompt: "p".into(),
            answer: "a".into(),
            source: ExampleSource::Escalated,
            context: serde_json::json!({}),
        };

        assert!(append_example(&mut counters, &mut volume, &mut bus, example, 1).unwrap());
        assert_eq!(volume.0.len(), 1);
        assert_eq!(bus.0, 1);
    }
}
