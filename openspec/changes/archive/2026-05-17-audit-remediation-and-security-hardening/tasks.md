# Implementation Tasks

- [x] `task-1`: Create `nebula-tenant-core`, implement UUID mapping, and refactor `tenant-router` and `dataset-forge` to use it exclusively.
- [x] `task-2`: Upgrade `nebula-dpo-judge` to use `RegexSet` with proper boundary and case-insensitivity rules.
- [x] `task-3`: Refactor `nebula-tool-architect` to demand and parse strict JSON payloads from the LLM.
- [x] `task-4`: Refactor `nebula-wasm-foundry` to parse `cargo --message-format=json` for deterministic error handling.
- [x] `task-5`: Secure the VS Code Webview with a strict nonce-based CSP and implement `ts-rs` for IPC type synchronization.
- [x] `task-6`: Create a `tests/e2e_pipeline_test.rs` integration test proving a payload successfully traverses the entire system from router to Golden Dataset.
