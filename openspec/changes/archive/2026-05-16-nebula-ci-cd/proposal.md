# Proposal: Nebula CI/CD

## Why

Nebula contains Node.js packages, a VS Code extension, and Rust/Wasm FaaS components. Changes must be validated automatically, and release artifacts must be produced reproducibly.

## What Changes

- Add GitHub Actions CI to validate Node.js and Rust/Wasm.
- Install `cargo-component` and the Wasm target in runners.
- Add a release workflow triggered by tags.
- Add an OCI publication script for FaaS components through `wkg push`.
- Publish the compiled VS Code `.vsix` extension in GitHub Releases.
