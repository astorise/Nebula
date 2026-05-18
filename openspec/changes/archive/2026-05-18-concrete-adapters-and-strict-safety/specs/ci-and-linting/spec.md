# ci-and-linting Delta

## ADDED Requirements

### Requirement: Workspace denies production unwrap and expect
The FaaS workspace SHALL define `unwrap_used` and `expect_used` as denied Clippy lints and every workspace crate SHALL inherit those lint settings.

#### Scenario: Run workspace Clippy
- **WHEN** `cargo clippy --workspace --all-targets -- -D warnings` runs
- **THEN** production code using `.unwrap()` or `.expect()` fails the check
- **AND** test code may explicitly allow those lints under `#[cfg(test)]`

### Requirement: OpenSpec traceability detects common Rust test macros
The OpenSpec verification script SHALL detect `#[test]`, `#[tokio::test]`, `#[rstest]`, and `#[test_case]` annotations.

#### Scenario: Verify annotated tests
- **WHEN** `scripts/verify_openspec.sh` scans Rust test files
- **THEN** each supported test annotation requires a following `// spec:` comment

### Requirement: CI checks eval AST on wasm target
The CI wasm Clippy step SHALL run across the complete FaaS workspace instead of a hand-selected package list.

#### Scenario: Wasm Clippy
- **WHEN** the CI wasm Clippy step runs
- **THEN** it executes `cargo clippy --workspace --target wasm32-wasip1 -- -D warnings`
