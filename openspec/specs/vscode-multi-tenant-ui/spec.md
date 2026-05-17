# vscode-multi-tenant-ui Specification

## Purpose
TBD - created by archiving change multi-tenancy-architecture. Update Purpose after archive.
## Requirements
### Requirement: Dashboard displays tenants
The VS Code dashboard SHALL display tenant summaries with row counts and quotas.

#### Scenario: Tenant list event
- **GIVEN** a `nebula.tenant.list` socket event
- **WHEN** the dashboard receives it
- **THEN** the tenant grid updates

### Requirement: Dashboard can switch active tenant
The dashboard SHALL expose a global tenant context selector.

#### Scenario: Switch tenant
- **WHEN** the user selects a tenant
- **THEN** the extension sends `tenant.setActive`

### Requirement: Dashboard can request tenant purge
The dashboard SHALL provide a purge command for tenant data.

#### Scenario: Purge tenant
- **WHEN** the user requests a purge
- **THEN** the extension sends `tenant.purge`
