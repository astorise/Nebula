# Implementation Tasks

- [x] `task-1`: Update `faas/Cargo.toml` workspace definitions to include the updated `tachyon.wit` interface definitions containing `custom-metrics`.
- [x] `task-2`: Refactor the `nebula-deployment-policy` crate to replace all raw HTTP rollback mechanisms with native calls to `custom_metrics::push`.
- [x] `task-3`: Implement the metric tagging logic to isolate canary instance performance from standard production logs.
- [x] `task-4`: Update the VSCode `DeploymentPanel.tsx` component to pull the rollout status from Tachyon's custom metrics cache via the WebSocket proxy.
- [x] `task-5`: Run a local validation test simulating an unlearned concept injection to verify that Tachyon drops the canary route when `nebula.cognitive_divergence` spikes.
