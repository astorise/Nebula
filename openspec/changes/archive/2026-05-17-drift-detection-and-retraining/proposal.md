# Proposal: Model Drift Detection and Automated Retraining

## Context
Deploying a validated LoRA adapter to the Pulsar swarm solves known hallucinations at a specific point in time. However, as the operating environment evolves (e.g., users asking about newly released programming frameworks), the static model will begin to hallucinate again. This phenomenon, known as Model Drift or Concept Drift, requires continuous monitoring. Nebula needs a mechanism to detect when the swarm's confidence drops below an acceptable baseline over a rolling window, and automatically initiate a new data gathering and retraining cycle.

## Objectives
Implement a continuous monitoring and retraining loop:
1. **Drift Monitor (`nebula-drift-monitor`)**: A lightweight FaaS that aggregates inference metrics (specifically, the frequency of fallback tool calls or high-variance responses) over time.
2. **Threshold Alarming**: Define statistical thresholds (e.g., "hallucination rate exceeds 5% of total inferences over 1 hour") to trigger an alert.
3. **Automated Curriculum Trigger**: Upon detecting drift, automatically instruct the `curriculum-generator` to probe the swarm specifically around the topics causing the new errors, restarting the Active Learning pipeline without human intervention.