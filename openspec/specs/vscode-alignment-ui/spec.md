# vscode-alignment-ui Specification

## Purpose
TBD - created by archiving change dpo-model-alignment. Update Purpose after archive.
## Requirements
### Requirement: Dashboard manages alignment constitution
The VS Code dashboard SHALL provide an Alignment section that edits and saves constitution rules through WebSocket commands.

#### Scenario: Save constitution
- **GIVEN** a user edits constitution rules
- **WHEN** the save action is triggered
- **THEN** the extension sends `alignment.constitution.save`

### Requirement: Dashboard reviews pending preferences
The VS Code dashboard SHALL display pending preference pairs and allow accept or reject decisions.

#### Scenario: Review preference
- **GIVEN** a pending preference arrives from the socket
- **WHEN** the user accepts or rejects it
- **THEN** the extension sends `alignment.preference.review`
