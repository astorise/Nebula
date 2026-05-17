# ci-traceability Delta

## ADDED Requirements

### Requirement: CI enforces OpenSpec test traceability
The CI pipeline SHALL run `scripts/verify_openspec.sh` to ensure Rust tests declare `// spec:` traceability tags and FaaS-backed OpenSpec capabilities have at least one matching Rust test tag.

#### Scenario: Rust test has traceability tag
- **WHEN** the traceability script scans Rust test files under `faas/nebula-*`
- **THEN** each `#[test]` attribute is followed by a `// spec: <capability>` comment

#### Scenario: FaaS-backed spec has test coverage tag
- **WHEN** the traceability script finds an OpenSpec capability backed by a FaaS crate or explicit `faas/` location
- **THEN** at least one Rust test contains a matching `// spec: <capability>` tag

### Requirement: Wasm rootfs metadata includes checksum
The Wasm Foundry rootfs build script SHALL append a `sha256sum` line to the generated artifact metadata.

#### Scenario: Build rootfs metadata
- **WHEN** `build-rootfs.sh` writes the toolchain manifest
- **THEN** the manifest includes a SHA-256 checksum line for the generated metadata content
