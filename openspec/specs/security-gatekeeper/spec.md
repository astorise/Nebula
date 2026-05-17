# security-gatekeeper Specification

## Purpose
TBD - created by archiving change autonomous-tool-synthesis. Update Purpose after archive.
## Requirements
### Requirement: Tool synthesis gate honors synthesis mode
`nebula-tool-gap-analyzer` SHALL evaluate disabled, human-in-loop, and fully-autonomous synthesis modes.

#### Scenario: Human-in-loop mode
- **GIVEN** synthesis mode is `human-in-loop`
- **WHEN** a missing tool signal is evaluated
- **THEN** the gate emits an approval-required decision

### Requirement: Tool synthesis gate validates WIT imports
The gate SHALL reject missing tool signals that request imports outside the configured allow list.

#### Scenario: Disallowed import
- **GIVEN** a signal requests a disallowed WIT import
- **WHEN** the gate evaluates the signal
- **THEN** it rejects the request
