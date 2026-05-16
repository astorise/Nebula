## ADDED Requirements

### Requirement: Telemetry gateway routes inference triplets

The telemetry gateway SHALL subscribe to Pulsar inference triplets and route them to the appropriate Nebula evaluator topic.

#### Scenario: Code generation triplet

- **WHEN** an event arrives on `pulsar.telemetry.inference_triplets`
- **AND** its `task_type` is `code_generation` or contains a language tag such as `rust`, `cobol`, or `wasm`
- **THEN** the gateway publishes the payload to `nebula.eval.ast.pending`

#### Scenario: Free text triplet

- **WHEN** an event arrives on `pulsar.telemetry.inference_triplets`
- **AND** its `task_type` is `free_text` or `reasoning`
- **THEN** the gateway publishes the payload to `nebula.eval.semantic.pending`

#### Scenario: Unsupported source topic

- **WHEN** an event arrives from a topic other than `pulsar.telemetry.inference_triplets`
- **THEN** the gateway rejects the event without publishing an evaluator request
