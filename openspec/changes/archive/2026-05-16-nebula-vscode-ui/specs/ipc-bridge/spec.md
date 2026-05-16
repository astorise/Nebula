## ADDED Requirements

### Requirement: Extension provider synchronizes dashboard state

The VS Code extension SHALL use webview message passing to synchronize state from the extension host into the dashboard and commands back from the dashboard.

#### Scenario: Host state changes

- **WHEN** dataset, training, connection, or log state changes in the extension host
- **THEN** the provider posts a `STATE` message to the webview

#### Scenario: User command is submitted

- **WHEN** the webview posts a `COMMAND` message
- **THEN** the provider handles the command in the extension host

### Requirement: Extension relays commands over mTLS WebSocket

The extension SHALL maintain a persistent WSS connection to the Nebula CLI and relay dashboard commands using a standard JSON envelope.

#### Scenario: Curriculum command is received from webview

- **WHEN** the provider receives `curriculum.generate`
- **THEN** it sends `{ type: "COMMAND", action: "curriculum.generate", payload }` over the WebSocket

#### Scenario: Manual merge command is received from webview

- **WHEN** the provider receives `training.forceMerge`
- **THEN** it sends `{ type: "COMMAND", action: "training.forceMerge", payload: {} }` over the WebSocket

### Requirement: Extension maps Tachyon events to dashboard state

The extension SHALL map Tachyon-originated WebSocket events into dashboard state updates.

#### Scenario: Dataset append event arrives

- **WHEN** `nebula.dataset.append` arrives from the WebSocket
- **THEN** the provider updates the dataset gauge state

#### Scenario: Training event arrives

- **WHEN** `nebula.training.ready` or `nebula.training.complete` arrives from the WebSocket
- **THEN** the provider updates the training state and posts it to the webview
