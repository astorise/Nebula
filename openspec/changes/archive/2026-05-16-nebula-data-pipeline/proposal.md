# Proposal: Divergence Evaluation Pipeline (Data Pipeline)

## Context

To generate high-quality active-learning datasets for the Nebula forge, the system must mathematically isolate the moments when a local Pulsar swarm agent hallucinates because it lacks information. Since LLMs cannot reliably evaluate their own uncertainty, this pipeline introduces deterministic out-of-band validation based on response variance at different temperatures.

## Objectives

Implement a chain of four autonomous WebAssembly FaaS components under the repository-level `faas/` directory:

1. **`nebula-telemetry-gateway`**: Lightweight entry point that listens to the Tachyon event bus and captures response triplets (1x T=0.1, 2x T=0.8).
2. **`nebula-eval-ast`**: Syntax evaluation engine based on Tree-sitter for source code, detecting logical variation through structural hashing.
3. **`nebula-eval-semantic`**: Semantic evaluation engine that uses cosine similarity over lightweight embedding models (Candle) for free text or as a safety fallback.
4. **`nebula-divergence-aggregator`**: Final collector that assembles confirmed failure cases and pushes them into the Teacher model (Tier 3) queue.

## Communication Architecture

The FaaS components communicate exclusively through `tachyon:messaging/event-bus`, ensuring full decoupling and allowing vector-evaluation FaaS components to scale independently from syntax-evaluation components.
