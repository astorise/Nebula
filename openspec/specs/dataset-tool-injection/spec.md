# dataset-tool-injection Specification

## Purpose
TBD - created by archiving change autonomous-tool-synthesis. Update Purpose after archive.
## Requirements
### Requirement: Dataset forge stores tool-calling examples
Dataset forge SHALL persist tool-calling SFT/DPO examples with prompt, chosen answer, tool schema, and tool call payload.

#### Scenario: Tool example append
- **GIVEN** a generated tool schema and chosen tool call
- **WHEN** the example is appended
- **THEN** the tool dataset receives a JSONL record containing schema and call data
