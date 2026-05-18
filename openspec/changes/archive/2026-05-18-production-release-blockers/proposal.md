# Proposal: Production Release Blockers & Consolidation Sprint

## Context
The final pre-release audit (v1.0.0-RC) resulted in a strict "NO-GO for Production". The autonomous implementation of the Wormhole NAT traversal (Change 025) was delivered as a mocked stub rather than a functional network client. Furthermore, the Node.js CLI layer (916 LoC) completely lacks runtime testing, exposing a critical path traversal vulnerability via symlinks in the WebDAV server. Finally, the Rust Redis adapter lacks true integration tests to prove its atomic guarantees.

## Objectives
Execute a pure consolidation sprint to clear all release blockers:
1. **Wormhole Implementation**: Eradicate the mocked vendor stub and bind the actual network client.
2. **CLI Test Coverage & Security**: Introduce `vitest` to the monorepo, write runtime tests for the CLI, and fix the symlink path traversal vulnerability in `webdav.ts` using `fs.realpath`.
3. **SSRF Protection**: Harden the `nebula-doc-parser` to strictly validate incoming `tunnel_host` URIs.
4. **Concrete Integration Tests**: Implement `testcontainers-rs` in the Redis budget store to prove concurrent atomic reservations and fix the LUA script underflow bug.