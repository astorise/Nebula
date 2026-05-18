# redis-integration-tests Specification

## Purpose
TBD - created by archiving change production-release-blockers. Update Purpose after archive.
## Requirements
### Requirement: Redis reconciliation must clamp usage at zero

The Redis budget adapter MUST reconcile token corrections with a Lua script that prevents tenant usage from dropping below zero by using `math.max(0, current - delta)`.

#### Scenario: Exact usage is lower than the reservation

- **GIVEN** a tenant has reserved tokens in Redis
- **WHEN** reconciliation subtracts more reserved tokens than are currently stored
- **THEN** Redis stores zero usage rather than a negative value

### Requirement: Redis quota reservations must be integration-tested against Dockerized Redis

The Redis budget adapter MUST include a `testcontainers-rs` Redis integration test with 10 concurrent reservation attempts of 100 tokens against a 500-token budget.

#### Scenario: Concurrent reservations respect quota atomically

- **GIVEN** a Dockerized Redis instance and a tenant budget of 500 tokens
- **WHEN** 10 concurrent workers each try to reserve 100 tokens
- **THEN** exactly 5 reservations succeed and the remaining 5 fail
