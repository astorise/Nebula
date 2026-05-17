## ADDED Requirements

### Requirement: DPO judge injects constitutional rules
`nebula-dpo-judge` SHALL load constitutional rules from a store and inject them into Tier 3 arbitration prompts.

#### Scenario: Teacher prompt is built with constitution
- **GIVEN** active constitutional rules
- **WHEN** a judgement request is prepared
- **THEN** the teacher prompt includes each rule before the user prompt

### Requirement: DPO judge forwards accepted preference pairs
`nebula-dpo-judge` SHALL reject chosen answers that violate forbidden constitutional terms and forward valid `(prompt, chosen, rejected)` pairs to the preference dataset sink.

#### Scenario: Valid correction is accepted
- **GIVEN** the chosen answer does not violate any rule
- **WHEN** the judge evaluates the request
- **THEN** it forwards a preference pair with the chosen and rejected answers
