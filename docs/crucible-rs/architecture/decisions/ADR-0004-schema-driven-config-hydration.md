---
id: "ADR-0004"
title: "Schema-Driven Config Hydration"
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
  - "ssot"
  - "hydration"
related_adrs:
  - "ADR-0003"
adoption:
  gofulmen: "implemented"
  pyfulmen: "implemented"
  tsfulmen: "planned"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-0004: Schema-Driven Config Hydration

## Status

**Current Status**: Proposal – ready for maintainer confirmation.

## Context

Crucible ships configuration defaults (YAML/JSON) for logging, config paths, observability, and future modules. Helper libraries must merge three layers at runtime:

1. **Crucible Defaults** (embedded SSOT snapshot)
2. **User Overrides** (files under `${FULMEN_CONFIG_DIR}`)
3. **Application Overrides** (parameters or BYOC inputs)

During PyFulmen’s progressive logger work, schema/runtime drift caused repeated audit cycles because hydration logic was scattered, type aliases weren’t enforced, and policy enforcement happened late. Similar pain surfaced in gofulmen during config refactors. This ADR standardizes a **single normalization pipeline per library** that converts schema-authored documents (camelCase) into idiomatic language structures while applying validation and policy checks.

## Decision

Each helper library MUST implement a dedicated hydration module that:

1. **Loads and Merges Layers**
   - Reads Crucible defaults from embedded assets.
   - Overlays user-managed config directories (`GetFulmenConfigDir` et al.).
   - Applies programmatic overrides last.

2. **Normalizes Schema Fields**
   - Converts camelCase keys to language conventions (snake_case for Python, exported struct fields for Go, lowerCamelCase for TS).
   - Flattens nested `config` blocks (e.g., `middleware[].config`) into typed parameters.
   - Preserves explicit zero/empty values (no accidental defaulting).

3. **Validates Against Schemas**
   - Validates hydrated configuration using language-appropriate tooling (Pydantic, gojsonschema, AJV).
   - Validates emitted artefacts (e.g., log events) against corresponding schemas when applicable.

4. **Enforces Policy Files**
   - Resolves policy search order (`.goneat/`, `/etc/fulmen/`, `/org/`).
   - Applies allow/deny rules prior to returning configuration.
   - Honors strict mode by raising exceptions on violation.

5. **Exposes Pure APIs**
   - Expose functions like `normalize_logger_config(raw: Mapping)` that are deterministic and side-effect free.
   - Make hydration idempotent to simplify testing.
   - Keep the pure normalizers free of hidden caches, external I/O, or global mutations; perform those operations in orchestration layers so maintainers can reason about behaviour during reviews and audits.

6. **Ships Fixtures & Tests**
   - Provide canonical fixtures (under `tests/fixtures/` or `internal/.../testdata`) for each supported profile/use case.
   - Include golden round-trip tests to detect regressions promptly.

## Rationale

- **Reliability**: Centralizing hydration reduces schema drift and ensures consistent defaults.
- **Testability**: Pure normalization functions enable exhaustive unit tests and shared fixtures.
- **Cross-Language Parity**: With the same pipeline structure, behaviour differences between languages become auditable.
- **Compliance**: Early policy enforcement prevents non-compliant configs from reaching production.
- **DX**: Applications receive validated, idiomatic structures and concise error messages.

## Alternatives Considered

### Alternative 1: Ad-Hoc Field Mapping in Call Sites

**Description**: Let each component parse the sections it needs directly from YAML/JSON.

**Pros**:

- Less upfront code
- Localized knowledge per module

**Cons**:

- Inconsistent casing and default handling
- Harder to validate globally
- Duplicated logic, increased maintenance burden

**Decision**: Rejected – leads to schema/runtime drift (observed in practice).

### Alternative 2: Tooling-Only Transformation (Goneat Generates Code)

**Description**: Use build-time code generation to produce structs/classes from schemas.

**Pros**:

- Automatic casing conversion
- Potentially less manual code

**Cons**:

- Complex tooling story across languages
- Does not cover runtime overlays (user/app overrides)
- Still requires policy enforcement and validation glue

**Decision**: Rejected – future enhancement, but runtime hydrators still required.

### Alternative 3: Accept CamelCase Throughout

**Description**: Expose schema field names directly (camelCase) in every language.

**Pros**:

- Simplifies loading (no mapping layer)
- Avoids alias configuration

**Cons**:

- Breaks language idioms (Python snake_case expectations)
- Painful for autocomplete/type checking
- Confusing for developers reading structs/classes

**Decision**: Rejected – poor developer ergonomics.

## Consequences

### Positive

- ✅ Reduced audit cycles; schema changes surface immediately in shared tests.
- ✅ Predictable config behaviour across libraries; easier documentation and support.
- ✅ Facilitates future automation (linting, fixtures, code generation) built on consistent APIs.

### Negative

- ⚠️ Requires upfront investment to refactor existing code into normalization modules.
- ⚠️ Demands ongoing fixture maintenance whenever schemas evolve.

### Neutral

- ℹ️ Libraries may implement hydration lazily (cache results) provided pure functions exist for testing.
- ℹ️ Applications can still bypass convenience APIs if necessary, but doing so forfeits guarantees.

## Implementation

1. **Create Normalization Module**
   - Python: `pyfulmen.config.normalize_logger_config`.
   - Go: `foundation/config/normalize.go`.
   - TypeScript: `packages/config/src/normalizeLoggerConfig.ts`.

2. **Adopt Shared Fixtures**
   - Add configs to `examples/config/logging/` in Crucible; sync to language repos.
   - Round-trip fixtures through normalizers during CI.

3. **Enforce Policy Early**
   - Implement search order and strict mode handling within normalizers.
   - Provide descriptive errors (include policy rule that failed).

4. **Validate Output**
   - Run schema validation in `make test` (AJV/gojsonschema/Pydantic).
   - Add golden snapshot tests for hydrated configs and log events.

5. **Document Contract**
   - Update language coding standards (already drafted) with expectations.
   - Link to this ADR from README/architecture docs.

### Implementation Status

| Library  | Status         | Notes                                                        | PR/Issue |
| -------- | -------------- | ------------------------------------------------------------ | -------- |
| gofulmen | implemented    | Three-layer config loader + normalization helpers in place   | N/A      |
| pyfulmen | implemented    | Dedicated mapper with schema + policy validation (Phase 6)   | N/A      |
| tsfulmen | planned        | Hydrator planned alongside progressive logger implementation | N/A      |
| rsfulmen | not-applicable | Rust foundation still exploratory                            | N/A      |
| csfulmen | not-applicable | .NET foundation not yet in scope                             | N/A      |

## Cross-Language Coordination

- Maintain shared documentation in Crucible (`docs/standards/coding/*.md`) capturing language-specific casing rules.
  - Python: `populate_by_name=True`, alias generation.
  - Go: struct field tags (`json:"alpha2"`).
  - TypeScript: type guards and Zod schemas.
- Keep fixtures synchronized through `make sync`.
- When schemas change, update normalization tests in all languages before publishing releases.
- Consider adding a lint rule (future) to verify frontmatter for ecosystem ADR compliance references normalization modules.

## References

- PyFulmen KAIZEN memo: Progressive Logger Phase 6 lessons.
- Coding standards updates (`docs/standards/coding/python.md`, `go.md`, `typescript.md`).
- Crucible helper library standard (Testing Expectations section).
- Logging schemas & config defaults under `schemas/observability/logging/v1.0.0/`.

## Revision History

| Date       | Status Change | Summary                          | Updated By           |
| ---------- | ------------- | -------------------------------- | -------------------- |
| 2025-10-15 | → proposal    | Drafted ecosystem ADR for review | @schema-cartographer |

---

**Schema Conformance**: This ADR conforms to [ADR Frontmatter Schema v1.0.0](https://schemas.fulmenhq.dev/standards/adr-frontmatter-v1.0.0.json)
