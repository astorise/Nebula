use anyhow::Result;
use nebula_tenant_core::{
    tenant_dataset_path as core_tenant_dataset_path, tenant_dataset_path_with_prefix, TenantId,
    TenantRegistry,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const DATASET_FILE: &str = "dataset_v1.jsonl";
pub const PREFERENCE_DATASET_FILE: &str = "preference_v1.jsonl";
pub const TOOL_DATASET_FILE: &str = "tool_calls_v1.jsonl";
pub const GOLDEN_DATASET_FILE: &str = "golden_v1.jsonl";
pub const TRAINING_READY_TOPIC: &str = "nebula.training.ready";
pub const DATASET_INDEX_PREFIX: &str = "nebula.dataset.index";
pub const TENANT_ROW_QUOTA: usize = 50_000;
pub const ESCALATED_TARGET: f32 = 0.60;
pub const DIRECT_TARGET: f32 = 0.40;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExampleSource {
    Escalated,
    Direct,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingExample {
    pub prompt: String,
    pub answer: String,
    pub source: ExampleSource,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PreferenceExample {
    pub prompt: String,
    pub chosen: String,
    pub rejected: String,
    #[serde(default)]
    pub tenant_id: Option<String>,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolTrainingExample {
    pub prompt: String,
    pub chosen: String,
    pub rejected: Option<String>,
    pub tool_schema: serde_json::Value,
    pub tool_call: serde_json::Value,
    #[serde(default)]
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoldenTrainingRow {
    pub prompt: String,
    pub answer: String,
    pub vector_score: f32,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReplayConfig {
    pub live_rows: usize,
    pub golden_rows: usize,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            live_rows: 500,
            golden_rows: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingReadyEvent {
    pub dataset_path: String,
    pub examples: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DatasetCounters {
    pub escalated: usize,
    pub direct: usize,
}

impl DatasetCounters {
    pub fn total(self) -> usize {
        self.escalated + self.direct
    }
}

pub trait CounterStore {
    fn counters(&self) -> Result<DatasetCounters>;
    fn increment(&mut self, source: &ExampleSource) -> Result<DatasetCounters>;
}

pub trait VolumeStore {
    fn append_line(&mut self, path: &str, line: &str) -> Result<()>;
}

pub trait IndexStore {
    fn put_index_key(&mut self, key: &str) -> Result<()>;
}

pub trait EventBus {
    fn publish_training_ready(&mut self, topic: &str, event: &TrainingReadyEvent) -> Result<()>;
}

pub fn append_example(
    counters: &mut impl CounterStore,
    volume: &mut impl VolumeStore,
    index: &mut impl IndexStore,
    bus: &mut impl EventBus,
    example: TrainingExample,
    threshold: usize,
) -> Result<bool> {
    let current = counters.counters()?;
    if !ratio_allows(current, &example.source) {
        return Ok(false);
    }

    let line = serde_json::to_string(&example)?;
    volume.append_line(DATASET_FILE, &line)?;
    index.put_index_key(&dataset_index_key(&example.prompt))?;
    let updated = counters.increment(&example.source)?;

    if updated.total() >= threshold {
        bus.publish_training_ready(
            TRAINING_READY_TOPIC,
            &TrainingReadyEvent {
                dataset_path: DATASET_FILE.into(),
                examples: updated.total(),
            },
        )?;
    }

    Ok(true)
}

pub fn append_preference_example(
    volume: &mut impl VolumeStore,
    index: &mut impl IndexStore,
    example: PreferenceExample,
) -> Result<()> {
    let path = example
        .tenant_id
        .as_deref()
        .map(tenant_preference_dataset_path)
        .unwrap_or_else(|| PREFERENCE_DATASET_FILE.into());
    let line = serde_json::to_string(&example)?;
    volume.append_line(&path, &line)?;
    index.put_index_key(&dataset_index_key(&example.prompt))?;
    Ok(())
}

pub fn append_tool_example(
    volume: &mut impl VolumeStore,
    example: ToolTrainingExample,
) -> Result<()> {
    let path = example
        .tenant_id
        .as_deref()
        .map(tenant_tool_dataset_path)
        .unwrap_or_else(|| TOOL_DATASET_FILE.into());
    volume.append_line(&path, &serde_json::to_string(&example)?)?;
    Ok(())
}

pub fn dataset_index_key(prompt: &str) -> String {
    let digest = Sha256::digest(prompt.as_bytes());
    format!("{DATASET_INDEX_PREFIX}:{}", hex(&digest))
}

pub fn tenant_dataset_path(tenant_id: &str) -> String {
    tenant_dataset_path_checked(tenant_id, DATASET_FILE).expect("tenant id must be a valid UUID")
}

pub fn tenant_preference_dataset_path(tenant_id: &str) -> String {
    tenant_dataset_path_checked(tenant_id, PREFERENCE_DATASET_FILE)
        .expect("tenant id must be a valid UUID")
}

pub fn tenant_tool_dataset_path(tenant_id: &str) -> String {
    tenant_dataset_path_checked(tenant_id, TOOL_DATASET_FILE)
        .expect("tenant id must be a valid UUID")
}

pub fn tenant_golden_dataset_path(tenant_id: &str) -> String {
    tenant_dataset_path_checked(tenant_id, GOLDEN_DATASET_FILE)
        .expect("tenant id must be a valid UUID")
}

pub fn tenant_dataset_path_with_registry(
    raw_tenant_id: &str,
    registry: &impl TenantRegistry,
) -> Result<String> {
    let tenant_id = nebula_tenant_core::resolve_tenant(raw_tenant_id, registry)?;
    Ok(core_tenant_dataset_path(tenant_id, DATASET_FILE))
}

pub fn tenant_dataset_path_with_prefix_checked(
    tenant_id: &str,
    prefix: &str,
    file_name: &str,
) -> Result<String> {
    let tenant_id = parse_tenant_id(tenant_id)?;
    Ok(tenant_dataset_path_with_prefix(
        tenant_id, prefix, file_name,
    ))
}

pub fn tenant_index_key(tenant_id: &str, prompt: &str) -> String {
    let digest = Sha256::digest(prompt.as_bytes());
    format!(
        "tenant:{}:dataset:index:{}",
        parse_tenant_id(tenant_id).expect("tenant id must be a valid UUID"),
        hex(&digest)
    )
}

pub fn enforce_tenant_quota(current_rows: usize) -> Result<()> {
    anyhow::ensure!(
        current_rows < TENANT_ROW_QUOTA,
        "tenant dataset quota exceeded"
    );
    Ok(())
}

pub fn mix_with_golden(
    live_rows: &[TrainingExample],
    golden_rows: &[GoldenTrainingRow],
    config: ReplayConfig,
) -> Vec<TrainingExample> {
    let mut batch: Vec<TrainingExample> =
        live_rows.iter().take(config.live_rows).cloned().collect();
    let mut golden: Vec<GoldenTrainingRow> = golden_rows.to_vec();
    golden.sort_by(|left, right| {
        right
            .locked
            .cmp(&left.locked)
            .then_with(|| right.vector_score.total_cmp(&left.vector_score))
    });
    batch.extend(
        golden
            .into_iter()
            .take(config.golden_rows)
            .map(|row| TrainingExample {
                prompt: row.prompt,
                answer: row.answer,
                source: ExampleSource::Escalated,
                context: serde_json::json!({ "source": "golden_dataset" }),
            }),
    );
    batch
}

fn tenant_dataset_path_checked(tenant_id: &str, file_name: &str) -> Result<String> {
    Ok(core_tenant_dataset_path(
        parse_tenant_id(tenant_id)?,
        file_name,
    ))
}

fn parse_tenant_id(tenant_id: &str) -> Result<TenantId> {
    Ok(TenantId(uuid::Uuid::parse_str(tenant_id)?))
}

fn ratio_allows(counters: DatasetCounters, source: &ExampleSource) -> bool {
    let next = match source {
        ExampleSource::Escalated => DatasetCounters {
            escalated: counters.escalated + 1,
            ..counters
        },
        ExampleSource::Direct => DatasetCounters {
            direct: counters.direct + 1,
            ..counters
        },
    };
    let total = next.total() as f32;

    if total < 10.0 {
        return true;
    }

    let escalated_ratio = next.escalated as f32 / total;
    let direct_ratio = next.direct as f32 / total;
    escalated_ratio <= ESCALATED_TARGET + 0.10 && direct_ratio <= DIRECT_TARGET + 0.10
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

    #[derive(Default)]
    struct Counters(DatasetCounters);
    struct Volume(Vec<String>);
    struct Index(Vec<String>);
    struct Bus(usize);

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

    impl EventBus for Bus {
        fn publish_training_ready(
            &mut self,
            _topic: &str,
            _event: &TrainingReadyEvent,
        ) -> Result<()> {
            self.0 += 1;
            Ok(())
        }
    }

    #[test]
    // spec: dataset-forge
    fn appends_jsonl_and_emits_threshold() {
        let mut counters = Counters::default();
        let mut volume = Volume(Vec::new());
        let mut index = Index(Vec::new());
        let mut bus = Bus(0);
        let example = TrainingExample {
            prompt: "p".into(),
            answer: "a".into(),
            source: ExampleSource::Escalated,
            context: serde_json::json!({}),
        };

        assert!(
            append_example(&mut counters, &mut volume, &mut index, &mut bus, example, 1).unwrap()
        );
        assert_eq!(volume.0.len(), 1);
        assert_eq!(index.0, vec![dataset_index_key("p")]);
        assert_eq!(bus.0, 1);
    }

    #[test]
    // spec: dataset-forge
    fn writes_preference_triplets() {
        let mut volume = Volume(Vec::new());
        let mut index = Index(Vec::new());

        append_preference_example(
            &mut volume,
            &mut index,
            PreferenceExample {
                prompt: "p".into(),
                chosen: "safe".into(),
                rejected: "unsafe".into(),
                tenant_id: Some(nebula_tenant_core::deterministic_test_tenant("acme").to_string()),
                context: serde_json::json!({}),
            },
        )
        .unwrap();

        assert!(volume.0[0].contains("\"chosen\":\"safe\""));
        assert_eq!(index.0, vec![dataset_index_key("p")]);
    }

    #[test]
    // spec: dataset-forge
    #[should_panic(expected = "tenant id must be a valid UUID")]
    fn tenant_path_rejects_unresolved_tenant_strings() {
        let _ = tenant_dataset_path("acme/prod");
    }

    #[test]
    // spec: dataset-forge
    fn builds_tenant_paths_and_mixes_replay_rows() {
        let tenant_id = nebula_tenant_core::deterministic_test_tenant("acme/prod").to_string();
        assert_eq!(
            tenant_dataset_path(&tenant_id),
            format!(
                "/mnt/forge/tenants/{}/dataset_v1.jsonl",
                nebula_tenant_core::deterministic_test_tenant("acme/prod")
            )
        );

        let live = vec![TrainingExample {
            prompt: "live".into(),
            answer: "a".into(),
            source: ExampleSource::Direct,
            context: serde_json::json!({}),
        }];
        let golden = vec![GoldenTrainingRow {
            prompt: "gold".into(),
            answer: "b".into(),
            vector_score: 0.9,
            locked: false,
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
        assert_eq!(batch[1].prompt, "gold");
    }
}
