# Implementation Tasks

- [x] `task-1`: Create the `nebula-tenant-router` Wasm crate to extract and validate `x-tenant-id`.
- [x] `task-2`: Refactor `nebula-dataset-forge` and `nebula-crdt-merger` to implement dynamic volume paths and KV store namespaces.
- [x] `task-3`: Update `nebula-training-orchestrator` to scope the training loop and execute `wkg push` with tenant-specific tags.
- [x] `task-4`: Implement the `TenantPanel.tsx` in the VSCode extension and the global tenant context switcher.
- [x] `task-5`: Update downstream evaluation components (`eval-ast`, `eval-semantic`, `dpo-judge`) to pass the `tenant_id` state transparently.
