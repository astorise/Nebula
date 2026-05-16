# Design : Pipeline d'evaluation de divergence

## Architecture

Le pipeline est decoupe en quatre crates Rust sous `faas/`, chacune representant une fonction Wasm deployable separement :

- `nebula-telemetry-gateway` route les triplets d'inference depuis `tachyon:messaging/event-bus`.
- `nebula-eval-ast` evalue les generations de code via un hachage structurel.
- `nebula-eval-semantic` evalue les textes libres via embeddings et similarite cosinus.
- `nebula-divergence-aggregator` transforme les divergences confirmees en taches Tier 3 persistantes.

Les integrations Tachyon sont modelisees par des traits (`EventBus`, `InferenceHost`, `KvListStore`, `GrammarRegistry`) afin que les crates restent compilables et testables hors du runtime Wasm. Le host Tachyon fournira les implementations concretes lors du packaging FaaS.

## Flux

1. Le gateway consomme `pulsar.telemetry.inference_triplets`.
2. Les payloads code sont envoyes vers `nebula.eval.ast.pending`; les payloads texte vers `nebula.eval.semantic.pending`.
3. Les evaluateurs publient uniquement les divergences confirmees sur `nebula.eval.results`.
4. L'aggregator filtre `diverged == true` et pousse une tache JSON dans la liste KV `nebula:tier3:arbitration`.

## Compromis

Le binding Tree-sitter Wasm est expose comme dependance de registre via `GrammarRegistry`; l'implementation actuelle valide le chargement dynamique et applique un hachage structurel deterministe sur l'arbre produit par `tree-sitter`. Le branchement exact vers les grammaires Wasm du registre Tachyon reste isole derriere ce trait.
