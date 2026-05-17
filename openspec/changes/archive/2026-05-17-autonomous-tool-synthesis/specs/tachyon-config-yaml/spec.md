## ADDED Requirements

### Requirement: Config AI WIT declares synthesis mode
The repository SHALL provide `wit/config-ai.wit` with disabled, human-in-loop, and fully-autonomous synthesis modes.

#### Scenario: WIT contract exists
- **WHEN** downstream components need synthesis configuration
- **THEN** they can reference the `config-ai` WIT package
