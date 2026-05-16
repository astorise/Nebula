# Design: Nebula CI/CD

## Architecture

CI is split into two parallel jobs:

- `validate-node` installs npm workspaces, then runs linting, build, and Node tests.
- `validate-rust-wasm` installs Rust stable, `cargo-component`, `clang`, `wasi-libc`, the `wasm32-wasip1` target, then runs formatting, clippy, and tests.

The release workflow runs only on `v*` tags. It builds Node packages, produces the `.vsix`, builds and pushes FaaS components to GHCR, then creates a GitHub Release.

## FaaS Publication

The `scripts/publish-faas-oci.sh` script discovers crates under `faas/*/Cargo.toml`, runs `cargo component build --release --package <crate>`, and pushes each component with `wkg push`.

The registry and tag are configurable through `OCI_REGISTRY` and `OCI_TAG`, allowing the release workflow to use `ghcr.io/<owner>/nebula` and the current Git tag.

## Notes

The modern Rust target equivalent to WASI preview 1 is `wasm32-wasip1`. The workflow installs this target to stay compatible with current Rust toolchains while covering the Wasm/WASI requirement.
