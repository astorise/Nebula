#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const DATASET_FILE: &str = "dataset_v1.jsonl";
pub const TRAINING_READY_TOPIC: &str = "nebula.training.ready";
pub const TENANT_CRDT_PREFIX: &str = "tenant";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoteTrainingRow {
    pub prompt: String,
    pub correction: String,
    #[serde(default)]
    pub teacher_score: Option<f32>,
    #[serde(default)]
    pub source_node: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatasetRow {
    pub id: String,
    pub prompt: String,
    pub correction: String,
    #[serde(default)]
    pub teacher_score: Option<f32>,
    #[serde(default)]
    pub source_node: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrainingReadyEvent {
    pub dataset_path: String,
    pub examples: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MergeReport {
    pub inserted: usize,
    pub skipped: usize,
    pub total: usize,
    pub training_triggered: bool,
}

pub trait DatasetStore {
    fn contains_row(&mut self, id: &str) -> Result<bool>;
    fn append_row(&mut self, row: &DatasetRow) -> Result<()>;
    fn increment_counter(&mut self) -> Result<usize>;
}

pub trait EventBus {
    fn publish_training_ready(&mut self, topic: &str, event: &TrainingReadyEvent) -> Result<()>;
}

pub fn merge_remote_rows(
    store: &mut impl DatasetStore,
    bus: &mut impl EventBus,
    rows: &[RemoteTrainingRow],
    threshold: usize,
) -> Result<MergeReport> {
    let mut inserted = 0;
    let mut skipped = 0;
    let mut total = 0;
    let mut training_triggered = false;

    for row in rows {
        let normalized = normalize_row(row);
        if store.contains_row(&normalized.id)? {
            skipped += 1;
            continue;
        }

        store.append_row(&normalized)?;
        total = store.increment_counter()?;
        inserted += 1;

        if total >= threshold && !training_triggered {
            bus.publish_training_ready(
                TRAINING_READY_TOPIC,
                &TrainingReadyEvent {
                    dataset_path: DATASET_FILE.into(),
                    examples: total,
                },
            )?;
            training_triggered = true;
        }
    }

    Ok(MergeReport {
        inserted,
        skipped,
        total,
        training_triggered,
    })
}

pub fn normalize_row(row: &RemoteTrainingRow) -> DatasetRow {
    DatasetRow {
        id: deterministic_row_id(&row.prompt),
        prompt: row.prompt.clone(),
        correction: row.correction.clone(),
        teacher_score: row.teacher_score,
        source_node: row.source_node.clone(),
        tenant_id: row.tenant_id.clone(),
    }
}

pub fn deterministic_row_id(prompt: &str) -> String {
    let digest = Sha256::digest(prompt.as_bytes());
    hex(&digest)
}

pub fn tenant_crdt_key(tenant_id: &str, prompt: &str) -> String {
    format!(
        "{TENANT_CRDT_PREFIX}:{}:crdt:hash:{}",
        sanitize_tenant_id(tenant_id),
        deterministic_row_id(prompt)
    )
}

fn sanitize_tenant_id(tenant_id: &str) -> String {
    tenant_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect()
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
    use std::collections::BTreeSet;

    #[derive(Default)]
    struct Store {
        ids: BTreeSet<String>,
        rows: Vec<DatasetRow>,
        counter: usize,
    }

    struct Bus(usize);

    impl DatasetStore for Store {
        fn contains_row(&mut self, id: &str) -> Result<bool> {
            Ok(self.ids.contains(id))
        }

        fn append_row(&mut self, row: &DatasetRow) -> Result<()> {
            self.ids.insert(row.id.clone());
            self.rows.push(row.clone());
            Ok(())
        }

        fn increment_counter(&mut self) -> Result<usize> {
            self.counter += 1;
            Ok(self.counter)
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
    // spec: crdt-merger
    fn uses_prompt_hash_as_stable_row_id() {
        assert_eq!(deterministic_row_id("same"), deterministic_row_id("same"));
        assert_ne!(deterministic_row_id("same"), deterministic_row_id("other"));
    }

    #[test]
    // spec: crdt-merger
    fn skips_existing_rows() {
        let existing = deterministic_row_id("p");
        let mut store = Store {
            ids: BTreeSet::from([existing]),
            ..Store::default()
        };
        let mut bus = Bus(0);

        let report = merge_remote_rows(&mut store, &mut bus, &[row("p")], 1).unwrap();

        assert_eq!(report.inserted, 0);
        assert_eq!(report.skipped, 1);
        assert_eq!(store.rows.len(), 0);
        assert_eq!(bus.0, 0);
    }

    #[test]
    // spec: crdt-merger
    fn appends_unique_rows_and_triggers_training() {
        let mut store = Store::default();
        let mut bus = Bus(0);

        let report = merge_remote_rows(&mut store, &mut bus, &[row("p")], 1).unwrap();

        assert_eq!(report.inserted, 1);
        assert_eq!(store.rows.len(), 1);
        assert!(report.training_triggered);
        assert_eq!(bus.0, 1);
    }

    fn row(prompt: &str) -> RemoteTrainingRow {
        RemoteTrainingRow {
            prompt: prompt.into(),
            correction: "fixed".into(),
            teacher_score: Some(0.99),
            source_node: Some("node-b".into()),
            tenant_id: None,
        }
    }

    #[test]
    // spec: crdt-merger
    fn prefixes_crdt_key_with_tenant_namespace() {
        let key = tenant_crdt_key("acme/prod", "prompt");
        assert!(key.starts_with("tenant:acme_prod:crdt:hash:"));
    }
}
