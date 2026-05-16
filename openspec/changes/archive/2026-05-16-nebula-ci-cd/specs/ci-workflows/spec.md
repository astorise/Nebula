## ADDED Requirements

### Requirement: CI validates Node workspaces

The repository SHALL run a GitHub Actions CI workflow for Node.js packages on pushes to `main` and pull requests.

#### Scenario: Node CI runs

- **WHEN** the CI workflow is triggered
- **THEN** it installs dependencies with `npm ci`
- **AND** runs linting, build, and tests for the npm workspaces

### Requirement: CI validates Rust/Wasm FaaS

The repository SHALL run a GitHub Actions CI workflow for the Rust FaaS workspace with Wasm tooling installed.

#### Scenario: Rust CI runs

- **WHEN** the CI workflow is triggered
- **THEN** it installs Rust stable with `clippy`, `rustfmt`, and the WASI target
- **AND** installs `cargo-component`
- **AND** runs `cargo fmt --all -- --check`
- **AND** runs `cargo clippy --workspace --target wasm32-wasip1 -- -D warnings`
- **AND** runs `cargo test --workspace`

### Requirement: CI caches Cargo dependencies

The Rust CI job SHALL cache Cargo dependencies to reduce repeated build time.

#### Scenario: Rust job starts

- **WHEN** the Rust validation job starts
- **THEN** it configures `Swatinem/rust-cache` for the `faas` workspace
