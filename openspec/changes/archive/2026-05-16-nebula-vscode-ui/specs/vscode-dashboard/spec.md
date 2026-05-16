## ADDED Requirements

### Requirement: Dashboard webview is bundled as frontend assets

The VS Code extension SHALL compile dashboard webview assets from `packages/extension/src/webview/` into extension media assets.

#### Scenario: Extension build runs

- **WHEN** the extension build script is executed
- **THEN** the webview TypeScript entrypoint is bundled into `packages/extension/media/webview.js`

### Requirement: Dashboard displays dataset ratio

The dashboard SHALL show the current dataset size and the contrastive confidence ratio between escalated failures and direct successes.

#### Scenario: Dataset append event is received

- **WHEN** the extension receives `nebula.dataset.append`
- **THEN** the dashboard total count is incremented
- **AND** the ratio bar updates escalated and direct percentages

### Requirement: Dashboard sends curriculum commands

The dashboard SHALL provide a curriculum form that sends user requests to the extension host.

#### Scenario: User launches curriculum evaluation

- **WHEN** the user submits a subject and exercise count
- **THEN** the webview posts a `curriculum.generate` command
- **AND** the extension relays the command over the secure WebSocket

### Requirement: Dashboard monitors Tier 3 work

The dashboard SHALL show live divergence and training events from the Teacher forge.

#### Scenario: Divergence event is received

- **WHEN** the extension receives `nebula.eval.results`
- **THEN** the dashboard appends a visible log entry for the divergence

#### Scenario: Training state event is received

- **WHEN** the extension receives `nebula.training.ready` or `nebula.training.complete`
- **THEN** the dashboard updates the LoRA training status indicator
