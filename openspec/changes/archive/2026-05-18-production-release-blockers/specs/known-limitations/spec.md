# Known Limitations Delta

## ADDED Requirements

### Requirement: Release limitations must document parser heading heuristics

The repository MUST contain a root `KNOWN_LIMITATIONS.md` file documenting accepted technical debt for the v1.0.0 release, including the aggressive `nebula-doc-parser::looks_like_heading` uppercase-line behavior.

#### Scenario: Users review parser limitations

- **GIVEN** a user opens `KNOWN_LIMITATIONS.md`
- **WHEN** they read the document parser section
- **THEN** they are warned that short all-uppercase lines can be promoted to Markdown headings
