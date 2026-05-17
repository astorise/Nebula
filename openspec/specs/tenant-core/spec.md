# tenant-core Specification

## Purpose
TBD - created by archiving change audit-remediation-and-security-hardening. Update Purpose after archive.
## Requirements
### Requirement: Tenant core resolves registered tenants to UUIDs
`nebula-tenant-core` SHALL resolve raw tenant identifiers through a registry and return a UUID only when the tenant is registered.

#### Scenario: Registered tenant
- **GIVEN** a registry contains tenant `acme`
- **WHEN** the router resolves `acme`
- **THEN** a tenant UUID is returned

#### Scenario: Unregistered tenant
- **GIVEN** a registry does not contain tenant `acme/prod`
- **WHEN** the router resolves `acme/prod`
- **THEN** an error is returned and the payload is dropped

### Requirement: Tenant core centralizes quota checks
`nebula-tenant-core` SHALL expose quota enforcement using tenant UUIDs and registry row counts.

#### Scenario: Quota exceeded
- **GIVEN** the row count is greater than or equal to the tenant quota
- **WHEN** quota enforcement runs
- **THEN** a quota error is returned

### Requirement: Dataset forge uses tenant UUID paths
Dataset forge SHALL build tenant paths from UUID tenant identifiers instead of lossy string sanitization.

#### Scenario: Tenant dataset path
- **GIVEN** a tenant UUID
- **WHEN** a tenant dataset path is built
- **THEN** the UUID appears in the tenant volume path
