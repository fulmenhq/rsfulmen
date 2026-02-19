---
title: "Schema Stability Classification"
description: "Schema evolution and stability classification standard"
category: "standards"
status: "stable"
version: "1.0.0"
lastUpdated: "2026-01-22"
maintainer: "3leaps-core"
reviewers: ["platform", "api-standards"]
approvers: ["3leapsdave"]
tags: ["classification", "schema", "versioning", "api", "stability"]
content_license: "CC0"
relatedDocs:
  - "docs/decisions/ADR-0001-schema-config-versioning.md"
  - "config/classifiers/dimensions/schema-stability.dimension.json"
audience: "all"
---

# Schema Stability Classification

This standard defines stability levels for schemas, APIs, and configuration formats across all 3leaps ecosystems. It provides a consistent framework for:

- **Compatibility Expectations** - What consumers can rely on
- **Versioning Cadence** - How often changes occur
- **Deprecation Windows** - How much notice before breaking changes
- **Consumer Guidance** - Whether to adopt and how to track changes

---

## Stability Levels

### Unknown

**Stability not yet classified; must be classified before consumers adopt.**

| Aspect               | Requirement                           |
| -------------------- | ------------------------------------- |
| **Compatibility**    | Unknown; assume none                  |
| **Breaking Changes** | Possible at any time                  |
| **Versioning**       | Not yet versioned                     |
| **Deprecation**      | N/A                                   |
| **Consumer Advice**  | Do not adopt; wait for classification |

**Use Cases**: New schemas under initial development, imported definitions pending review.

**Operational Notes**: Block production adoption of `unknown` stability schemas. Require explicit classification before publishing to consumers.

---

### Experimental

**No stability guarantees; may change without notice.**

| Aspect               | Requirement                                  |
| -------------------- | -------------------------------------------- |
| **Compatibility**    | None guaranteed                              |
| **Breaking Changes** | Any time, no notice required                 |
| **Versioning**       | v0.x or -alpha/-preview suffix               |
| **Deprecation**      | None required                                |
| **Consumer Advice**  | For evaluation only; expect breaking changes |

**Use Cases**: Early prototypes, alpha features, proof-of-concept APIs, `v0/` schema directories.

**Naming Conventions**:

- Version: `v0.x.x`, `0.x`, or `vX-alpha`
- Path: `/v0/`, `/preview/`, `/alpha/`

---

### Evolving

**Active development; additive changes expected, breaking changes with notice.**

| Aspect               | Requirement                                 |
| -------------------- | ------------------------------------------- |
| **Compatibility**    | Additive changes backward-compatible        |
| **Breaking Changes** | With notice (changelog, deprecation period) |
| **Versioning**       | v0.x with clear changelog                   |
| **Deprecation**      | Minimum 1 minor version warning             |
| **Consumer Advice**  | Safe for development; track changelog       |

**Use Cases**: Beta features, active development APIs, schemas under iteration.

**Naming Conventions**:

- Version: `v0.x.x` or `vX-beta`
- Path: `/v0/`, `/beta/`

---

### Stable

**Production-ready; breaking changes require major version bump and deprecation period.**

| Aspect               | Requirement                                      |
| -------------------- | ------------------------------------------------ |
| **Compatibility**    | Full backward compatibility within major version |
| **Breaking Changes** | Major version bump required                      |
| **Versioning**       | Semantic versioning (v1.x.x, v2.x.x)             |
| **Deprecation**      | Minimum 6 months or 2 minor versions             |
| **Consumer Advice**  | Production safe; follow semver guidelines        |

**Use Cases**: Production APIs, released schemas, public interfaces.

**Naming Conventions**:

- Version: `v1.x.x`, `v2.x.x` (semver)
- Path: `/v1/`, `/v2/`

---

### Frozen

**No changes expected; security patches only.**

| Aspect               | Requirement                                 |
| -------------------- | ------------------------------------------- |
| **Compatibility**    | No changes (security-only exceptions)       |
| **Breaking Changes** | Only for critical security fixes            |
| **Versioning**       | Patch versions only (vX.Y.Z → vX.Y.Z+1)     |
| **Deprecation**      | N/A (may transition to deprecated)          |
| **Consumer Advice**  | Maximum stability; plan for eventual sunset |

**Use Cases**: Legacy APIs maintained for compatibility, long-term support versions.

---

### Deprecated

**Scheduled for removal; migration path documented.**

| Aspect               | Requirement                                      |
| -------------------- | ------------------------------------------------ |
| **Compatibility**    | Maintained until removal date                    |
| **Breaking Changes** | None (frozen until removal)                      |
| **Versioning**       | No new versions                                  |
| **Deprecation**      | Removal date published, migration guide provided |
| **Consumer Advice**  | Migrate immediately; removal imminent            |

**Use Cases**: Sunset APIs, replaced schemas, legacy versions.

**Required Documentation**:

- Removal date
- Replacement/migration path
- Migration guide

---

## Lifecycle Transitions

```
experimental → evolving → stable → frozen → deprecated → removed
                  ↑           |
                  └───────────┘ (reactivation rare)
```

### Typical Progression

1. **experimental**: Initial development, rapid iteration
2. **evolving**: Beta users onboarded, stabilizing
3. **stable**: GA release, production use
4. **frozen**: Maintenance mode, new version available
5. **deprecated**: Sunset announced, migration period
6. **removed**: No longer available

---

## Versioning Guidelines

| Stability Level | Version Pattern | Example      |
| --------------- | --------------- | ------------ |
| experimental    | v0.x.x          | v0.3.0-alpha |
| evolving        | v0.x.x          | v0.8.0-beta  |
| stable          | vX.Y.Z          | v1.2.3       |
| frozen          | vX.Y.Z (patch)  | v1.2.4       |
| deprecated      | vX.Y.Z (frozen) | v1.2.4       |

---

## Consumer Guidance Matrix

| If you need...              | Use stability level |
| --------------------------- | ------------------- |
| Bleeding edge features      | experimental        |
| New features with some risk | evolving            |
| Production reliability      | stable              |
| Maximum stability           | frozen              |
| (Don't use)                 | deprecated          |

---

## Machine-Readable Definition

- **Dimension Config**: `config/classifiers/dimensions/schema-stability.dimension.json`
- **Schema**: `schemas/classifiers/v0/dimension-definition.schema.json`

---

## Attribution

This standard is the canonical reference for schema stability classification across 3leaps ecosystems. Downstream consumers should reference or vendor this standard rather than maintaining independent copies.

**Review Cycle**: Quarterly with platform and API standards teams.
