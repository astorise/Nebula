# Design : CI/CD Nebula

## Architecture

La CI est separee en deux jobs paralleles :

- `validate-node` installe les workspaces npm, lance le lint, le build et les tests Node.
- `validate-rust-wasm` installe Rust stable, `cargo-component`, `clang`, `wasi-libc`, la cible `wasm32-wasip1`, puis lance formatage, clippy et tests.

Le workflow release se declenche uniquement sur les tags `v*`. Il construit les packages Node, genere le `.vsix`, construit et pousse les composants FaaS vers GHCR, puis cree une GitHub Release.

## Publication FaaS

Le script `scripts/publish-faas-oci.sh` decouvre les crates sous `faas/*/Cargo.toml`, execute `cargo component build --release --package <crate>` et pousse chaque composant avec `wkg push`.

Le registre et le tag sont parametrables via `OCI_REGISTRY` et `OCI_TAG`, ce qui permet au workflow release d'utiliser `ghcr.io/<owner>/nebula` et le tag Git courant.

## Notes

La cible Rust moderne equivalente a WASI preview 1 est `wasm32-wasip1`. Le workflow installe cette cible pour rester compatible avec les toolchains Rust actuelles tout en couvrant l'exigence Wasm/WASI.
