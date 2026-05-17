use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const TRAINING_READY_TOPIC: &str = "nebula.training.ready";
pub const TRAINING_COMPLETE_TOPIC: &str = "nebula.training.complete";
pub const QUANTIZATION_COMPLETE_TOPIC: &str = "nebula.quantization.completed";

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantizedVariant {
    pub title: String,
    pub artifact_ref: String,
    pub digest: String,
    pub size_bytes: u64,
    pub min_vram_gb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantizationCompletedEvent {
    pub source_artifact: String,
    pub output_model: String,
    pub variants: Vec<QuantizedVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OciDescriptor {
    pub artifact_ref: String,
    pub digest: String,
    pub size_bytes: u64,
    pub annotations: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OciImageIndex {
    pub tag: String,
    pub manifests: Vec<OciDescriptor>,
}

pub trait Trainer {
    fn train_lora(&mut self, dataset_path: &str, config: &TrainingConfig) -> Result<String>;
    fn merge_adapter(&mut self, adapter_path: &str, config: &TrainingConfig) -> Result<String>;
}

pub trait ArtifactPublisher {
    fn publish_with_wkg(&mut self, model_path: &str, oci_ref: &str) -> Result<String>;
    fn publish_index(&mut self, index: &OciImageIndex) -> Result<String>;
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

pub fn handle_quantization_completed(
    publisher: &mut impl ArtifactPublisher,
    event: QuantizationCompletedEvent,
    config: &TrainingConfig,
) -> Result<OciImageIndex> {
    let index = build_oci_image_index(&event, &config.oci_ref);
    publisher.publish_index(&index)?;
    Ok(index)
}

pub fn build_oci_image_index(event: &QuantizationCompletedEvent, tag: &str) -> OciImageIndex {
    OciImageIndex {
        tag: tag.to_string(),
        manifests: event
            .variants
            .iter()
            .map(|variant| OciDescriptor {
                artifact_ref: variant.artifact_ref.clone(),
                digest: variant.digest.clone(),
                size_bytes: variant.size_bytes,
                annotations: vec![
                    (
                        "org.opencontainers.image.title".into(),
                        variant.title.clone(),
                    ),
                    (
                        "tachyon.mesh/min-vram".into(),
                        format!("{}GB", variant.min_vram_gb),
                    ),
                ],
            })
            .collect(),
    }
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

        fn merge_adapter(
            &mut self,
            _adapter_path: &str,
            config: &TrainingConfig,
        ) -> Result<String> {
            Ok(config.output_model.clone())
        }
    }

    impl ArtifactPublisher for PublisherStub {
        fn publish_with_wkg(&mut self, _model_path: &str, oci_ref: &str) -> Result<String> {
            Ok(oci_ref.into())
        }

        fn publish_index(&mut self, _index: &OciImageIndex) -> Result<String> {
            Ok("oci://localhost:5000/pulsar-models/base:v2".into())
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
            TrainingReadyEvent {
                dataset_path: "dataset_v1.jsonl".into(),
                examples: 500,
            },
            TrainingConfig::default(),
        )
        .unwrap();

        assert_eq!(
            complete.artifact_ref,
            "oci://localhost:5000/pulsar-models/base:v2"
        );
        assert!(notifier.0);
    }

    #[test]
    fn builds_quantized_oci_index_with_vram_annotations() {
        let event = QuantizationCompletedEvent {
            source_artifact: "fp16".into(),
            output_model: "model.safetensors".into(),
            variants: vec![QuantizedVariant {
                title: "q4_k".into(),
                artifact_ref: "oci://localhost:5000/model:q4_k".into(),
                digest: "sha256:abc".into(),
                size_bytes: 128,
                min_vram_gb: 4,
            }],
        };

        let index = build_oci_image_index(&event, "oci://localhost:5000/model:v3");

        assert_eq!(index.manifests.len(), 1);
        assert_eq!(
            index.manifests[0].annotations,
            vec![
                ("org.opencontainers.image.title".into(), "q4_k".into()),
                ("tachyon.mesh/min-vram".into(), "4GB".into())
            ]
        );
    }
}
