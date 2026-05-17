## ADDED Requirements

### Requirement: VS Code dashboard uses nonce-only CSP
The dashboard Webview SHALL remove `unsafe-inline` and use nonce-based CSP for both script and style execution.

#### Scenario: Render dashboard
- **WHEN** the Webview HTML is rendered
- **THEN** the CSP contains `script-src 'nonce-...'`
- **AND** `style-src 'nonce-...'`
- **AND** no `unsafe-inline` directive

### Requirement: Webview styles receive the nonce
The Webview runtime SHALL apply the provided nonce to its dynamically injected style element.

#### Scenario: Inject runtime styles
- **GIVEN** the root element contains the style nonce
- **WHEN** the webview script creates a style element
- **THEN** the element nonce matches the CSP nonce

### Requirement: Rust IPC types export TypeScript bindings
Rust IPC payload structs SHALL derive `ts-rs` exports for synchronization with the VS Code extension.

#### Scenario: Tenant summary export
- **WHEN** Rust tests run
- **THEN** `packages/extension/src/types/generated.ts` contains the generated tenant summary type
