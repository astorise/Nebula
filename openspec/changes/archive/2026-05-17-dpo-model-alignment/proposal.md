# Proposal: Direct Preference Optimization (DPO) Alignment Pipeline

## Context
Nebula currently employs Supervised Fine-Tuning (SFT) to correct hallucinations. While SFT is excellent for injecting missing knowledge, it struggles to enforce stylistic guidelines, behavioral guardrails, or safety constraints (e.g., "always write safe Rust," "do not apologize," "be concise"). To align the Pulsar swarm with specific enterprise or developer guidelines, Nebula must implement a Direct Preference Optimization (DPO) pipeline. DPO is mathematically stable, avoids the need for complex Reward Models (unlike PPO/RLHF), and runs efficiently on Edge hardware.

## Objectives
1. **Preference Dataset Generation**: Convert the existing hallucination loop into a preference loop. The original Tier 1 hallucination automatically becomes the `rejected` response, and the Tier 3 Teacher's correction becomes the `chosen` response.
2. **Constitutional AI Arbitrator (`nebula-dpo-judge`)**: A new FaaS that ensures the Tier 3 model adheres to a user-defined "Constitution" (guidelines) when generating the `chosen` response.
3. **DPO Training Engine**: Extend the `nebula-training-orchestrator` to support the DPO loss function via the Candle ML framework, baking behavioral alignment into a dedicated LoRA adapter.
4. **Alignment Dashboard (VSCode)**: A UI allowing developers to define their Swarm Constitution and review the DPO dataset pairs.