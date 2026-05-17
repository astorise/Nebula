# telemetry-gateway Specification

## MODIFIED Requirements

### Requirement: Telemetry gateway routes inference triplets

The telemetry gateway SHALL subscribe to sanitized inference triplets emitted by the data anonymizer and route them to the appropriate Nebula evaluator topic.

#### Scenario: Code generation triplet

- **GIVEN** a sanitized inference triplet with task type `code_generation`
- **WHEN** an event arrives on `pulsar.telemetry.inference_triplets`
- **THEN** the gateway publishes the payload to `nebula.eval.ast.pending`

#### Scenario: Free-text triplet

- **GIVEN** a sanitized inference triplet with task type `reasoning`
- **WHEN** an event arrives on `pulsar.telemetry.inference_triplets`
- **THEN** the gateway publishes the payload to `nebula.eval.semantic.pending`

#### Scenario: Raw telemetry is rejected

- **GIVEN** an event arrives on `nebula.telemetry.raw_inferences`
- **WHEN** the telemetry gateway handles the event directly
- **THEN** the gateway rejects the event without publishing an evaluator request
