# oci-manifest-builder Specification

## Purpose
TBD - created by archiving change model-quantization-optimization. Update Purpose after archive.
## Requirements
### Requirement: Orchestrator builds multi-variant OCI indexes

The training orchestrator SHALL publish fp16, Q8_0, and Q4_K artifacts as a single OCI image index with hardware annotations.

#### Scenario: Quantization completes

- **GIVEN** a `nebula.quantization.completed` event includes fp16, Q8_0, and Q4_K variants
- **WHEN** the orchestrator publishes the artifact
- **THEN** it creates an OCI image index under the configured artifact tag
- **AND** each descriptor includes `org.opencontainers.image.title`
- **AND** each descriptor includes `tachyon.mesh/min-vram`

### Requirement: OCI index preserves variant metadata

The OCI manifest builder SHALL expose variant title, digest, size, and VRAM requirements to downstream configuration consumers.

#### Scenario: Variant annotations are requested

- **GIVEN** the index contains quantized variants
- **WHEN** a consumer reads the manifest index
- **THEN** it can list every variant and its `min-vram` annotation
