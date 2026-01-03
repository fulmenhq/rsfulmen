---
id: "ADR-0008"
title: "Helper Library Instrumentation Patterns"
status: "accepted"
date: "2025-10-24"
last_updated: "2025-10-24"
deciders:
  - "@fulmen-ea-steward"
  - "@schema-cartographer"
scope: "Ecosystem"
tags:
  - "telemetry"
  - "metrics"
  - "standards"
related_adrs:
  - "ADR-0003"
  - "ADR-0007"
adoption:
  gofulmen: "planned"
  pyfulmen: "planned"
  tsfulmen: "planned"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-0008: Helper Library Instrumentation Patterns

## Status

**Current Status**: Accepted

## Context

Fulmen helper libraries (gofulmen, pyfulmen, tsfulmen) expose shared building blocks—config loaders, schema
validators, hashing utilities, catalog lookups, Docscribe processors—that must emit consistent telemetry. Prior
standards cover taxonomy alignment (metrics naming) and histogram bucket defaults (ADR-0007), but we lacked
documented guidance for **when** to apply histograms versus cheaper counters. As new performance-sensitive
modules (FulHash, Foundry similarity v2) scale to tens of thousands of calls per CLI run, library teams requested
clear criteria for selecting instrumentation strategies that balance observability with CPU overhead and timing
side-channel risk.

Without an ecosystem-level decision, each language implementation risked choosing different patterns, leading to
drift across helper libraries, divergent metrics payloads, and inconsistent operator expectations. PyFulmen’s
FulHash work surfaced the need to codify three usage patterns so all foundations stay aligned.

## Decision

Standardise three instrumentation patterns for helper libraries. Each module chooses the pattern that matches its
workload characteristics; libraries MAY document additional rationale via local ADRs but MUST adhere to the shared
patterns.

1. **Standard Pattern (Histogram + Counter)**
   - Apply to moderate-frequency operations where latency visibility matters: file/config I/O, network sync, schema
     validation.
   - Implementation: measure elapsed time with the default `_ms` histogram buckets, increment success and error
     counters, wrap via `defer`/`try/finally`.
   - Outcome metrics: `<operation>_ms` histogram, `<operation>_total`, `<operation>_error_total`.

2. **Performance-Sensitive Pattern (Counter Only)**
   - Apply to hot loops invoked thousands of times per run: hashing, in-memory catalog lookups, text normalization,
     similarity scoring.
   - Implementation: increment success/error counters only; omit histogram timing to avoid ~50–100 ns overhead per
     call (measured in Go/Python/TypeScript).
   - Outcome metrics: `<operation>_total`, `<operation>_error_total`, optional “miss” counters where helpful.

3. **Audit & Compliance Pattern (Counter + Audit Event)**
   - Apply to security- or policy-sensitive actions: credential handling, authorization checks, configuration
     mutations, cache invalidation triggered by privileged commands.
   - Implementation: increment attempt/success/failure counters and emit a structured audit log entry (per
     progressive logging profile, ADR-0003). Avoid histograms when timing data could reveal sensitive behaviour;
     instead, optionally expose a gauge for active sessions/state.
   - Outcome metrics: `<operation>_attempt_total`, `<operation>_success_total`, `<operation>_failure_total`,
     optional gauges; companion audit log event with redacted context.

Helper library standards (Telemetry & Metrics module) now reference these patterns, providing module-specific
recommendations. Future modules must explicitly map to one of the three patterns during design reviews.

## Rationale

- **Consistency**: Provides a common vocabulary for instrumentation design reviews across languages.
- **Performance Awareness**: Codifies when to avoid histograms, preventing regressions in hot paths like hashing.
- **Security Compliance**: Ensures sensitive operations pair metrics with audit events without leaking timing
  information.
- **Traceability**: Aligns with taxonomy enforcement and logging standards, simplifying downstream dashboards and
  incident investigations.
- **Scalability**: Reduces ad-hoc decisions and documentation churn as new helper modules are added.

## Alternatives Considered

### Alternative 1: Single Universal Instrumentation Pattern

**Description**: Mandate histogram + counter instrumentation for all operations.

**Pros**:

- Simplifies guidance and implementation.
- Provides latency data everywhere.

**Cons**:

- Adds measurable overhead to hot loops (50–100 ns per invocation in benchmarks).
- Increases risk of side-channel leaks for security-sensitive flows.
- Leads to resistance/ad-hoc overrides by performance-focused teams.

**Decision**: Rejected – Overhead and security drawbacks outweigh simplicity benefits.

### Alternative 2: Repository-Level Autonomy

**Description**: Let each helper library define instrumentation via local ADRs without a shared standard.

**Pros**:

- Greater flexibility per language/runtime.
- Faster iteration for local teams.

**Cons**:

- High risk of divergence; consumers receive different metrics per language.
- Duplicated documentation and review effort.
- Difficult to maintain shared dashboards and alerting rules.

**Decision**: Rejected – Ecosystem consistency is a core Fulmen principle.

## Consequences

### Positive

- ✅ Shared patterns streamline reviews and reduce back-and-forth during module design.
- ✅ Performance-sensitive modules preserve CPU budgets without sacrificing visibility into operation counts.
- ✅ Security/audit workflows gain explicit guidance for pairing metrics with audit logs.

### Negative

- ⚠️ Requires authors to justify pattern selection during design reviews, adding upfront documentation work.
- ⚠️ Teams must maintain per-module adoption notes until libraries align with the new guidance.

### Neutral

- ℹ️ Existing modules may remain on their current instrumentation approach until touched; migrations will occur
  opportunistically.
- ℹ️ Additional patterns may be introduced later if new workloads emerge (e.g., long-running batch jobs).

## Implementation

1. Update `docs/standards/library/modules/telemetry-metrics.md` with the three patterns and module-specific mapping
   (completed).
2. Communicate the ADR to gofulmen, pyfulmen, tsfulmen maintainers; ensure future PR templates reference pattern
   selection.
3. During FulHash and Foundry similarity updates, document the chosen pattern in release notes and local ADRs.
4. Extend telemetry test fixtures where needed to cover counter-only vs histogram flows.

### Implementation Status

| Library  | Status         | Notes                                          | PR/Issue |
| -------- | -------------- | ---------------------------------------------- | -------- |
| gofulmen | planned        | Align FulHash + Foundry modules with patterns  | N/A      |
| pyfulmen | planned        | FulHash counter-only instrumentation underway  | N/A      |
| tsfulmen | planned        | Will adopt during similarity v2 implementation | N/A      |
| rsfulmen | not-applicable | No active Rust foundation yet                  | N/A      |
| csfulmen | not-applicable | C# foundation not yet onboarded                | N/A      |

## Cross-Language Coordination

- Embed pattern selection in helper library review checklists and design briefs.
- Schema Cartographer to verify future Crucible standards reference the appropriate pattern.
- EA Steward to monitor upcoming ADRs for new modules to ensure they map correctly and extend this ADR if a new
  pattern is justified.
