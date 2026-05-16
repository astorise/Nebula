# Tâches d'implémentation

- [x] `task-1` : Créer le fichier `.github/workflows/ci.yml` pour le linting et le build de test (Node.js + Rust/Wasm).
- [x] `task-2` : Configurer `cargo-component` et la cible `wasm32-wasi` dans l'image runner des GitHub Actions.
- [x] `task-3` : Créer le fichier `.github/workflows/release.yml` pour déclencher les builds sur l'événement de création de tag.
- [x] `task-4` : Implémenter le script de publication OCI via `wkg push` pour itérer sur les 4 dossiers du répertoire `faas/`.
- [x] `task-5` : Intégrer l'action `softprops/action-gh-release` pour uploader le fichier `.vsix` compilé par `vsce`.
