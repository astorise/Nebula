# vscode-golden-ui Specification

## Purpose
TBD - created by archiving change experience-replay-and-golden-dataset. Update Purpose after archive.
## Requirements
### Requirement: Dashboard displays golden vault rows
The VS Code dashboard SHALL display promoted golden rows and lock state.

#### Scenario: Golden rows event
- **GIVEN** a `nebula.golden.rows` socket event
- **WHEN** the dashboard receives it
- **THEN** the Golden Vault table updates

### Requirement: Dashboard controls replay ratio
The dashboard SHALL expose a replay ratio control and send updates over WebSocket.

#### Scenario: Change replay ratio
- **WHEN** the user changes the replay ratio
- **THEN** the extension sends `golden.replayRatio.set`
