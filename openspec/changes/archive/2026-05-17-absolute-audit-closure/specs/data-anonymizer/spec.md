# data-anonymizer Delta

## MODIFIED Requirements

### Requirement: Data anonymizer masks sensitive telemetry
The data anonymizer SHALL match UUID and IP address rules before evaluating credit-card-like numeric strings, and SHALL only replace a credit-card candidate with `<CREDIT_CARD>` when its digits pass the Luhn checksum.

#### Scenario: Mask valid payment card
- **WHEN** text contains a credit-card candidate that passes Luhn validation
- **THEN** the candidate is replaced with `<CREDIT_CARD>`
- **AND** the credit-card mask count is incremented

#### Scenario: Preserve non-Luhn numeric identifier
- **WHEN** text contains a 13-19 digit candidate that fails Luhn validation
- **THEN** the candidate remains unchanged
- **AND** the credit-card mask count is not incremented

#### Scenario: Prioritize structured identifiers
- **WHEN** text contains UUID or IPv6 values before credit-card-like candidates are evaluated
- **THEN** UUID and IPv6 values are masked by their dedicated rules
- **AND** they are not counted as credit-card masks
