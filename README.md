# Nebula 🌌

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **Nebula** is the centralized training forge designed to power localized, autonomous swarms. It orchestrates knowledge distillation, automated dataset generation, and the fine-tuning of LoRA (Low-Rank Adaptation) layers.



## Overview

Nebula bridges the gap between massive, hardware-intensive Large Language Models (the "Teachers") and highly efficient, edge-deployed inference engines (the "Students"). By processing raw execution logs and utilizing active learning, Nebula identifies the epistemic boundaries of smaller models and automatically generates the fine-tuning data required to push past those limits. 

Whether it is injecting specialized domain knowledge or teaching a system exactly when to request assistance, Nebula outputs ready-to-deploy `.safetensors` artifacts.

## Core Capabilities

### 🧠 Automated Knowledge Distillation
Nebula converts raw, unstructured logs and complex architectural constraints into high-quality synthetic datasets. It uses advanced Teacher models to parse edge cases and distill their reasoning into contrastive pairs, creating highly focused training data for smaller Wasm-based agents.

### ⚖️ Uncertainty & Escalation Alignment
Nebula maps the "hallucination boundaries" of smaller models through temperature-variance sampling (Self-Consistency Uncertainty Estimation). It trains behavioral LoRAs that teach smaller agents exactly when they lack the necessary context, enforcing strict tool-calling fallbacks instead of generating false assumptions.

### ⚙️ VRAM-Optimized Processing
Designed to run on localized hardware configurations, Nebula implements progressive layer-by-layer loading and horizontal micro-batching. This allows massive Teacher models to process vast amounts of synthetic data without saturating local GPU memory or bottlenecking the PCIe bus.

### 📦 Seamless OCI Distribution
Once a LoRA layer is trained or merged into a base model, Nebula automatically packages the resulting `.safetensors` files as OCI artifacts. Using standard tools like `wkg`, these adapters are published directly to the local registry, ready to be pulled and hot-swapped by the underlying service mesh.

## Ecosystem Integration

Nebula is a standalone component, strictly isolated from runtime environments. 
- It ingests telemetry and error logs from distributed execution meshes (e.g., **Tachyon**).
- It produces specialized cognitive layers for autonomous agent swarms (e.g., **Pulsar**).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.