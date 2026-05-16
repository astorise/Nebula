use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const TRAINING_READY_TOPIC: &str = "nebula.training.ready";
pub const TRAINING_COMPLETE_TOPIC: &str = "nebula.training.complete";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingReadyEvent {
    pub dataset_path: String,
    pub examples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingConfig {
    pub base_model: String,
    pub output_model: String,
    pub oci_ref: String,
    pub lora_dim: usize,
    pub lora_alpha: usize,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            base_model: "pulsar-base-v1.safetensors".into(),
            output_model: "pulsar-base-v2.safetensors".into(),
            oci_ref: "oci://localhost:5000/pulsar-models/base:v2".into(),
            lora_dim: 16,
            lora_alpha: 32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingCompleteEvent {
    pub artifact_ref: String,
    pub output_model: String,
    pub examples: usize,
}

pub trait Trainer {
    fn train_lora(&mut self, dataset_path: &str, config: &TrainingConfig) -> Result<String>;
    fn merge_adapter(&mut self, adapter_path: &str, config: &TrainingConfig) -> Result<String>;
}

pub trait ArtifactPublisher {
    fn publish_with_wkg(&mut self, model_path: &str, oci_ref: &str) -> Result<String>;
}

pub trait UiNotifier {
    fn notify(&mut self, topic: &str, event: &TrainingCompleteEvent) -> Result<()>;
}

pub fn handle_training_ready(
    trainer: &mut impl Trainer,
    publisher: &mut impl ArtifactPublisher,
    notifier: &mut impl UiNotifier,
    event: TrainingReadyEvent,
    config: TrainingConfig,
) -> Result<TrainingCompleteEvent> {
    let adapter = trainer.train_lora(&event.dataset_path, &config)?;
    let merged_model = trainer.merge_adapter(&adapter, &config)?;
    let artifact_ref = publisher.publish_with_wkg(&merged_model, &config.oci_ref)?;
    let complete = TrainingCompleteEvent {
        artifact_ref,
        output_model: merged_model,
        examples: event.examples,
    };
    notifier.notify(TRAINING_COMPLETE_TOPIC, &complete)?;
    Ok(complete)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TrainerStub;
    struct PublisherStub;
    struct NotifierStub(bool);

    impl Trainer for TrainerStub {
        fn train_lora(&mut self, _dataset_path: &str, _config: &TrainingConfig) -> Result<String> {
            Ok("adapter.safetensors".into())
        }

        fn merge_adapter(&mut self, _adapter_path: &str, config: &TrainingConfig) -> Result<String> {
            Ok(config.output_model.clone())
        }
    }

    impl ArtifactPublisher for PublisherStub {
        fn publish_with_wkg(&mut self, _model_path: &str, oci_ref: &str) -> Result<String> {
            Ok(oci_ref.into())
        }
    }

    impl UiNotifier for NotifierStub {
        fn notify(&mut self, _topic: &str, _event: &TrainingCompleteEvent) -> Result<()> {
            self.0 = true;
            Ok(())
        }
    }

    #[test]
    fn trains_merges_publishes_and_notifies() {
        let mut trainer = TrainerStub;
        let mut publisher = PublisherStub;
        let mut notifier = NotifierStub(false);

        let complete = handle_training_ready(
            &mut trainer,
            &mut publisher,
            &mut notifier,
            TrainingReadyEvent { dataset_path: "dataset_v1.jsonl".into(), examples: 500 },
            TrainingConfig::default(),
        ).unwrap();

        assert_eq!(complete.artifact_ref, "oci://localhost:5000/pulsar-models/base:v2");
        assert!(notifier.0);
    }
}
