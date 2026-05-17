# Proposal: Data Privacy and PII Anonymization Pipeline

## Context
Nebula's Active Learning loop relies on capturing failed production prompts from the Pulsar swarm. In a real-world environment, these prompts frequently contain sensitive data (Passwords, API Keys, Credit Card numbers, Names, Emails). Leaking this data to the Tier 3 Teacher API or baking it into the `.safetensors` LoRA weights constitutes a severe security breach and compliance violation. All telemetry data must be strictly scrubbed and anonymized before entering the evaluation and distillation pipeline.

## Objectives
Implement a robust, deterministic data masking gateway:
1. **PII Scrubber (`nebula-data-anonymizer`)**: A high-performance WebAssembly FaaS acting as a middleware filter. It will use deterministic pattern matching (Regex) and lightweight Named Entity Recognition (NER) to mask sensitive entities (e.g., replacing "John Doe" with `<PERSON_1>`).
2. **Telemetry Interception**: Update the `nebula-telemetry-gateway` to route raw inference results through the anonymizer *before* passing them to the AST Evaluator or the Divergence Aggregator.
3. **Privacy Dashboard (VSCode)**: Add a configuration UI to the VSCode extension allowing developers to define custom regex rules, manage compliance dictionaries, and audit the masking engine's effectiveness.