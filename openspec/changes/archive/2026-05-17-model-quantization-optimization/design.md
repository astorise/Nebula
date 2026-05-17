# Design: Model Quantization and VRAM Optimization

The quantization engine is implemented as a host-agnostic Rust crate. It depends on traits for artifact volume access, quantization, and event publication so production can wire Candle or host-delegated tensor conversion without changing the orchestration logic. The current converter records deterministic Q8_0 and Q4_K outputs and calculates expected size and minimum VRAM metadata.

The training orchestrator gains an OCI image index path for quantization completion events. It maps each variant into an OCI descriptor with standard title annotations and Tachyon `min-vram` annotations, then delegates the actual registry push to its publisher boundary.

The CLI Tachyon router exposes artifact metadata and variant-ceiling configuration through existing WebSocket commands. The VS Code dashboard stores quantization state, renders an artifact size table, compares variants against reported host VRAM, and lets the user set a maximum deployable variant.
