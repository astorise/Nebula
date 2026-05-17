use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const GOSSIP_TOPIC: &str = "nebula.knowledge.manifest";
const BLOOM_BITS: usize = 256;
const RECENT_ID_LIMIT: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeRecord {
    pub id: String,
    pub prompt: String,
    pub response: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeManifest {
    pub node_id: String,
    pub root_ca_fingerprint: String,
    pub sync_endpoint: String,
    pub record_count: usize,
    pub bloom: Vec<u8>,
    pub recent_ids: Vec<String>,
}

pub trait DatasetStore {
    fn record_ids(&mut self) -> Result<Vec<String>>;
    fn upsert_records(&mut self, records: &[KnowledgeRecord]) -> Result<usize>;
}

pub trait GossipBus {
    fn publish(&mut self, topic: &str, manifest: &KnowledgeManifest) -> Result<()>;
}

pub trait SyncClient {
    fn pull_delta(&mut self, endpoint: &str, known_ids: &[String]) -> Result<Vec<KnowledgeRecord>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncOutcome {
    RejectedUntrusted,
    AlreadyCurrent,
    Pulled { inserted: usize },
}

pub fn publish_manifest(
    store: &mut impl DatasetStore,
    bus: &mut impl GossipBus,
    node_id: &str,
    root_ca_fingerprint: &str,
    sync_endpoint: &str,
) -> Result<KnowledgeManifest> {
    let ids = store.record_ids()?;
    let manifest = build_manifest(node_id, root_ca_fingerprint, sync_endpoint, &ids);
    bus.publish(GOSSIP_TOPIC, &manifest)?;
    Ok(manifest)
}

pub fn handle_remote_manifest(
    store: &mut impl DatasetStore,
    client: &mut impl SyncClient,
    local_root_ca_fingerprint: &str,
    manifest: &KnowledgeManifest,
) -> Result<SyncOutcome> {
    if manifest.root_ca_fingerprint != local_root_ca_fingerprint {
        return Ok(SyncOutcome::RejectedUntrusted);
    }

    let local_ids = store.record_ids()?;
    if !has_remote_delta(&local_ids, manifest) {
        return Ok(SyncOutcome::AlreadyCurrent);
    }

    let delta = client.pull_delta(&manifest.sync_endpoint, &local_ids)?;
    let local_set = local_ids.into_iter().collect::<BTreeSet<_>>();
    let missing = delta
        .into_iter()
        .filter(|record| !local_set.contains(&record.id))
        .collect::<Vec<_>>();
    let inserted = store.upsert_records(&missing)?;
    Ok(SyncOutcome::Pulled { inserted })
}

pub fn build_manifest(
    node_id: &str,
    root_ca_fingerprint: &str,
    sync_endpoint: &str,
    ids: &[String],
) -> KnowledgeManifest {
    let ordered = ids.iter().cloned().collect::<BTreeSet<_>>();
    let recent_ids = ordered
        .iter()
        .rev()
        .take(RECENT_ID_LIMIT)
        .cloned()
        .collect::<Vec<_>>();

    KnowledgeManifest {
        node_id: node_id.to_string(),
        root_ca_fingerprint: root_ca_fingerprint.to_string(),
        sync_endpoint: sync_endpoint.to_string(),
        record_count: ordered.len(),
        bloom: bloom_for_ids(ordered.iter()),
        recent_ids,
    }
}

pub fn deterministic_record_id(prompt: &str, response: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prompt.as_bytes());
    hasher.update(b"\0");
    hasher.update(response.as_bytes());
    hex(&hasher.finalize())
}

fn has_remote_delta(local_ids: &[String], manifest: &KnowledgeManifest) -> bool {
    let local = local_ids.iter().collect::<BTreeSet<_>>();
    manifest.record_count > local.len() || manifest.recent_ids.iter().any(|id| !local.contains(id))
}

fn bloom_for_ids<'a>(ids: impl Iterator<Item = &'a String>) -> Vec<u8> {
    let mut bloom = vec![0_u8; BLOOM_BITS / 8];
    for id in ids {
        for index in bloom_indexes(id) {
            bloom[index / 8] |= 1 << (index % 8);
        }
    }
    bloom
}

fn bloom_indexes(id: &str) -> [usize; 3] {
    let digest = Sha256::digest(id.as_bytes());
    [
        u16::from_be_bytes([digest[0], digest[1]]) as usize % BLOOM_BITS,
        u16::from_be_bytes([digest[2], digest[3]]) as usize % BLOOM_BITS,
        u16::from_be_bytes([digest[4], digest[5]]) as usize % BLOOM_BITS,
    ]
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

pub fn decode_manifest(raw: &[u8]) -> Result<KnowledgeManifest> {
    serde_json::from_slice(raw).map_err(|error| anyhow!("invalid knowledge manifest: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Store {
        records: Vec<KnowledgeRecord>,
    }

    struct Bus(Option<KnowledgeManifest>);
    struct Client {
        called: bool,
        delta: Vec<KnowledgeRecord>,
    }

    impl DatasetStore for Store {
        fn record_ids(&mut self) -> Result<Vec<String>> {
            Ok(self
                .records
                .iter()
                .map(|record| record.id.clone())
                .collect())
        }

        fn upsert_records(&mut self, records: &[KnowledgeRecord]) -> Result<usize> {
            let existing = self
                .records
                .iter()
                .map(|record| record.id.clone())
                .collect::<BTreeSet<_>>();
            let mut inserted = 0;
            for record in records {
                if !existing.contains(&record.id) {
                    self.records.push(record.clone());
                    inserted += 1;
                }
            }
            Ok(inserted)
        }
    }

    impl GossipBus for Bus {
        fn publish(&mut self, _topic: &str, manifest: &KnowledgeManifest) -> Result<()> {
            self.0 = Some(manifest.clone());
            Ok(())
        }
    }

    impl SyncClient for Client {
        fn pull_delta(
            &mut self,
            _endpoint: &str,
            _known_ids: &[String],
        ) -> Result<Vec<KnowledgeRecord>> {
            self.called = true;
            Ok(self.delta.clone())
        }
    }

    #[test]
    fn publishes_deterministic_manifest() {
        let mut store = Store {
            records: vec![record("p1", "r1"), record("p2", "r2")],
        };
        let mut bus = Bus(None);

        let manifest = publish_manifest(
            &mut store,
            &mut bus,
            "node-a",
            "root-ca",
            "https://node-a/sync",
        )
        .unwrap();

        assert_eq!(manifest.record_count, 2);
        assert_eq!(manifest.bloom.len(), 32);
        assert_eq!(bus.0.unwrap(), manifest);
    }

    #[test]
    fn pulls_and_inserts_missing_remote_records() {
        let mut store = Store {
            records: vec![record("p1", "r1")],
        };
        let remote_record = record("p2", "r2");
        let manifest = build_manifest(
            "node-b",
            "root-ca",
            "https://node-b/sync",
            &[remote_record.id.clone()],
        );
        let mut client = Client {
            called: false,
            delta: vec![remote_record],
        };

        let outcome =
            handle_remote_manifest(&mut store, &mut client, "root-ca", &manifest).unwrap();

        assert_eq!(outcome, SyncOutcome::Pulled { inserted: 1 });
        assert!(client.called);
        assert_eq!(store.records.len(), 2);
    }

    #[test]
    fn rejects_untrusted_manifest_without_delta_request() {
        let mut store = Store::default();
        let mut client = Client {
            called: false,
            delta: Vec::new(),
        };
        let manifest = build_manifest("node-b", "other-root", "https://node-b/sync", &[]);

        let outcome =
            handle_remote_manifest(&mut store, &mut client, "root-ca", &manifest).unwrap();

        assert_eq!(outcome, SyncOutcome::RejectedUntrusted);
        assert!(!client.called);
    }

    fn record(prompt: &str, response: &str) -> KnowledgeRecord {
        KnowledgeRecord {
            id: deterministic_record_id(prompt, response),
            prompt: prompt.into(),
            response: response.into(),
            metadata: serde_json::json!({}),
        }
    }
}
