---
title: "Retention & Lifecycle Classification"
description: "Data retention period classification standard"
category: "standards"
status: "stable"
version: "1.0.0"
lastUpdated: "2026-01-22"
maintainer: "3leaps-core"
reviewers: ["compliance", "legal"]
approvers: ["3leapsdave"]
tags: ["classification", "retention", "lifecycle", "compliance", "data-governance"]
content_license: "CC0"
relatedDocs:
  - "docs/standards/data-sensitivity-classification.md"
  - "config/classifiers/dimensions/retention-lifecycle.dimension.json"
audience: "all"
---

# Retention & Lifecycle Classification

This standard defines retention period classifications for data across all 3leaps ecosystems. It provides a consistent framework for:

- **Deletion SLAs** - When data must be deleted
- **Archival Policies** - When and how to archive
- **Compliance Requirements** - Regulatory retention obligations
- **Storage Optimization** - Tiered storage decisions
- **Legal Hold Procedures** - Preservation requirements

---

## Retention Levels

### Unknown

**Retention not yet classified; must be classified before storage provisioning.**

| Aspect        | Requirement                                     |
| ------------- | ----------------------------------------------- |
| **Retention** | Undefined; treat as indefinite until classified |
| **Backup**    | Basic backup until policy determined            |
| **Storage**   | Staging/quarantine                              |
| **Deletion**  | Prohibited until classified                     |
| **Use Cases** | New data sources, imported datasets             |

**Operational Notes**: Gate storage tier decisions and deletion policies on explicit classification. Do not auto-delete `unknown` retention data.

---

### Transient

**Short-lived data (≤7 days); auto-deleted, no backup required.**

| Aspect        | Requirement                              |
| ------------- | ---------------------------------------- |
| **Retention** | ≤7 days                                  |
| **Backup**    | Not required                             |
| **Storage**   | Hot/ephemeral                            |
| **Deletion**  | Automatic expiry                         |
| **Use Cases** | Session cache, temp files, debug buffers |

**Operational Notes**:

- Use TTL-based storage (Redis, S3 lifecycle)
- No archive or backup pipelines needed
- Safe to delete without approval

---

### Short

**Short retention (≤90 days); routine cleanup, basic backup.**

| Aspect        | Requirement                             |
| ------------- | --------------------------------------- |
| **Retention** | ≤90 days                                |
| **Backup**    | Basic (daily snapshots)                 |
| **Storage**   | Standard                                |
| **Deletion**  | Scheduled cleanup jobs                  |
| **Use Cases** | Debug logs, dev environments, test data |

**Operational Notes**:

- Configure automated cleanup jobs
- Basic backup for recovery during retention window
- Review before deletion if business value uncertain

---

### Standard

**Standard retention (≤2 years); regular backup, tiered storage.**

| Aspect        | Requirement                                             |
| ------------- | ------------------------------------------------------- |
| **Retention** | ≤2 years                                                |
| **Backup**    | Regular (with point-in-time recovery)                   |
| **Storage**   | Tiered (hot → warm → cold)                              |
| **Deletion**  | Policy-driven with approval                             |
| **Use Cases** | Transaction history, operational data, business records |

**Operational Notes**:

- Default retention tier for most business data
- Implement storage tiering for cost optimization
- Document deletion policy and approval process

---

### Long

**Long retention (>2 years); archive storage, compliance backup.**

| Aspect        | Requirement                                        |
| ------------- | -------------------------------------------------- |
| **Retention** | >2 years (often 7+ years)                          |
| **Backup**    | Compliance-grade (immutable, verified)             |
| **Storage**   | Archive/cold (Glacier, etc.)                       |
| **Deletion**  | Regulatory approval required                       |
| **Use Cases** | Audit records, financial data, regulatory archives |

**Operational Notes**:

- Use archive storage classes for cost efficiency
- Ensure compliance with industry regulations (SOX, HIPAA, etc.)
- Implement integrity verification (checksums, attestation)

---

### Legal Hold

**Indefinite retention; immutable, protected from deletion.**

| Aspect        | Requirement                                                     |
| ------------- | --------------------------------------------------------------- |
| **Retention** | Indefinite (until released)                                     |
| **Backup**    | Immutable, geographically redundant                             |
| **Storage**   | WORM (Write-Once-Read-Many)                                     |
| **Deletion**  | Prohibited without legal release                                |
| **Use Cases** | Litigation evidence, regulatory investigation, breach forensics |

**Operational Notes**:

- Implement legal hold flag in data management systems
- Chain of custody documentation required
- Legal team controls release authorization
- Prevent any modification or deletion

---

## Decision Guide

```
How long must we keep this data?

├── Only while actively needed (hours/days) → transient
├── Weeks to months (debugging, dev cycles) → short
├── 1-2 years (operational history) → standard
├── Years (regulatory, audit requirements) → long
└── Until legal/regulatory release → legal-hold
```

---

## Compliance Mapping

| Regulation | Typical Retention | Recommended Tier |
| ---------- | ----------------- | ---------------- |
| SOX        | 7 years           | long             |
| HIPAA      | 6 years           | long             |
| GDPR       | Varies (minimize) | standard or less |
| PCI DSS    | 1 year minimum    | standard         |
| SEC 17a-4  | 6 years           | long             |
| Litigation | Until resolved    | legal-hold       |

---

## Handling Matrix

| Retention Level | Storage Tier  | Backup Strategy  | Deletion Process    | Cost Profile |
| --------------- | ------------- | ---------------- | ------------------- | ------------ |
| **transient**   | Hot/ephemeral | None             | Auto-expiry         | Lowest       |
| **short**       | Standard      | Daily snapshots  | Scheduled jobs      | Low          |
| **standard**    | Tiered        | Regular + PITR   | Policy + approval   | Medium       |
| **long**        | Archive/cold  | Compliance-grade | Regulatory approval | Higher       |
| **legal-hold**  | WORM          | Immutable        | Legal release only  | Highest      |

---

## Machine-Readable Definition

- **Dimension Config**: `config/classifiers/dimensions/retention-lifecycle.dimension.json`
- **Schema**: `schemas/classifiers/v0/dimension-definition.schema.json`

---

## Attribution

This standard is the canonical reference for retention classification across 3leaps ecosystems. Downstream consumers should reference or vendor this standard rather than maintaining independent copies.

**Review Cycle**: Semiannual with compliance and legal teams.
