# canary-telemetry-provider Specification

## Purpose
TBD - created by archiving change cognitive-canary-rollbacks. Update Purpose after archive.
## Requirements
### Requirement: Canary telemetry provider publishes cognitive divergence metrics

The `nebula-deployment-policy` Wasm component SHALL publish `nebula.cognitive_divergence` through the Tachyon `custom-metrics` WIT boundary instead of raw HTTP rollback calls.

#### Scenario: Canary inference window is evaluated

- **GIVEN** inference results include a canary model version tag
- **WHEN** the policy evaluates the rolling canary window
- **THEN** it pushes a gauge metric named `nebula.cognitive_divergence`
- **AND** the metric includes `model_version` and `rollout_track` tags

### Requirement: Canary telemetry provider isolates canary and stable tracks

The provider SHALL tag canary metrics separately from stable production metrics.

#### Scenario: Stable and canary events are mixed

- **GIVEN** inference results arrive for stable and canary model versions
- **WHEN** metrics are calculated
- **THEN** canary metrics use `rollout_track=canary`
- **AND** stable metrics use `rollout_track=stable`

### Requirement: Canary telemetry provider recommends rollback

The provider SHALL identify canary rollback candidates when the canary cognitive divergence rate exceeds the configured threshold.

#### Scenario: Canary divergence spikes

- **GIVEN** canary cognitive divergence exceeds the configured maximum
- **WHEN** the rolling window is evaluated
- **THEN** the provider marks the rollout decision as rollback
- **AND** the pushed metric is available for Tachyon's rollout engine
