# p2p-sync-agent Specification

## ADDED Requirements

### Requirement: P2P sync agent advertises local dataset state

The `nebula-p2p-sync-agent` Wasm component SHALL publish a knowledge manifest over the Tachyon gossip boundary containing node identity, trust zone, dataset count, and a compact dataset fingerprint.

#### Scenario: Local dataset changes

- **GIVEN** the local dataset contains contrastive training records
- **WHEN** the sync agent publishes a manifest
- **THEN** it includes the local node id
- **AND** it includes the trusted Root CA fingerprint
- **AND** it includes a deterministic Bloom-style fingerprint of local record ids

### Requirement: P2P sync agent pulls only missing records

The sync agent SHALL compare remote manifests with the local dataset state and pull only records the local node does not already have.

#### Scenario: Remote node has new records

- **GIVEN** a trusted remote manifest advertises records not present locally
- **WHEN** the sync agent handles the manifest
- **THEN** it requests a delta from the remote sync endpoint
- **AND** it persists only records whose ids are not already present locally

#### Scenario: Remote node has no new records

- **GIVEN** a trusted remote manifest matches the local dataset state
- **WHEN** the sync agent handles the manifest
- **THEN** it does not request a delta from the remote endpoint

### Requirement: P2P sync agent enforces trust zone boundaries

The sync agent SHALL reject manifests from nodes outside the configured Tachyon identity Root CA zone before any delta request is made.

#### Scenario: Remote Root CA does not match

- **GIVEN** a remote manifest contains a different Root CA fingerprint
- **WHEN** the sync agent handles the manifest
- **THEN** it rejects the manifest
- **AND** no sync endpoint request is made
