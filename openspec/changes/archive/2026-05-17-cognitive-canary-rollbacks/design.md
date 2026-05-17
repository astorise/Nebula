# Design: Cognitive Canary Rollbacks

The deployment policy is modeled as a Rust FaaS crate with a `CustomMetrics` trait representing Tachyon's `tachyon:telemetry/custom-metrics` WIT import. The code no longer models HTTP rollback calls; it emits gauge metrics that Tachyon's rollout engine can evaluate natively.

Inference outcomes are grouped by model version and tagged as `canary` or `stable` based on the version suffix. The policy computes cognitive divergence over a small rolling window and pushes `nebula.cognitive_divergence` with `model_version` and `rollout_track` tags.

The crate also renders a declarative Tachyon canary deployment template with the cognitive divergence rule. The CLI exposes a custom-metrics cache over the existing WebSocket bridge, and the VS Code dashboard renders the comparative canary/stable health.
