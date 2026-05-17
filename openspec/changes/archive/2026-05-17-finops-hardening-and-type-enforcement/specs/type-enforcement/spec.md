## ADDED Requirements

### Requirement: Tenant core exposes TenantId newtype
`nebula-tenant-core` SHALL expose a `TenantId` newtype wrapper around `Uuid`.

#### Scenario: Resolve tenant
- **GIVEN** a registered raw tenant id
- **WHEN** tenant resolution succeeds
- **THEN** the caller receives a `TenantId`

### Requirement: FinOps crates use TenantId
`nebula-economic-governor` and `nebula-semantic-deduplicator` SHALL use `TenantId` in event payloads instead of raw string tenant ids.

#### Scenario: Budget request
- **WHEN** an arbitration request is created
- **THEN** its tenant field is a `TenantId`

#### Scenario: Divergence event
- **WHEN** a divergence event is classified
- **THEN** its tenant field is a `TenantId`
