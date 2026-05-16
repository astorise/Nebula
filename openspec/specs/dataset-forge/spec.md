# dataset-forge Specification

## Purpose
TBD - created by archiving change nebula-teacher-forge. Update Purpose after archive.
## Requirements
### Requirement: Dataset forge balances confidence sources

The dataset forge SHALL maintain an approximate 60% escalated and 40% direct-success training mix.

#### Scenario: New example preserves ratio

- **WHEN** an incoming example keeps the dataset within the accepted ratio tolerance
- **THEN** the forge accepts the example

#### Scenario: New example would skew ratio

- **WHEN** an incoming example would exceed the accepted ratio tolerance
- **THEN** the forge rejects or defers the example without writing to the dataset

### Requirement: Dataset forge appends JSONL records

The dataset forge SHALL persist accepted training examples as append-only JSONL records on the Tachyon volume store.

#### Scenario: Example is accepted

- **WHEN** the forge accepts an example
- **THEN** it appends one JSON object line to `dataset_v1.jsonl`

### Requirement: Dataset forge signals training readiness

The dataset forge SHALL emit a training-ready event when the configured dataset threshold is reached.

#### Scenario: Dataset reaches threshold

- **WHEN** the total accepted example count reaches the training threshold
- **THEN** the forge publishes `nebula.training.ready` with the dataset path

