# vscode-finops-ui Specification

## Purpose
TBD - created by archiving change economic-governor-and-semantic-deduplication. Update Purpose after archive.
## Requirements
### Requirement: Dashboard displays FinOps burn metrics
The VS Code dashboard SHALL display daily cost, monthly cost, token usage, token budget, saved tokens, and deduplicated request counts.

#### Scenario: FinOps metrics event
- **GIVEN** the extension receives `nebula.finops.metrics`
- **WHEN** the dashboard state updates
- **THEN** the FinOps panel displays the latest burn and savings metrics

### Requirement: Dashboard updates deduplication savings
The dashboard SHALL increment saved token and deduplicated request metrics when deduplication events arrive.

#### Scenario: Deduplicated event
- **GIVEN** the dashboard receives `nebula.finops.deduplicated`
- **WHEN** the event includes saved tokens
- **THEN** saved tokens increase
- **AND** deduplicated request count increases

### Requirement: Dashboard can set tenant token cap
The dashboard SHALL allow administrators to set the active tenant token cap through WebSocket.

#### Scenario: Save token budget
- **WHEN** an administrator saves a token cap
- **THEN** the extension sends `finops.budget.set`
