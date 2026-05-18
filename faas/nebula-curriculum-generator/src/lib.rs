#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const AGENT_INFERENCE_TOPIC: &str = "tachyon.agents.inference.pending";
pub const CORRELATION_HEADER: &str = "x-nebula-curriculum-id";
pub const DRIFT_CORRELATION_HEADER: &str = "x-nebula-drift-topic";
pub const DRIFT_TOPIC: &str = "nebula.drift.detected";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurriculumRequest {
    pub curriculum_id: String,
    pub subject: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurriculumTask {
    pub title: String,
    pub description: String,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentInferenceEvent {
    pub topic: String,
    pub prompt: String,
    pub headers: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriftDetectedEvent {
    pub topic: String,
    pub confidence_score: f32,
    pub threshold: f32,
    pub sample_count: usize,
    pub uncertain_count: usize,
}

pub trait TeacherModel {
    fn generate_curriculum(
        &self,
        subject: &str,
        count: usize,
        schema: &serde_json::Value,
    ) -> Result<Vec<CurriculumTask>>;
}

pub trait EventBus {
    fn publish(&mut self, topic: &str, event: &AgentInferenceEvent) -> Result<()>;
}

pub fn generate_and_inject(
    teacher: &impl TeacherModel,
    bus: &mut impl EventBus,
    request: CurriculumRequest,
) -> Result<Vec<AgentInferenceEvent>> {
    if request.count == 0 {
        return Err(anyhow!("curriculum count must be greater than zero"));
    }

    let schema = curriculum_schema();
    let tasks = teacher.generate_curriculum(&request.subject, request.count, &schema)?;
    let mut events = Vec::with_capacity(tasks.len());

    for task in tasks {
        let event = AgentInferenceEvent {
            topic: AGENT_INFERENCE_TOPIC.to_string(),
            prompt: format!(
                "{}\n\n{}\n\nConstraints:\n{}",
                task.title,
                task.description,
                task.constraints.join("\n")
            ),
            headers: serde_json::json!({ CORRELATION_HEADER: request.curriculum_id }),
        };
        bus.publish(AGENT_INFERENCE_TOPIC, &event)?;
        events.push(event);
    }

    Ok(events)
}

pub fn handle_drift_detected(
    teacher: &impl TeacherModel,
    bus: &mut impl EventBus,
    topic: &str,
    raw_payload: &[u8],
    count: usize,
) -> Result<Vec<AgentInferenceEvent>> {
    if topic != DRIFT_TOPIC {
        return Err(anyhow!("unsupported topic: {topic}"));
    }

    let drift: DriftDetectedEvent = serde_json::from_slice(raw_payload)?;
    generate_drift_curriculum(teacher, bus, drift, count)
}

pub fn generate_drift_curriculum(
    teacher: &impl TeacherModel,
    bus: &mut impl EventBus,
    drift: DriftDetectedEvent,
    count: usize,
) -> Result<Vec<AgentInferenceEvent>> {
    if count == 0 {
        return Err(anyhow!("curriculum count must be greater than zero"));
    }

    let subject = format!(
        "{} drift confidence {:.2} below {:.2}",
        drift.topic, drift.confidence_score, drift.threshold
    );
    let tasks = teacher.generate_curriculum(&subject, count, &curriculum_schema())?;
    let mut events = Vec::with_capacity(tasks.len());

    for task in tasks {
        let event = AgentInferenceEvent {
            topic: AGENT_INFERENCE_TOPIC.to_string(),
            prompt: format!(
                "The swarm is failing on tasks related to '{}'.\n{}\n\n{}\n\nConstraints:\n{}",
                drift.topic,
                task.title,
                task.description,
                task.constraints.join("\n")
            ),
            headers: serde_json::json!({
                CORRELATION_HEADER: format!("drift:{}", drift.topic),
                DRIFT_CORRELATION_HEADER: drift.topic
            }),
        };
        bus.publish(AGENT_INFERENCE_TOPIC, &event)?;
        events.push(event);
    }

    Ok(events)
}

fn curriculum_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "array",
        "items": {
            "type": "object",
            "required": ["title", "description", "constraints"],
            "properties": {
                "title": { "type": "string" },
                "description": { "type": "string" },
                "constraints": { "type": "array", "items": { "type": "string" } }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Teacher;
    struct Bus(Vec<AgentInferenceEvent>);

    impl TeacherModel for Teacher {
        fn generate_curriculum(
            &self,
            _subject: &str,
            count: usize,
            _schema: &serde_json::Value,
        ) -> Result<Vec<CurriculumTask>> {
            Ok((0..count)
                .map(|idx| CurriculumTask {
                    title: format!("Task {idx}"),
                    description: "Solve it".into(),
                    constraints: vec!["No docs".into()],
                })
                .collect())
        }
    }

    impl EventBus for Bus {
        fn publish(&mut self, _topic: &str, event: &AgentInferenceEvent) -> Result<()> {
            self.0.push(event.clone());
            Ok(())
        }
    }

    #[test]
    // spec: curriculum-generator
    fn injects_correlation_header() {
        let mut bus = Bus(Vec::new());
        let events = generate_and_inject(
            &Teacher,
            &mut bus,
            CurriculumRequest {
                curriculum_id: "cur-1".into(),
                subject: "Cobol".into(),
                count: 2,
            },
        )
        .unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(bus.0[0].headers[CORRELATION_HEADER], "cur-1");
    }

    #[test]
    // spec: curriculum-generator
    fn drift_event_generates_targeted_curriculum() {
        let mut bus = Bus(Vec::new());
        let events = generate_drift_curriculum(
            &Teacher,
            &mut bus,
            DriftDetectedEvent {
                topic: "React 19 hooks".into(),
                confidence_score: 0.82,
                threshold: 0.90,
                sample_count: 1000,
                uncertain_count: 180,
            },
            1,
        )
        .unwrap();

        assert!(events[0].prompt.contains("React 19 hooks"));
        assert_eq!(
            events[0].headers[DRIFT_CORRELATION_HEADER],
            "React 19 hooks"
        );
    }
}
