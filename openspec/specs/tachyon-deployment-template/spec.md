# tachyon-deployment-template Specification

## Purpose
TBD - created by archiving change cognitive-canary-rollbacks. Update Purpose after archive.
## Requirements
### Requirement: Deployment template includes cognitive canary rules

Nebula SHALL generate a Tachyon deployment template whose canary strategy includes a `nebula.cognitive_divergence` analysis rule.

#### Scenario: Canary rollout template is requested

- **GIVEN** a deployment uses the canary strategy
- **WHEN** Nebula renders the Tachyon deployment template
- **THEN** the template includes staged rollout percentages
- **AND** it includes a `nebula.cognitive_divergence` threshold below 4 percent

### Requirement: Dashboard displays canary rollout health

The VS Code deployment dashboard SHALL display stable and canary cognitive divergence metrics streamed through the WebSocket proxy.

#### Scenario: Canary metric cache is received

- **GIVEN** the extension receives `nebula.canary.metrics`
- **WHEN** dashboard state updates
- **THEN** the deployment panel shows stable and canary divergence rates
- **AND** it shows whether the current rollout status is healthy or rollback
