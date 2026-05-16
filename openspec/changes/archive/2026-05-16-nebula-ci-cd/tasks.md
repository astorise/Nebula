# Implementation Tasks

- [x] `task-1`: Create `.github/workflows/ci.yml` for linting and test builds (Node.js + Rust/Wasm).
- [x] `task-2`: Configure `cargo-component` and the `wasm32-wasi` target in the GitHub Actions runner image.
- [x] `task-3`: Create `.github/workflows/release.yml` to trigger builds on tag creation events.
- [x] `task-4`: Implement the OCI publication script using `wkg push` to iterate over the FaaS directories.
- [x] `task-5`: Integrate `softprops/action-gh-release` to upload the `.vsix` file compiled by `vsce`.
