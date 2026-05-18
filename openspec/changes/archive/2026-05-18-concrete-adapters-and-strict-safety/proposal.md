# Proposal: Concrete Adapters, Strict Safety, and CI Expansion

## Context
The third architectural audit validated the successful implementation of the FinOps governor and the absolute audit closure (Luhn check, type enforcement). However, it identified a self-invalidating cryptographic checksum in the build scripts, two remaining crates ignoring the `TenantId` type enforcement, a dormant panic in the public API (`tenant_index_key`), and a fragile OpenSpec verification script.

Crucially, the audit highlighted a strategic vulnerability: the entire Nebula mesh relies on "Trait-only" mock implementations. To prove the resilience of the Atomic FinOps architecture (Change 021), we must implement our first concrete backend adapter.

## Objectives
1. **Security & Type Remediation**: Fix the self-corrupting `build-rootfs.sh` hash and enforce `TenantId` globally, completely eliminating `Option<String>` and `String` for tenant identifiers.
2. **Eradicate Panics**: Enforce `#![deny(clippy::unwrap_used, clippy::expect_used)]` at the workspace level and fix the dormant panic in `nebula-dataset-forge`.
3. **Concrete Architecture Validation**: Build `nebula-redis-budget-store`, a concrete implementation of the `BudgetStore` trait using Redis, proving the atomicity of the `reserve_if_under_quota` FinOps logic via Lua scripting.
4. **CI Hardening**: Expand the Wasm Clippy matrix to all FaaS crates and make the `verify_openspec.sh` script resilient to various test macros (`tokio::test`, `rstest`).