# logic-and-security Specification

## Purpose
TBD - created by archiving change finops-hardening-and-type-enforcement. Update Purpose after archive.
## Requirements
### Requirement: Tenant router permissive mode forwards with default tenant
`nebula-tenant-router` SHALL fall back to a deterministic default tenant when `require_registered_tenant` is `false` and the supplied tenant cannot be resolved.

#### Scenario: Unregistered tenant in permissive mode
- **GIVEN** `require_registered_tenant` is `false`
- **AND** the tenant id is not registered
- **WHEN** a triplet is routed
- **THEN** the triplet is forwarded with the default tenant id

### Requirement: Tenant router strict mode drops unregistered tenants
`nebula-tenant-router` SHALL drop unregistered tenant traffic when `require_registered_tenant` is `true`.

#### Scenario: Unregistered tenant in strict mode
- **GIVEN** `require_registered_tenant` is `true`
- **AND** the tenant id is not registered
- **WHEN** a triplet is routed
- **THEN** no routed event is emitted

### Requirement: VS Code CSP nonce is cryptographically secure
The VS Code dashboard SHALL generate Webview CSP nonces with cryptographic randomness.

#### Scenario: Render dashboard
- **WHEN** the dashboard HTML is rendered
- **THEN** the nonce is generated from `node:crypto` random bytes
