# Architecture Decision Records (ADRs)

This directory contains **Tier 1 (Ecosystem)** Architecture Decision Records that affect the entire Fulmen ecosystem. These ADRs are the authoritative source for cross-project architectural decisions.

## ADR Index

### Versioning & Repository Structure

#### [ADR-0010: Semantic Versioning Adoption](ADR-0010-semantic-versioning-adoption.md)

**Status**: Accepted | **Date**: 2025-10-29

Crucible adopted Semantic Versioning (SemVer) for Go module compatibility, replacing Calendar Versioning (CalVer).

**Impact**: Version format changed from `YYYY.0M.MICRO` to `MAJOR.MINOR.PATCH`. Affects all consumers using `go get`.

**Key Decisions**:

- SemVer 0.x phase during stabilization
- Retroactive tagging of CalVer releases (v2025.10.1-v2025.10.5 → v0.1.0-v0.1.4)
- CalVer tags preserved for historical record

**Related**: [ADR-0009](#adr-0009-go-module-root-relocation)

---

#### [ADR-0009: Go Module Root Relocation](ADR-0009-go-module-root-relocation.md)

**Status**: Accepted | **Date**: 2025-10-28

Moved Go module from `lang/go/` to repository root, enabling standard `go get` installation and direct SSOT embedding via `//go:embed`.

**Impact**: Breaking change requiring dependency updates. Improved Go developer experience significantly.

**Key Decisions**:

- Asymmetric structure: Go at root, Python/TypeScript in `lang/`
- Direct embedding vs sync for Go
- Standard Go module conventions

**Related**: [Pseudo-Monorepo](../pseudo-monorepo.md), [ADR-0010](#adr-0010-semantic-versioning-adoption)

---

### Telemetry & Observability

#### [ADR-0008: Helper Library Instrumentation Patterns](ADR-0008-helper-library-instrumentation-patterns.md)

**Status**: Accepted | **Date**: 2025-10-25

Standard patterns for exposing telemetry (metrics, traces, logs) from helper libraries without forcing specific backends.

**Impact**: All Fulmen helper libraries follow consistent instrumentation patterns.

**Key Decisions**:

- Provider-agnostic instrumentation interfaces
- Optional telemetry (zero-cost when disabled)
- Histogram bucket standardization (ADR-0007)
- Middleware patterns for HTTP/gRPC

**Related**: [ADR-0007](#adr-0007-telemetry-default-histogram-buckets), [Telemetry Metrics Module](../../standards/library/modules/telemetry-metrics.md)

---

#### [ADR-0007: Telemetry Default Histogram Buckets](ADR-0007-telemetry-default-histogram-buckets.md)

**Status**: Accepted | **Date**: 2025-10-23

Standardized histogram bucket definitions for latency and size metrics across all Fulmen services.

**Impact**: Consistent histogram granularity enabling cross-service comparison.

**Key Decisions**:

- Latency buckets: exponential 1ms to 30s (16 buckets)
- Size buckets: powers of 2 from 1KB to 16MB (16 buckets)
- Standard buckets in Crucible config

**Related**: [ADR-0008](#adr-0008-helper-library-instrumentation-patterns)

---

### Data Modeling & Configuration

#### [ADR-0006: Error Data Models](ADR-0006-error-data-models.md)

**Status**: Accepted | **Date**: 2025-10-23

Standardized error data structures and propagation patterns for APIs and internal services.

**Impact**: All Fulmen services use consistent error formats.

**Key Decisions**:

- Structured error response format (code, message, details, request_id)
- HTTP status code mapping
- Error detail schemas for validation errors

**Related**: [Error Handling Module](../../standards/library/modules/error-handling-propagation.md)

---

#### [ADR-0005: CamelCase to Language Conventions](ADR-0005-camelcase-to-language-conventions.md)

**Status**: Accepted | **Date**: 2025-10-15

Schema normalization contract: Crucible schemas use camelCase, language wrappers transform to idiomatic conventions.

**Impact**: Schema definitions remain language-neutral while code feels native.

**Key Decisions**:

- Schemas: camelCase (JSON convention)
- Go: PascalCase for exports, camelCase for internal
- Python: snake_case
- TypeScript: camelCase

**Related**: [Schema Normalization](../../standards/schema-normalization.md), [Library Ecosystem](../library-ecosystem.md)

---

#### [ADR-0004: Schema-Driven Config Hydration](ADR-0004-schema-driven-config-hydration.md)

**Status**: Accepted | **Date**: 2025-10-15

Type-safe configuration hydration pattern using JSON schemas to validate and transform config files into strongly-typed objects.

**Impact**: All configuration in Fulmen libraries is schema-validated.

**Key Decisions**:

- JSON Schema validation before hydration
- Type generation from schemas
- Fail-fast on invalid config

**Related**: [Three-Layer Config Module](../../standards/library/modules/enterprise-three-layer-config.md)

---

#### [ADR-0003: Progressive Logging Profiles](ADR-0003-progressive-logging-profiles.md)

**Status**: Accepted | **Date**: 2025-10-15

Tiered logging profiles (silent, terse, normal, verbose, debug) enabling appropriate verbosity for different contexts.

**Impact**: All Fulmen libraries support progressive logging.

**Key Decisions**:

- Five standard profiles
- Profile selection via environment or config
- Middleware logging respects profiles

**Related**: [Logging Standard](../../standards/observability/logging.md)

---

### Catalog & Data Management

#### [ADR-0002: Triple Index Catalog Strategy](ADR-0002-triple-index-catalog-strategy.md)

**Status**: Accepted | **Date**: 2025-10-15

Pattern for maintaining catalogs with primary, secondary, and tertiary indexes for efficient lookups.

**Impact**: All Foundry catalogs (country codes, HTTP statuses, MIME types) use this pattern.

**Key Decisions**:

- Primary index: natural key (e.g., country code alpha-2)
- Secondary index: alternative key (e.g., alpha-3, numeric)
- Tertiary index: common lookups (e.g., by region)

**Related**: [Foundry Catalog Standard](../../standards/library/foundry/README.md)

---

### Governance

#### [ADR-0001: Two-Tier ADR System](ADR-0001-two-tier-adr-system.md)

**Status**: Accepted | **Date**: 2025-10-15

Establishes two ADR tiers: Tier 1 (ecosystem-wide, in Crucible) and Tier 2 (library-specific, in individual repos).

**Impact**: Clear distinction between ecosystem vs library decisions.

**Key Decisions**:

- Tier 1 (this directory): Synced to all libraries
- Tier 2 (`docs/development/adr/` in libraries): Not synced from Crucible
- Tier 1 ADRs require broader stakeholder approval

**Related**: This README

---

## ADR States

ADRs can have the following statuses:

- **Proposed**: Under discussion, not yet approved
- **Accepted**: Approved and implemented
- **Superseded**: Replaced by a newer ADR (reference successor)
- **Deprecated**: No longer recommended but not formally replaced
- **Rejected**: Proposed but declined

## Creating a New ADR

1. **Copy the template**: Use [template.md](template.md) as starting point
2. **Number sequentially**: Next available number (currently ADR-0011)
3. **Fill out sections**: Problem, decision, consequences, alternatives
4. **Open discussion**: https://github.com/fulmenhq/crucible/discussions
5. **Submit PR**: Include ADR in `docs/architecture/decisions/`
6. **Get approval**: Requires review by @3leapsdave and ecosystem maintainers

## ADR Guidelines

### When to Create an ADR

**Do create an ADR for**:

- Architectural decisions affecting multiple Fulmen projects
- Changes to core patterns (config, logging, error handling)
- New ecosystem-wide standards or conventions
- Significant dependency choices
- Breaking changes to APIs or data models

**Don't create an ADR for**:

- Library-specific implementation details (use Tier 2 in that library)
- Routine bug fixes or performance improvements
- Documentation-only changes
- Temporary workarounds

### ADR Quality

**Good ADRs**:

- Clearly describe the problem/context
- List alternatives considered with pros/cons
- Document the decision and its consequences
- Are concise (2-4 pages typically)
- Link to related ADRs and documentation

**Avoid**:

- Explaining implementation details (save for code/docs)
- Revisiting decisions already made (create new ADR if needed)
- Vague problem descriptions
- Missing alternatives analysis

## Related Documentation

### Architecture

- [Fulmen Technical Manifesto](../fulmen-technical-manifesto.md) - Core principles informing ADRs
- [Architecture README](../README.md) - Overview of all architecture docs
- [ADR Template](template.md) - Starting point for new ADRs

### Standards

- [Standards README](../../standards/README.md) - Implementation standards derived from ADRs
- [Repository Versioning](../../standards/repository-versioning.md) - Follows ADR-0010
- [Schema Normalization](../../standards/schema-normalization.md) - Follows ADR-0005

### Governance

- [Repository Operations SOP](../../sop/repository-operations-sop.md) - Operational procedures
- [ops/adr/README.md](../../ops/adr/README.md) - Operational ADRs (local to Crucible)

## Questions?

- **Propose an ADR**: https://github.com/fulmenhq/crucible/discussions
- **Report issues**: https://github.com/fulmenhq/crucible/issues
- **Ask questions**: https://github.com/fulmenhq/crucible/discussions

---

**Note**: Tier 1 ADRs are maintained in the Crucible SSOT and automatically synced to all language wrappers (`lang/*/docs/architecture/decisions/`). This ensures ecosystem-wide consistency.
