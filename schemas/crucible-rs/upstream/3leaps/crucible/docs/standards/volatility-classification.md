---
title: "Volatility & Update Cadence Classification"
description: "Standard for classifying data and configuration update frequency"
category: "standards"
status: "stable"
version: "1.0.0"
lastUpdated: "2026-01-22"
maintainer: "3leaps-core"
reviewers: ["platform", "data-engineering"]
approvers: ["3leapsdave"]
tags: ["classification", "volatility", "cadence", "scheduling", "data-lifecycle"]
content_license: "CC0"
relatedDocs:
  - "schemas/classifiers/v0/dimension-definition.schema.json"
  - "config/classifiers/dimensions/volatility.dimension.json"
  - "docs/standards/data-sensitivity-classification.md"
audience: "all"
---

# Volatility & Update Cadence Classification

This standard defines update cadence levels for data and configuration across all 3leaps ecosystems. It provides a consistent framework for:

- **Freshness SLAs** - Setting expectations for data currency
- **Scheduling** - Determining batch job and pipeline frequencies
- **Partitioning** - Informing time-based partitioning strategies
- **Caching** - Setting appropriate TTLs and invalidation policies
- **Resource Planning** - Estimating compute and storage requirements

## Volatility Levels

Volatility is an **ordinal** dimension—higher values indicate more frequent updates.

| Level | Key         | Description             | Typical Use Cases                                       |
| ----- | ----------- | ----------------------- | ------------------------------------------------------- |
| 0     | `unknown`   | Not yet classified      | New data sources pending classification                 |
| 1     | `static`    | No scheduled updates    | Reference data, schemas, standards, one-time snapshots  |
| 2     | `monthly`   | Roughly monthly batches | Financial reports, compliance audits, capacity planning |
| 3     | `weekly`    | Roughly weekly batches  | Product catalogs, pricing updates, aggregated metrics   |
| 4     | `daily`     | Daily batches           | Transaction summaries, daily snapshots, ETL pipelines   |
| 5     | `hourly`    | Sub-daily batches       | Operational metrics, near-real-time dashboards, alerts  |
| 6     | `streaming` | Event-driven continuous | Real-time telemetry, live feeds, event sourcing         |

---

## Level Details

### Unknown (0)

**Volatility not yet classified; must be classified before operational use.**

| Aspect           | Requirement                                                  |
| ---------------- | ------------------------------------------------------------ |
| **Scheduling**   | Prohibited until classified                                  |
| **Caching**      | Conservative defaults only (no long TTL assumptions)         |
| **Partitioning** | Avoid production partition strategy decisions                |
| **Use Cases**    | New data feeds, imported datasets, unprofiled or new sources |

**Operational Notes**: Gate operational use on explicit classification. Do not treat missing or unknown volatility as “static” or “daily” by default.

---

### Static (1)

**No scheduled updates—one-time or ad-hoc changes only.**

- Changes require explicit versioning and release process
- Safe to cache indefinitely (until version changes)
- Examples: JSON schemas, role definitions, reference taxonomies

**Operational Implications**:

- No scheduled refresh jobs needed
- Version-based cache invalidation
- Changes go through PR/review process

---

### Monthly (2)

**Batch updates roughly monthly.**

- Typically aligned with business cycles (month-end close, reporting periods)
- Allow 24-48 hour processing windows
- Examples: Financial statements, compliance reports, capacity forecasts

**Operational Implications**:

- Schedule during low-traffic windows
- Plan for larger batch sizes
- Coordinate with downstream consumers on refresh dates

---

### Weekly (3)

**Batch updates roughly weekly.**

- Common for curated datasets that balance freshness and processing cost
- Examples: Product catalogs, aggregated analytics, weekly digests

**Operational Implications**:

- Typical refresh: weekends or early morning
- Moderate batch sizes
- Weekly SLA monitoring

---

### Daily (4)

**Daily batches—the most common cadence for operational data.**

- Standard for transactional summaries and operational reporting
- Examples: Daily sales, order summaries, log aggregations

**Operational Implications**:

- Nightly batch windows (typically 00:00-06:00)
- Date-partitioned storage recommended
- T+1 data availability expectations

---

### Hourly (5)

**Sub-daily batches—hourly or more frequent.**

- Bridges gap between batch and streaming
- Examples: Operational dashboards, alerting thresholds, rate limit counters

**Operational Implications**:

- Micro-batch processing
- Hour-partitioned or rolling windows
- Higher compute costs than daily
- Consider streaming if approaching minute-level freshness needs

---

### Streaming (6)

**Event-driven continuous updates—sub-minute latency.**

- True real-time processing
- Examples: Live telemetry, event sourcing, real-time fraud detection

**Operational Implications**:

- Requires streaming infrastructure (Kafka, Kinesis, Pulsar)
- Continuous compute costs
- Complex exactly-once semantics
- Backpressure and scaling considerations

---

## Decision Guide

```
How quickly must consumers see new data?

├── "Whenever we release a new version" → static
├── "By the end of the month" → monthly
├── "Within a week" → weekly
├── "Next business day" → daily
├── "Within hours" → hourly
└── "Immediately / real-time" → streaming
```

### Cost-Freshness Tradeoff

| Volatility | Relative Cost | Freshness           | Complexity |
| ---------- | ------------- | ------------------- | ---------- |
| static     | Lowest        | Stale until release | Simplest   |
| monthly    | Low           | Up to 30 days       | Simple     |
| weekly     | Low-Medium    | Up to 7 days        | Simple     |
| daily      | Medium        | Up to 24 hours      | Moderate   |
| hourly     | Medium-High   | Up to 1 hour        | Moderate   |
| streaming  | Highest       | Sub-minute          | Complex    |

**Guidance**: Start with the lowest volatility that meets business requirements. Upgrading to higher frequency is easier than optimizing an over-engineered streaming system.

---

## Combining with Other Dimensions

Volatility works alongside other classifiers:

| Combination                                         | Implication                                           |
| --------------------------------------------------- | ----------------------------------------------------- |
| `sensitivity: 4-personal` + `volatility: streaming` | Real-time PII requires streaming encryption and audit |
| `sensitivity: 0-public` + `volatility: static`      | Cacheable forever, CDN-friendly                       |
| `volatility: daily` + partitioning                  | Use date-based partitions                             |
| `volatility: streaming` + storage                   | Consider append-only / event log storage              |

---

## Machine-Readable Definition

- **Dimension Config**: `config/classifiers/dimensions/volatility.dimension.json`
- **Schema**: `schemas/classifiers/v0/dimension-definition.schema.json`

---

## Attribution

This standard is the canonical reference for volatility classification across 3leaps ecosystems. Downstream consumers should reference or vendor this standard rather than maintaining independent copies.

**Review Cycle**: Semiannual with platform and data engineering teams.
