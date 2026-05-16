# Implementation Tasks

- [x] `task-1`: Update `webdav.ts` in the Node.js CLI to watch the directory and emit `nebula.fs.file_updated` events over the WebSocket bridge.
- [x] `task-2`: Create the `nebula-doc-parser` crate. Implement the PDF-to-Markdown layout parser using pure Rust dependencies compatible with `wasm32-wasip1`.
- [x] `task-3`: Create the `nebula-semantic-chunker` crate. Implement the context-aware Markdown splitting algorithm.
- [x] `task-4`: Integrate `tachyon:inference` inside the chunker to generate vectors.
- [x] `task-5`: Integrate `tachyon:store/vector` inside the chunker to persist the final RAG database.
