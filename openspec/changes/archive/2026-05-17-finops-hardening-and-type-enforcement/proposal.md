# Proposal: FinOps Hardening, Type Enforcement, and Logic Remediation

## Context
A secondary deep architectural audit of the `main` branch (commits up to `f38d8ad`) revealed critical logic bugs introduced during the previous security remediation sprint, as well as concurrency flaws in the newly introduced FinOps crates. Specifically, the Tenant Router's permissive mode drops traffic unexpectedly, the VS Code CSP nonce is cryptographically weak, and the Economic Governor suffers from a race condition that allows budget overruns. Furthermore, the newly created `nebula-tenant-core` was ignored by subsequent crates, reintroducing "stringly-typed" tenant IDs.

## Objectives
Execute a final stabilization sprint before v1.0.0:
1. **Logic & Security Fixes**: Resolve the permissive routing bug in `nebula-tenant-router` and implement a cryptographically secure `nonce` in the VS Code UI.
2. **Strict Type Enforcement**: Refactor all newly created crates (`economic-governor`, `semantic-deduplicator`) to strictly utilize `nebula_tenant_core::TenantId` (UUID) instead of raw Strings.
3. **Atomic FinOps & Accurate Tokenization**: Eliminate the race condition in the Economic Governor via atomic Compare-and-Swap (CAS) or Reservation APIs, and implement `tiktoken-rs` for safe token estimation.
4. **CI Enforcement**: Add `cargo-deny` / `cargo-audit` to the pipeline and remove the dangerous `--exclude nebula-eval-ast` from the WebAssembly Clippy job.