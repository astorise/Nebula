# webdav-bridge Specification

## Purpose
TBD - created by archiving change deterministic-doc-ingestion. Update Purpose after archive.
## Requirements
### Requirement: WebDAV bridge emits file update events

The CLI SHALL watch the configured WebDAV document root and emit ingestion events when files are added or modified.

#### Scenario: File is added or changed

- **WHEN** a file inside the WebDAV root is added or modified
- **THEN** the CLI waits for the debounce interval
- **AND** computes a SHA-256 hash of the file contents
- **AND** emits `nebula.fs.file_updated` over the WebSocket bridge

#### Scenario: Directory event is observed

- **WHEN** the watcher observes a directory path
- **THEN** the CLI ignores the event

#### Scenario: File is deleted before debounce completes

- **WHEN** a file disappears before the debounced event is processed
- **THEN** the CLI ignores the missing file without crashing

### Requirement: File update events include ingestion metadata

The WebDAV bridge SHALL include enough metadata for downstream FaaS components to retrieve and verify the source file.

#### Scenario: File event payload is emitted

- **WHEN** `nebula.fs.file_updated` is emitted
- **THEN** the payload includes the WebDAV-relative `path`
- **AND** the payload includes `mime_type`
- **AND** the payload includes `sha256`

