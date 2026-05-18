# Known Limitations

## Document Parser Heading Heuristic

The `nebula-doc-parser` deterministic text parser uses an intentionally aggressive heading heuristic for plain text and PDF-derived text. Any line with more than three alphabetic characters, no more than 96 bytes, and all-uppercase content is treated as a Markdown heading.

This favors stable, dependency-light extraction for early ingestion pipelines, but it can misclassify uppercase acronyms, legal clauses, table labels, or log-like content as headings. Callers that require high-fidelity document structure should treat generated Markdown as a best-effort representation until a richer parser is introduced.
