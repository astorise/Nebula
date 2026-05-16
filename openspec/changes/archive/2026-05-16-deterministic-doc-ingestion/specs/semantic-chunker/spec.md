## ADDED Requirements

### Requirement: Semantic chunker splits Markdown by hierarchy

The `nebula-semantic-chunker` FaaS SHALL split Markdown by structural heading boundaries instead of blind character counts.

#### Scenario: Nested headings are chunked

- **WHEN** Markdown contains nested `#`, `##`, and `###` headings
- **THEN** each emitted chunk retains the full heading path for its section

#### Scenario: Markdown has no body content

- **WHEN** Markdown does not produce any chunk body
- **THEN** the chunker reports that no semantic chunks were produced

### Requirement: Semantic chunker generates embeddings

The chunker SHALL invoke the Tachyon inference host to embed each semantic chunk.

#### Scenario: Chunk is ready for vectorization

- **WHEN** a semantic chunk is produced
- **THEN** the chunker calls `tachyon:inference` with the configured embedding model

### Requirement: Semantic chunker persists vector records

The chunker SHALL persist each vector with text and source metadata in Tachyon's vector store.

#### Scenario: Embedding is generated

- **WHEN** an embedding is returned for a chunk
- **THEN** the chunker stores the vector, chunk text, heading path, source path, and source SHA-256 through `tachyon:store/vector`
