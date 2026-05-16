# curriculum-generator Specification

## Purpose
TBD - created by archiving change nebula-teacher-forge. Update Purpose after archive.
## Requirements
### Requirement: Curriculum generator produces synthetic exams

The curriculum generator SHALL request a bounded set of synthetic technical tasks from the Tier 3 model using a strict JSON schema.

#### Scenario: Manual curriculum request

- **WHEN** the CLI or VS Code extension requests `N` exercises for a subject
- **THEN** the generator asks the Teacher model for `N` tasks with title, description, and constraints

### Requirement: Curriculum generator injects correlated inference tasks

The curriculum generator SHALL publish each generated task to the Tier 1/2 inference queue with a curriculum correlation header.

#### Scenario: Generated task is injected

- **WHEN** a curriculum task is generated
- **THEN** it is published to `tachyon.agents.inference.pending`
- **AND** the event includes `x-nebula-curriculum-id`

#### Scenario: Invalid curriculum count

- **WHEN** the requested exercise count is zero
- **THEN** the generator rejects the request without publishing events

