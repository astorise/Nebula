## ADDED Requirements

### Requirement: Training artifacts include tenant tags
Training orchestrator SHALL append the tenant id to OCI artifact tags when handling tenant-scoped training events.

#### Scenario: Tenant training ready event
- **GIVEN** a training-ready event with tenant `acme`
- **WHEN** the orchestrator publishes the artifact
- **THEN** the OCI reference tag is suffixed with `acme`

### Requirement: Training config preserves tenant context
Training config SHALL carry the tenant id when a tenant-scoped event starts training.

#### Scenario: Config receives tenant
- **GIVEN** a tenant training event
- **WHEN** training starts
- **THEN** the trainer receives a config containing the tenant id
