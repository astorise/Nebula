# tool-architect Specification

## Purpose
TBD - created by archiving change autonomous-tool-synthesis. Update Purpose after archive.
## Requirements
### Requirement: Tool architect generates component sources
`nebula-tool-architect` SHALL build Tier 3 prompts for missing tool events and extract `src/lib.rs` plus `Cargo.toml` from the generated answer.

#### Scenario: Generated tool extraction
- **GIVEN** a Tier 3 response containing fenced source blocks
- **WHEN** the architect parses it
- **THEN** it returns a generated tool with source and manifest
