# Implementation Tasks

- [x] `task-1`: Fix the `build-rootfs.sh` hash invalidation bug to write to a `.sha256` detached file.
- [x] `task-2`: Update `tenant_index_key` signature to require `TenantId` and eradicate the `.expect()` call.
- [x] `task-3`: Migrate `nebula-dpo-judge` and `nebula-golden-dataset-manager` to strictly use `TenantId`.
- [x] `task-4`: Inject `[workspace.lints.clippy]` rules to deny `unwrap_used` and `expect_used` globally, and fix all resulting compiler errors by properly propagating `Result`.
- [x] `task-5`: Update the CI pipeline to run `wasm32-wasip1` clippy checks across the *entire* workspace and update the OpenSpec verification script regex.
- [x] `task-6`: Create the `nebula-redis-budget-store` adapter and implement the Lua-backed atomic `reserve_if_under_quota` function.
