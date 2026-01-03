---
title: "Spec-Host Category Standards"
description: "Standards for spec-host repositories - machine-first specification artifact hosting"
author: "Fulmen Enterprise Architect (@fulmen-ea-steward)"
date: "2025-12-20"
last_updated: "2025-12-20"
status: "active"
tags: ["standards", "repository-category", "spec-host", "publishing", "v0.2.26"]
---

# Spec-Host Category Standards

## Purpose

This document defines the requirements and standards for `spec-host` category repositories. Spec-host repositories provide machine-first static hosting for versioned specification artifacts (JSON Schema, OpenAPI, AsyncAPI) with canonical URL resolution as the primary invariant.

## Category Definition

**Key**: `spec-host`

**Summary**: Machine-first static hosting for versioned specification artifacts.

**Primary invariant**: Canonical IDs embedded in spec assets (`$id`, `x-fulmen-id`) MUST resolve over HTTPS to the asset content.

## Differentiation from Codex

| Aspect           | spec-host                               | Codex                        |
| ---------------- | --------------------------------------- | ---------------------------- |
| Primary consumer | Machines (validators, IDEs, generators) | Humans (developers, writers) |
| Content          | Raw specs (JSON/YAML)                   | Rich documentation (MDX)     |
| Build            | None or minimal publisher               | Full SSG (Astro/Starlight)   |
| UI               | Optional index page                     | Required navigation/search   |
| Dependencies     | Zero runtime                            | Framework-dependent          |
| Core invariant   | Canonical URL resolution                | Human-readable presentation  |

**Relationship**: Codex is "human-first docs/UI"; spec-host is "canonical raw spec hosting." A Codex site MAY layer browsable UI over a spec-host corpus, but the spec-host provides the machine-resolvable foundation.

## Requirements

### MUST

1. **Resolve canonical IDs over HTTPS**: Every `$id` (JSON Schema) or `x-fulmen-id` (OpenAPI/AsyncAPI) MUST resolve to the asset at that URL.

2. **Use versioned paths for immutability**: Assets MUST be organized under version prefixes (e.g., `v1.0.0/`, `v2025.12.0/`). The version scheme is corpus-specific (SemVer, CalVer, or custom).

3. **Be static files only**: No runtime server required. The publish tree is deployable to any static hosting provider.

4. **Implement Spec Publishing Standard**: Follow all requirements in [Spec Publishing Standard](../../publishing/spec-publishing.md).

### SHOULD

1. **Provide `index.json` catalog**: A machine-readable catalog of all published assets for discovery. See [Spec Catalog Schema](../../../../schemas/standards/publishing/v1.0.0/spec-catalog.schema.json).

2. **Use recommended file naming**: `*.schema.json` for JSON Schema, `*.openapi.json` for OpenAPI, `*.asyncapi.json` for AsyncAPI.

### MAY

1. **Provide human-readable index page**: A simple static HTML page listing available specs/versions.

2. **Provide `latest/` convenience alias**: If provided, MUST be implemented via CI-copy or host redirects (not symlinks). Production consumers MUST NOT use `latest/`.

## Repository Structure

Recommended layout:

```
spec-host-repo/
├── v1.0.0/
│   ├── core.schema.json
│   ├── events.schema.json
│   └── api.openapi.json
├── v1.1.0/
│   ├── core.schema.json
│   ├── events.schema.json
│   └── api.openapi.json
├── latest/                    # Optional: CI-managed copy
│   └── ...
├── index.json                 # Optional: asset catalog
├── index.html                 # Optional: human-readable index
└── .github/
    └── workflows/
        └── publish.yml        # CI pipeline calling publisher tool
```

## Publishing Workflow

Spec-host repositories implement the publishing workflow defined in the [Spec Publishing Standard](../../publishing/spec-publishing.md):

1. **Corpus discovery**: Publisher tool discovers spec assets in source directories.
2. **Canonical ID extraction**: Extract `$id` or `x-fulmen-id` from each asset.
3. **Validation**: Verify domain match, uniqueness, path safety.
4. **Materialization**: Generate publish tree with assets at canonical paths.
5. **Deployment**: Deploy static files to hosting provider.

## Template Naming

Forge templates for spec-host follow the pattern:

```
forge-spec-host-{name}
```

Where `{name}` reflects the corpus being hosted (naming pattern TBD).

## Examples

### Crucible Schemas

Publishing Crucible's `schemas/` directory to `schemas.fulmenhq.dev`:

- **Source**: `crucible/schemas/`
- **Host domain**: `schemas.fulmenhq.dev`
- **Example canonical ID**: `https://schemas.fulmenhq.dev/enact/v1.0.0/configuration/recipe.schema.json`

### Enact Specs

Publishing Enact specification artifacts:

- **Source**: `enact/specs/`
- **Host domain**: `specs.enact.fulmenhq.dev`
- **Example canonical ID**: `https://specs.enact.fulmenhq.dev/v1.0.0/deployment.openapi.json`

## Related Documentation

- [Spec Publishing Standard](../../publishing/spec-publishing.md) — Publishing workflow contract
- [Schema Normalization](../../schema-normalization.md) — `$id` requirements for JSON Schema
- [Repository Category Standards](../README.md) — Overview of all categories
- [Codex Category Standards](../codex/config-standard.md) — Codex configuration (for comparison)

---

**Status**: Active (v0.2.26+)

**Maintainers**: Crucible Team
