# concrete-adapters Delta

## ADDED Requirements

### Requirement: Redis budget store reserves quota atomically
Nebula SHALL provide a concrete `nebula-redis-budget-store` adapter that implements the economic governor `BudgetStore` trait with Redis-backed atomic token reservation.

#### Scenario: Reserve under quota
- **WHEN** `reserve_if_under_quota` is called and current usage plus estimated tokens is within the daily limit
- **THEN** a single Redis Lua script increments usage atomically
- **AND** the adapter returns a reservation receipt

#### Scenario: Reject exhausted quota
- **WHEN** current usage plus estimated tokens exceeds the daily limit
- **THEN** the Lua script does not increment usage
- **AND** the adapter returns no reservation receipt

### Requirement: Redis adapter remains host-only for Wasm checks
The Redis adapter SHALL compile in the full workspace and SHALL avoid pulling Redis socket dependencies into `wasm32` builds.

#### Scenario: Wasm workspace Clippy
- **WHEN** the full workspace is checked for `wasm32-wasip1`
- **THEN** the Redis adapter compiles without Redis network dependencies on that target
