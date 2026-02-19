---
title: "Volume Tier Classification"
description: "Data volume classification standard for scale planning"
category: "standards"
status: "stable"
version: "1.0.0"
lastUpdated: "2026-01-22"
maintainer: "3leaps-core"
reviewers: ["platform", "data-engineering"]
approvers: ["3leapsdave"]
tags: ["classification", "volume", "scale", "data-engineering", "partitioning"]
content_license: "CC0"
relatedDocs:
  - "docs/standards/velocity-mode-classification.md"
  - "config/classifiers/dimensions/volume-tier.dimension.json"
audience: "all"
---

# Volume Tier Classification

This standard defines volume tier levels for data across all 3leaps ecosystems. It provides a consistent framework for:

- **Scale Planning** - Estimating infrastructure requirements
- **Partitioning Strategies** - Optimal data organization
- **File Sizing** - Appropriate chunk sizes for processing
- **Storage Selection** - Choosing storage systems and formats
- **Benchmark Profiles** - Performance testing at appropriate scale

Volume tier is an **ordinal** dimension—higher values indicate larger scale.

---

## Volume Tiers

| Tier        | Row Count | Typical Size | Processing Model           |
| ----------- | --------- | ------------ | -------------------------- |
| **unknown** | Unknown   | Unknown      | Cannot provision           |
| **tiny**    | ≤100K     | <100 MB      | In-memory, single file     |
| **small**   | ≤10M      | <10 GB       | Single-node                |
| **medium**  | ≤1B       | <1 TB        | Distributed beneficial     |
| **large**   | ≤100B     | <100 TB      | Distributed required       |
| **massive** | >100B     | >100 TB      | Specialized infrastructure |

---

## Tier Details

### Unknown

**Volume not yet classified; must be classified before infrastructure provisioning.**

| Aspect             | Guidance                                  |
| ------------------ | ----------------------------------------- |
| **Processing**     | Unknown; cannot provision infrastructure  |
| **Storage Format** | Staging only                              |
| **Partitioning**   | Cannot determine                          |
| **Infrastructure** | Quarantine/staging environment            |
| **Use Cases**      | New data feeds, imports pending profiling |

**Operational Notes**: Gate infrastructure provisioning decisions on explicit classification. Profile data to determine appropriate tier before production deployment.

---

### Tiny (≤100K rows)

**Very small datasets; single-file, in-memory processing.**

| Aspect             | Guidance                                  |
| ------------------ | ----------------------------------------- |
| **Processing**     | In-memory (pandas, DuckDB, etc.)          |
| **Storage Format** | CSV, JSON, single Parquet file            |
| **Partitioning**   | None needed                               |
| **Infrastructure** | Local machine, small container            |
| **Use Cases**      | Test fixtures, config data, lookup tables |

---

### Small (≤10M rows)

**Small datasets; single-node processing, moderate file sizes.**

| Aspect             | Guidance                                      |
| ------------------ | --------------------------------------------- |
| **Processing**     | Single-node (laptop, small VM)                |
| **Storage Format** | Parquet, CSV with compression                 |
| **Partitioning**   | Optional (by date if time-series)             |
| **Infrastructure** | Standard compute, local SSD                   |
| **Use Cases**      | Product catalogs, user tables, reference data |

---

### Medium (≤1B rows)

**Medium datasets; partitioned storage, distributed processing beneficial.**

| Aspect             | Guidance                                   |
| ------------------ | ------------------------------------------ |
| **Processing**     | Distributed beneficial (Spark, Dask)       |
| **Storage Format** | Columnar (Parquet, ORC) required           |
| **Partitioning**   | Required (date, key columns)               |
| **Infrastructure** | Cloud data warehouse, distributed compute  |
| **Use Cases**      | Transaction history, event logs, analytics |

**Optimization Tips**:

- Partition by date for time-series data
- Use predicate pushdown for queries
- Consider data lake with metadata layer

---

### Large (≤100B rows)

**Large datasets; distributed processing required, columnar formats.**

| Aspect             | Guidance                                 |
| ------------------ | ---------------------------------------- |
| **Processing**     | Distributed required (Spark, Presto)     |
| **Storage Format** | Columnar with compression (Parquet+Zstd) |
| **Partitioning**   | Multi-level (date + key)                 |
| **Infrastructure** | Data lake, distributed compute clusters  |
| **Use Cases**      | Telemetry, clickstream, IoT sensors      |

**Optimization Tips**:

- Aggressive partitioning and clustering
- Z-ordering or data skipping indexes
- Consider separate hot/warm/cold storage

---

### Massive (>100B rows)

**Massive datasets; specialized infrastructure, aggressive partitioning.**

| Aspect             | Guidance                                         |
| ------------------ | ------------------------------------------------ |
| **Processing**     | Specialized systems (BigQuery, Redshift, custom) |
| **Storage Format** | Native formats, custom codecs                    |
| **Partitioning**   | Heavy (multi-dimension, sharding)                |
| **Infrastructure** | Enterprise data platforms, dedicated clusters    |
| **Use Cases**      | Global clickstream, genomics, simulation         |

**Optimization Tips**:

- Work with platform specialists
- Consider materialized views/aggregates
- Pre-compute common queries
- Evaluate specialized databases

---

## Decision Guide

```
How many rows in your dataset?

├── Thousands (≤100K) → tiny
├── Millions (≤10M) → small
├── Hundreds of millions (≤1B) → medium
├── Tens of billions (≤100B) → large
└── Hundreds of billions+ → massive
```

---

## Infrastructure Recommendations

| Volume Tier | Storage System             | Compute                | Format             |
| ----------- | -------------------------- | ---------------------- | ------------------ |
| **tiny**    | Local FS, S3 single file   | Local, small container | CSV, JSON          |
| **small**   | S3/GCS, local SSD          | Single VM, serverless  | Parquet            |
| **medium**  | Data lake (Delta, Iceberg) | Spark, serverless SQL  | Parquet + metadata |
| **large**   | Data lake, warehouse       | Spark cluster, Presto  | Parquet + Zstd     |
| **massive** | Enterprise DW, BigQuery    | Dedicated clusters     | Native formats     |

---

## Combining with Other Dimensions

| Combination                             | Implication                                |
| --------------------------------------- | ------------------------------------------ |
| `volume: large` + `velocity: streaming` | Requires streaming infrastructure at scale |
| `volume: tiny` + `sensitivity: 4`       | Small but needs secure handling            |
| `volume: massive` + `retention: long`   | Archive storage strategy critical          |

---

## Machine-Readable Definition

- **Dimension Config**: `config/classifiers/dimensions/volume-tier.dimension.json`
- **Schema**: `schemas/classifiers/v0/dimension-definition.schema.json`

---

## Attribution

This standard is the canonical reference for volume tier classification across 3leaps ecosystems. Downstream consumers should reference or vendor this standard rather than maintaining independent copies.

**Review Cycle**: Semiannual with platform and data engineering teams.
