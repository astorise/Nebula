# Doc Parser SSRF Protection Delta

## ADDED Requirements

### Requirement: Document parser tunnel endpoints must be Wormhole-only

The document parser MUST validate `tunnel_host` before fetching document bytes. The URL scheme MUST be `http` or `https`, and the host MUST end with `.wormhole.internal`. Invalid endpoints MUST abort ingestion with an `InvalidTunnelEndpoint` error.

#### Scenario: Metadata IP endpoints are rejected

- **GIVEN** a file update event with `tunnel_host` set to `http://169.254.169.254`
- **WHEN** the document parser ingests the event
- **THEN** ingestion aborts with an `InvalidTunnelEndpoint` error before any fetch

#### Scenario: Non-Wormhole internal domains are rejected

- **GIVEN** a file update event with `tunnel_host` set to `https://attacker.internal`
- **WHEN** the document parser validates the endpoint
- **THEN** validation returns `InvalidTunnelEndpoint`

### Requirement: Heading heuristic limitations must be documented in code

The `looks_like_heading` heuristic MUST include rustdocs explaining that short uppercase lines are aggressively promoted to headings.

#### Scenario: Maintainers inspect parser heuristics

- **GIVEN** a maintainer reads the `looks_like_heading` implementation
- **WHEN** they inspect its documentation
- **THEN** the aggressive uppercase-line behavior is called out as a known limitation
