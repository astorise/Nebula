use anyhow::Result;
use nebula_dataset_forge::{
    mix_with_golden, ExampleSource, GoldenTrainingRow, ReplayConfig, TrainingExample,
};
use nebula_golden_dataset_manager::{promote_if_stable, CandidateRow, GoldenRow, GoldenStore};
use nebula_tenant_core::{deterministic_test_tenant, TenantId, TenantRegistry};
use nebula_tenant_router::{
    route_tenant_triplet_with_registry, TelemetryTriplet, TenantRouterConfig,
};
use std::collections::BTreeMap;
use uuid::Uuid;

struct Registry;

impl TenantRegistry for Registry {
    fn lookup_tenant_uuid(&self, raw_id: &str) -> Result<Option<Uuid>> {
        Ok((raw_id == "acme").then(|| deterministic_test_tenant(raw_id).0))
    }

    fn tenant_row_count(&self, _tenant_id: TenantId) -> Result<usize> {
        Ok(1)
    }
}

#[derive(Default)]
struct Store {
    promoted: Vec<GoldenRow>,
}

impl GoldenStore for Store {
    fn append(&mut self, row: &GoldenRow) -> Result<()> {
        self.promoted.push(row.clone());
        Ok(())
    }

    fn vectorize(&mut self, _namespace: &str, _row: &GoldenRow) -> Result<()> {
        Ok(())
    }
}

#[test]
fn routes_registered_payload_into_golden_replay_batch() {
    let mut context = BTreeMap::new();
    context.insert("x-tenant-id".into(), "acme".into());

    let routed = route_tenant_triplet_with_registry(
        TelemetryTriplet {
            prompt: "Write safe Rust file IO".into(),
            answer: "Return Result from read_to_string".into(),
            context,
        },
        &TenantRouterConfig::default(),
        &Registry,
    )
    .unwrap()
    .unwrap();

    let mut store = Store::default();
    let promoted = promote_if_stable(
        &mut store,
        CandidateRow {
            prompt: routed.payload.prompt.clone(),
            answer: routed.payload.answer.clone(),
            tenant_id: routed.tenant_id.to_string(),
            days_in_production: 8,
            rollback_count: 0,
            drift_count: 0,
        },
    )
    .unwrap()
    .unwrap();

    let live = vec![TrainingExample {
        prompt: "live".into(),
        answer: "answer".into(),
        source: ExampleSource::Direct,
        context: serde_json::json!({ "tenant_id": routed.tenant_id }),
    }];
    let golden = vec![GoldenTrainingRow {
        prompt: promoted.prompt,
        answer: promoted.answer,
        vector_score: 1.0,
        locked: true,
    }];

    let batch = mix_with_golden(
        &live,
        &golden,
        ReplayConfig {
            live_rows: 1,
            golden_rows: 1,
        },
    );

    assert_eq!(batch.len(), 2);
    assert_eq!(batch[1].prompt, "Write safe Rust file IO");
}
