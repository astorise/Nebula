## ADDED Requirements

### Requirement: Teacher arbitrator consumes divergence batches

The teacher arbitrator SHALL consume accumulated hallucination batches from the Tier 3 arbitration queue.

#### Scenario: Queue has work

- **WHEN** the arbitration queue returns one or more divergence tasks
- **THEN** the arbitrator processes the tasks as a micro-batch

#### Scenario: Queue is empty

- **WHEN** the arbitration queue returns no tasks
- **THEN** the arbitrator completes without publishing dataset entries

### Requirement: Teacher arbitrator runs layer-by-layer inference

The teacher arbitrator SHALL use `tachyon:inference` to run the Teacher model under VRAM limits by loading, forwarding, and unloading one layer at a time.

#### Scenario: Batch inference starts

- **WHEN** a micro-batch is processed
- **THEN** each model layer is loaded
- **AND** the layer is forwarded over the whole batch
- **AND** the layer is unloaded before the next layer

### Requirement: Teacher arbitrator emits constrained answers

The teacher arbitrator SHALL decode Teacher outputs using a strict JSON schema and publish corrected answers to the dataset append queue.

#### Scenario: Corrected answers are decoded

- **WHEN** constrained decoding succeeds
- **THEN** each corrected answer is published to `nebula.dataset.append`
