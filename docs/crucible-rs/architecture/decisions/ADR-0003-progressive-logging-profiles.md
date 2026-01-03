---
id: "ADR-0003"
title: "Progressive Logging Profiles"
status: "proposal"
date: "2025-10-15"
last_updated: "2025-10-15"
deciders:
  - "@schema-cartographer"
  - "@pipeline-architect"
scope: "Ecosystem"
supersedes: []
tags:
  - "logging"
  - "observability"
  - "standards"
related_adrs:
  - "ADR-0002"
adoption:
  gofulmen: "planned"
  pyfulmen: "implemented"
  tsfulmen: "planned"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-0003: Progressive Logging Profiles

## Status

**Current Status**: Proposal – awaiting ecosystem approval.

## Context

Fulmen helper libraries expose structured logging utilities that applications embed directly. To balance ease of adoption with enterprise observability requirements, Crucible defines a **progressive profile model** (SIMPLE → STRUCTURED → ENTERPRISE → CUSTOM) in `schemas/observability/logging/v1.0.0/logger-config.schema.json`. PyFulmen’s Phase 6 implementation surfaced gaps in cross-language guidance: without a shared contract, Go and TypeScript teams risk diverging on default sinks, middleware chains, and policy enforcement.

This ADR promotes the profile definitions and required behaviours to an ecosystem standard so every foundation delivers identical semantics.

## Decision

All helper libraries MUST implement the following profiles with equivalent behaviour:

| Profile        | Intended Use Cases                                     | Mandatory Capabilities                                                                                           |
| -------------- | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------- |
| **SIMPLE**     | CLI tools, local experimentation                       | Console sink (stderr JSON), static fields support, optional throttling, zero external dependencies               |
| **STRUCTURED** | Long-running daemons, services with audit requirements | Console + optional file sink, correlation/context middleware, throttling, redaction hooks, policy enforcement    |
| **ENTERPRISE** | Production services with compliance/SOC/SIEM needs     | Multi-sink routing (file + external transports), full middleware registry, strict policy enforcement, throttling |
| **CUSTOM**     | Advanced scenarios requiring bespoke pipelines         | Arbitrary sink/middleware combinations defined by callers but still validated against schema + policy            |

### Required Behaviours

1. **Profile-to-Configuration Mapping**
   - Provide constructors/factories that accept a profile enum and hydrate default sinks, middleware, and throttling options.
   - Allow callers to override defaults while preserving schema validity.

2. **Middleware Registry**
   - Expose a registry API that resolves middleware by name (`redact-secrets`, `throttle`, `annotate-trace`, etc.) using strongly typed configs.
   - Ensure order and enabled flags match schema definitions.

3. **Policy Enforcement**
   - Evaluate optional policy files (`logging-policy.yaml`) during hydration.
   - Honour `enforceStrictMode` by failing fast when configuration violates policy (no silent fallbacks).

4. **Schema Validation**
   - Validate both hydrated configuration and emitted events against Crucible schemas (`logger-config`, `log-event`) during tests and optionally at runtime.

5. **Golden Event Fixtures**
   - Maintain shared fixtures per profile so all languages emit structurally identical events for audit scenarios.

6. **Observability Metadata**
   - Always populate correlation/request IDs when available.
   - Ensure severity enums/numeric levels match Crucible definitions.

## Rationale

- **Consistency**: Ensures Fulmen applications can swap helper libraries without behavioural surprises.
- **Compliance**: Codifies policy enforcement and middleware expectations, reducing audit risk.
- **Developer Experience**: Provides clear upgrade path from SIMPLE to ENTERPRISE as needs grow.
- **Quality**: Encourages schema validation and shared fixtures, catching regressions during CI.
- **Reuse**: Aligns with PyFulmen’s progressive logger lessons, preventing repeat pain in other languages.

## Alternatives Considered

### Alternative 1: Ad-Hoc Profile Definitions per Language

**Description**: Allow each language team to define its own profiles and defaults.

**Pros**:

- Maximizes local flexibility
- Lower upfront coordination cost

**Cons**:

- Fragmented behaviour (“INFO” in one language might differ in another)
- Harder to publish unified documentation
- Complicates policy enforcement and tooling

**Decision**: Rejected – undermines SSOT principles.

### Alternative 2: Single “Enterprise” Profile Only

**Description**: Ship only a fully featured configuration, require callers to opt-out manually.

**Pros**:

- Simplifies implementation
- Ensures advanced features always available

**Cons**:

- Overkill for CLI tools and local scripts
- Requires manual configuration pruning for lightweight scenarios
- Higher barrier to entry for new consumers

**Decision**: Rejected – fails ease-of-use goal.

### Alternative 3: Externalize Profiles to Goneat Only

**Description**: Generate language defaults via toolchains (goneat templates) instead of runtime libraries.

**Pros**:

- Centralized configuration
- Potentially less runtime code

**Cons**:

- Loses runtime validation and convenience APIs
- Harder to update existing deployments
- Still requires shared semantics; this ADR codifies them regardless of delivery mechanism

**Decision**: Rejected – tooling cannot replace library-level guarantees.

## Consequences

### Positive

- ✅ Unified documentation and support playbooks across languages.
- ✅ Simplified QA—golden fixtures and schema validation catch drift quickly.
- ✅ Clear upgrade path for customers as observability needs evolve.

### Negative

- ⚠️ Requires ongoing coordination when adding new middleware or sinks (update schemas, registry, fixtures).
- ⚠️ Slightly higher baseline implementation effort for languages that have not yet delivered progressive logging.

### Neutral

- ℹ️ Libraries retain freedom to expose additional profile-specific convenience APIs so long as baseline behaviour matches.
- ℹ️ Policy files remain optional for SIMPLE/STRUCTURED but MUST be honoured when present.

## Implementation

1. **Schema Review** – Keep Crucible logging schemas authoritative; update when introducing new middleware, sinks, or metadata.
2. **Hydration Helpers** – Implement `normalizeLoggerConfig` (TS), `NormalizeLoggerConfig` (Go), `normalize_logger_config` (Python) to map camelCase schema fields into language idioms.
3. **Middleware Registry** – Define registries keyed by middleware name with typed constructors.
4. **Fixtures & Tests** – Add profile fixtures (`simple.yaml`, `structured.yaml`, `enterprise.yaml`) and golden event snapshots. Validate via AJV/gojsonschema/Pydantic.
5. **Docs** – Update language-specific README/API docs describing profiles and expected defaults.
6. **Policy Integration** – Ensure strict mode halts start-up when violations detected. Provide actionable error messages.

### Implementation Status

| Library  | Status         | Notes                                                                | PR/Issue |
| -------- | -------------- | -------------------------------------------------------------------- | -------- |
| gofulmen | planned        | Progressive logger slated for upcoming observability sprint          | N/A      |
| pyfulmen | implemented    | Phase 6 complete with profile hydration, middleware registry, policy | N/A      |
| tsfulmen | planned        | Design in progress; will follow Crucible guidance                    | N/A      |
| rsfulmen | not-applicable | No Rust foundation in active development                             | N/A      |
| csfulmen | not-applicable | .NET foundation out of current scope                                 | N/A      |

## Cross-Language Coordination

- **Fixtures**: Publish canonical fixtures in Crucible (under `examples/logging/`) so each language imports identical configs for testing.
- **Middleware Naming**: Align registry identifiers (`redact-secrets`, `correlation-context`, `throttle`) to simplify documentation.
- **Policy Files**: Share sample policy files in Crucible `examples/logging/policy/` and document search order (.goneat → /etc → /org).
- **Benchmarking**: Track logger initialization latency and per-event overhead to ensure parity.
- **Release Cadence**: Announce new middleware or sink additions via ADR updates and release notes before implementation lands.

## References

- PyFulmen Progressive Logger Phase 6 lessons (`.plans/kaizen/progressive-logger-phase6-lessons.md`).
- Crucible Logging Standard (`docs/standards/observability/logging.md`).
- Logging schemas under `schemas/observability/logging/v1.0.0/`.
- PyFulmen implementation: `pyfulmen/logging/progressive/` modules.
- Goneat tooling roadmap for logging validation.

## Revision History

| Date       | Status Change | Summary                                     | Updated By           |
| ---------- | ------------- | ------------------------------------------- | -------------------- |
| 2025-10-15 | → proposal    | Drafted ecosystem ADR for maintainer review | @schema-cartographer |

---

**Schema Conformance**: This ADR conforms to [ADR Frontmatter Schema v1.0.0](https://schemas.fulmenhq.dev/standards/adr-frontmatter-v1.0.0.json)
