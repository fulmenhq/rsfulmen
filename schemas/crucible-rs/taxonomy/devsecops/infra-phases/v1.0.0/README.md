---
title: "Fulmen Infra Phases Taxonomy"
description: "Canonical JSON Schema definitions for infrastructure deployment phase keys and metadata"
author: "Schema Cartographer"
date: "2025-11-10"
last_updated: "2025-11-10"
status: "experimental"
tags: ["schema", "taxonomy", "phases", "devsecops", "v1.0.0", "experimental"]
---

# Fulmen Infra Phases Taxonomy

⚠️ **EXPERIMENTAL STATUS**: This taxonomy is under active development for L'Orage Central (v0.1.x → v0.2.0 cycle).
Breaking changes may occur without version bumps per [ADR-0011](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md).

**Do not depend on this taxonomy in production applications** until experimental status is removed (target: L'Orage Central v0.2.0).

## Purpose

Machine-readable enums for infrastructure deployment phases used in IaC orchestration. This taxonomy provides SSOT for dependency ordering in L'Orage Central recipe execution, enabling automatic topological sorting and parallel execution planning.

## Files

- `infra-phases-key.schema.json` – Enumerates the canonical phase keys (`bootstrap`, `secrets`, `network`, `storage`, `compute`).
- `infra-phases-metadata.schema.json` – Describes the metadata objects stored in `config/taxonomy/devsecops/infra-phases.yaml`.

## Usage

```bash
goneat schema validate-data \
  --schema schemas/taxonomy/devsecops/infra-phases/v1.0.0/infra-phases-metadata.schema.json \
  --data config/taxonomy/devsecops/infra-phases.yaml
```

Downstream specs (e.g., L'Orage Central recipe schemas) can `$ref` `infra-phases-key.schema.json` to stay aligned with the canonical taxonomy.

## Phase Ordering

Phases execute in order sequence (0=first). Phases with the same order number are parallelizable if `parallel: true`. The `deps` array defines explicit dependencies that must complete before the phase starts.

## Procedural Flag

When `procedural: true`, the phase allows script/SDK calls beyond pure declarative IaC. This is typical for bootstrap (key generation) and networking (complex routing logic) phases.

## References

- [ADR-0011: Temporary SemVer Flexibility for L'Orage Central DevSecOps Schemas](../../../../architecture/decisions/ADR-0011-lorage-devsecops-schemas-semver-experimental.md)
- [L'Orage Central Feature Brief](.plans/fulmen-lorage-central/feature-brief.md)
