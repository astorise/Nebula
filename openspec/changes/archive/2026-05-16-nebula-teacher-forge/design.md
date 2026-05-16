# Design: Tier 3 Teacher Forge

## Architecture

The Teacher forge adds four Rust/Wasm functions under `faas/`:

- `nebula-curriculum-generator` creates synthetic exams and injects them into the Tier 1/2 inference queue with a correlation header.
- `nebula-teacher-arbitrator` consumes divergent cases, orchestrates the Tier 3 model layer by layer, and emits corrected answers decoded strictly as JSON.
- `nebula-dataset-forge` applies the 60/40 ratio between resolved escalations and direct successes, then persists the dataset as append-only JSONL.
- `nebula-training-orchestrator` launches LoRA training, merges the adapter, and publishes the model through `wkg`.

Runtime integrations are represented by injectable traits so the crates remain deterministic and testable without a GPU, local OCI registry, or active Tachyon runtime.

## Flow

1. The curriculum generator produces proactive tasks and pushes them to Tachyon.
2. The arbitrator processes `nebula:tier3:arbitration` batches emitted by the divergence pipeline.
3. Tier 3 corrections and direct successes feed `nebula.dataset.append`.
4. The dataset forge emits `nebula.training.ready` when the threshold is reached.
5. The training orchestrator produces and publishes `pulsar-base-v2.safetensors`, then notifies the extension.
