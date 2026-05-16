# Design : Forge Teacher Tier 3

## Architecture

La forge Teacher ajoute quatre fonctions Rust/Wasm sous `faas/` :

- `nebula-curriculum-generator` cree des examens synthetiques et les injecte dans la file d'inference Tier 1/2 avec un header de correlation.
- `nebula-teacher-arbitrator` consomme les cas divergents, orchestre le modele Tier 3 couche par couche et emet des reponses corrigees strictement decodees en JSON.
- `nebula-dataset-forge` applique le ratio 60/40 entre escalades resolues et succes directs, puis persiste le dataset append-only en JSONL.
- `nebula-training-orchestrator` lance l'entrainement LoRA, fusionne l'adaptateur et publie le modele via `wkg`.

Les integrations runtime sont representees par des traits injectables pour conserver des crates deterministes et testables sans GPU, sans registre OCI local et sans runtime Tachyon actif.

## Flux

1. Le curriculum generator produit des taches proactives et les pousse vers Tachyon.
2. L'arbitrator traite les lots `nebula:tier3:arbitration` issus du pipeline de divergence.
3. Les corrections Tier 3 et les succes directs alimentent `nebula.dataset.append`.
4. Le dataset forge emet `nebula.training.ready` lorsque le seuil est atteint.
5. Le training orchestrator produit et publie `pulsar-base-v2.safetensors`, puis notifie l'extension.
