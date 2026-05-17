# dpo-judge-hardening Specification

## Purpose
TBD - created by archiving change audit-remediation-and-security-hardening. Update Purpose after archive.
## Requirements
### Requirement: DPO judge uses boundary-aware RegexSet
`nebula-dpo-judge` SHALL compile forbidden constitutional terms into a case-insensitive `RegexSet` with word boundaries.

#### Scenario: Capitalization bypass is rejected
- **GIVEN** `unwrap` is forbidden
- **WHEN** the chosen answer contains `Unwrap()`
- **THEN** the answer is rejected

#### Scenario: Identifier substring is allowed
- **GIVEN** `unwrap` is forbidden
- **WHEN** the chosen answer contains `unwrap_internal_macro`
- **THEN** the term matcher does not flag it as a standalone violation

### Requirement: DPO judge writes audit hashes
Every approved preference pair SHALL produce a SHA-256 audit hash over `(prompt, chosen, rejected)`.

#### Scenario: Approved preference
- **WHEN** a valid preference pair is forwarded
- **THEN** the preference sink receives a 64-character audit hash
