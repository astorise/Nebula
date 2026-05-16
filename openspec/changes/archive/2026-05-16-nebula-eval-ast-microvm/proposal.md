# Proposition : Parseur AST via SmolVM et gRPC (virtio-vsock)

## Contexte
L'évaluation syntaxique des LLMs (détection d'hallucinations) requiert le moteur C `tree-sitter`. L'environnement WASI étant incompatible avec cette dépendance native, le composant `nebula-eval-ast` sera packagé sous forme de MicroVM Linux (SmolVM) orchestrée par Tachyon.

## Objectifs
Pour garantir un overhead quasi-nul lors des transferts mémoire entre le maillage WebAssembly et la MicroVM, l'architecture réseau reposera sur un canal `virtio-vsock` multiplexé via le protocole gRPC.

1. **Protocole Binaire (Protobuf)** : Éliminer la coûteuse sérialisation JSON en définissant un contrat Protobuf strict pour l'envoi des réponses LLM et la réception du statut de divergence.
2. **Serveur gRPC (Tonic/UDS)** : Implémenter le binaire Rust interne à la MicroVM via le framework `tonic`, configuré pour écouter sur un Unix Domain Socket (UDS) mappé au périphérique VSOCK.
3. **Packaging OCI** : Encapsuler ce binaire et les bibliothèques C partagées dans un système de fichiers `rootfs.ext4` distribué sur le registre d'artefacts.