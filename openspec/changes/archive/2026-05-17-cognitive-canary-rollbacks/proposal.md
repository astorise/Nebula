# Proposal: Cognitive Canary Rollouts via Native Telemetry WIT Bindings

## Context
Tachyon v1.1.x has officially implemented our RFC by introducing the `tachyon:telemetry/custom-metrics` WIT contract. This enables any WebAssembly FaaS to natively publish domain-specific metrics directly into the host's telemetry router. We are deprecating the previous design that relied on raw HTTP POST requests to the configuration API. Instead, Nebula will stream real-time hallucination rates and cognitive uncertainty vectors straight into the Tachyon telemetry layer.

## Objectives
1. **Leverage Native WIT Telemetry**: Update our policy provider to import and invoke `custom-metrics/push` instead of exposing an HTTP endpoint or triggering webhook-driven rollbacks.
2. **Cognitive Canary Orchestration**: Map the streamed metric `nebula.cognitive_divergence` directly into Tachyon’s deployment manifests, allowing Tachyon's native rollout engine to execute automatic rollbacks.
3. **Real-time VSCode Analytics**: Stream the comparative cognitive health of the Canary vs. Stable models up to the developer's workspace panel.