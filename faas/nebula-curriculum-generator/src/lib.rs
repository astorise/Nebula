use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const AGENT_INFERENCE_TOPIC: &str = "tachyon.agents.inference.pending";
pub const CORRELATION_HEADER: &str = "x-nebula-curriculum-id";

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
}
