# vscode-foundry-ui Specification

## Purpose
TBD - created by archiving change autonomous-tool-synthesis. Update Purpose after archive.
## Requirements
### Requirement: Dashboard displays foundry approvals
The VS Code dashboard SHALL display synthesized tool approval requests.

#### Scenario: Approval required event
- **GIVEN** a `nebula.foundry.approval_required` socket event
- **WHEN** the dashboard receives it
- **THEN** the Foundry panel lists the pending tool

### Requirement: Dashboard can approve injection
The Foundry panel SHALL send approval commands for pending tools.

#### Scenario: Approve tool
- **WHEN** the user approves a tool
- **THEN** the extension sends `foundry.approve`
