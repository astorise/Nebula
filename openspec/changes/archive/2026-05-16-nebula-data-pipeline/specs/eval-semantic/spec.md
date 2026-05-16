## ADDED Requirements

### Requirement: Semantic evaluator uses Tachyon inference embeddings

The semantic evaluator SHALL request embeddings through `tachyon:inference` and compare the deterministic response against the high-temperature responses.

#### Scenario: Embeddings are available

- **WHEN** the evaluator receives a semantic evaluation request
- **THEN** it embeds all three responses using a lightweight embedding model
- **AND** it computes cosine similarity between the `T=0.1` response and each `T=0.8` response

### Requirement: Semantic evaluator applies divergence thresholds

The semantic evaluator SHALL emit divergence results only when semantic similarity is below the configured failure threshold.

#### Scenario: Responses are semantically equivalent

- **WHEN** the average similarity is greater than or equal to `0.95`
- **THEN** the evaluator ignores the event

#### Scenario: Responses diverge semantically

- **WHEN** the average similarity is lower than `0.85`
- **THEN** the evaluator publishes `EvaluationResult { diverged: true }` to `nebula.eval.results`

#### Scenario: Similarity is inconclusive

- **WHEN** the average similarity is between `0.85` and `0.95`
- **THEN** the evaluator marks the result ambiguous without publishing a confirmed divergence
