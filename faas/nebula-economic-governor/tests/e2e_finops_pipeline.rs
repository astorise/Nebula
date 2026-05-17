use anyhow::Result;
use nebula_dataset_forge::{
    append_example, dataset_index_key, CounterStore, DatasetCounters, EventBus as DatasetEventBus,
    ExampleSource, IndexStore, TrainingExample, TrainingReadyEvent, VolumeStore,
};
use nebula_economic_governor::{
    evaluate_budget, ArbitrationRequest, BudgetDecision, BudgetStore, ReservationReceipt,
};
use nebula_semantic_deduplicator::{
    classify_divergence, CachedEmbedding, DeduplicationDecision, DivergenceEvent, EmbeddingModel,
    VectorStore,
};
use nebula_teacher_arbitrator::{
    arbitrate_batch, ArbitrationQueue, ArbitrationTask, EventBus as TeacherEventBus,
    LayeredInference, PerfectAnswer,
};
use nebula_tenant_core::{deterministic_test_tenant, TenantId};

struct Model;

impl EmbeddingModel for Model {
    fn embed(&self, _prompt: &str) -> Result<Vec<f32>> {
        Ok(vec![1.0, 0.0])
    }
}

struct Vectors(Vec<CachedEmbedding>);

impl VectorStore for Vectors {
    fn query_recent(&self, _tenant_id: TenantId) -> Result<Vec<CachedEmbedding>> {
        Ok(self.0.clone())
    }
}

struct Budget {
    limit: u64,
    used: u64,
}

impl BudgetStore for Budget {
    fn reserve_if_under_quota(
        &mut self,
        tenant_id: TenantId,
        estimated_tokens: u64,
    ) -> Result<Option<ReservationReceipt>> {
        if self.used + estimated_tokens > self.limit {
            return Ok(None);
        }
        self.used += estimated_tokens;
        Ok(Some(ReservationReceipt {
            tenant_id,
            reserved_tokens: estimated_tokens,
            used_after_reservation: self.used,
        }))
    }

    fn reconcile_tokens(&mut self, _tenant_id: TenantId, estimated: u64, exact: u64) -> Result<()> {
        if exact > estimated {
            self.used += exact - estimated;
        } else {
            self.used -= estimated - exact;
        }
        Ok(())
    }
}

struct Queue(Vec<ArbitrationTask>);
struct Inference;
struct TeacherBus(Vec<PerfectAnswer>);

impl ArbitrationQueue for Queue {
    fn pop_batch(&mut self, limit: usize) -> Result<Vec<ArbitrationTask>> {
        Ok(self.0.drain(..limit.min(self.0.len())).collect())
    }
}

impl LayeredInference for Inference {
    fn layer_count(&self, _model: &str) -> Result<usize> {
        Ok(1)
    }

    fn load_layer(&mut self, _model: &str, _layer: usize) -> Result<()> {
        Ok(())
    }

    fn forward_layer(
        &mut self,
        _model: &str,
        _layer: usize,
        _batch: &[ArbitrationTask],
    ) -> Result<()> {
        Ok(())
    }

    fn unload_layer(&mut self, _model: &str, _layer: usize) -> Result<()> {
        Ok(())
    }

    fn decode_json(
        &mut self,
        _model: &str,
        _schema: &serde_json::Value,
        batch: &[ArbitrationTask],
    ) -> Result<Vec<PerfectAnswer>> {
        Ok(batch
            .iter()
            .map(|task| PerfectAnswer {
                prompt: task.prompt.clone(),
                answer: "corrected".into(),
                source: "tier3".into(),
                context: task.pulsar_context.clone(),
            })
            .collect())
    }
}

impl TeacherEventBus for TeacherBus {
    fn publish(&mut self, _topic: &str, answer: &PerfectAnswer) -> Result<()> {
        self.0.push(answer.clone());
        Ok(())
    }
}

#[derive(Default)]
struct Counters(DatasetCounters);
#[derive(Default)]
struct Volume(Vec<String>);
#[derive(Default)]
struct Index(Vec<String>);
#[derive(Default)]
struct ReadyBus(usize);

impl CounterStore for Counters {
    fn counters(&self) -> Result<DatasetCounters> {
        Ok(self.0)
    }

    fn increment(&mut self, source: &ExampleSource) -> Result<DatasetCounters> {
        match source {
            ExampleSource::Escalated => self.0.escalated += 1,
            ExampleSource::Direct => self.0.direct += 1,
        }
        Ok(self.0)
    }
}

impl VolumeStore for Volume {
    fn append_line(&mut self, _path: &str, line: &str) -> Result<()> {
        self.0.push(line.into());
        Ok(())
    }
}

impl IndexStore for Index {
    fn put_index_key(&mut self, key: &str) -> Result<()> {
        self.0.push(key.into());
        Ok(())
    }
}

impl DatasetEventBus for ReadyBus {
    fn publish_training_ready(&mut self, _topic: &str, _event: &TrainingReadyEvent) -> Result<()> {
        self.0 += 1;
        Ok(())
    }
}

#[test]
fn e2e_finops_pipeline_forwards_only_first_duplicate_to_dataset() {
    let tenant_id = deterministic_test_tenant("acme");
    let mut cache = Vec::new();
    let mut budget = Budget {
        limit: 10_000,
        used: 0,
    };
    let mut teacher_calls = 0;
    let mut counters = Counters::default();
    let mut volume = Volume::default();
    let mut index = Index::default();
    let mut ready_bus = ReadyBus::default();

    for _ in 0..5 {
        let event = DivergenceEvent {
            tenant_id,
            prompt: "same failing prompt".into(),
            context: serde_json::json!({}),
        };
        let decision = classify_divergence(&Model, &Vectors(cache.clone()), &event).unwrap();
        if matches!(decision, DeduplicationDecision::Duplicate { .. }) {
            continue;
        }

        let budget_decision = evaluate_budget(
            &mut budget,
            &ArbitrationRequest {
                tenant_id,
                prompt: event.prompt.clone(),
                context: event.context.clone(),
            },
        )
        .unwrap();
        assert!(matches!(budget_decision, BudgetDecision::Forward { .. }));

        let mut queue = Queue(vec![ArbitrationTask {
            prompt: event.prompt.clone(),
            pulsar_context: event.context,
            hallucinations: ["a".into(), "b".into(), "c".into()],
            evaluator: "semantic".into(),
            reason: "diverged".into(),
        }]);
        let mut inference = Inference;
        let mut bus = TeacherBus(Vec::new());
        let answers = arbitrate_batch(&mut queue, &mut inference, &mut bus, "teacher", 1).unwrap();
        teacher_calls += 1;

        for answer in answers {
            append_example(
                &mut counters,
                &mut volume,
                &mut index,
                &mut ready_bus,
                TrainingExample {
                    prompt: answer.prompt,
                    answer: answer.answer,
                    source: ExampleSource::Escalated,
                    context: serde_json::json!({ "tenant_id": tenant_id }),
                },
                1,
            )
            .unwrap();
        }

        cache.push(CachedEmbedding {
            namespace: "pending_arbitrations".into(),
            prompt: event.prompt,
            embedding: vec![1.0, 0.0],
        });
    }

    assert_eq!(teacher_calls, 1);
    assert_eq!(volume.0.len(), 1);
    assert_eq!(index.0, vec![dataset_index_key("same failing prompt")]);
}
