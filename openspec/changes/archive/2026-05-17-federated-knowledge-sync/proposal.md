# Proposal: Federated Knowledge Synchronization (P2P Swarm Learning)

## Context
Nebula currently operates as a localized forge. However, Pulsar swarms are often deployed across heterogeneous, multi-node edge networks without a centralized master server. To prevent duplicated effort and wasted compute (Tier 3 Teacher invocations), Nebula instances must be able to securely gossip and merge their contrastive datasets (`.jsonl` files) across the Tachyon P2P overlay before baking new LoRA weights.

## Objectives
Implement a decentralized knowledge-sharing pipeline:
1. **P2P Discovery & Trust**: Utilize Tachyon's `system-faas-gossip` and identity suite to discover other trusted Nebula instances within the same cryptographic zone.
2. **CRDT Dataset Merging**: Prevent merge conflicts when two nodes learn simultaneously by treating the training dataset as a Grow-Only Set (G-Set CRDT) using mathematical hashing.
3. **Delta Sync Protocol**: Ensure nodes only transmit the *diffs* (newly learned prompt/response pairs) rather than transmitting the entire megabyte-sized `.jsonl` file over constrained edge networks.