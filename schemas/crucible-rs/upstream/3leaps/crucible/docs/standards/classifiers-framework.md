---
title: "Classifiers Framework"
description: "How 3leaps defines and uses orthogonal classification dimensions (docs + config + schemas)"
category: "standards"
status: "stable"
version: "1.0.0"
lastUpdated: "2026-01-22"
maintainer: "3leaps-core"
reviewers: ["platform", "security", "data-engineering"]
approvers: ["3leapsdave"]
tags: ["classification", "classifiers", "metadata", "governance", "schemas"]
content_license: "CC0"
relatedDocs:
  - "schemas/classifiers/v0/dimension-definition.schema.json"
  - "docs/standards/data-sensitivity-classification.md"
  - "docs/standards/volatility-classification.md"
  - "docs/standards/access-tier-classification.md"
  - "docs/standards/retention-lifecycle-classification.md"
  - "docs/standards/schema-stability-classification.md"
  - "docs/standards/volume-tier-classification.md"
  - "docs/standards/velocity-mode-classification.md"
  - "docs/operations/upstream-sync-guide.md"
audience: "all"
---

# Classifiers Framework

Crucible classifiers are a lightweight framework for describing data and artifacts using **orthogonal dimensions**. The goal is to make classification:

- **Consistent** across projects and ecosystems
- **Machine-readable** for automation and policy enforcement
- **Reference-friendly** (linkable docs, vendorable config)

This framework is intentionally composable: consumers can adopt a single dimension (e.g., `sensitivity`) or a full set.

---

## What Lives Where

Each dimension is expressed in three forms:

1. **Standard (narrative)**: the human policy and examples (`docs/standards/*.md`)
2. **Dimension definition (machine)**: a canonical list of values and metadata (`config/classifiers/dimensions/*.dimension.json`)
3. **Meta-schema (validation)**: how dimension definitions are structured (`schemas/classifiers/v0/*.schema.json`)

If you vendor anything, vendor **dimension configs + schemas together** so validation stays aligned.

---

## Current Dimension Set

| Dimension Key         | Tier | Narrative Standard                                     | Machine Definition                                                 |
| --------------------- | ---- | ------------------------------------------------------ | ------------------------------------------------------------------ |
| `sensitivity`         | 1    | `docs/standards/data-sensitivity-classification.md`    | `config/classifiers/dimensions/sensitivity.dimension.json`         |
| `volatility`          | 1    | `docs/standards/volatility-classification.md`          | `config/classifiers/dimensions/volatility.dimension.json`          |
| `access-tier`         | 1    | `docs/standards/access-tier-classification.md`         | `config/classifiers/dimensions/access-tier.dimension.json`         |
| `retention-lifecycle` | 1    | `docs/standards/retention-lifecycle-classification.md` | `config/classifiers/dimensions/retention-lifecycle.dimension.json` |
| `schema-stability`    | 1    | `docs/standards/schema-stability-classification.md`    | `config/classifiers/dimensions/schema-stability.dimension.json`    |
| `volume-tier`         | 2    | `docs/standards/volume-tier-classification.md`         | `config/classifiers/dimensions/volume-tier.dimension.json`         |
| `velocity-mode`       | 2    | `docs/standards/velocity-mode-classification.md`       | `config/classifiers/dimensions/velocity-mode.dimension.json`       |

Tier meaning:

- **Tier 1**: universal infrastructure dimensions (applies almost everywhere)
- **Tier 2**: data platform fundamentals (useful for pipelines and systems design)

---

## Minimal Data Model

Crucible does not currently impose a single universal “classification object” schema. A common, portable pattern is:

```json
{
  "classifiers": {
    "sensitivity": "3-proprietary",
    "access-tier": "restricted",
    "retention-lifecycle": "standard",
    "volatility": "daily",
    "volume-tier": "medium",
    "velocity-mode": "batch",
    "schema-stability": "stable"
  }
}
```

Guidance:

- Use **dimension keys** exactly as declared in the dimension definition JSON (`key`).
- Treat classifier values as **opaque identifiers** (compare as strings, don’t parse them).
- Do not rely on numeric ordinals or level numbers in docs; they may shift when new values (like `unknown`) are introduced.
- Prefer **single source of truth** per artifact (don’t duplicate the same classification in multiple places).

---

## UNKNOWN and Missing Values

Classification is safe-by-default—**missing classification is a policy error**.

All dimensions include an explicit `unknown` value:

| Dimension             | Unknown Meaning                                                      |
| --------------------- | -------------------------------------------------------------------- |
| `sensitivity`         | Unclassified; isolate until classified within 24h                    |
| `volatility`          | Not yet classified; must classify before operational use             |
| `access-tier`         | Not yet classified; must classify before sharing                     |
| `retention-lifecycle` | Not yet classified; must classify before storage provisioning        |
| `schema-stability`    | Not yet classified; must classify before consumers adopt             |
| `volume-tier`         | Not yet classified; must classify before infrastructure provisioning |
| `velocity-mode`       | Not yet classified; must classify before pipeline design             |

For categorical dimensions, configs set `index_strategy.missing_handling: "error"` to enforce explicit classification. For sortable dimensions, treat missing values as invalid by policy—require explicit `unknown` or a concrete value.

## Indexing Metadata (Non-Policy)

Dimension configs include metadata intended for indexing and user interfaces:

- `ordinal_mapping` and `default_order`: sorting and ordering hints
- `sentinel`: an ordering/indexing placeholder (often `0` for `unknown`)
- `is_none`: UI hint for a commonly selected “baseline” value (not a permission to omit classification)

These fields MUST NOT be interpreted as policy defaults. Classification remains required even when a value is marked `is_none: true`.

**Pattern**: When ingesting unclassified data, explicitly set `unknown` and gate downstream operations on classification completion.

---

## Volatility vs Velocity

These dimensions are intentionally separate:

- **Volatility**: how often the underlying data changes (freshness / cadence)
- **Velocity mode**: how you process the data (batch / micro-batch / streaming / hybrid)

See `docs/standards/velocity-mode-classification.md` for the relationship table and common pairings.

---

## Stability and Versioning

During Crucible’s alpha phase:

- Dimension definitions and docs may be marked `status: stable` to indicate the team’s intent that the meaning is not expected to churn.
- Schemas live under `schemas/**/v0/` which signals the interface may still change.

If you need strong stability guarantees, pin to a specific Crucible git commit (and document provenance) rather than relying on a moving `v0` URL.

---

## Extending the Framework

When adding a new dimension:

1. Add a narrative standard in `docs/standards/`
2. Add a machine definition in `config/classifiers/dimensions/` validated by `schemas/classifiers/v0/dimension-definition.schema.json`
3. Add examples and governance metadata (owner, reviewers, review cycle)
4. Ensure the dimension is **orthogonal** (avoid overlapping meanings with existing dimensions)
