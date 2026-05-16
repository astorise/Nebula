use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const DATASET_APPEND_TOPIC: &str = "nebula.dataset.append";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArbitrationTask {
    pub prompt: String,
    pub pulsar_context: serde_json::Value,
    pub hallucinations: [String; 3],
    pub evaluator: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerfectAnswer {
    pub prompt: String,
    pub answer: String,
    pub source: String,
    pub context: serde_json::Value,
}

pub trait ArbitrationQueue {
    fn pop_batch(&mut self, limit: usize) -> Result<Vec<ArbitrationTask>>;
}

pub trait LayeredInference {
    fn layer_count(&self, model: &str) -> Result<usize>;
    fn load_layer(&mut self, model: &str, layer: usize) -> Result<()>;
    fn forward_layer(&mut self, model: &str, layer: usize, batch: &[ArbitrationTask]) -> Result<()>;
    fn unload_layer(&mut self, model: &str, layer: usize) -> Result<()>;
    fn decode_json(&mut self, model: &str, schema: &serde_json::Value, batch: &[ArbitrationTask]) -> Result<Vec<PerfectAnswer>>;
}

pub trait EventBus {
    fn publish(&mut self, topic: &str, answer: &PerfectAnswer) -> Result<()>;
}

pub fn arbitrate_batch(
    queue: &mut impl ArbitrationQueue,
    inference: &mut impl LayeredInference,
    bus: &mut impl EventBus,
    model: &str,
    limit: usize,
) -> Result<Vec<PerfectAnswer>> {
    if limit == 0 {
        return Err(anyhow!("batch limit must be greater than zero"));
    }

    let batch = queue.pop_batch(limit)?;
    if batch.is_empty() {
        return Ok(Vec::new());
    }

    let layer_count = inference.layer_count(model)?;
    for layer in 0..layer_count {
        inference.load_layer(model, layer)?;
        inference.forward_layer(model, layer, &batch)?;
        inference.unload_layer(model, layer)?;
    }

    let answers = inference.decode_json(model, &answer_schema(), &batch)?;
    for answer in &answers {
        bus.publish(DATASET_APPEND_TOPIC, answer)?;
    }

    Ok(answers)
}

fn answer_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["answer"],
        "properties": {
            "answer": { "type": "string" }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Queue(Vec<ArbitrationTask>);
    struct Inference(Vec<String>);
    struct Bus(usize);

    impl ArbitrationQueue for Queue {
        fn pop_batch(&mut self, limit: usize) -> Result<Vec<ArbitrationTask>> {
            Ok(self.0.drain(..limit.min(self.0.len())).collect())
        }
    }

    impl LayeredInference for Inference {
        fn layer_count(&self, _model: &str) -> Result<usize> { Ok(2) }
        fn load_layer(&mut self, _model: &str, layer: usize) -> Result<()> { self.0.push(format!("load:{layer}")); Ok(()) }
        fn forward_layer(&mut self, _model: &str, layer: usize, _batch: &[ArbitrationTask]) -> Result<()> { self.0.push(format!("forward:{layer}")); Ok(()) }
        fn unload_layer(&mut self, _model: &str, layer: usize) -> Result<()> { self.0.push(format!("unload:{layer}")); Ok(()) }
        fn decode_json(&mut self, _model: &str, _schema: &serde_json::Value, batch: &[ArbitrationTask]) -> Result<Vec<PerfectAnswer>> {
            Ok(batch.iter().map(|task| PerfectAnswer {
                prompt: task.prompt.clone(),
                answer: "fixed".into(),
                source: "tier3".into(),
                context: task.pulsar_context.clone(),
            }).collect())
        }
    }

    impl EventBus for Bus {
        fn publish(&mut self, _topic: &str, _answer: &PerfectAnswer) -> Result<()> {
            self.0 += 1;
            Ok(())
        }
    }

    #[test]
    fn runs_layer_by_layer_before_publish() {
        let task = ArbitrationTask {
            prompt: "p".into(),
            pulsar_context: serde_json::json!({}),
            hallucinations: ["a".into(), "b".into(), "c".into()],
            evaluator: "eval".into(),
            reason: "diverged".into(),
        };
        let mut queue = Queue(vec![task]);
        let mut inference = Inference(Vec::new());
        let mut bus = Bus(0);

        let answers = arbitrate_batch(&mut queue, &mut inference, &mut bus, "deepseek", 8).unwrap();

        assert_eq!(answers.len(), 1);
        assert_eq!(bus.0, 1);
        assert_eq!(inference.0[0], "load:0");
    }
}
