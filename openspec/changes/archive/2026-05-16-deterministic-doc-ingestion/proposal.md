# Proposal: Deterministic Document Ingestion & RAG Pipeline

## Context
Nebula features a read-only WebDAV server designed to expose local user documentation (PDFs, DOCX, Markdown) to the training forge. To provide the Tier 3 Teacher model with accurate ground-truth context for Active Learning, this raw documentation must be ingested, structured, and vectorized. 

Relying on LLMs (Vision models) for PDF extraction introduces unacceptable risks of structural hallucination. Therefore, the extraction process must be strictly deterministic: parsing layouts, extracting hierarchical trees (Titles, Subtitles, Paragraphs), converting to Markdown, and performing semantic chunking *before* any vector embedding occurs.

## Objectives
Implement a 3-stage asynchronous ingestion pipeline:
1. **WebDAV Event Bridge**: Enhance the Node.js CLI to emit events on the Tachyon bus when files are added or modified in the WebDAV directory.
2. **Deterministic Parser (`nebula-doc-parser`)**: A WebAssembly FaaS that receives raw file bytes, performs layout analysis, and outputs a canonical Markdown representation preserving the document's logical tree.
3. **Semantic Chunker & Embedder (`nebula-semantic-chunker`)**: A WebAssembly FaaS that splits the Markdown by structural boundaries (Headers), invokes the local embedding model via `tachyon:inference`, and persists the vectors into `tachyon:store/vector`.