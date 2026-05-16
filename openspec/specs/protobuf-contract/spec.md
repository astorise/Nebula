# protobuf-contract Specification

## Purpose
TBD - created by archiving change nebula-eval-ast-microvm. Update Purpose after archive.
## Requirements
### Requirement: AST evaluator uses a Protobuf contract

The AST evaluator microVM SHALL expose a strict Protobuf contract for triplet evaluation.

#### Scenario: Protobuf contract is generated

- **WHEN** `nebula-eval-ast` is built
- **THEN** `tonic-build` compiles `proto/ast_evaluator.proto`
- **AND** the generated Rust types include `EvaluationRequest` and `EvaluationResponse`

### Requirement: Evaluation request carries language and responses

The Protobuf request SHALL include the target language and the generated response triplet.

#### Scenario: Gateway sends a code triplet

- **WHEN** the telemetry gateway sends an AST evaluation request
- **THEN** the request contains `language`
- **AND** the request contains exactly the LLM response triplet in `responses`

### Requirement: Evaluation response carries divergence and fallback

The Protobuf response SHALL report whether structural hashes diverged and why semantic fallback is required.

#### Scenario: Structural hashes differ

- **WHEN** the evaluator computes different structural hashes
- **THEN** it returns `diverged: true`

#### Scenario: Structural evaluation cannot run

- **WHEN** the evaluator cannot parse or hash the payload
- **THEN** it returns `diverged: false`
- **AND** it sets `fallback_reason`

