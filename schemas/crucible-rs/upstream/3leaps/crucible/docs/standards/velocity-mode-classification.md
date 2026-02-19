---
title: "Velocity Mode Classification"
description: "Data processing velocity pattern classification standard"
category: "standards"
status: "stable"
version: "1.0.0"
lastUpdated: "2026-01-22"
maintainer: "3leaps-core"
reviewers: ["platform", "data-engineering"]
approvers: ["3leapsdave"]
tags: ["classification", "velocity", "streaming", "batch", "data-engineering"]
content_license: "CC0"
relatedDocs:
  - "docs/standards/volatility-classification.md"
  - "docs/standards/volume-tier-classification.md"
  - "config/classifiers/dimensions/velocity-mode.dimension.json"
audience: "all"
---

# Velocity Mode Classification

This standard defines velocity mode classifications for data processing across all 3leaps ecosystems. It provides a consistent framework for:

- **Pipeline Topology** - Choosing batch vs. streaming architecture
- **Infrastructure Requirements** - Selecting appropriate platforms
- **Processing Semantics** - At-least-once, exactly-once, etc.
- **Latency Expectations** - Setting SLAs for data freshness
- **Cost Planning** - Understanding operational cost profiles

---

## Relationship to Volatility

Velocity mode and volatility are complementary:

- **Volatility** describes _how often data changes_ (update cadence)
- **Velocity mode** describes _how data is processed_ (processing pattern)

Common pairings:

| Volatility | Typical Velocity Mode    |
| ---------- | ------------------------ |
| static     | batch                    |
| monthly    | batch                    |
| weekly     | batch                    |
| daily      | batch                    |
| hourly     | micro-batch or batch     |
| streaming  | streaming or micro-batch |

---

## Velocity Modes

### Unknown

**Velocity mode not yet classified; must be classified before pipeline design.**

| Aspect           | Characteristic         |
| ---------------- | ---------------------- |
| **Latency**      | Unknown                |
| **Processing**   | Cannot design pipeline |
| **Data State**   | Unknown                |
| **Semantics**    | Cannot determine       |
| **Cost Profile** | Cannot estimate        |

**Use Cases**: New data sources, requirements gathering phase.

**Operational Notes**: Gate pipeline design and infrastructure provisioning on explicit classification. Analyze source characteristics and latency requirements to determine appropriate mode.

---

### Batch

**Scheduled batch processing; data at rest, periodic execution.**

| Aspect           | Characteristic                       |
| ---------------- | ------------------------------------ |
| **Latency**      | Hours to days                        |
| **Processing**   | Scheduled jobs (cron, Airflow, etc.) |
| **Data State**   | At rest (complete dataset available) |
| **Semantics**    | Full reprocessing possible           |
| **Cost Profile** | Lowest (compute only when running)   |

**Use Cases**: ETL pipelines, nightly aggregations, reporting, data warehouse loads.

**Infrastructure**: Airflow, dbt, Spark batch, serverless functions.

---

### Micro-batch

**Frequent small batches (minutes); near-real-time with batch semantics.**

| Aspect           | Characteristic                         |
| ---------------- | -------------------------------------- |
| **Latency**      | Minutes (1-15 min typical)             |
| **Processing**   | Triggered batches, windowed processing |
| **Data State**   | Recent windows (tumbling/sliding)      |
| **Semantics**    | Windowed exactly-once achievable       |
| **Cost Profile** | Moderate (frequent compute cycles)     |

**Use Cases**: Dashboard metrics, near-real-time analytics, alerting with delay tolerance.

**Infrastructure**: Spark Structured Streaming, Flink (batch mode), scheduled lambdas.

---

### Streaming

**Event-driven continuous processing; data in motion, sub-second latency.**

| Aspect           | Characteristic                       |
| ---------------- | ------------------------------------ |
| **Latency**      | Sub-second to seconds                |
| **Processing**   | Continuous (always-on)               |
| **Data State**   | In motion (event at a time)          |
| **Semantics**    | Exactly-once requires careful design |
| **Cost Profile** | Highest (always-on compute)          |

**Use Cases**: Real-time fraud detection, live dashboards, event sourcing, IoT processing.

**Infrastructure**: Kafka Streams, Flink, Kinesis, Pulsar.

**Considerations**:

- Requires streaming infrastructure (message brokers)
- Complex exactly-once semantics
- Backpressure and scaling considerations
- Higher operational complexity

---

### Hybrid

**Combined batch and streaming; lambda/kappa architecture patterns.**

| Aspect           | Characteristic                     |
| ---------------- | ---------------------------------- |
| **Latency**      | Mixed (real-time + historical)     |
| **Processing**   | Parallel batch and streaming paths |
| **Data State**   | Both at rest and in motion         |
| **Semantics**    | Varies by path                     |
| **Cost Profile** | High (maintaining both systems)    |

**Use Cases**: Analytics platforms needing both historical and real-time, ML feature stores.

**Architecture Patterns**:

**Lambda Architecture**:

```
Events → [Streaming Layer] → Real-time Views
      ↘ [Batch Layer] → Batch Views
                     → [Serving Layer] → Query
```

**Kappa Architecture**:

```
Events → [Streaming Layer] → Views
      → [Reprocessing] (replay from log)
```

---

## Decision Guide

```
How quickly must data be processed after arrival?

├── Hours/days acceptable → batch
├── Minutes acceptable → micro-batch
├── Seconds/sub-second required → streaming
└── Need both real-time and historical → hybrid
```

### Cost-Latency Tradeoff

| Velocity Mode | Latency    | Cost     | Complexity |
| ------------- | ---------- | -------- | ---------- |
| batch         | Hours-days | Lowest   | Simplest   |
| micro-batch   | Minutes    | Moderate | Low        |
| streaming     | Sub-second | High     | High       |
| hybrid        | Mixed      | Highest  | Highest    |

**Guidance**: Start with batch unless latency requirements demand otherwise. Micro-batch is often a good compromise.

---

## Processing Semantics

| Velocity Mode   | At-least-once | Exactly-once | At-most-once |
| --------------- | ------------- | ------------ | ------------ |
| **batch**       | Easy          | Easy         | Easy         |
| **micro-batch** | Easy          | Achievable   | Easy         |
| **streaming**   | Default       | Complex      | Possible     |
| **hybrid**      | By path       | By path      | By path      |

---

## Combining with Other Dimensions

| Combination                                      | Implication                               |
| ------------------------------------------------ | ----------------------------------------- |
| `velocity: streaming` + `volume: large`          | Requires streaming at scale (Kafka+Flink) |
| `velocity: batch` + `volatility: streaming`      | Mismatch—reconsider architecture          |
| `velocity: micro-batch` + `retention: transient` | Good for dashboards with short history    |

---

## Machine-Readable Definition

- **Dimension Config**: `config/classifiers/dimensions/velocity-mode.dimension.json`
- **Schema**: `schemas/classifiers/v0/dimension-definition.schema.json`

---

## Attribution

This standard is the canonical reference for velocity mode classification across 3leaps ecosystems. Downstream consumers should reference or vendor this standard rather than maintaining independent copies.

**Review Cycle**: Semiannual with platform and data engineering teams.
