# vscode-vram-ui Specification

## Purpose
TBD - created by archiving change model-quantization-optimization. Update Purpose after archive.
## Requirements
### Requirement: Dashboard displays artifact size table

The VS Code deployment dashboard SHALL display quantized artifact variants with size and minimum VRAM requirements.

#### Scenario: Quantization completion arrives

- **GIVEN** the extension receives a `nebula.quantization.completed` event
- **WHEN** dashboard state is updated
- **THEN** the deployment panel shows fp16, Q8_0, and Q4_K rows
- **AND** each row shows size and minimum VRAM

### Requirement: Dashboard displays VRAM safety

The VS Code deployment dashboard SHALL compare artifact minimum VRAM with host VRAM reported by the CLI configuration API.

#### Scenario: Host VRAM is known

- **GIVEN** the CLI reports available VRAM
- **WHEN** the artifact table is rendered
- **THEN** each variant shows a Green, Yellow, or Red safety label

### Requirement: Dashboard can set deployment variant ceiling

The VS Code deployment dashboard SHALL allow the user to set a maximum quantization variant for swarm deployment.

#### Scenario: User selects Q8_0 ceiling

- **GIVEN** quantized variants are available
- **WHEN** the user selects `q8_0`
- **THEN** the extension sends `{ type: "COMMAND", action: "deployment.variant.setMax", payload: { maxVariant: "q8_0" } }`
