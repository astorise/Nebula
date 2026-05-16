## ADDED Requirements

### Requirement: Read-only WebDAV document mount

The Nebula CLI SHALL expose the configured documentation directory through a WebDAV-compatible HTTPS endpoint that cannot mutate source files.

#### Scenario: Read request for an existing document

- **WHEN** a client sends `GET` for a file inside the configured documentation root
- **THEN** the CLI returns the file contents with status `200`

#### Scenario: WebDAV property discovery

- **WHEN** a client sends `PROPFIND` for a file or directory inside the configured documentation root
- **THEN** the CLI returns a `207 Multi-Status` XML response with basic resource metadata

#### Scenario: Mutating WebDAV method

- **WHEN** a client sends `PUT`, `POST`, `DELETE`, `MKCOL`, `MOVE`, or `COPY`
- **THEN** the CLI returns `405 Method Not Allowed`
- **AND** the response advertises the allowed read-only methods

### Requirement: Secure WebSocket bridge

The Nebula CLI SHALL expose a WebSocket bridge for asynchronous Tachyon events over the same mTLS-protected HTTPS server.

#### Scenario: Authorized client connects

- **WHEN** a client connects to `/ws` with a certificate trusted by the configured CA
- **THEN** the WebSocket bridge accepts the connection

#### Scenario: Unauthorized client connects

- **WHEN** a client connects without an authorized certificate
- **THEN** the WebSocket bridge closes the connection with a policy violation

### Requirement: Tachyon IPC routing stub

The Nebula CLI SHALL include a routing boundary between WebSocket messages and the future Tachyon IPC adapter.

#### Scenario: Dashboard message is routed

- **WHEN** the WebSocket bridge receives a JSON message
- **THEN** the Tachyon router stub acknowledges the message with a stable `tachyon.stub.routed` event
