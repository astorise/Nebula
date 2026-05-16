# Design: Divergence Evaluation Pipeline

## Architecture

The pipeline is split into four Rust crates under `faas/`, each representing a separately deployable Wasm function:

- `nebula-telemetry-gateway` routes inference triplets from `tachyon:messaging/event-bus`.
- `nebula-eval-ast` evaluates code generations through structural hashing.
- `nebula-eval-semantic` evaluates free text through embeddings and cosine similarity.
- `nebula-divergence-aggregator` turns confirmed divergences into persistent Tier 3 tasks.

Tachyon integrations are modeled as traits (`EventBus`, `InferenceHost`, `KvListStore`, `GrammarRegistry`) so the crates remain compilable and testable outside the Wasm runtime. The Tachyon host supplies concrete implementations during FaaS packaging.

## Flow

1. The gateway consumes `pulsar.telemetry.inference_triplets`.
2. Code payloads are sent to `nebula.eval.ast.pending`; text payloads are sent to `nebula.eval.semantic.pending`.
3. Evaluators publish only confirmed divergences to `nebula.eval.results`.
4. The aggregator filters `diverged == true` and pushes a JSON task into the `nebula:tier3:arbitration` KV list.

## Tradeoffs

The Tree-sitter Wasm binding is exposed as a registry dependency through `GrammarRegistry`; the current implementation validates dynamic loading and applies deterministic structural hashing. The exact connection to Tachyon registry grammar modules remains isolated behind that trait.
