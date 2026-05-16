## ADDED Requirements

### Requirement: VS Code learning dashboard

The Nebula VS Code extension SHALL provide a dashboard command for controlling and monitoring learning workflows.

#### Scenario: User opens the dashboard

- **WHEN** the user runs `Nebula: Open Dashboard`
- **THEN** the extension opens a webview panel for workflow controls and pipeline events

### Requirement: Certificate-authenticated WebSocket client

The Nebula VS Code extension SHALL connect to the CLI WebSocket endpoint with client certificate authentication.

#### Scenario: Complete TLS configuration

- **WHEN** `nebula.tls.keyPath`, `nebula.tls.certPath`, and `nebula.tls.caPath` are configured
- **THEN** the extension creates a `wss` client using those certificate files
- **AND** server certificate validation remains enabled

#### Scenario: Incomplete TLS configuration

- **WHEN** one or more certificate paths are missing
- **THEN** the extension reports that the Nebula mTLS settings are incomplete
