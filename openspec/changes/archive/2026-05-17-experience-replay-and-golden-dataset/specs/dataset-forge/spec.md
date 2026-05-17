## ADDED Requirements

### Requirement: Dataset forge mixes golden replay rows
Dataset forge SHALL mix live training rows with golden dataset rows before training notification using a default 500 live / 100 golden replay batch.

#### Scenario: Replay batch creation
- **GIVEN** live rows and eligible golden rows
- **WHEN** replay mixing runs with default configuration
- **THEN** the output contains live rows followed by selected golden rows

### Requirement: Golden selection prefers locked and diverse rows
Golden replay selection SHALL prefer locked rows and higher diversity scores.

#### Scenario: Locked row priority
- **GIVEN** locked and unlocked golden rows
- **WHEN** replay rows are selected
- **THEN** locked rows are ranked first
