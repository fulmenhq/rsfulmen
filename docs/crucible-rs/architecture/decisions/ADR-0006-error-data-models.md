---
id: "ADR-0006"
title: "Fulmen Errors as Schema-Backed Data Models"
status: "accepted"
date: "2025-10-23"
last_updated: "2025-10-23"
deciders:
  - "@fulmen-ea-steward"
scope: "Ecosystem"
supersedes: []
superseded_by: ""
deprecation_date: ""
tags:
  - "errors"
  - "telemetry"
related_adrs:
  - "ADR-0003"
adoption:
  gofulmen: "planned"
  pyfulmen: "implemented"
  tsfulmen: "planned"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-0006: Fulmen Errors as Schema-Backed Data Models

## Status

**Current Status**: Accepted

## Context

The error-handling-propagation module extends the Pathfinder error envelope with telemetry metadata. PyFulmen
(v0.1.6) provided the first concrete implementation and demonstrated that modelling errors as Pydantic data models
made schema validation, JSON serialisation, and cross-language parity straightforward. Without guidance, future
implementations (Go/TypeScript) could drift into language-native error hierarchies, making it harder to share
fixtures, validate schemas, and serialise errors consistently.

## Decision

Fulmen helper libraries MUST represent the canonical error contract as a schema-backed data model (struct,
interface, record, class) that matches `schemas/error-handling/v1.0.0/error-response.schema.json`. Language-native
exception mechanisms MAY wrap the data model, but they MUST NOT replace it. The data model includes Pathfinder
base fields plus telemetry extensions (`severity`, `correlation_id`, `trace_id`, `exit_code`, `context`, `original`).

Libraries MUST provide helper functions to:

- Construct/wrap errors using the shared data shape
- Serialise to JSON (ensuring schema compliance)
- Validate payloads against the schema
- Exit/log using the structured payload

## Rationale

- **Cross-language consistency**: Go and TypeScript do not share Python’s exception semantics; a data-first contract
  keeps APIs aligned.
- **Schema validation**: Pydantic/struct tags/interfaces map cleanly onto JSON Schema. Exceptions would require
  duplicating field extraction logic.
- **Testing & tooling**: Data models are easy to instantiate, inspect, and serialise in fixtures and automated tests.
- **Extensibility**: Wrappers (e.g., raising exceptions) can embed the data model without breaking consumers.

## Alternatives Considered

### Alternative 1: Language-Specific Exceptions

**Description**: Each language exposes native exception types (Python `Exception`, Go `error`, TS `Error`).

**Pros**:

- Feels idiomatic per language
- Automatic stack traces in languages with exception support

**Cons**:

- Hard to serialise consistently
- Difficult to validate against shared schema
- Go/TypeScript would need bespoke wrappers, causing drift

**Decision**: Rejected – sacrifices schema compliance and parity.

### Alternative 2: Hybrid (Data Model + Exception Subclass)

**Description**: Provide both a data model and a companion exception type per language.

**Pros**:

- Allows raising exceptions while preserving data
- Familiar API for consumers

**Cons**:

- Larger surface area to maintain
- Risk consumers depend on exceptions and ignore the data model

**Decision**: Deferred – wrappers are allowed but not standardised yet. Libraries may add them locally.

## Consequences

### Positive

- ✅ Consistent error payloads across Go/Python/TypeScript
- ✅ Simplified schema validation and fixture reuse
- ✅ Easier future enhancements (additional telemetry fields) without breaking exceptions

### Negative

- ⚠️ Developers must wrap/raise errors manually when idiomatic exceptions are required
- ⚠️ Additional documentation required to explain separation between data model and exception behaviour

### Neutral

- ℹ️ Allows future auto-generation of client SDKs based on the schema

## Implementation

- Update `docs/standards/library/modules/error-handling-propagation.md` to reference this ADR and list the shared
  fields.
- Ensure language implementations expose constructors/wrappers that produce the canonical data model.
- Provide fixtures in each language repository that serialise the data model and validate against the schema.

### Implementation Status

| Library  | Status         | Notes                                 | PR/Issue |
| -------- | -------------- | ------------------------------------- | -------- |
| gofulmen | planned        | To implement after FulCache/telemetry | TBA      |
| pyfulmen | implemented    | Implemented in v0.1.6 (`FulmenError`) | TBA      |
| tsfulmen | planned        | Pending module implementation         | TBA      |
| rsfulmen | not-applicable | Language not yet onboarded            | N/A      |
| csfulmen | not-applicable | Language not yet onboarded            | N/A      |

## Cross-Language Coordination

- Share fixtures (`tests/fixtures/errors/`) across repositories.
- Align helper function names (`wrap`, `validate`, `exitWithError`) to reduce documentation divergence.
- Coordinate validation tooling (goneat assess, AJV, gojsonschema) to ensure schema compliance.

## References

- [PyFulmen ADR-0008: Error Handling as Data Models](../../../pyfulmen/docs/development/adr/ADR-0008-error-handling-as-data-models.md)
- [Error Handling Schema](../../../schemas/error-handling/v1.0.0/error-response.schema.json)
- [Pathfinder Error Schema](../../../schemas/pathfinder/v1.0.0/error-response.schema.json)

## Revision History

| Date       | Status Change | Summary                            | Updated By         |
| ---------- | ------------- | ---------------------------------- | ------------------ |
| 2025-10-23 | → accepted    | Promoted PyFulmen ADR to ecosystem | @fulmen-ea-steward |

---

**Schema Conformance**: This ADR conforms to [ADR Frontmatter Schema v1.0.0](https://schemas.fulmenhq.dev/standards/adr-frontmatter-v1.0.0.json)
