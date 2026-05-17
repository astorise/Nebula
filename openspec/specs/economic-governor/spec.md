# economic-governor Specification

## Purpose
TBD - created by archiving change economic-governor-and-semantic-deduplication. Update Purpose after archive.
## Requirements
### Requirement: Economic governor reserves tenant token budget
`nebula-economic-governor` SHALL estimate token usage for Teacher requests and reserve tenant budget before forwarding.

#### Scenario: Tenant is under budget
- **GIVEN** a tenant has remaining daily token budget
- **WHEN** an arbitration request is evaluated
- **THEN** estimated tokens are reserved
- **AND** the request is forwarded to `nebula.teacher.arbitration.request`

### Requirement: Economic governor blocks exhausted tenants
`nebula-economic-governor` SHALL drop Teacher requests when the tenant budget is exhausted.

#### Scenario: Tenant has no remaining tokens
- **GIVEN** a tenant has reached the daily token limit
- **WHEN** an arbitration request is evaluated
- **THEN** the request is not forwarded
- **AND** `nebula.finops.budget_exhausted` is emitted

### Requirement: Teacher token usage is reconciled exactly
Teacher arbitration SHALL publish exact token usage after successful Tier 3 output so the governor can reconcile reservations.

#### Scenario: Teacher returns API usage
- **WHEN** arbitration completes and token usage is available
- **THEN** `nebula.finops.token_usage` is emitted with total token usage
