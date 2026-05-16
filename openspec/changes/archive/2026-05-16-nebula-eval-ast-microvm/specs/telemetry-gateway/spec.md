## MODIFIED Requirements

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

#### Scenario: Code generation triplet is dispatched to AST microVM

- **WHEN** the gateway needs to call the AST microVM
- **THEN** it encodes `EvaluationRequest` as Protobuf
- **AND** it wraps the payload in a gRPC frame
- **AND** it sends the frame to `http://nebula-eval-ast.microvm.internal/nebula.ast.AstEvaluator/EvaluateTriplets` through the host HTTP bridge
