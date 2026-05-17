## ADDED Requirements

### Requirement: Semantic deduplicator classifies duplicate divergence alerts
`nebula-semantic-deduplicator` SHALL embed divergence prompts and compare them against recent pending arbitrations and corrections with cosine similarity.

#### Scenario: Duplicate divergence
- **GIVEN** a cached embedding has cosine similarity greater than `0.95`
- **WHEN** a new divergence event is classified
- **THEN** the deduplicator emits `nebula.finops.deduplicated`
- **AND** the request is not forwarded to the economic governor

### Requirement: Semantic deduplicator forwards unique divergence alerts
`nebula-semantic-deduplicator` SHALL forward unique divergence events to the economic governor.

#### Scenario: Unique divergence
- **GIVEN** no cached embedding exceeds the similarity threshold
- **WHEN** a divergence event is classified
- **THEN** the event is forwarded to `nebula.finops.governor.request`

### Requirement: Identical floods trigger one paid arbitration
The deduplicator SHALL prevent a flood of identical failing queries from triggering more than one external Teacher request.

#### Scenario: Ten identical alerts
- **GIVEN** ten identical divergence events arrive for one tenant
- **WHEN** each event is classified after the first is cached as pending
- **THEN** only the first event is forwarded
