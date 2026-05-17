# Proposal: Model Quantization and VRAM Optimization Pipeline

## Context
Nebula's `training-orchestrator` produces standard `fp16` (16-bit floating point) `.safetensors` LoRA adapters and merged base models. While highly accurate, these artifacts are large and consume significant VRAM, making them unsuitable for constrained edge devices (e.g., IoT gateways, older laptops) running the Pulsar swarm. To deploy knowledge universally across the Tachyon mesh, Nebula must compress these neural weights immediately after validation.

## Objectives
Implement an automated quantization pipeline:
1. **Quantization Engine (`nebula-quantization-engine`)**: A new FaaS that intercepts validated `fp16` artifacts and compresses them into memory-efficient formats (e.g., `GGUF` using Q4_K or Q8_0 quantization) via Candle's tensor operations.
2. **Multi-Architecture OCI Manifests**: Update the publishing pipeline to generate standard OCI Image Indexes, linking the `fp16`, `q8_0`, and `q4_k` variants under a single artifact tag.
3. **Hardware-Aware UI Dashboard**: Update the VSCode extension to display the size and VRAM requirements of the quantized models before swarm deployment.