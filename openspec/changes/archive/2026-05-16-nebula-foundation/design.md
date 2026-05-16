# Design: Nebula Foundation

## Architecture

The repository is initialized as an npm monorepo with two workspaces:

- `packages/cli` exposes the local Nebula engine.
- `packages/extension` exposes the VS Code interface.

The CLI starts a single mTLS-enabled HTTPS server. The server handles read-only WebDAV requests at the root path and attaches a WebSocket bridge at `/ws`.

## CLI

The WebDAV server resolves all requests within `NEBULA_DOCS_ROOT` and rejects any path escape from that directory. Allowed methods are `GET`, `HEAD`, `OPTIONS`, and `PROPFIND`; mutating methods return `405 Method Not Allowed`.

The WebSocket bridge uses `ws` and verifies that the client TLS socket is authorized. Incoming messages are passed to a Tachyon router stub, which returns a stable routing event until the real Tachyon IPC adapter is available.

## VS Code Extension

The extension declares the `Nebula: Open Dashboard` command. It opens a control webview and creates a `wss` WebSocket client with the certificate paths declared in the `nebula.tls.*` configuration.

## Security

Server, client, and CA certificates are supplied through local configuration. The CLI enforces `requestCert: true` and `rejectUnauthorized: true`; the extension also validates the server through the configured CA.
