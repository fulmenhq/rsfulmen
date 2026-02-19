---
title: "Access Tier Classification"
description: "Distribution and access control classification standard"
category: "standards"
status: "stable"
version: "1.0.0"
lastUpdated: "2026-01-22"
maintainer: "3leaps-core"
reviewers: ["security", "compliance"]
approvers: ["3leapsdave"]
tags: ["classification", "access-control", "distribution", "acl"]
content_license: "CC0"
relatedDocs:
  - "docs/standards/data-sensitivity-classification.md"
  - "config/classifiers/dimensions/access-tier.dimension.json"
audience: "all"
---

# Access Tier Classification

This standard defines access tier levels for distribution and access control across all 3leaps ecosystems. Access tier is often derived from sensitivity but can be overridden to further restrict distribution.

## Relationship to Sensitivity

Access tier complements sensitivity classification:

- **Sensitivity** determines _what the data is_ (how sensitive)
- **Access tier** determines _who can access it_ (distribution scope)

Common pattern: Access tier ≥ Sensitivity level (you can restrict further but not loosen)

| Sensitivity Level     | Minimum Access Tier |
| --------------------- | ------------------- |
| 0-Public              | public              |
| 1-Confidential        | internal            |
| 2-Blinded             | internal            |
| 3-Proprietary         | restricted          |
| 4-Personal/Secret     | privileged          |
| 5-Privileged/Sysadmin | privileged          |
| 6-Eyes Only           | eyes-only           |

---

## Access Tiers

### Unknown

**Access tier not yet classified; must be classified before sharing or distribution.**

| Aspect           | Requirement                                |
| ---------------- | ------------------------------------------ |
| **Access**       | Restricted to classification pipeline only |
| **Distribution** | Prohibited until classified                |
| **Audit**        | All access logged                          |
| **Use Cases**    | Newly uploaded content, unreviewed imports |

**Operational Notes**: Gate all sharing and distribution operations on explicit classification. Systems should reject requests to share `unknown` tier content.

---

### Public

**Unrestricted access; suitable for public distribution.**

| Aspect           | Requirement                             |
| ---------------- | --------------------------------------- |
| **Access**       | No authentication required              |
| **Distribution** | CDN, public repos, external APIs        |
| **Audit**        | Optional                                |
| **Use Cases**    | Open source, public docs, public status |

---

### Internal

**Organization-wide access; authentication required.**

| Aspect           | Requirement                             |
| ---------------- | --------------------------------------- |
| **Access**       | Authenticated org members               |
| **Distribution** | Internal repos, intranet, VPN-protected |
| **Audit**        | Access logging recommended              |
| **Use Cases**    | Internal docs, runbooks, dev resources  |

---

### Restricted

**Team or project-level access; explicit authorization required.**

| Aspect           | Requirement                                |
| ---------------- | ------------------------------------------ |
| **Access**       | Explicitly authorized team members         |
| **Distribution** | Access-controlled repos, gated APIs        |
| **Audit**        | Access logging required                    |
| **Use Cases**    | Project roadmaps, team configs, pilot data |

---

### Privileged

**Named individuals only; documented business need.**

| Aspect           | Requirement                                    |
| ---------------- | ---------------------------------------------- |
| **Access**       | Named individuals with documented need         |
| **Distribution** | Direct share only, no group access             |
| **Audit**        | Comprehensive logging, periodic review         |
| **Use Cases**    | Security findings, exec comms, sensitive plans |

---

### Eyes Only

**Executive/legal authorization required; immutable audit trail.**

| Aspect           | Requirement                                           |
| ---------------- | ----------------------------------------------------- |
| **Access**       | Executive or legal authorization                      |
| **Distribution** | Controlled handoff with chain of custody              |
| **Audit**        | Immutable audit trail, legal compliance               |
| **Use Cases**    | Legal discovery, breach evidence, regulatory response |

---

## Decision Guide

```
Who needs access to this?

├── Anyone (public internet) → public
├── Anyone in the organization → internal
├── Specific teams/projects → restricted
├── Named individuals with documented need → privileged
└── Executive/legal approval required → eyes-only
```

---

## Handling Matrix

| Access Tier    | Storage              | Sharing           | Audit Level   | Review Cycle |
| -------------- | -------------------- | ----------------- | ------------- | ------------ |
| **public**     | Any                  | Unrestricted      | Optional      | None         |
| **internal**   | Private repos        | Org-wide          | Recommended   | Annual       |
| **restricted** | Access-controlled    | Explicit grants   | Required      | Quarterly    |
| **privileged** | Isolated             | Named individuals | Comprehensive | Monthly      |
| **eyes-only**  | Legal-grade controls | Executive handoff | Immutable     | Per-access   |

---

## Machine-Readable Definition

- **Dimension Config**: `config/classifiers/dimensions/access-tier.dimension.json`
- **Schema**: `schemas/classifiers/v0/dimension-definition.schema.json`

---

## Attribution

This standard is the canonical reference for access tier classification across 3leaps ecosystems. Downstream consumers should reference or vendor this standard rather than maintaining independent copies.

**Review Cycle**: Quarterly with security and compliance teams.
