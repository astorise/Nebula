# data-anonymizer Specification

## Purpose
TBD - created by archiving change data-privacy-and-anonymization. Update Purpose after archive.
## Requirements
### Requirement: Data anonymizer masks sensitive telemetry

The `nebula-data-anonymizer` Wasm component SHALL consume raw inference telemetry and mask sensitive values before the payload enters the evaluation pipeline.

#### Scenario: Raw telemetry contains PII and secrets

- **GIVEN** a raw inference triplet contains email addresses, bearer tokens, or payment card numbers
- **WHEN** the anonymizer handles the event
- **THEN** it replaces each sensitive value with a stable masking token
- **AND** it preserves the triplet shape for downstream evaluators

### Requirement: Data anonymizer provides compliance presets

The anonymizer SHALL include default compliance rules for tokens, PII, and financial data.

#### Scenario: Default rules are loaded

- **WHEN** the anonymizer starts without custom rules
- **THEN** it can mask emails, bearer tokens, JWT-like tokens, IPv4 addresses, UUIDs, and credit card-like numbers

### Requirement: Data anonymizer reports masking audit metrics

The anonymizer SHALL report how many entities were masked per event.

#### Scenario: Entities are masked

- **GIVEN** a telemetry payload contains sensitive entities
- **WHEN** anonymization completes
- **THEN** the anonymizer publishes `nebula.privacy.entities_masked`
- **AND** the metric includes per-rule counts
