## ADDED Requirements

### Requirement: Tool architect parses strict JSON
`nebula-tool-architect` SHALL instruct Tier 3 to return strict JSON and parse generated files with `serde_json`.

#### Scenario: Valid generated payload
- **GIVEN** a response shaped as `{ "files": [{ "name": "src/lib.rs", "content": "..." }] }`
- **WHEN** the architect extracts the generated tool
- **THEN** it returns the source and manifest without Markdown parsing

### Requirement: Tool architect rejects Markdown payloads
The architect SHALL fail extraction when the response is not valid JSON.

#### Scenario: Markdown response
- **GIVEN** a response contains fenced code blocks
- **WHEN** extraction runs
- **THEN** JSON parsing fails

### Requirement: Wasm foundry parses cargo JSON diagnostics
`nebula-wasm-foundry` SHALL treat compiler output as failed only when a JSON diagnostic message has level `error`.

#### Scenario: Warning contains error text
- **GIVEN** a cargo JSON warning message contains the word `error`
- **WHEN** diagnostics are evaluated
- **THEN** the build is not marked failed

#### Scenario: Structured error
- **GIVEN** a cargo JSON diagnostic has `"message": { "level": "error" }`
- **WHEN** diagnostics are evaluated
- **THEN** the build is marked failed
