# security-and-types Delta

## ADDED Requirements

### Requirement: Rootfs checksum is detached
The Wasm Foundry rootfs build script SHALL write the manifest checksum to a detached `.sha256` file instead of appending the checksum into the manifest being hashed.

#### Scenario: Build rootfs metadata
- **WHEN** `build-rootfs.sh` writes `manifest.txt`
- **THEN** it writes `manifest.txt.sha256`
- **AND** `manifest.txt` remains unchanged by the checksum output

### Requirement: Tenant index keys require TenantId
Dataset forge tenant index keys SHALL require a validated `TenantId` rather than parsing tenant strings internally.

#### Scenario: Build tenant index key
- **WHEN** `tenant_index_key` is called
- **THEN** the caller passes a `TenantId`
- **AND** the function cannot panic on invalid tenant text

### Requirement: DPO and golden dataset types use TenantId
`nebula-dpo-judge` and `nebula-golden-dataset-manager` SHALL use `TenantId` for tenant identifiers.

#### Scenario: DPO judgement
- **WHEN** a DPO judgement carries tenant context
- **THEN** the tenant field is `Option<TenantId>`

#### Scenario: Golden row promotion
- **WHEN** a candidate row is promoted
- **THEN** candidate and golden row tenant fields are `TenantId`
