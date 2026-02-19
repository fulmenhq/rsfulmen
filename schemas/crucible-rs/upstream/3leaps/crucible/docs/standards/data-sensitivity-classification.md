---
title: "Data Sensitivity Classification Standard"
description: "Comprehensive data sensitivity levels for all 3leaps ecosystems"
category: "standards"
status: "stable"
version: "1.0.0"
lastUpdated: "2026-01-22"
maintainer: "3leaps-core"
reviewers: ["security", "compliance"]
approvers: ["3leapsdave"]
tags: ["classification", "sensitivity", "security", "data-handling"]
content_license: "CC0"
relatedDocs:
  - "schemas/classifiers/v0/sensitivity-level.schema.json"
  - "config/classifiers/dimensions/sensitivity.dimension.json"
audience: "all"
---

# Data Sensitivity Classification Standard

This standard defines sensitivity levels for data across all 3leaps ecosystems. It provides a consistent framework for:

- **Automated Security Controls** - Classification-driven access and audit requirements
- **Risk Assessment** - Clear understanding of data handling implications
- **Compliance Management** - Structured approach to regulatory requirements
- **Operational Safety** - Appropriate handling procedures by sensitivity level

Missing classification is a policy error. At ingestion boundaries, explicitly set `unknown` until classification is complete.

## Sensitivity Levels

### UNKNOWN - Unclassified

**Data classification unknown; must be isolated until classified.**

| Aspect         | Requirement                                              |
| -------------- | -------------------------------------------------------- |
| **Handling**   | Isolate at data ingestion boundaries                     |
| **Processing** | Use classification functions before storage/transmission |
| **Access**     | Restricted to classification pipeline components only    |
| **Audit**      | All handling logged until proper classification assigned |
| **Timeline**   | Reclassify within 24 hours                               |

**Examples**: Secrets discovered during scanning without context, config files from external sources, API keys without classification, logs that may contain credentials or PII.

---

### Level 0 - Public

**Information known or intended to be in the public domain.**

| Aspect              | Requirement   |
| ------------------- | ------------- |
| **Access Controls** | None required |
| **Audit**           | None required |
| **Retention**       | Unlimited     |

**Examples**: Open source dependencies, public API documentation, published config templates, public status pages.

**Permitted Operations**: Public repository storage, unrestricted sharing, external integration without controls, logging without redaction.

---

### Level 1 - Confidential

**Information not available unrestricted; NDA or confidential marking required.**

| Aspect              | Requirement                                |
| ------------------- | ------------------------------------------ |
| **Access Controls** | Authentication required, role-based access |
| **Audit**           | Access logging recommended                 |
| **Retention**       | Follow organizational retention policies   |

**Examples**: Internal documentation and runbooks, non-production configs, development procedures, internal correspondence, business logic details.

**Security Requirements**: Private repositories with access controls, encrypted storage at rest, VPN/secure network for remote access, team-based access with regular review.

---

### Level 2 - Blinded

**Information obfuscated to protect identity of persons and enterprises.**

| Aspect              | Requirement                                            |
| ------------------- | ------------------------------------------------------ |
| **Access Controls** | Authenticated access with blinding verification        |
| **Audit**           | Audit blinding processes and access attempts           |
| **Retention**       | Verify blinding effectiveness before long-term storage |

**Examples**: Anonymized telemetry, sanitized logs with PII redaction, test data with real structure but fake identities, debugging info with customer data obfuscated.

**Processing Requirements**:

1. Use consistent, auditable obfuscation processes
2. Regularly validate blinding effectiveness
3. Document blinding methods used
4. Ensure blinding cannot be easily reversed

---

### Level 3 - Proprietary

**Enterprise information including MNPI, trade secrets, financial data.**

| Aspect              | Requirement                                        |
| ------------------- | -------------------------------------------------- |
| **Access Controls** | Firewall-protected regions, need-to-know basis     |
| **Audit**           | All access attempts logged and reviewed            |
| **Retention**       | Business retention with secure deletion procedures |

**Examples**: Production configs with business logic, database schemas with proprietary structures, enterprise client integration details, financial/performance data, strategic roadmaps.

**Security Requirements**: Dedicated secure environments, multi-factor authentication, encrypted storage and transmission, regular access reviews, incident response procedures.

---

### Level 4 - Personal/Secret

**Information containing NPPII or requiring specialized access controls.**

| Aspect              | Requirement                                             |
| ------------------- | ------------------------------------------------------- |
| **Access Controls** | Specialized auditing, limited authorized personnel only |
| **Audit**           | All access logged, success/failure monitoring           |
| **Retention**       | Minimum retention, secure deletion with verification    |

**Examples**: Production database credentials, encryption/signing keys, service account tokens with admin access, OAuth client secrets, personal information (emails, names, addresses), authentication tokens, password hashes.

**Critical Security Requirements**:

1. **Secrets Management**: Never store in code repositories or logs
2. **Access Auditing**: Real-time monitoring of all access attempts
3. **Rotation Policies**: Regular credential rotation with automation
4. **Breach Response**: Immediate revocation and rotation if compromised
5. **Environmental Isolation**: Separate from lower-sensitivity data

---

### Level 5 - Privileged/Sysadmin

**Information pertaining to platform operations with attack potential.**

| Aspect              | Requirement                                              |
| ------------------- | -------------------------------------------------------- |
| **Access Controls** | Information security team only, documented business need |
| **Audit**           | Comprehensive logging, anomaly detection                 |
| **Retention**       | Security-driven retention, tamper-proof logging          |

**Examples**: System administration credentials (root, admin), infrastructure access keys (cloud admin), security monitoring data, vulnerability assessments, penetration test reports, backup encryption keys, network security configs.

**Operational Security Requirements**: Air-gapped or highly isolated storage, hardware security modules (HSM), break-glass emergency procedures, continuous security monitoring, regular security audits.

---

### Level 6 - Eyes Only/Legal Hold

**Information with extreme access restrictions and deletion protection.**

| Aspect              | Requirement                                                   |
| ------------------- | ------------------------------------------------------------- |
| **Access Controls** | Executive/legal authorization required, immutable audit trail |
| **Audit**           | Complete access logging, legal compliance tracking            |
| **Retention**       | Legal hold procedures, protected against alteration/deletion  |

**Examples**: Incident response data under investigation, security breach evidence, regulatory compliance data subject to discovery, executive communications on security matters, legal counsel privileged information.

**Legal and Compliance Requirements**:

1. **Immutable Storage**: Write-once, read-many systems
2. **Legal Authorization**: Written approval for all access
3. **Chain of Custody**: Complete audit trail for legal proceedings
4. **Compliance Documentation**: Full regulatory compliance tracking
5. **Executive Oversight**: Board/CEO level awareness and control

---

## Classification Decision Tree

```
Does the data contain credentials, keys, or authentication tokens?
├── YES → Level 4+ (Personal/Secret or higher based on scope)
└── NO → Does it contain personal or enterprise-identifying information?
    ├── YES → Can identity be safely removed/blinded?
    │   ├── YES → Level 2 (Blinded)
    │   └── NO → Level 3+ (Proprietary or higher)
    └── NO → Is it available publicly or intended for public use?
        ├── YES → Level 0 (Public)
        └── NO → Level 1 (Confidential)
```

---

## Handling Matrix

| Level                | Storage           | Transmission       | Logging           | Backup              | Sharing              |
| -------------------- | ----------------- | ------------------ | ----------------- | ------------------- | -------------------- |
| **UNKNOWN**          | Isolated staging  | Encrypted only     | Full audit        | Encrypted           | Prohibited           |
| **0 - Public**       | Any location      | Any method         | Optional          | Any method          | Unrestricted         |
| **1 - Confidential** | Private repos     | VPN/TLS            | Access logs       | Encrypted           | Team only            |
| **2 - Blinded**      | Secure storage    | Encrypted          | Blinding audit    | Verified encryption | Authorized only      |
| **3 - Proprietary**  | Isolated regions  | Dedicated channels | Comprehensive     | Secure deletion     | Need-to-know         |
| **4 - Personal**     | HSM/Vault         | Zero-trust         | Real-time monitor | Immutable backup    | Authorized personnel |
| **5 - Privileged**   | Air-gapped        | Secure channels    | Anomaly detection | Disaster recovery   | Security team only   |
| **6 - Eyes Only**    | Immutable storage | Legal channels     | Legal compliance  | Legal hold          | Executive/legal only |

---

## Transitions

### Downgrade Paths

| From           | To             | Requirements                                                   |
| -------------- | -------------- | -------------------------------------------------------------- |
| 4-Personal     | 2-Blinded      | Tokenization/masking/synthetic surrogates; verification report |
| 5-Privileged   | 3-Proprietary  | Security review and approval                                   |
| 6-Eyes Only    | 3-Proprietary  | Legal release authorization                                    |
| 2-Blinded      | 1-Confidential | Statistical privacy checks (k-anonymity, membership inference) |
| 1-Confidential | 0-Public       | Publication review and approval                                |

### Upgrade Triggers

| Condition              | Action                                            |
| ---------------------- | ------------------------------------------------- |
| Secrets/PII discovered | Immediate upgrade to Level 4+                     |
| Legal hold imposed     | Upgrade to Level 6                                |
| Risk indicators found  | Trigger incident playbook, reclassify immediately |

---

## Incident Response

### Level 4+ Breach Response

1. **Immediate** (< 5 minutes): Revoke/rotate all potentially compromised credentials
2. **Assessment** (< 30 minutes): Determine scope and potential impact
3. **Notification** (< 1 hour): Inform security team and stakeholders
4. **Remediation**: Deploy patches, update procedures, conduct post-mortem

### Level 6 Breach Response

1. **Executive Notification**: Immediate contact to CEO/legal counsel
2. **Legal Assessment**: Determine regulatory and legal implications
3. **Forensic Preservation**: Preserve evidence with chain of custody
4. **Regulatory Compliance**: Follow notification and reporting requirements

---

## Machine-Readable Definitions

- **Schema**: `schemas/classifiers/v0/sensitivity-level.schema.json`
- **Dimension Config**: `config/classifiers/dimensions/sensitivity.dimension.json`

---

## Attribution

This standard is the canonical reference for data sensitivity across 3leaps ecosystems. Downstream consumers (fulmenhq, practicingdata, etc.) should reference or vendor this standard rather than maintaining independent copies.

**Review Cycle**: Quarterly with security and compliance teams.
