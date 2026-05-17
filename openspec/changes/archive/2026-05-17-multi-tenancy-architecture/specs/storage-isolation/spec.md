## ADDED Requirements

### Requirement: Dataset paths are tenant scoped
Dataset forge SHALL build tenant dataset paths under `/mnt/forge/tenants/{tenant_id}/dataset_v1.jsonl`.

#### Scenario: Tenant dataset path
- **GIVEN** tenant id `acme/prod`
- **WHEN** the dataset path is built
- **THEN** unsafe path characters are sanitized

### Requirement: CRDT keys are tenant scoped
CRDT merger SHALL prefix row hashes as `tenant:{tenant_id}:crdt:hash:{sha256}`.

#### Scenario: Tenant CRDT key
- **GIVEN** a tenant and prompt
- **WHEN** the CRDT key is built
- **THEN** the key includes the sanitized tenant namespace

### Requirement: Tenant row quota is enforced
Dataset forge SHALL reject appends beyond the tenant row quota of 50,000 rows.

#### Scenario: Quota exceeded
- **GIVEN** a tenant already has 50,000 rows
- **WHEN** quota enforcement runs
- **THEN** it returns an error
