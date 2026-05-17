## ADDED Requirements

### Requirement: Wasm foundry builds requested tools
`nebula-wasm-foundry` SHALL accept build requests containing Rust source, `Cargo.toml`, and an OCI artifact reference.

#### Scenario: Successful build
- **GIVEN** the runner reports no build errors
- **WHEN** a build request is handled
- **THEN** the foundry pushes the `.wasm` artifact and returns the artifact reference

### Requirement: Wasm foundry reports diagnostics on failure
The foundry SHALL return diagnostics without pushing an artifact when build output contains errors.

#### Scenario: Failed build
- **GIVEN** build diagnostics include an error
- **WHEN** the build request completes
- **THEN** the response is unsuccessful and contains diagnostics
