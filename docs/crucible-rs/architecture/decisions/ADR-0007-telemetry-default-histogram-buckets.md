---
id: "ADR-0007"
title: "Default Histogram Buckets for Millisecond Metrics"
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
  - "metrics"
  - "telemetry"
related_adrs:
  - "ADR-0003"
  - "ADR-0006"
adoption:
  gofulmen: "planned"
  pyfulmen: "implemented"
  tsfulmen: "planned"
  rsfulmen: "not-applicable"
  csfulmen: "not-applicable"
---

# ADR-0007: Default Histogram Buckets for Millisecond Metrics

## Status

**Current Status**: Accepted

## Context

The telemetry-metrics module allows helper libraries to emit counter, gauge, and histogram events aligned with the
metrics taxonomy. Histogram accuracy depends on consistent bucket boundaries; PyFulmen (v0.1.6) introduced a
practical default covering 1 ms to 10 s. Without a shared default, each language could publish incompatible
histogram structures, complicating downstream aggregation in Prometheus/OTLP backends.

## Decision

Libraries MUST use the following bucket boundaries (in milliseconds) when creating histograms for metrics whose
names end with `_ms`, unless a caller explicitly overrides the buckets:

```
[1, 5, 10, 50, 100, 500, 1000, 5000, 10000]
```

The buckets are cumulative (`<=` semantics) and map directly onto the OpenTelemetry histogram data model. Helper
APIs SHOULD pick these buckets automatically when the taxonomy unit is `ms`. Overrides remain allowed for niche
cases, but the default ensures consistent dashboards and alert thresholds.

## Rationale

- **Operational consistency**: Aggregating latency metrics across Go/Python/TypeScript relies on identical bucket
  boundaries.
- **Practical coverage**: The range captures fast in-memory operations (≤10 ms), mid-range I/O (10–100 ms), slower
  network calls (100–1 000 ms), and long-running tasks (1–10 s).
- **Simplicity**: A single nine-bucket array keeps implementation lightweight and easily serialised.
- **Proven in practice**: PyFulmen implementation validated the defaults against the existing taxonomy and schema.

## Alternatives Considered

### Alternative 1: Prometheus-Style Buckets (Seconds)

**Description**: Adopt Prometheus defaults (seconds) and convert to milliseconds.

**Pros**:

- Familiar to operators using Prometheus
- Well-tested production default

**Cons**:

- First bucket (5 ms) misses sub-5 ms operations
- Non-intuitive boundaries (2.5 ms, 25 ms, etc.) when converted

**Decision**: Rejected – less intuitive for Fulmen library latencies.

### Alternative 2: Taxonomy-Defined Buckets Per Metric

**Description**: Store bucket arrays alongside each metric entry in the taxonomy.

**Pros**:

- Maximum flexibility per metric
- SSOT enforcement

**Cons**:

- Increases taxonomy maintenance burden
- Requires schema changes and sync overhead for minor tweaks

**Decision**: Deferred – may revisit if we need non-`ms` defaults (bytes, counts). For now, we document defaults in
this ADR and the telemetry standard.

## Consequences

### Positive

- ✅ Consistent histogram structure across languages and tools
- ✅ Immediate adoption path for gofulmen/tsfulmen implementations
- ✅ Eases onboarding of dashboards/alerts in Prometheus, Datadog, etc.

### Negative

- ⚠️ Libraries with ultra-low-latency requirements may need overrides (supported via optional parameters)

### Neutral

- ℹ️ Future taxonomy updates can formalise the defaults without breaking existing code

## Implementation

- Update `docs/standards/library/modules/telemetry-metrics.md` to list the default buckets and reference this ADR.
- Ensure Fulmen telemetry helpers in each language use the defaults when no override is provided.
- Provide fixtures demonstrating histogram serialisation with these buckets (`tests/fixtures/metrics/histogram_ms.json`).

### Implementation Status

| Library  | Status         | Notes                                  | PR/Issue |
| -------- | -------------- | -------------------------------------- | -------- |
| gofulmen | planned        | To adopt during telemetry module build | TBA      |
| pyfulmen | implemented    | Implemented in v0.1.6 defaults         | TBA      |
| tsfulmen | planned        | Pending telemetry implementation       | TBA      |
| rsfulmen | not-applicable | Language not yet onboarded             | N/A      |
| csfulmen | not-applicable | Language not yet onboarded             | N/A      |

## References

- [PyFulmen ADR-0009: Telemetry Default Histogram Buckets](../../../pyfulmen/docs/development/adr/ADR-0009-telemetry-histogram-buckets.md)
- [Telemetry Metrics Standard](../modules/telemetry-metrics.md)
- [Metrics Taxonomy](../../../config/taxonomy/metrics.yaml)

## Revision History

| Date       | Status Change | Summary                              | Updated By         |
| ---------- | ------------- | ------------------------------------ | ------------------ |
| 2025-10-23 | → accepted    | Promoted PyFulmen histogram defaults | @fulmen-ea-steward |

---

**Schema Conformance**: This ADR conforms to [ADR Frontmatter Schema v1.0.0](https://schemas.fulmenhq.dev/standards/adr-frontmatter-v1.0.0.json)
