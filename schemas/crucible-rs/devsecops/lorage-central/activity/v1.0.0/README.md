---
title: "L'Orage Central Activity Record Schema"
description: "Schema for audit event records tracking REPL/CLI operations"
author: "Schema Cartographer"
date: "2025-11-10"
last_updated: "2025-11-10"
status: "experimental"
tags:
  [
    "schema",
    "lorage-central",
    "activity",
    "audit",
    "devsecops",
    "v1.0.0",
    "experimental",
  ]
---

# L'Orage Central Activity Record Schema

⚠️ **EXPERIMENTAL STATUS**: This schema is under active development for L'Orage Central (v0.1.x → v0.2.0 cycle).
Breaking changes may occur without version bumps per [ADR-0011](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md).

**Do not depend on this schema in production applications** until experimental status is removed (target: L'Orage Central v0.2.0).

## Purpose

Defines audit event structure for L'Orage Central REPL/CLI operations. Auto-emitted via gofulmen observability and stored per policy.audit configuration. **No sensitive data included** - only metadata for compliance and review.

## Key Features

### Event Types

- `unlock`: MFA unlock attempts
- `deploy`: Recipe deployment operations
- `seed`: Data seeding operations
- `query`: Tenant status queries
- `validate`: Schema/config validation
- `error`: Error conditions

### Outcome Tracking

- `success`: Operation completed successfully
- `failure`: Operation failed
- `warning`: Completed with warnings

### Metadata Captured

- **sessionId**: REPL session identifier
- **userId**: Anonymized user (from MFA)
- **duration**: Operation timing (for metrics)
- **ip**: Anonymized client IP
- **details**: Operation-specific context (e.g., recipe name)

## Files

- `activity.schema.json` – Activity record schema

## Usage

```bash
goneat schema validate-schema schemas/devsecops/lorage-central/activity/v1.0.0/activity.schema.json
```

## Example

```json
{
  "id": "evt-acme-unlock-001",
  "timestamp": "2025-11-09T12:00:00Z",
  "tenant": "tnt-uuid-123-prod-us",
  "eventType": "unlock",
  "outcome": "success",
  "metadata": {
    "sessionId": "sess-acme-001",
    "userId": "user-123",
    "duration": "5s",
    "ip": "192.168.1.1",
    "details": {
      "method": "totp",
      "attempts": "1"
    }
  },
  "backend": {
    "level": "structured",
    "retain": "30d"
  }
}
```

## Integration

- **gofulmen observability**: Auto-emitted after each operation
- **Policy storage**: Backend/retention per `policy.audit`
- **Compliance**: Supports GDPR/HIPAA audit requirements
- **Metrics**: Duration tracking for Prometheus export

## References

- [ADR-0011: Temporary SemVer Flexibility for L'Orage Central DevSecOps Schemas](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md)
- [L'Orage Central Feature Brief](.plans/fulmen-lorage-central/feature-brief.md)
- [Policy Schema](../policy/v1.0.0/README.md)
- [Tenant Schema](../tenant/v1.0.0/README.md)
