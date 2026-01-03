---
title: "L'Orage Central Tenant Registry Schema"
description: "Schema for tenant registry defining infra instantiations with confidential public IDs"
author: "Schema Cartographer"
date: "2025-11-10"
last_updated: "2025-11-10"
status: "experimental"
tags:
  ["schema", "lorage-central", "tenant", "devsecops", "v1.0.0", "experimental"]
---

# L'Orage Central Tenant Registry Schema

⚠️ **EXPERIMENTAL STATUS**: This schema is under active development for L'Orage Central (v0.1.x → v0.2.0 cycle).
Breaking changes may occur without version bumps per [ADR-0011](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md).

**Do not depend on this schema in production applications** until experimental status is removed (target: L'Orage Central v0.2.0).

## Purpose

Defines the tenant registry structure for L'Orage Central multi-tenant orchestration. A **tenant** represents an infrastructure instantiation serving a specific purpose (e.g., `acme-prod-us-east`), while a **client** represents the owning organization. Multiple tenants per client are supported (e.g., production, development, testing environments).

## Key Features

### Confidential Public IDs

- Public IDs are globally unique identifiers for tenants
- For confidential clients, IDs must be obscured (e.g., `tnt-uuid-123-prod-us` instead of `acme-prod-us`)
- UUID/hash-based IDs recommended for anonymous clients

### Geographic & Cloud Restrictions

- `geo`: Allowed geographic regions (references geo taxonomy)
- `cloud`: Allowed cloud providers (references infra-providers taxonomy)
- Enforced during deployment to ensure compliance

### Data Sensitivity Tagging

- `pii`: PII flag triggers GDPR compliance (e.g., EU geo enforcement)
- `phi`: PHI flag triggers HIPAA compliance (e.g., encryption + audit requirements)
- `other`: Additional compliance frameworks (e.g., `pci-dss`, `soc2`)

## Files

- `tenant.schema.json` – Tenant registry schema structure

## Usage

```bash
goneat schema validate-schema schemas/devsecops/lorage-central/tenant/v1.0.0/tenant.schema.json
```

## Schema Structure

```yaml
client:
  id: acme-internal
  name: Acme Corporation
  confidential: true
tenants:
  - publicId: tnt-uuid-123-prod-us
    purpose: production-mattermost
    geo: [us, eu]
    cloud: [doc, aws]
    dataSensitivity:
      pii: true
      phi: false
      other: []
globalUniqueness: true
```

## Integration with L'Orage Central

- Loaded via gofulmen Crucible shim
- REPL enforces tenant scoping via policy references
- Public IDs used throughout (client info never exposed)
- Policy schemas reference tenant `publicId` for session management

## References

- [ADR-0011: Temporary SemVer Flexibility for L'Orage Central DevSecOps Schemas](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md)
- [L'Orage Central Feature Brief](.plans/fulmen-lorage-central/feature-brief.md)
- [Geo Taxonomy](../../../../taxonomy/devsecops/geo/v1.0.0/README.md)
- [Infra Providers Taxonomy](../../../../taxonomy/devsecops/infra-providers/v1.0.0/README.md)
