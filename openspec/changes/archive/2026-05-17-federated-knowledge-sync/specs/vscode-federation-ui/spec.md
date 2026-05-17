# vscode-federation-ui Specification

## ADDED Requirements

### Requirement: Dashboard displays federation peers

The VS Code dashboard SHALL display discovered Nebula peers and recent sync activity from federation events.

#### Scenario: Peer manifest arrives

- **GIVEN** the extension receives a `nebula.federation.peer` event
- **WHEN** the dashboard state is updated
- **THEN** the federation panel shows the peer node id
- **AND** it shows the advertised record count

### Requirement: Dashboard displays contribution metrics

The VS Code dashboard SHALL display local and remote contribution counts for the current dataset.

#### Scenario: Contribution event arrives

- **GIVEN** the extension receives a `nebula.federation.contribution` event
- **WHEN** the dashboard state is updated
- **THEN** the federation panel shows the source node and contributed row count

### Requirement: Dashboard can pause federated sync

The VS Code dashboard SHALL provide a control to pause or resume federated sync through the CLI WebSocket bridge.

#### Scenario: User pauses federation

- **GIVEN** federated sync is currently enabled
- **WHEN** the user toggles pause
- **THEN** the extension sends `{ type: "COMMAND", action: "federation.sync.setPaused", payload: { paused: true } }`
