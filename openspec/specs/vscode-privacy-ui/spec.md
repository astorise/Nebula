# vscode-privacy-ui Specification

## Purpose
TBD - created by archiving change data-privacy-and-anonymization. Update Purpose after archive.
## Requirements
### Requirement: Dashboard displays privacy audit metrics

The VS Code dashboard SHALL display privacy masking metrics received from the CLI WebSocket bridge.

#### Scenario: Privacy metrics arrive

- **GIVEN** the extension receives `nebula.privacy.metrics`
- **WHEN** dashboard state updates
- **THEN** the privacy panel shows total prompts scanned
- **AND** it shows masked entity counts by rule

### Requirement: Dashboard provides live anonymization sandbox

The VS Code dashboard SHALL allow developers to test custom text against the anonymizer through the CLI WebSocket bridge.

#### Scenario: User runs privacy sandbox

- **GIVEN** the user enters raw text in the privacy sandbox
- **WHEN** the user runs the test
- **THEN** the extension sends `{ type: "COMMAND", action: "privacy.sandbox.test", payload: { text } }`
- **AND** the dashboard displays the masked output returned by the CLI
