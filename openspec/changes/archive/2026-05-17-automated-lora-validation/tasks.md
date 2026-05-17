# Implementation Tasks

- [x] `task-1`: Create the `nebula-lora-validator` Wasm crate in the `faas/` directory.
- [x] `task-2`: Implement the prompt replay logic using the `tachyon:inference` bindings with the newly compiled adapter applied.
- [x] `task-3`: Connect the validator to the existing gRPC Protobuf client to request AST evaluation from the MicroVM.
- [x] `task-4`: Build the `DeploymentPanel` React/Web component in the VSCode extension to visualize the A/B test results.
- [x] `task-5`: Wire the "Deploy" button to the CLI WebSocket, and implement the Tachyon config API call to trigger the cluster-wide hot-swap.
