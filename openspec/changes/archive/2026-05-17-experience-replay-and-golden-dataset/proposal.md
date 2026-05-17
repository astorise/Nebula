# Proposal: Experience Replay and Golden Dataset Management

## Context
Nebula's continuous learning loop allows the Pulsar swarm to rapidly adapt to concept drift. However, training new LoRA adapters strictly on recent error batches introduces a severe risk of **Catastrophic Forgetting**. As the neural gradients aggressively adjust to fix new specific hallucinations (e.g., a new frontend framework), they inherently degrade previously learned behaviors (e.g., backend database tuning). To maintain a monotonic increase in general intelligence, Nebula must implement an "Experience Replay" mechanism.

## Objectives
Implement a Golden Dataset manager to stabilize neural gradients:
1. **Golden Buffer (`nebula-golden-dataset-manager`)**: A system that curates a persistent, high-quality reference dataset of past successful alignments and corrections.
2. **Semantic Diversity Sampling**: Before baking a new LoRA, the `dataset-forge` will query Tachyon's vector store to retrieve historical "Golden" examples that are *semantically distant* from the current training batch.
3. **Data Mixing**: Dynamically merge the live drift dataset (e.g., 80%) with the sampled Golden replay data (e.g., 20%) to act as an anchor, preventing the model from overfitting to the recent drift.