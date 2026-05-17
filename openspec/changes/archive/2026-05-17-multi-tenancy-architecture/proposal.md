# Proposal: Multi-Tenancy Architecture and Data Isolation

## Context
As the Pulsar swarm expands into enterprise environments, a single physical Edge node managed by Tachyon may serve multiple distinct departments (e.g., HR, Finance, Engineering) or entirely different clients. To prevent data leakage and ensure that specialized knowledge (LoRAs) is only applied to authorized users, Nebula must natively support Multi-Tenancy. This requires strict cryptographic and logical isolation of telemetry, `.jsonl` datasets, DPO preferences, and neural weights.

## Objectives
Implement a secure, multi-tenant training forge:
1. **Tenant Router Middleware (`nebula-tenant-router`)**: A lightweight FaaS that intercepts all incoming mesh telemetry, validates the `x-tenant-id`, and securely tags all downstream events.
2. **Storage Isolation**: Refactor the `dataset-forge` and `crdt-merger` to use namespace-prefixed keys in `tachyon:store/kv` and isolated directory paths in `tachyon:store/volume`.
3. **Tenant-Aware Publishing**: Update the `training-orchestrator` to bake and push tenant-specific LoRA adapters to the OCI registry using `wkg`, tagged with their respective tenant identifiers.
4. **Dynamic LoRA Dispatch**: Ensure the inference host dynamically swaps the active LoRA adapter into VRAM per-request based on the incoming tenant ID.