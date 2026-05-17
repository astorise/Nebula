## ADDED Requirements

### Requirement: Economic governor reserves budget atomically
`nebula-economic-governor` SHALL reserve budget through a single `reserve_if_under_quota` store operation.

#### Scenario: Under quota
- **GIVEN** enough tenant budget remains
- **WHEN** budget is evaluated
- **THEN** the store returns a reservation receipt
- **AND** the request is forwarded

#### Scenario: Over quota
- **GIVEN** insufficient tenant budget remains
- **WHEN** budget is evaluated
- **THEN** no reservation is created
- **AND** the request is blocked

### Requirement: Economic governor estimates tokens with tiktoken
The governor SHALL use `tiktoken-rs` for pre-reservation token estimation.

#### Scenario: Estimate prompt cost
- **WHEN** a prompt is evaluated
- **THEN** the estimate is produced by the `cl100k_base` tokenizer when available

### Requirement: Economic governor reconciles reservation deltas
The governor SHALL debit or credit the delta between estimated and exact billed tokens.

#### Scenario: Exact usage exceeds estimate
- **WHEN** exact tokens are greater than estimated tokens
- **THEN** the difference is debited

#### Scenario: Exact usage is below estimate
- **WHEN** exact tokens are lower than estimated tokens
- **THEN** the difference is credited
