# Proposal: VS Code Dashboard for the Nebula Forge

## Context

The knowledge distillation architecture (Teacher -> Student) runs asynchronously through WebAssembly FaaS components on the Tachyon mesh. Users need an ergonomic interface integrated into their development environment (VS Code) to trigger workflows, monitor contrastive dataset construction, and visualize LoRA layer publication.

## Objectives

Develop the user interface inside `packages/extension`:

1. **Dashboard Webview**: A rich view integrated into VS Code for visualizing swarm and forge state.
2. **IPC & WebSocket Bridge**: Wire secure mTLS communication between the VS Code extension and the Node.js daemon (CLI) so Tachyon bus events surface in real time.
3. **Interactive Control**: Add manual commands to trigger `nebula-curriculum-generator` (Zero-Doc Learning) and force a model merge (Baking).
