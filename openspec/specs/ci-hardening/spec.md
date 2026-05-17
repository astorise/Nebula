# ci-hardening Specification

## Purpose
TBD - created by archiving change finops-hardening-and-type-enforcement. Update Purpose after archive.
## Requirements
### Requirement: CI runs Cargo vulnerability audit
The Rust CI job SHALL install and run `cargo-audit` with warnings denied.

#### Scenario: Known advisory
- **WHEN** dependency advisories are present
- **THEN** CI fails during the audit step

### Requirement: CI runs Cargo dependency policy
The Rust CI job SHALL install and run `cargo-deny`.

#### Scenario: Policy violation
- **WHEN** a banned source, license, or wildcard dependency is present
- **THEN** CI fails during dependency policy validation

### Requirement: CI checks eval AST on wasm target
The wasm Clippy CI step SHALL include `nebula-eval-ast` and SHALL NOT use `--exclude nebula-eval-ast`.

#### Scenario: Wasm Clippy
- **WHEN** the CI wasm Clippy step runs
- **THEN** `nebula-eval-ast` is part of the package list
