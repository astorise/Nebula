use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const INPUT_TOPIC: &str = "nebula.validation.success";
pub const OUTPUT_TOPIC: &str = "nebula.quantization.completed";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationSuccessEvent {
    pub artifact_ref: String,
    pub output_model: String,
    #[serde(default)]
    pub pass_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuantizedVariant {
    pub title: String,
    pub artifact_ref: String,
    pub path: String,
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

pub trait VolumeStore {
    fn read_artifact(&mut self, artifact_ref: &str) -> Result<Vec<u8>>;
    fn write_artifact(&mut self, path: &str, bytes: &[u8]) -> Result<String>;
}

pub trait Quantizer {
    fn quantize(&mut self, source: &[u8], format: QuantizationFormat) -> Result<Vec<u8>>;
}

pub trait EventBus {
    fn publish(&mut self, topic: &str, event: &QuantizationCompletedEvent) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationFormat {
    Fp16,
    Q8_0,
    Q4K,
}

impl QuantizationFormat {
    fn title(self) -> &'static str {
        match self {
            QuantizationFormat::Fp16 => "fp16",
            QuantizationFormat::Q8_0 => "q8_0",
            QuantizationFormat::Q4K => "q4_k",
        }
    }

    fn min_vram_gb(self) -> u32 {
        match self {
            QuantizationFormat::Fp16 => 16,
            QuantizationFormat::Q8_0 => 8,
            QuantizationFormat::Q4K => 4,
        }
    }
}

pub struct DeterministicQuantizer;

impl Quantizer for DeterministicQuantizer {
    fn quantize(&mut self, source: &[u8], format: QuantizationFormat) -> Result<Vec<u8>> {
        match format {
            QuantizationFormat::Fp16 => Ok(source.to_vec()),
            QuantizationFormat::Q8_0 => Ok(compact(source, 2, b"Q8_0")),
            QuantizationFormat::Q4K => Ok(compact(source, 4, b"Q4_K")),
        }
    }
}

pub fn handle_validation_success(
    volume: &mut impl VolumeStore,
    quantizer: &mut impl Quantizer,
    bus: &mut impl EventBus,
    topic: &str,
    raw_payload: &[u8],
) -> Result<QuantizationCompletedEvent> {
    if topic != INPUT_TOPIC {
        return Err(anyhow!("unsupported topic: {topic}"));
    }

    let event: ValidationSuccessEvent = serde_json::from_slice(raw_payload)?;
    quantize_validated_artifact(volume, quantizer, bus, event)
}

pub fn quantize_validated_artifact(
    volume: &mut impl VolumeStore,
    quantizer: &mut impl Quantizer,
    bus: &mut impl EventBus,
    event: ValidationSuccessEvent,
) -> Result<QuantizationCompletedEvent> {
    let source = volume.read_artifact(&event.artifact_ref)?;
    if source.is_empty() {
        return Err(anyhow!("source artifact is empty"));
    }

    let mut variants = Vec::new();
    for format in [
        QuantizationFormat::Fp16,
        QuantizationFormat::Q8_0,
        QuantizationFormat::Q4K,
    ] {
        let bytes = quantizer.quantize(&source, format)?;
        let path = variant_path(&event.output_model, format);
        let artifact_ref = volume.write_artifact(&path, &bytes)?;
        variants.push(QuantizedVariant {
            title: format.title().into(),
            artifact_ref,
            path,
            digest: digest(&bytes),
            size_bytes: bytes.len() as u64,
            min_vram_gb: format.min_vram_gb(),
        });
    }

    let completed = QuantizationCompletedEvent {
        source_artifact: event.artifact_ref,
        output_model: event.output_model,
        variants,
    };
    bus.publish(OUTPUT_TOPIC, &completed)?;
    Ok(completed)
}

fn variant_path(output_model: &str, format: QuantizationFormat) -> String {
    let stem = output_model
        .strip_suffix(".safetensors")
        .unwrap_or(output_model);
    format!("{stem}.{}.safetensors", format.title())
}

fn compact(source: &[u8], divisor: usize, tag: &[u8]) -> Vec<u8> {
    let target_len = (source.len() / divisor).max(1);
    let mut out = Vec::with_capacity(tag.len() + target_len);
    out.extend_from_slice(tag);
    out.extend(source.iter().step_by(divisor).take(target_len));
    out
}

fn digest(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    hex(&hash)
}

fn hex(bytes: &[u8]) -> String {
    const CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(CHARS[(byte >> 4) as usize] as char);
        out.push(CHARS[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct Volume {
        source: Vec<u8>,
        writes: BTreeMap<String, Vec<u8>>,
    }

    struct Bus(Option<QuantizationCompletedEvent>);

    impl VolumeStore for Volume {
        fn read_artifact(&mut self, _artifact_ref: &str) -> Result<Vec<u8>> {
            Ok(self.source.clone())
        }

        fn write_artifact(&mut self, path: &str, bytes: &[u8]) -> Result<String> {
            self.writes.insert(path.into(), bytes.to_vec());
            Ok(format!("oci://localhost:5000/{path}"))
        }
    }

    impl EventBus for Bus {
        fn publish(&mut self, _topic: &str, event: &QuantizationCompletedEvent) -> Result<()> {
            self.0 = Some(event.clone());
            Ok(())
        }
    }

    #[test]
    fn creates_fp16_q8_and_q4_variants() {
        let mut volume = Volume {
            source: (0..64).collect(),
            writes: BTreeMap::new(),
        };
        let mut bus = Bus(None);

        let event = quantize_validated_artifact(
            &mut volume,
            &mut DeterministicQuantizer,
            &mut bus,
            ValidationSuccessEvent {
                artifact_ref: "oci://localhost:5000/pulsar:v3".into(),
                output_model: "pulsar-v3.safetensors".into(),
                pass_rate: 1.0,
            },
        )
        .unwrap();

        assert_eq!(event.variants.len(), 3);
        assert_eq!(event.variants[0].title, "fp16");
        assert_eq!(event.variants[1].title, "q8_0");
        assert_eq!(event.variants[2].min_vram_gb, 4);
        assert_eq!(volume.writes.len(), 3);
        assert_eq!(bus.0.unwrap(), event);
    }
}
