# Design: Automated LoRA Validation and Deployment

The change adds a validation boundary between LoRA publication and deployment. The Rust validator stays deterministic and host-agnostic by depending on traits for prompt retrieval, Tachyon inference, AST evaluation, and event publishing. Production bindings can supply the real `tachyon:store/kv`, `tachyon:inference`, and MicroVM gRPC transports without changing validation logic.

The validator samples failed prompts from the training batch, generates one conservative response and two exploratory responses with the newly published adapter, then scores each triplet through the existing AST evaluator Protobuf contract. A validation succeeds only when all sampled triplets no longer diverge.

The VS Code dashboard keeps the latest validation result in state and renders a deployment panel with before/after evidence, pass rate, and the artifact being considered. The deploy action reuses the existing mTLS WebSocket bridge and sends a `DEPLOY_LORA` command.

The CLI router translates that command into a Tachyon config API boundary. The current implementation provides a stub config client that records the hot-swap request and emits status events, matching the existing router style while keeping the external Tachyon API replaceable.
