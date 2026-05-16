use anyhow::{anyhow, Result};
use prost::Message;
use serde::{Deserialize, Serialize};

pub const INPUT_TOPIC: &str = "pulsar.telemetry.inference_triplets";
pub const AST_TOPIC: &str = "nebula.eval.ast.pending";
pub const SEMANTIC_TOPIC: &str = "nebula.eval.semantic.pending";
pub const AST_MICROVM_URL: &str =
    "http://nebula-eval-ast.microvm.internal/nebula.ast.AstEvaluator/EvaluateTriplets";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InferenceTriplet {
    pub prompt: String,
    pub task_type: String,
    pub responses: [String; 3],
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutedEvent {
    pub topic: String,
    pub payload: InferenceTriplet,
}

pub trait EventBus {
    fn publish(&mut self, topic: &str, payload: &InferenceTriplet) -> Result<()>;
}

pub trait WasiHttpClient {
    fn post_grpc(&mut self, url: &str, body: &[u8]) -> Result<Vec<u8>>;
}

#[derive(Clone, PartialEq, Message)]
pub struct AstEvaluationRequest {
    #[prost(string, tag = "1")]
    pub language: String,
    #[prost(string, repeated, tag = "2")]
    pub responses: Vec<String>,
}

pub fn route_triplet(payload: InferenceTriplet) -> RoutedEvent {
    let normalized = payload.task_type.to_ascii_lowercase();
    let topic = if is_code_task(&normalized) {
        AST_TOPIC
    } else {
        SEMANTIC_TOPIC
    };

    RoutedEvent {
        topic: topic.to_string(),
        payload,
    }
}

pub fn handle_event(
    bus: &mut impl EventBus,
    topic: &str,
    raw_payload: &[u8],
) -> Result<RoutedEvent> {
    if topic != INPUT_TOPIC {
        return Err(anyhow!("unsupported topic: {topic}"));
    }

    let triplet: InferenceTriplet = serde_json::from_slice(raw_payload)?;
    let event = route_triplet(triplet);
    bus.publish(&event.topic, &event.payload)?;
    Ok(event)
}

pub fn dispatch_ast_to_microvm(
    client: &mut impl WasiHttpClient,
    triplet: &InferenceTriplet,
) -> Result<Vec<u8>> {
    let request = AstEvaluationRequest {
        language: infer_language(&triplet.task_type),
        responses: triplet.responses.to_vec(),
    };
    let frame = encode_grpc_frame(&request)?;
    client.post_grpc(AST_MICROVM_URL, &frame)
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

fn is_code_task(task_type: &str) -> bool {
    matches!(task_type, "code_generation" | "rust" | "cobol" | "wasm")
        || ["code", "rust", "cobol", "wasm", "typescript", "python"]
            .iter()
            .any(|tag| task_type.contains(tag))
}

fn infer_language(task_type: &str) -> String {
    let normalized = task_type.to_ascii_lowercase();
    ["rust", "cobol", "wasm", "typescript", "python"]
        .iter()
        .find(|language| normalized.contains(**language))
        .copied()
        .unwrap_or("unknown")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_code_to_ast() {
        let event = route_triplet(InferenceTriplet {
            prompt: "write rust".into(),
            task_type: "rust_code_generation".into(),
            responses: ["a".into(), "b".into(), "c".into()],
            context: serde_json::json!({}),
        });

        assert_eq!(event.topic, AST_TOPIC);
    }

    #[test]
    fn routes_free_text_to_semantic() {
        let event = route_triplet(InferenceTriplet {
            prompt: "reason".into(),
            task_type: "reasoning".into(),
            responses: ["a".into(), "b".into(), "c".into()],
            context: serde_json::json!({}),
        });

        assert_eq!(event.topic, SEMANTIC_TOPIC);
    }

    #[test]
    fn encodes_grpc_frame_for_ast_microvm() {
        let frame = encode_grpc_frame(&AstEvaluationRequest {
            language: "rust".into(),
            responses: vec!["a".into(), "b".into(), "c".into()],
        })
        .unwrap();

        assert_eq!(frame[0], 0);
        assert!(frame.len() > 5);
    }
}
