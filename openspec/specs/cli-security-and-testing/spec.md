# cli-security-and-testing Specification

## Purpose
TBD - created by archiving change production-release-blockers. Update Purpose after archive.
## Requirements
### Requirement: CLI WebDAV requests must be confined by real paths

The Node.js CLI WebDAV server MUST resolve both the configured document root and the requested target with `fs.realpath` before serving an existing file or directory. Requests whose real target is outside the real document root MUST be rejected.

#### Scenario: Symlink escapes are rejected

- **GIVEN** a document root containing a symlink to a file outside the document root
- **WHEN** a WebDAV client requests the symlink path
- **THEN** the CLI rejects the request instead of serving the external file

#### Scenario: Lexical traversal is rejected

- **GIVEN** a WebDAV request path containing `..` traversal outside the document root
- **WHEN** the CLI resolves the path
- **THEN** the CLI rejects the request before file access

### Requirement: CLI runtime security behavior must be covered by Vitest

The CLI package MUST include Vitest-based runtime tests for WebDAV traversal rejection, PROPFIND XML escaping, WebSocket mTLS rejection, and WebSocket message parser validation.

#### Scenario: Malformed WebSocket payloads are rejected

- **GIVEN** a WebSocket message containing malformed JSON or an invalid Tachyon message structure
- **WHEN** the CLI parses the message
- **THEN** parsing fails without crashing the CLI process

#### Scenario: PROPFIND hrefs are XML escaped

- **GIVEN** a file name containing XML-sensitive characters
- **WHEN** the CLI renders a PROPFIND multistatus response
- **THEN** the response escapes those characters in the `<D:href>` value
