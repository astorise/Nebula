# Proposition : CI/CD Nebula

## Why

Nebula contient des packages Node.js, une extension VS Code et des FaaS Rust/Wasm. Les changements doivent etre valides automatiquement et les artefacts de release doivent etre produits de maniere reproductible.

## What Changes

- Ajouter une CI GitHub Actions pour valider Node.js et Rust/Wasm.
- Installer `cargo-component` et la cible Wasm dans les runners.
- Ajouter un workflow de release declenche par tag.
- Ajouter un script de publication OCI des composants FaaS via `wkg push`.
- Publier l'extension VS Code compilee en `.vsix` dans les GitHub Releases.
