## ADDED Requirements

### Requirement: Golden manager promotes stable rows
`nebula-golden-dataset-manager` SHALL promote rows that have survived more than seven days in production without rollback or drift.

#### Scenario: Stable row promotion
- **GIVEN** a row has eight production days
- **AND** no rollback or drift was recorded
- **WHEN** promotion runs
- **THEN** the row is appended to the golden dataset

### Requirement: Golden rows are vectorized
Promoted rows SHALL be vectorized under the `golden_dataset` namespace.

#### Scenario: Vectorization after promotion
- **WHEN** a row is promoted
- **THEN** the vector store receives the row in the golden namespace
