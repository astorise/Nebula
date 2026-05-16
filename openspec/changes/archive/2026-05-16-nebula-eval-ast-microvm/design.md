# Design : AST evaluator microVM

## Architecture

`nebula-eval-ast` devient un binaire Linux natif execute dans une microVM SmolVM. Les autres FaaS restent des composants Wasm.

Le contrat entre le mesh Tachyon et la microVM est defini dans `proto/ast_evaluator.proto`. Le binaire microVM utilise `tonic` et ecoute sur un Unix Domain Socket, par defaut `/run/guest.sock`, qui est mappe au transport `virtio-vsock` par le host.

## Flux

1. `nebula-telemetry-gateway` detecte une tache de code.
2. Il encode `EvaluationRequest` en Protobuf et l'encapsule dans une frame gRPC HTTP/2.
3. Le core-host Tachyon route la requete `wasi:http` vers le socket de la microVM.
4. Le serveur `nebula-eval-ast` calcule les hash structurels et renvoie `EvaluationResponse`.
5. En cas d'echec de parsing ou de payload invalide, `fallback_reason` permet de basculer vers l'evaluation semantique.

## Packaging

`scripts/build-eval-ast-rootfs.sh` construit le binaire musl release et cree une image `rootfs.ext4` avec le binaire comme processus de demarrage.

La CI valide `nebula-eval-ast` en natif et exclut ce crate du clippy Wasm. Le script OCI ignore aussi ce crate car son artefact est un rootfs microVM, pas un composant Wasm.
