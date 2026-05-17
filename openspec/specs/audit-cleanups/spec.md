# audit-cleanups Specification

## Purpose
TBD - created by archiving change finops-hardening-and-type-enforcement. Update Purpose after archive.
## Requirements
### Requirement: Dataset forge rejects unresolved tenant path ids
Dataset forge SHALL reject non-UUID tenant ids when constructing tenant-scoped paths directly.

#### Scenario: Raw tenant string
- **GIVEN** tenant id `acme/prod`
- **WHEN** a direct tenant dataset path is requested
- **THEN** the function fails loudly instead of falling back to a test UUID

### Requirement: Semantic deduplicator reports invalid vector math
`nebula-semantic-deduplicator` SHALL return an error for empty, zero-norm, or mismatched embedding vectors.

#### Scenario: Mismatched dimensions
- **WHEN** cosine similarity receives vectors with different dimensions
- **THEN** it returns an error

### Requirement: Golden manager documents production day contract
`nebula-golden-dataset-manager` SHALL document that callers own the durable evidence behind `days_in_production`.

#### Scenario: Promote stable row
- **WHEN** callers use `promote_if_stable`
- **THEN** Rustdoc explains how `days_in_production` is supplied
