---
title: "Fulmen Auth Methods Taxonomy"
description: "Canonical JSON Schema definitions for authentication/MFA method keys and metadata"
author: "Schema Cartographer"
date: "2025-11-10"
last_updated: "2025-11-10"
status: "experimental"
tags: ["schema", "taxonomy", "auth", "devsecops", "v1.0.0", "experimental"]
---

# Fulmen Auth Methods Taxonomy

⚠️ **EXPERIMENTAL STATUS**: This taxonomy is under active development for L'Orage Central (v0.1.x → v0.2.0 cycle).
Breaking changes may occur without version bumps per [ADR-0011](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md).

**Do not depend on this taxonomy in production applications** until experimental status is removed (target: L'Orage Central v0.2.0).

## Purpose

Machine-readable enums for authentication and MFA methods used across DevSecOps tooling. This taxonomy provides SSOT for security method selection in L'Orage Central policy schemas, AAA systems, and future identity management tools.

## Files

- `auth-methods-key.schema.json` – Enumerates the canonical auth method keys (`totp`, `webauthn`, `yubikey`, `sms`).
- `auth-methods-metadata.schema.json` – Describes the metadata objects stored in `config/taxonomy/devsecops/auth-methods.yaml`.

## Usage

```bash
goneat schema validate-data \
  --schema schemas/taxonomy/devsecops/auth-methods/v1.0.0/auth-methods-metadata.schema.json \
  --data config/taxonomy/devsecops/auth-methods.yaml
```

Downstream specs (e.g., L'Orage Central policy schemas) can `$ref` `auth-methods-key.schema.json` to stay aligned with the canonical taxonomy.

## Security Levels

- **low**: Basic security (e.g., SMS OTP) - fallback only, deprecated for production
- **medium**: Standard security (e.g., TOTP app-based codes)
- **high**: Hardware-backed security (e.g., YubiKey OTP/U2F)
- **phishing-resistant**: NIST-aligned highest security (e.g., WebAuthn/FIDO2)

## References

- [ADR-0011: Temporary SemVer Flexibility for L'Orage Central DevSecOps Schemas](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md)
- [L'Orage Central Feature Brief](.plans/fulmen-lorage-central/feature-brief.md)
