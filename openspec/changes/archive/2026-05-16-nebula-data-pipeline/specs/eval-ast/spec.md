## ADDED Requirements

### Requirement: AST evaluator extracts code blocks

The AST evaluator SHALL isolate fenced code blocks from each of the three generated responses before syntax evaluation.

#### Scenario: All responses contain code fences

- **WHEN** the evaluator receives three responses with fenced code blocks
- **THEN** it extracts the code contents from each response

#### Scenario: A response has no code fence

- **WHEN** at least one response has no fenced code block
- **THEN** the evaluator falls back to semantic evaluation

### Requirement: AST evaluator computes structural hashes

The AST evaluator SHALL load the requested Tree-sitter Wasm grammar and compute structural hashes that ignore lexical naming noise.

#### Scenario: Equivalent structures with different names

- **WHEN** the three responses have equivalent control-flow structure but different identifiers or literals
- **THEN** the evaluator treats the structural hashes as identical
- **AND** no divergence result is emitted

#### Scenario: Different structures

- **WHEN** the three structural hashes differ
- **THEN** the evaluator publishes `EvaluationResult { diverged: true }` to `nebula.eval.results`

#### Scenario: Fatal parsing setup error

- **WHEN** the evaluator cannot load the requested grammar or cannot derive structural features
- **THEN** the payload is forwarded to `nebula.eval.semantic.pending`
