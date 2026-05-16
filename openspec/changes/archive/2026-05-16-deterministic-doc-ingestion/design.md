# Design: Deterministic Document Ingestion

## Architecture

The ingestion flow is asynchronous and deterministic:

- The Node.js CLI watches the WebDAV document root and emits `nebula.fs.file_updated` after debounced writes.
- `nebula-doc-parser` fetches the file through WebDAV and converts supported document bytes into canonical Markdown without LLM-based extraction.
- `nebula-semantic-chunker` splits Markdown by heading hierarchy, embeds each chunk through `tachyon:inference`, and persists records through `tachyon:store/vector`.

## CLI Watcher

The watcher uses Node's native `fs.watch` to avoid another runtime dependency. Events are debounced for 2000ms, file hashes are computed with SHA-256, and payloads are emitted through the existing WebSocket event bridge.

## Parser

The initial parser is pure Rust and compatible with `wasm32-wasip1`. Markdown is passed through directly, while plain text and PDF bytes are normalized into deterministic Markdown using text layout heuristics. The parser preserves a stable output hash so downstream indexing can be idempotent.

## Chunker

The chunker keeps heading ancestry in every chunk, so an H3 section carries its H1 and H2 context. Vector generation and persistence are represented by injectable traits, matching the Tachyon host contracts while keeping the crate testable outside the runtime.
