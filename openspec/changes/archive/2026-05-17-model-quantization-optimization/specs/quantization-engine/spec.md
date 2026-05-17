# quantization-engine Specification

## ADDED Requirements

### Requirement: Quantization engine converts validated artifacts

The `nebula-quantization-engine` Wasm component SHALL react to `nebula.validation.success` events and generate Q8_0 and Q4_K variants from the validated fp16 artifact.

#### Scenario: Validated adapter is available

- **GIVEN** a validation success event contains an fp16 artifact reference
- **WHEN** the quantization engine handles the event
- **THEN** it reads the source artifact from the Tachyon volume boundary
- **AND** it writes Q8_0 and Q4_K variants
- **AND** it keeps the original fp16 variant in the result set

### Requirement: Quantization engine reports memory footprint

The quantization engine SHALL calculate artifact size and minimum VRAM metadata for fp16, Q8_0, and Q4_K variants.

#### Scenario: Quantized variants are produced

- **GIVEN** the source artifact byte size is known
- **WHEN** quantization completes
- **THEN** the result includes size bytes and `min_vram_gb` for each variant

### Requirement: Quantization engine emits completion event

The quantization engine SHALL emit `nebula.quantization.completed` after all variants are generated.

#### Scenario: Quantization succeeds

- **GIVEN** fp16, Q8_0, and Q4_K variants were recorded
- **WHEN** the engine finishes
- **THEN** it publishes `nebula.quantization.completed`
- **AND** the payload includes the source artifact and all generated variants
