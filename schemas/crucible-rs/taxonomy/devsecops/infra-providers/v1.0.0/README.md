---
title: "Fulmen Infra Providers Taxonomy"
description: "Canonical JSON Schema definitions for infrastructure provider keys and metadata"
author: "Schema Cartographer"
date: "2025-11-10"
last_updated: "2025-11-10"
status: "experimental"
tags: ["schema", "taxonomy", "providers", "devsecops", "v1.0.0", "experimental"]
---

# Fulmen Infra Providers Taxonomy

⚠️ **EXPERIMENTAL STATUS**: This taxonomy is under active development for L'Orage Central (v0.1.x → v0.2.0 cycle).
Breaking changes may occur without version bumps per [ADR-0011](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md).

**Do not depend on this taxonomy in production applications** until experimental status is removed (target: L'Orage Central v0.2.0).

## Purpose

Machine-readable enums for infrastructure and cloud providers supported across DevSecOps tooling. This taxonomy provides SSOT for provider selection in L'Orage Central recipe schemas, multi-cloud orchestration, and infrastructure management tools.

**Provider regions**: Each provider includes region mappings with geographic references to the [geo taxonomy](../../geo/v1.0.0/README.md), enabling compliance tracking and data residency validation.

## Files

- `infra-providers-key.schema.json` – Enumerates the canonical provider keys (`doc`, `aws`, `gcp`, `azure`, `cloudflare`, `hetzner`).
- `infra-providers-metadata.schema.json` – Describes the metadata objects stored in `config/taxonomy/devsecops/infra-providers.yaml`.

## Usage

```bash
goneat schema validate-data \
  --schema schemas/taxonomy/devsecops/infra-providers/v1.0.0/infra-providers-metadata.schema.json \
  --data config/taxonomy/devsecops/infra-providers.yaml
```

Downstream specs (e.g., L'Orage Central recipe schemas) can `$ref` `infra-providers-key.schema.json` to stay aligned with the canonical taxonomy.

## Provider Regions

Each provider defines regions as objects containing:

- **key**: Provider-specific region identifier (e.g., `us-east-1`, `nyc3`)
- **geo**: Geographic reference to [geo taxonomy](../../geo/v1.0.0/README.md) (country or region group)
- **description**: Location and characteristics

**Example**:

```yaml
regions:
  - key: us-east-1
    geo: us # References geo taxonomy country
    description: Northern Virginia, US; HIPAA eligible
  - key: eu-west-1
    geo: eu # References geo taxonomy region group
    description: Ireland, EU; GDPR compliant
```

The `geo` field allows either:

- **Country code**: `us`, `de`, `gb` (when provider specifies exact country)
- **Region group**: `eu`, `na` (when provider uses broader geographic designation)

## OpenTofu Support Levels

- **official**: Full OpenTofu provider maintained by HashiCorp or equivalent
- **community**: Third-party OpenTofu provider with community maintenance
- **none**: No OpenTofu provider available (manual SDK/API integration required)

## References

- [ADR-0011: Temporary SemVer Flexibility for L'Orage Central DevSecOps Schemas](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md)
- [L'Orage Central Feature Brief](.plans/fulmen-lorage-central/feature-brief.md)
