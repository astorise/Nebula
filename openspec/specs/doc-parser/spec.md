# doc-parser Specification

## Purpose
TBD - created by archiving change deterministic-doc-ingestion. Update Purpose after archive.
## Requirements
### Requirement: Document parser retrieves updated files

The `nebula-doc-parser` FaaS SHALL consume file update events and fetch source bytes through WebDAV.

#### Scenario: File update event is received

- **WHEN** the parser receives `nebula.fs.file_updated`
- **THEN** it fetches the file bytes from the provided WebDAV path

### Requirement: Document parser converts documents deterministically

The parser SHALL convert supported document bytes into canonical Markdown without LLM-based extraction.

#### Scenario: Markdown file is parsed

- **WHEN** the source MIME type is `text/markdown`
- **THEN** the parser preserves the Markdown payload

#### Scenario: Text-like document is parsed

- **WHEN** the source MIME type is `text/plain` or `application/pdf`
- **THEN** the parser normalizes extractable text into deterministic Markdown
- **AND** uppercase heading-like lines are mapped to Markdown headings

#### Scenario: Unsupported MIME type is received

- **WHEN** the parser receives an unsupported MIME type
- **THEN** it rejects the document without publishing a Markdown-ready event

### Requirement: Document parser emits Markdown-ready events

The parser SHALL emit structured Markdown with source metadata for downstream chunking.

#### Scenario: Document parsing succeeds

- **WHEN** Markdown conversion succeeds
- **THEN** the parser emits `nebula.doc.markdown_ready`
- **AND** the event includes source path, source SHA-256, Markdown, and Markdown SHA-256

