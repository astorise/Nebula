# vscode-deployment-ui Specification

## ADDED Requirements

### Requirement: Dashboard displays LoRA validation results

The VS Code dashboard SHALL display the latest LoRA validation A/B result with before and after outputs, pass rate, and current adapter artifact.

#### Scenario: Validation success arrives

- **GIVEN** the extension WebSocket receives a `nebula.validation.success` event
- **WHEN** the dashboard state is updated
- **THEN** the deployment panel shows the adapter artifact
- **AND** it shows the before/after sample output
- **AND** it shows the validation pass rate

### Requirement: Dashboard can request LoRA deployment

The VS Code dashboard SHALL expose a deploy control for validated adapters and send a deploy command through the CLI WebSocket bridge.

#### Scenario: User deploys a validated adapter

- **GIVEN** a validation result contains an adapter artifact
- **WHEN** the user selects deploy
- **THEN** the extension sends `{ type: "COMMAND", action: "DEPLOY_LORA", payload: { artifact } }` over the WebSocket bridge

### Requirement: CLI routes deployment commands to Tachyon configuration

The CLI Tachyon router SHALL translate `DEPLOY_LORA` commands into a Tachyon configuration update for cluster-wide adapter hot-swap.

#### Scenario: CLI accepts deployment command

- **GIVEN** the WebSocket bridge receives a `DEPLOY_LORA` command with an artifact
- **WHEN** the Tachyon router handles the command
- **THEN** it requests active LoRA routing update through the Tachyon config API boundary
- **AND** it emits a deployment status event for dashboard subscribers
