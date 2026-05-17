# Design: Federated Knowledge Synchronization

The P2P sync agent models the dataset as a grow-only set keyed by deterministic record ids. Local state is advertised as a `KnowledgeManifest` containing node identity, trust-zone fingerprint, record count, recent ids, and a compact Bloom-style fingerprint. The implementation keeps Bloom generation deterministic with SHA-256 so two nodes derive identical fingerprints from identical record sets.

The agent rejects manifests whose Root CA fingerprint does not match the local Tachyon identity zone. Trusted manifests are compared against local ids, and a delta pull is initiated only when the remote node advertises ids absent locally. Pulled records are upserted by id, preserving grow-only CRDT behavior.

Runtime integrations are represented as traits: `DatasetStore` for Tachyon KV or volume-backed JSONL state, `GossipBus` for `tachyon:network/gossip`, and `SyncClient` for secure UDS/HTTP3 delta pulls. This keeps the FaaS crate compilable under WASI while leaving concrete Tachyon bindings replaceable.
