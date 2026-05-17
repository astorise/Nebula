# crdt-dataset-merger Specification

## Purpose
TBD - created by archiving change federated-knowledge-sync. Update Purpose after archive.
## Requirements
### Requirement: CRDT merger assigns deterministic row ids

The CRDT dataset merger SHALL identify incoming training rows with a deterministic SHA-256 id derived from the prompt.

#### Scenario: Remote row is normalized

- **GIVEN** an incoming remote training row contains a prompt and correction
- **WHEN** the merger normalizes the row
- **THEN** it computes the same id for the same prompt on every node

### Requirement: CRDT merger appends only unique rows

The CRDT dataset merger SHALL treat the dataset as a grow-only set and silently drop incoming rows whose ids already exist locally.

#### Scenario: Remote row already exists

- **GIVEN** the local dataset already contains a row id
- **WHEN** a remote row with the same id is merged
- **THEN** the merger does not append a duplicate JSONL row
- **AND** it does not increment the training counter

#### Scenario: Remote row is new

- **GIVEN** the local dataset does not contain the remote row id
- **WHEN** the row is merged
- **THEN** the merger appends it to the local dataset volume
- **AND** it increments the training counter

### Requirement: CRDT merger triggers training threshold

The CRDT dataset merger SHALL trigger the training orchestrator when federated inserts raise the dataset counter to the configured threshold.

#### Scenario: Federated merge reaches threshold

- **GIVEN** the current dataset counter is one below the training threshold
- **WHEN** a unique remote row is merged
- **THEN** the merger publishes `nebula.training.ready`
