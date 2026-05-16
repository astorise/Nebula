# divergence-aggregator Specification

## Purpose
TBD - created by archiving change nebula-data-pipeline. Update Purpose after archive.
## Requirements
### Requirement: Divergence aggregator persists confirmed failures

The divergence aggregator SHALL collect confirmed divergence results and enqueue Tier 3 arbitration work in the Tachyon KV store.

#### Scenario: Diverged evaluation result

- **WHEN** an event arrives on `nebula.eval.results`
- **AND** `diverged` is `true`
- **THEN** the aggregator builds a JSON arbitration task containing the prompt, Pulsar context, evaluator metadata, and the three generated responses
- **AND** it appends the task to the KV list `nebula:tier3:arbitration`

#### Scenario: Non-divergent evaluation result

- **WHEN** an event arrives with `diverged` set to `false`
- **THEN** the aggregator ignores the event
- **AND** no KV store write is performed

