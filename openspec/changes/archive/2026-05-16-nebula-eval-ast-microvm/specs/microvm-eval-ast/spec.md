## ADDED Requirements

### Requirement: AST evaluator runs as a microVM gRPC server

The `nebula-eval-ast` crate SHALL provide a native Linux gRPC server for execution inside a SmolVM.

#### Scenario: Server starts on Unix socket

- **WHEN** the microVM server starts on Linux
- **THEN** it binds a `tonic` server to `/run/guest.sock`
- **AND** it does not bind a TCP address

#### Scenario: Server is checked on non-Unix host

- **WHEN** the crate is checked on a non-Unix developer host
- **THEN** the binary provides a non-Unix stub instead of attempting to bind a Unix socket

### Requirement: AST evaluator computes structural hashes natively

The microVM evaluator SHALL instantiate the native Tree-sitter runtime and compute structural hashes for the response triplet.

#### Scenario: Three valid code responses are received

- **WHEN** `EvaluateTriplets` receives three code responses
- **THEN** the evaluator extracts fenced code blocks
- **AND** it computes SHA-256 structural hashes
- **AND** it reports divergence when the hashes differ

#### Scenario: Invalid request is received

- **WHEN** `EvaluateTriplets` receives a response count other than three
- **THEN** the evaluator returns a fallback reason

### Requirement: AST evaluator rootfs can be assembled

The repository SHALL provide a script that builds a microVM root filesystem for the AST evaluator.

#### Scenario: Rootfs script runs

- **WHEN** `scripts/build-eval-ast-rootfs.sh` runs on Linux
- **THEN** it builds `nebula-eval-ast` for `x86_64-unknown-linux-musl`
- **AND** it creates `artifacts/microvm/nebula-eval-ast/rootfs.ext4`
