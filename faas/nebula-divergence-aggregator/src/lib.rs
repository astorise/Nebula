use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const RESULTS_TOPIC: &str = "nebula.eval.results";
pub const TIER3_QUEUE_KEY: &str = "nebula:tier3:arbitration";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationResult {
    pub diverged: bool,
    pub evaluator: String,
    pub reason: String,
    pub prompt: String,
    pub responses: [String; 3],
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArbitrationTask {
    pub prompt: String,
    pub pulsar_context: serde_json::Value,
    pub hallucinations: [String; 3],
    pub evaluator: String,
    pub reason: String,
}

pub trait KvListStore {
    fn push_json(&mut self, key: &str, value: &ArbitrationTask) -> Result<usize>;
}

pub fn aggregate_and_persist(store: &mut impl KvListStore, topic: &str, result: EvaluationResult) -> Result<Option<usize>> {
    if topic != RESULTS_TOPIC || !result.diverged {
        return Ok(None);
    }

    let task = ArbitrationTask {
        prompt: result.prompt,
        pulsar_context: result.context,
        hallucinations: result.responses,
        evaluator: result.evaluator,
        reason: result.reason,
    };

    store.push_json(TIER3_QUEUE_KEY, &task).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Store(Vec<ArbitrationTask>);

    impl KvListStore for Store {
        fn push_json(&mut self, _key: &str, value: &ArbitrationTask) -> Result<usize> {
            self.0.push(value.clone());
            Ok(self.0.len())
        }
    }

    #[test]
    fn persists_only_divergent_results() {
        let mut store = Store(Vec::new());
        let result = EvaluationResult {
            diverged: true,
            evaluator: "eval".into(),
            reason: "r".into(),
            prompt: "p".into(),
            responses: ["a".into(), "b".into(), "c".into()],
            context: serde_json::json!({ "agent": "pulsar" }),
        };

        assert_eq!(aggregate_and_persist(&mut store, RESULTS_TOPIC, result).unwrap(), Some(1));
    }
}
