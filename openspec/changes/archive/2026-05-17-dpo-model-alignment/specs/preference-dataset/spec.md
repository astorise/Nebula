## ADDED Requirements

### Requirement: Dataset forge persists DPO preference triplets
`nebula-dataset-forge` SHALL persist DPO examples as JSONL records containing `prompt`, `chosen`, and `rejected` fields.

#### Scenario: Constitutional judge forwards a preference
- **GIVEN** a Tier 1 answer triggered hallucination or divergence
- **AND** a Tier 3 arbitrator produced the corrected answer
- **WHEN** the preference is appended
- **THEN** the JSONL record contains the Tier 3 output as `chosen`
- **AND** the original Tier 1 output as `rejected`

### Requirement: Preference records can be tenant scoped
Preference dataset paths SHALL be tenant scoped when a `tenant_id` is present.

#### Scenario: Tenant preference append
- **GIVEN** a preference example for tenant `acme`
- **WHEN** the forge appends the example
- **THEN** it writes under the tenant preference dataset path
