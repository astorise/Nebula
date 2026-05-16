use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const INPUT_TOPIC: &str = "pulsar.telemetry.inference_triplets";
pub const AST_TOPIC: &str = "nebula.eval.ast.pending";
pub const SEMANTIC_TOPIC: &str = "nebula.eval.semantic.pending";

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

fn is_code_task(task_type: &str) -> bool {
    matches!(task_type, "code_generation" | "rust" | "cobol" | "wasm")
        || ["code", "rust", "cobol", "wasm", "typescript", "python"]
            .iter()
            .any(|tag| task_type.contains(tag))
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
}
