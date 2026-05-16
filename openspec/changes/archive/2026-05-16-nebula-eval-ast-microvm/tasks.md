# Tâches d'implémentation

- [x] `task-1` : Créer le fichier `proto/ast_evaluator.proto` et configurer `tonic-build` dans le `build.rs` de la caisse `nebula-eval-ast`.
- [x] `task-2` : Implémenter le serveur gRPC avec `tonic` écoutant spécifiquement sur le `UnixListener` (`/run/guest.sock`).
- [x] `task-3` : Implémenter la logique métier d'appel à `tree-sitter` pour générer le hash depuis la `EvaluationRequest`.
- [x] `task-4` : Écrire le script `build-rootfs.sh` générant l'image `ext4` packagée avec le binaire gRPC et les dépendances natives.
- [x] `task-5` : Mettre à jour `nebula-telemetry-gateway` pour générer le binaire Protobuf (`prost`) et l'envoyer via `wasi:http`.
