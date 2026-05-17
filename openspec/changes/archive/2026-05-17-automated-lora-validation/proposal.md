# Proposal: Automated LoRA Validation and Swarm Deployment

## Context
The `nebula-training-orchestrator` successfully bakes contrastive datasets into new LoRA adapters and publishes them as OCI artifacts. However, deploying unchecked weights to a live swarm risks introducing regressions (Catastrophic Forgetting). Nebula requires an automated validation step that replays the previously failed prompts against the newly trained adapter to mathematically prove the hallucination is resolved before authorizing deployment.

## Objectives
Implement the Validation and Deployment pipeline:
1. **Validation Sandbox (`nebula-lora-validator`)**: A new WebAssembly FaaS that intercepts the end of a training job, loads the new LoRA adapter into a temporary Candle inference context, and replays the original failed questions.
2. **Regression Scoring**: Feed the new outputs back through the MicroVM AST Evaluator (gRPC) to ensure the divergence variance (uncertainty) has dropped to zero.
3. **Deployment UI (VSCode)**: Enhance the VSCode Dashboard to display the "Before/After" A/B test results of the LoRA, accompanied by a 1-click "Deploy to Swarm" button that triggers a hot-swap across Tachyon nodes.