# Implementation Tasks

- [x] `task-1`: Fix `nebula-tenant-router` logic and `dashboardProvider.ts` crypto nonce.
- [x] `task-2`: Introduce `TenantId(Uuid)` and refactor `economic-governor` and `semantic-deduplicator` to strictly use it.
- [x] `task-3`: Update `nebula-tenant-core` to accurately propagate `DatabaseUnavailable` errors instead of masking them as `QuotaExceeded`.
- [x] `task-4`: Implement atomic budget reservation and `tiktoken-rs` in `nebula-economic-governor`.
- [x] `task-5`: Harden the CI pipeline (`cargo-audit`, `cargo-deny`) and fix the `nebula-eval-ast` Wasm target exclusion.
- [x] `task-6`: Implement the E2E FinOps integration test suite.
