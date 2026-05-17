## ADDED Requirements

### Requirement: Training orchestrator computes DPO loss
`nebula-training-orchestrator` SHALL expose DPO loss calculation using sequential policy and reference forward-pass log probabilities.

#### Scenario: DPO loss favors chosen answer
- **GIVEN** policy and reference log probabilities for chosen and rejected responses
- **WHEN** the DPO loss is computed with default beta `0.1`
- **THEN** the loss reflects the policy margin relative to the reference margin

### Requirement: DPO beta defaults to 0.1
The DPO training configuration SHALL default `beta` to `0.1`.

#### Scenario: Default config
- **WHEN** a DPO training config is created without overrides
- **THEN** beta is `0.1`
