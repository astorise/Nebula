use anyhow::{anyhow, Result};
use prost::Message;
use serde::{Deserialize, Serialize};

pub const INPUT_TOPIC: &str = "nebula.training.complete";
pub const VALIDATION_SUCCESS_TOPIC: &str = "nebula.validation.success";
pub const VALIDATION_FAILED_TOPIC: &str = "nebula.validation.failed";
pub const AST_MICROVM_URL: &str =
    "http://nebula-eval-ast.microvm.internal/nebula.ast.AstEvaluator/EvaluateTriplets";
const SAMPLE_LIMIT: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingCompleteEvent {
    pub artifact_ref: String,
    pub output_model: String,
    pub examples: usize,
    #[serde(default)]
    pub batch_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FailedPrompt {
    pub prompt: String,
    pub language: String,
    pub before: [String; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplaySample {
    pub prompt: String,
    pub language: String,
    pub before: [String; 3],
    pub after: [String; 3],
    pub diverged: bool,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationReport {
    pub artifact_ref: String,
    pub output_model: String,
    pub pass_rate: f32,
    pub samples: Vec<ReplaySample>,
}

pub trait FailedPromptStore {
    fn sample_failed_prompts(
        &mut self,
        batch_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<FailedPrompt>>;
}

pub trait InferenceHost {
    fn generate_with_adapter(
        &mut self,
        artifact_ref: &str,
        output_model: &str,
        prompt: &str,
        temperature: f32,
    ) -> Result<String>;
}

pub trait AstEvaluator {
    fn evaluate_triplets(
        &mut self,
        language: &str,
        responses: &[String; 3],
    ) -> Result<AstEvaluationResponse>;
}

pub trait EventBus {
    fn publish(&mut self, topic: &str, report: &ValidationReport) -> Result<()>;
}

#[derive(Clone, PartialEq, Message)]
pub struct AstEvaluationRequest {
    #[prost(string, tag = "1")]
    pub language: String,
    #[prost(string, repeated, tag = "2")]
    pub responses: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct AstEvaluationResponse {
    #[prost(bool, tag = "1")]
    pub diverged: bool,
    #[prost(string, optional, tag = "2")]
    pub fallback_reason: Option<String>,
}

pub trait WasiHttpClient {
    fn post_grpc(&mut self, url: &str, body: &[u8]) -> Result<Vec<u8>>;
}

pub fn validate_lora(
    store: &mut impl FailedPromptStore,
    inference: &mut impl InferenceHost,
    evaluator: &mut impl AstEvaluator,
    bus: &mut impl EventBus,
    event: TrainingCompleteEvent,
) -> Result<ValidationReport> {
    let prompts = store.sample_failed_prompts(event.batch_id.as_deref(), SAMPLE_LIMIT)?;
    if prompts.is_empty() {
        return Err(anyhow!("no failed prompts available for validation"));
    }

    let mut samples = Vec::with_capacity(prompts.len());
    for failed in prompts {
        let after = replay_prompt(inference, &event, &failed.prompt)?;
        let evaluation = evaluator.evaluate_triplets(&failed.language, &after)?;
        samples.push(ReplaySample {
            prompt: failed.prompt,
            language: failed.language,
            before: failed.before,
            after,
            diverged: evaluation.diverged,
            fallback_reason: evaluation.fallback_reason,
        });
    }

    let passed = samples.iter().filter(|sample| !sample.diverged).count();
    let pass_rate = passed as f32 / samples.len() as f32;
    let report = ValidationReport {
        artifact_ref: event.artifact_ref,
        output_model: event.output_model,
        pass_rate,
        samples,
    };
    let topic = if passed == report.samples.len() {
        VALIDATION_SUCCESS_TOPIC
    } else {
        VALIDATION_FAILED_TOPIC
    };
    bus.publish(topic, &report)?;
    Ok(report)
}

pub fn handle_training_complete(
    store: &mut impl FailedPromptStore,
    inference: &mut impl InferenceHost,
    evaluator: &mut impl AstEvaluator,
    bus: &mut impl EventBus,
    topic: &str,
    raw_payload: &[u8],
) -> Result<ValidationReport> {
    if topic != INPUT_TOPIC {
        return Err(anyhow!("unsupported topic: {topic}"));
    }

    let event: TrainingCompleteEvent = serde_json::from_slice(raw_payload)?;
    validate_lora(store, inference, evaluator, bus, event)
}

pub fn evaluate_triplets_via_grpc(
    client: &mut impl WasiHttpClient,
    language: &str,
    responses: &[String; 3],
) -> Result<AstEvaluationResponse> {
    let request = AstEvaluationRequest {
        language: language.to_string(),
        responses: responses.to_vec(),
    };
    let response = client.post_grpc(AST_MICROVM_URL, &encode_grpc_frame(&request)?)?;
    decode_grpc_response(&response)
}

pub fn encode_grpc_frame(message: &impl Message) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    message.encode(&mut payload)?;

    let mut frame = Vec::with_capacity(payload.len() + 5);
    frame.push(0);
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

pub fn decode_grpc_response(frame: &[u8]) -> Result<AstEvaluationResponse> {
    if frame.len() < 5 {
        return Err(anyhow!("invalid gRPC frame"));
    }

    let len = u32::from_be_bytes([frame[1], frame[2], frame[3], frame[4]]) as usize;
    let payload = frame
        .get(5..5 + len)
        .ok_or_else(|| anyhow!("truncated gRPC frame"))?;
    Ok(AstEvaluationResponse::decode(payload)?)
}

fn replay_prompt(
    inference: &mut impl InferenceHost,
    event: &TrainingCompleteEvent,
    prompt: &str,
) -> Result<[String; 3]> {
    Ok([
        inference.generate_with_adapter(&event.artifact_ref, &event.output_model, prompt, 0.1)?,
        inference.generate_with_adapter(&event.artifact_ref, &event.output_model, prompt, 0.8)?,
        inference.generate_with_adapter(&event.artifact_ref, &event.output_model, prompt, 0.8)?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Store(Vec<FailedPrompt>);
    struct Inference;
    struct Evaluator(bool);
    struct Bus(String);

    impl FailedPromptStore for Store {
        fn sample_failed_prompts(
            &mut self,
            _batch_id: Option<&str>,
            limit: usize,
        ) -> Result<Vec<FailedPrompt>> {
            Ok(self.0.iter().take(limit).cloned().collect())
        }
    }

    impl InferenceHost for Inference {
        fn generate_with_adapter(
            &mut self,
            _artifact_ref: &str,
            _output_model: &str,
            prompt: &str,
            temperature: f32,
        ) -> Result<String> {
            Ok(format!("fixed:{prompt}:{temperature}"))
        }
    }

    impl AstEvaluator for Evaluator {
        fn evaluate_triplets(
            &mut self,
            _language: &str,
            _responses: &[String; 3],
        ) -> Result<AstEvaluationResponse> {
            Ok(AstEvaluationResponse {
                diverged: self.0,
                fallback_reason: None,
            })
        }
    }

    impl EventBus for Bus {
        fn publish(&mut self, topic: &str, _report: &ValidationReport) -> Result<()> {
            self.0 = topic.to_string();
            Ok(())
        }
    }

    #[test]
    // spec: lora-validator
    fn emits_success_when_all_prompts_converge() {
        let mut store = Store(vec![FailedPrompt {
            prompt: "write rust".into(),
            language: "rust".into(),
            before: ["a".into(), "b".into(), "c".into()],
        }]);
        let mut bus = Bus(String::new());

        let report = validate_lora(
            &mut store,
            &mut Inference,
            &mut Evaluator(false),
            &mut bus,
            TrainingCompleteEvent {
                artifact_ref: "oci://localhost:5000/pulsar-lora:v2".into(),
                output_model: "pulsar-base-v2.safetensors".into(),
                examples: 500,
                batch_id: Some("batch-1".into()),
            },
        )
        .unwrap();

        assert_eq!(report.pass_rate, 1.0);
        assert_eq!(bus.0, VALIDATION_SUCCESS_TOPIC);
    }

    #[test]
    // spec: lora-validator
    fn emits_failure_when_any_prompt_still_diverges() {
        let mut store = Store(vec![FailedPrompt {
            prompt: "write rust".into(),
            language: "rust".into(),
            before: ["a".into(), "b".into(), "c".into()],
        }]);
        let mut bus = Bus(String::new());

        let report = validate_lora(
            &mut store,
            &mut Inference,
            &mut Evaluator(true),
            &mut bus,
            TrainingCompleteEvent {
                artifact_ref: "oci://localhost:5000/pulsar-lora:v2".into(),
                output_model: "pulsar-base-v2.safetensors".into(),
                examples: 500,
                batch_id: None,
            },
        )
        .unwrap();

        assert_eq!(report.pass_rate, 0.0);
        assert_eq!(bus.0, VALIDATION_FAILED_TOPIC);
    }

    #[test]
    // spec: lora-validator
    fn decodes_ast_microvm_grpc_response() {
        let response = AstEvaluationResponse {
            diverged: false,
            fallback_reason: None,
        };

        let frame = encode_grpc_frame(&response).unwrap();
        assert!(!decode_grpc_response(&frame).unwrap().diverged);
    }
}
