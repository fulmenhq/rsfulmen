---
title: "L'Orage Central Root Credentials Metadata Schema"
description: "Schema for root credential metadata with opaque references and expiry tracking"
author: "Schema Cartographer"
date: "2025-11-10"
last_updated: "2025-11-10"
status: "experimental"
tags:
  [
    "schema",
    "lorage-central",
    "credentials",
    "devsecops",
    "v1.0.0",
    "experimental",
  ]
---

# L'Orage Central Root Credentials Metadata Schema

⚠️ **EXPERIMENTAL STATUS**: This schema is under active development for L'Orage Central (v0.1.x → v0.2.0 cycle).
Breaking changes may occur without version bumps per [ADR-0011](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md).

**Do not depend on this schema in production applications** until experimental status is removed (target: L'Orage Central v0.2.0).

## Purpose

Defines metadata for root credentials used in L'Orage Central tenant bootstrap and seeding operations. **No plaintext secrets stored** - only opaque references that are decrypted at runtime by microtools.

## Key Features

### Opaque References

- **No values**: Only metadata and references (e.g., `gpg://keyring/acme-bootstrap`)
- **Runtime decryption**: Microtools decrypt at operation time
- **Audit trail**: Metadata tracks creation, expiry, purpose without exposing secrets

### Credential Types

- `gpg-key`: GPG keyring references
- `vaultwarden-token`: Vaultwarden API tokens
- `turso-root`: Turso database root credentials
- `yubikey-seed`: YubiKey seed values

### Expiry & Rotation

- **Automatic expiry validation**: REPL blocks expired credentials
- **Rotation policy**: Optional auto-rotation (e.g., every 90 days)
- **Purpose tracking**: Links credentials to specific use cases

## Files

- `credentials.schema.json` – Root credentials metadata schema

## Usage

```bash
goneat schema validate-schema schemas/devsecops/lorage-central/credentials/v1.0.0/credentials.schema.json
```

## Example

```json
{
  "id": "acme-bootstrap-key",
  "tenant": "tnt-uuid-123-prod-us",
  "type": "gpg-key",
  "ref": "gpg://keyring/acme-bootstrap",
  "metadata": {
    "created": "2025-11-09T12:00:00Z",
    "expires": "2026-11-09T12:00:00Z",
    "purpose": "tenant-bootstrap",
    "rotation": {
      "interval": "90d",
      "method": "manual"
    }
  },
  "backend": {
    "type": "gpg-file",
    "conn": {
      "path": "/secure/keyring"
    },
    "enc": true
  }
}
```

## Integration

- **Policy integration**: Backend refs `policy.isolation.store`
- **Tenant scoping**: Credentials linked to specific tenant
- **REPL validation**: Expiry checked at unlock/operation time
- **Activity logging**: Credential usage tracked in activity records

## References

- [ADR-0011: Temporary SemVer Flexibility for L'Orage Central DevSecOps Schemas](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md)
- [L'Orage Central Feature Brief](.plans/fulmen-lorage-central/feature-brief.md)
- [Policy Schema](../policy/v1.0.0/README.md)
- [Tenant Schema](../tenant/v1.0.0/README.md)
