# Wormhole Real Client Delta

## ADDED Requirements

### Requirement: CLI tunnel startup must use the real Wormhole client

The CLI MUST instantiate `@tachyon-mesh/wormhole` for tunnel startup and MUST remove the local mock package and hardcoded tunnel host fallback.

#### Scenario: Wormhole handshake succeeds

- **GIVEN** the real Wormhole client returns an endpoint
- **WHEN** the CLI starts the tunnel
- **THEN** emitted file update events use the dynamic ingress host derived from that endpoint

#### Scenario: Wormhole handshake fails

- **GIVEN** the real Wormhole client cannot establish the tunnel
- **WHEN** the CLI starts
- **THEN** startup fails fatally instead of falling back to a mock URL
