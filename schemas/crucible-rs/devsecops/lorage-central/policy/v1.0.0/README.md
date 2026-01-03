---
title: "L'Orage Central Tenant Policy Schema"
description: "Schema for per-tenant session management, MFA, and isolation policies"
author: "Schema Cartographer"
date: "2025-11-10"
last_updated: "2025-11-10"
status: "experimental"
tags:
  ["schema", "lorage-central", "policy", "devsecops", "v1.0.0", "experimental"]
---

# L'Orage Central Tenant Policy Schema

⚠️ **EXPERIMENTAL STATUS**: This schema is under active development for L'Orage Central (v0.1.x → v0.2.0 cycle).
Breaking changes may occur without version bumps per [ADR-0011](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md).

**Do not depend on this schema in production applications** until experimental status is removed (target: L'Orage Central v0.2.0).

## Purpose

Defines per-tenant policy rules for L'Orage Central REPL/CLI sessions. Enforces:

- **Session TTL**: Token timeouts with per-operation overrides
- **MFA Requirements**: Multi-factor authentication methods and fallbacks
- **Isolation**: Tenant data separation with pluggable secrets backends
- **Geographic/Cloud Restrictions**: Compliance-driven deployment constraints
- **Data Sensitivity Guards**: Automatic policy enforcement based on PII/PHI flags

## Key Features

### Session Management

- **TTL enforcement**: Default timeout with per-operation overrides (e.g., `deploy: 1h`, `query: 5m`)
- **Concurrent session limits**: Strict isolation (default: 1 session per tenant)
- **No persistence**: Force re-unlock per session for zero-trust

### MFA Configuration

- **Methods**: Ordered list from auth-methods taxonomy (first available used)
- **Required flag**: Auto-enabled for PII-flagged tenants
- **Fallback**: CLI prompt or none (for air-gapped environments)

### Isolation & Backends

- **Pluggable secrets**: Turso (HA), LibSQL, GPG-file (cloud-free), Vaultwarden
- **Cross-access control**: Admin override flag (default: false)
- **Encryption**: GPG file-level encryption option for CR-SQLite

### Compliance Guards

- **Geo restrictions**: Enforce deployment regions (e.g., EU-only for GDPR)
- **Cloud restrictions**: Limit providers (e.g., FedRAMP-approved only)
- **Auto-guards**: PII/PHI flags trigger automatic policy enforcement

## Files

- `policy.schema.json` – Tenant policy schema structure

## Usage

```bash
goneat schema validate-schema schemas/devsecops/lorage-central/policy/v1.0.0/policy.schema.json
```

## Schema Structure

```yaml
tenant: tnt-uuid-123-prod-us
session:
  ttl:
    default: 15m
    ops:
      deploy: 1h
      query: 5m
  maxConcurrent: 1
mfa:
  required: true
  methods: [totp, webauthn]
  fallback: cli-prompt
isolation:
  store:
    type: turso
    conn:
      url: turso://acme-db
      auth: { ref: gpg://keyring/acme }
    enc: false
  crossAccess: false
geoRestrictions: [eu]
cloudRestrictions: [doc, aws]
dataSensitivityGuards:
  pii: true
  phi: false
audit:
  level: structured
  retain: 30d
```

## Integration with L'Orage Central

- Loaded via gofulmen Crucible shim at REPL unlock
- Enforced at operation execution (deploy, seed, query)
- Validates against tenant registry for geo/cloud match
- Auto-applies guards based on dataSensitivity flags

## References

- [ADR-0011: Temporary SemVer Flexibility for L'Orage Central DevSecOps Schemas](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md)
- [L'Orage Central Feature Brief](.plans/fulmen-lorage-central/feature-brief.md)
- [Tenant Registry Schema](../tenant/v1.0.0/README.md)
- [Auth Methods Taxonomy](../../../../taxonomy/devsecops/auth-methods/v1.0.0/README.md)
- [Geo Taxonomy](../../../../taxonomy/devsecops/geo/v1.0.0/README.md)
- [Infra Providers Taxonomy](../../../../taxonomy/devsecops/infra-providers/v1.0.0/README.md)
