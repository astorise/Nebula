# vscode-tunnel-ui Delta

## ADDED Requirements

### Requirement: CLI starts Wormhole tunnel for WebDAV
The Nebula CLI SHALL create a Wormhole reverse tunnel for the local WebDAV endpoint and publish tunnel status events.

#### Scenario: Start CLI WebDAV server
- **WHEN** the CLI binds the local WebDAV server
- **THEN** it starts a Wormhole tunnel for the bound host and port
- **AND** it emits `nebula.wormhole.status` with the tunnel host

### Requirement: File update events include tunnel host
`nebula.fs.file_updated` events SHALL include a required `tunnel_host` property that points FaaS document ingestion at the mesh-routable Wormhole endpoint.

#### Scenario: Emit file update
- **WHEN** the WebDAV watcher emits a file update
- **THEN** the payload includes `path`, `mime_type`, `sha256`, and `tunnel_host`

### Requirement: Doc parser fetches through tunnel host
The document parser SHALL use the event `tunnel_host` when fetching updated document content.

#### Scenario: Ingest file update
- **WHEN** `nebula-doc-parser` receives a file update event
- **THEN** it passes `tunnel_host` and `path` to the WebDAV client fetch operation

### Requirement: Dashboard displays Wormhole status
The VS Code dashboard SHALL display the current Wormhole tunnel status and host.

#### Scenario: Receive tunnel status
- **WHEN** the extension receives `nebula.wormhole.status`
- **THEN** the webview state includes the Wormhole status
- **AND** the dashboard renders the status and host
