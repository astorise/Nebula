# Proposal: Dataset Forge and Tier 3 Arbitration

## Context

Once uncertainty and hallucination zones in local Tier 1/2 agents have been identified by the evaluation pipeline, those dead ends must be resolved by a model with stronger reasoning capability (Teacher Model / Tier 3, for example DeepSeek). This process must run under strict hardware constraints (limited VRAM) and produce automated neural weight artifacts (LoRA).

## Objectives

Implement the distillation and training chain under `faas/`:

1. **`nebula-curriculum-generator`**: FaaS that initiates synthetic knowledge exams (for example Cobol or Rust) to proactively test small models without documentation.
2. **`nebula-teacher-arbitrator`**: Heavy FaaS that orchestrates layer-by-layer Tier 3 model loading through `tachyon:inference` to correct hallucinations.
3. **`nebula-dataset-forge`**: Storage FaaS that assembles contrastive pairs into a persistent `.jsonl` volume while enforcing the confidence ratio (60% resolved failures / 40% direct successes).
4. **`nebula-training-orchestrator`**: FaaS triggered at a configured threshold (for example 500 examples) to drive LoRA training and publish the resulting layer to the local artifact registry.
