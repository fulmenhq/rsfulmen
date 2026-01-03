---
id: "ADR-0005"
title: "CamelCase to Language Convention Mapping"
status: "proposal"
date: "2025-10-15"
last_updated: "2025-10-15"
deciders:
  - "@schema-cartographer"
  - "@pipeline-architect"
scope: "Ecosystem"
supersedes: []
tags:
  - "configuration"
  - "naming"
  - "ssot"
related_adrs:
  - "ADR-0004"
adoption:
  gofulmen: "implemented"
  pyfulmen: "implemented"
  tsfulmen: "planned"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-0005: CamelCase to Language Convention Mapping

## Status

**Current Status**: Proposal – pending maintainer approval.

## Context

Crucible schemas and configuration files use **camelCase** keys to remain language-agnostic and align with JSON Schema conventions. Helper libraries, however, follow language-specific casing rules:

- Python prefers `snake_case` attributes (`populate_by_name=True` for Pydantic models).
- Go requires exported struct fields in `PascalCase` with struct tags mapping back to camelCase.
- TypeScript commonly exposes `lowerCamelCase` properties but may prefer `snake_case` for internal maps.

During PyFulmen Phase 6, inconsistent casing caused data to fall through validation, while gofulmen encountered mismatched struct tags. Without a documented, shared approach, new modules risk deviating, producing subtle bugs.

## Decision

Each helper library MUST implement a consistent mapping strategy from camelCase schema keys to language conventions:

1. **Explicit Mapping Layer**
   - Python: Pydantic models define `model_config = ConfigDict(populate_by_name=True, alias_generator=to_camel)` and expose snake_case attributes.
   - Go: Structs use `json:"camelCase"` tags and helper constructors to normalize user-provided maps.
   - TypeScript: Normalizers convert camelCase schema keys to idiomatic property names, while DTOs maintain camelCase for serialization.

2. **Bidirectional Support**
   - Accept both schema aliases and language-native names when deserializing user overrides (e.g., YAML authored by humans).
   - Emit camelCase when writing back to SSOT-aligned formats (JSON/YAML) to stay consistent with Crucible.

3. **Centralized Utility Functions**
   - Provide shared helpers (`to_camel`, `to_snake`) to avoid bespoke conversions scattered across code.
   - Enforce usage via linting/tests where feasible.

4. **Documentation & Templates**
   - Update language coding standards with rules and examples.
   - Bake casing conventions into templates (`docs/architecture/decisions/template.md`, Pydantic base classes, Go struct definitions).

5. **Testing**
   - Add tests ensuring both camelCase and native casing deserialize correctly.
   - Validate serialization round-trips: native struct → JSON/YAML → native struct.

## Rationale

- **Interoperability**: Guarantees that Crucible assets sync cleanly into every language without manual adjustments.
- **Developer Ergonomics**: Developers work with idiomatic casing while still consuming SSOT data.
- **Consistency**: Avoids fragmentation where identical fields have different names across languages.
- **Future-Proofing**: Simplifies adoption of new schemas (only mapping utilities need updates).

## Alternatives Considered

### Alternative 1: Retain CamelCase Everywhere

**Description**: Expose camelCase keys directly in every language (Python classes with camelCase properties, Go struct fields named `CamelCase`).

**Pros**:

- Simplifies parsing logic (no aliasing needed)
- Mirrors raw schema one-to-one

**Cons**:

- Violates language idioms (Python devs expect snake_case)
- Reduces readability and auto-complete ergonomics
- Increases likelihood of mistakes when interacting with other local code

**Decision**: Rejected – developer experience deemed more important.

### Alternative 2: Custom Field Names Per Language

**Description**: Allow each language to rename fields arbitrarily (even diverging from camelCase semantics).

**Pros**:

- Maximum freedom
- Potentially more descriptive names in certain languages

**Cons**:

- Breaks documentation parity
- Hard to maintain cross-language examples
- Increases chance of drift between schemas and runtime

**Decision**: Rejected – conflicts with SSOT goals.

### Alternative 3: Generate Language-Specific Schemas

**Description**: Maintain separate schema copies per language with native casing baked in.

**Pros**:

- Eliminates runtime mapping layer
- Potentially better tooling integration per language

**Cons**:

- Multiplicative maintenance cost (keep schemas in sync)
- Higher risk of divergence and stale documentation
- Complicates sync workflows and release processes

**Decision**: Rejected – SSOT discipline requires a single canonical schema.

## Consequences

### Positive

- ✅ Uniform casing strategy reduces onboarding time and documentation complexity.
- ✅ Shared helpers simplify future schema adoption.
- ✅ Enables consistent serialization/deserialization behaviour across libraries.

### Negative

- ⚠️ Requires ongoing diligence to ensure new structs/classes adopt helper utilities.
- ⚠️ Adds minor runtime cost for alias resolution (negligible in practice).

### Neutral

- ℹ️ Libraries may expose both camelCase and native casing in public APIs if necessary, but must document the behaviour.
- ℹ️ Tooling (e.g., goneat) can leverage helpers to generate language-specific code in the future.

## Implementation

1. **Update Base Models/Structs**
   - Python: Provide `FulmenBaseModel` with alias generator and reuse everywhere.
   - Go: Enforce struct tags and helper functions in shared packages.
   - TypeScript: Export `toCamel`/`toSnake` utilities within shared config module.

2. **Add Tests**
   - Round-trip tests ensuring both camelCase and native casing inputs load properly.
   - Snapshot tests verifying emitted JSON/YAML uses camelCase.

3. **Documentation**
   - Link this ADR from coding standards (`docs/standards/coding/*.md`).
   - Add examples in README/API docs showing casing expectations.

4. **Linting (Future)**
   - Consider adding lint rules or CI checks verifying struct tags / alias generators present on relevant types.

### Implementation Status

| Library  | Status         | Notes                                                             | PR/Issue |
| -------- | -------------- | ----------------------------------------------------------------- | -------- |
| gofulmen | implemented    | Uses struct tags + helper functions across config modules         | N/A      |
| pyfulmen | implemented    | Central `FulmenBaseModel` with alias generator + populate_by_name | N/A      |
| tsfulmen | planned        | Normalization helper to be added alongside config hydration       | N/A      |
| rsfulmen | not-applicable | Rust foundation not active                                        | N/A      |
| csfulmen | not-applicable | .NET foundation out of scope                                      | N/A      |

## Cross-Language Coordination

- Share helper implementations in Crucible snippets so language teams borrow proven patterns.
- Include casing expectations in shared fixtures (e.g., YAML examples) and unit tests.
- Update documentation and ADR references whenever schema naming conventions evolve.
- Coordinate with tooling teams (goneat) to reuse the same conversion utilities for generated code.

## References

- Coding standards updates for Python, Go, and TypeScript (`docs/standards/coding/*.md`).
- PyFulmen ADR-0001 (local) – prior work motivating alias handling.
- Crucible logging/config schemas demonstrating camelCase keys.
- Goneat manifests referencing schema paths.

## Revision History

| Date       | Status Change | Summary                          | Updated By           |
| ---------- | ------------- | -------------------------------- | -------------------- |
| 2025-10-15 | → proposal    | Drafted ecosystem ADR for review | @schema-cartographer |

---

**Schema Conformance**: This ADR conforms to [ADR Frontmatter Schema v1.0.0](https://schemas.fulmenhq.dev/standards/adr-frontmatter-v1.0.0.json)
