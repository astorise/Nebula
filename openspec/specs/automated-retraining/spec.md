# automated-retraining Specification

## Purpose
TBD - created by archiving change drift-detection-and-retraining. Update Purpose after archive.
## Requirements
### Requirement: Curriculum generator reacts to drift events

The curriculum generator SHALL convert `nebula.drift.detected` events into targeted curriculum requests for the affected topic.

#### Scenario: Drift event arrives

- **GIVEN** a drift event includes a topic and confidence score
- **WHEN** the curriculum generator handles the event
- **THEN** it creates a curriculum request focused on the drift topic
- **AND** it injects generated diagnostic prompts into `tachyon.agents.inference.pending`

### Requirement: Automated retraining preserves drift context

Generated curriculum prompts SHALL include the drift topic and confidence context so downstream dataset records can be correlated with the retraining trigger.

#### Scenario: Diagnostic prompt is generated

- **GIVEN** a drift-triggered curriculum is generated
- **WHEN** prompts are published
- **THEN** each event contains a drift correlation header
- **AND** the prompt text targets the drift topic
