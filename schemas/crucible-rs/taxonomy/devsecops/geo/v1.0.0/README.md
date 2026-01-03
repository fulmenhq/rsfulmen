---
title: "Fulmen Geo Taxonomy"
description: "Canonical JSON Schema definitions for geographic region keys and metadata"
author: "Schema Cartographer"
date: "2025-11-10"
last_updated: "2025-11-10"
status: "experimental"
tags: ["schema", "taxonomy", "geo", "devsecops", "v1.0.0", "experimental"]
---

# Fulmen Geo Taxonomy

⚠️ **EXPERIMENTAL STATUS**: This taxonomy is under active development for L'Orage Central (v0.1.x → v0.2.0 cycle).
Breaking changes may occur without version bumps per [ADR-0011](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md).

**Do not depend on this taxonomy in production applications** until experimental status is removed (target: L'Orage Central v0.2.0).

## Purpose

Machine-readable geographic taxonomy for DevSecOps orchestration, compliance, and data residency requirements. This taxonomy provides SSOT for geo-restrictions in L'Orage Central tenant policies and supports asymmetric regional groupings (e.g., "EU" includes UK/CH for practical hosting conventions).

**Provider-specific regions**: Cloud provider region identifiers (e.g., `nyc3`, `us-east-1`) are defined in the [infra-providers taxonomy](../../infra-providers/v1.0.0/README.md) with geographic references back to this taxonomy.

## Files

- `geo-key.schema.json` – Enumerates all canonical geographic identifiers (countries and region groups only).
- `geo-metadata.schema.json` – Describes the complete taxonomy structure stored in `config/taxonomy/devsecops/geo.yaml`.

## Usage

```bash
goneat schema validate-data \
  --schema schemas/taxonomy/devsecops/geo/v1.0.0/geo-metadata.schema.json \
  --data config/taxonomy/devsecops/geo.yaml
```

Downstream specs (e.g., L'Orage Central policy schemas, infra-providers regions) can `$ref` `geo-key.schema.json` to stay aligned with the canonical taxonomy.

## Taxonomy Structure

The geo taxonomy consists of two levels:

### Countries

ISO 3166-1 alpha-2 country codes (lowercase for keys, uppercase for isoAlpha2). Includes compliance framework associations (e.g., GDPR, CCPA, HIPAA).

**Examples**: `us` (United States), `de` (Germany), `gb` (United Kingdom)

### Regions

Grouped super-regions that handle practical IT conventions. For example, "eu" includes UK and Switzerland despite not being EU members, reflecting common hosting patterns (e.g., Amsterdam for Swiss data).

**Examples**: `eu` (European Union + Conventions), `na` (North America)

## Relationship to Infrastructure

Cloud provider regions reference this taxonomy via the `geo` field. See [infra-providers taxonomy](../../infra-providers/v1.0.0/README.md) for provider-specific region mappings (e.g., AWS `us-east-1` → `geo: us`).

## References

- [ADR-0011: Temporary SemVer Flexibility for L'Orage Central DevSecOps Schemas](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md)
- [L'Orage Central Feature Brief](.plans/fulmen-lorage-central/feature-brief.md)
- [ISO 3166-1](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) - Country code standard
