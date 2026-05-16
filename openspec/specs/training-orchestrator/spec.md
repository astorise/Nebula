# training-orchestrator Specification

## Purpose
TBD - created by archiving change nebula-teacher-forge. Update Purpose after archive.
## Requirements
### Requirement: Training orchestrator starts LoRA training

The training orchestrator SHALL react to `nebula.training.ready` by launching LoRA training with default hyperparameters.

#### Scenario: Training ready event is received

- **WHEN** `nebula.training.ready` is received
- **THEN** the orchestrator starts LoRA training with `dim=16` and `alpha=32`

### Requirement: Training orchestrator merges and publishes model artifacts

The training orchestrator SHALL merge the trained LoRA adapter into the base model and publish the resulting safetensors artifact through the local OCI workflow.

#### Scenario: LoRA training succeeds

- **WHEN** the adapter is produced
- **THEN** the orchestrator merges it into `pulsar-base-v2.safetensors`
- **AND** invokes the artifact publisher equivalent to `wkg`
- **AND** publishes to `oci://localhost:5000/pulsar-models/base:v2`

### Requirement: Training orchestrator notifies the UI

The training orchestrator SHALL notify the VS Code UI when the model is published.

#### Scenario: Artifact publication succeeds

- **WHEN** publication completes
- **THEN** the orchestrator emits `nebula.training.complete` with the artifact reference

