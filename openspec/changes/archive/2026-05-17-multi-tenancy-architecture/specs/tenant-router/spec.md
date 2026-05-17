## ADDED Requirements

### Requirement: Tenant router extracts tenant context
`nebula-tenant-router` SHALL read `x-tenant-id` from inference triplet context and emit a routed wrapper containing `tenant_id` and the original payload.

#### Scenario: Explicit tenant
- **GIVEN** a triplet contains `x-tenant-id`
- **WHEN** the router processes it
- **THEN** the routed event contains the sanitized tenant identifier

### Requirement: Tenant router supports strict mode
The router SHALL drop tenantless triplets when strict mode is enabled.

#### Scenario: Missing tenant in strict mode
- **GIVEN** strict mode is enabled
- **AND** no tenant id is present
- **WHEN** the router processes the triplet
- **THEN** no routed event is emitted
