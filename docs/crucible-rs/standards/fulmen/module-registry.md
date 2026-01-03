---
title: "Module Registry Standard"
description: "Weight classification and feature gate semantics for Fulmen helper library modules"
author: "entarch"
date: "2026-01-05"
status: "active"
tags: ["standards", "modules", "feature-gates", "weight-classification"]
---

# Module Registry Standard

This document defines the semantics for module weight classification, default inclusion flags, and feature gate mapping in the Fulmen helper library ecosystem.

## Overview

The module registry provides SSOT metadata for helper library modules, enabling consistent feature gate implementation across all languages. Two registries exist:

| Registry         | Purpose                                            | Location                                                        |
| ---------------- | -------------------------------------------------- | --------------------------------------------------------------- |
| Platform Modules | Code modules (config, logging, schema, etc.)       | `config/taxonomy/library/platform-modules/v1.1.0/modules.yaml`  |
| Foundry Catalogs | Reference data catalogs (countries, signals, etc.) | `config/taxonomy/library/foundry-catalogs/v1.1.0/catalogs.yaml` |

## Key Fields

### Tier

Indicates universality and expected usage across the ecosystem.

| Value         | Meaning                                               |
| ------------- | ----------------------------------------------------- |
| `core`        | Required by all applications; foundational capability |
| `common`      | Default install; widely useful                        |
| `specialized` | Opt-in; niche use cases or heavy dependencies         |

**Note**: Tier is about _universality_, not dependency footprint. A `core` module can be `heavy` (e.g., schema validation).

### Weight

Classifies dependency footprint for feature gate decisions.

| Value   | Meaning                                        |
| ------- | ---------------------------------------------- |
| `light` | Stdlib or serde-class dependencies only        |
| `heavy` | Significant dependency tree beyond serde-class |

### Default Inclusion

Whether the module is included in default/standard builds.

| Value   | Meaning                                      |
| ------- | -------------------------------------------- |
| `true`  | Included unless explicitly excluded          |
| `false` | Excluded unless explicitly included (opt-in) |

### Feature Group (Foundry Catalogs Only)

Maps catalogs to library feature gates.

| Value              | Catalogs                                      | Typical Cargo Feature |
| ------------------ | --------------------------------------------- | --------------------- |
| `foundry-core`     | signals, exit-codes, countries, http-statuses | `foundry-core`        |
| `foundry-patterns` | patterns                                      | `foundry-patterns`    |
| `foundry-mime`     | mime-types                                    | `foundry-mime-types`  |

## Weight Classification Criteria

### Serde-Class Dependencies (Exempt)

These dependencies don't count toward "heavy" classification:

| Language   | Exempt Dependencies                                                     |
| ---------- | ----------------------------------------------------------------------- |
| Rust       | `serde`, `serde_json`, `serde_yaml`, `once_cell`, `thiserror`, `anyhow` |
| Python     | `pydantic`, `pyyaml`, stdlib                                            |
| TypeScript | `yaml`, native JSON                                                     |
| Go         | stdlib                                                                  |

### Heavy Trigger Dependencies

These dependencies trigger "heavy" classification:

| Language   | Heavy Dependencies                                                                                |
| ---------- | ------------------------------------------------------------------------------------------------- |
| Rust       | `jsonschema`, `regex`, `glob`, `strsim`, `unicode-normalization`, `unicode-segmentation`, `icu-*` |
| Python     | `jsonschema`, `xxhash`, `google-crc32c`, `rapidfuzz`, `prometheus_client`                         |
| TypeScript | `ajv`, `ajv-formats`, `hash-wasm`, `archiver`, `tar-stream`, `@3leaps/string-metrics-wasm`        |
| Go         | Typically light due to stdlib-first approach                                                      |

### Native Extension Consideration

In Python and TypeScript, native extensions (C/Rust code) often correlate with "heavy" due to:

- Installation complexity (build tools, platform-specific binaries)
- Increased package size
- CI/CD complications

## Library Feature Gate Mapping

Each language maps registry metadata to its native feature system:

### Rust (Cargo Features)

```toml
[features]
default = ["core"]
core = ["foundry-core", "config"]
foundry-core = []                    # weight: light, default: true
foundry-patterns = ["dep:regex"]     # weight: heavy, default: false
similarity = ["dep:strsim"]          # weight: heavy, default: false
schema-validation = ["dep:jsonschema"] # weight: heavy, default: false
full = ["foundry-core", "foundry-patterns", "similarity", "schema-validation"]
```

### Python (Extras)

```toml
[project.optional-dependencies]
schema = ["jsonschema>=4.25"]        # weight: heavy, default: false
similarity = ["rapidfuzz>=3.0"]      # weight: heavy, default: false
hashing = ["xxhash>=3.5"]            # weight: heavy, default: true
telemetry = ["prometheus_client"]    # weight: heavy, default: false
full = ["pyfulmen[schema,similarity,hashing,telemetry]"]
```

**Naming Convention**:

- Use module name as extra name where 1:1 (`schema`, `similarity`, `telemetry`)
- Use `full` for everything
- Serde-class deps (`pydantic`, `pyyaml`) remain in base install

### TypeScript (Separate Entry Points)

```typescript
// Heavy modules have dedicated entry points
import { validateSchema } from "@fulmenhq/tsfulmen/schema"; // heavy
import { similarity } from "@fulmenhq/tsfulmen/similarity"; // heavy
import { getConfigDir } from "@fulmenhq/tsfulmen/config"; // light

// Light modules can be in main barrel (but heavy should NOT be)
import { signals, exitCodes } from "@fulmenhq/tsfulmen/foundry"; // light
```

**Pattern**: Separate entry points superior to `peerDependencies` for core functionality because:

- Tree-shaking works at module boundary
- No runtime feature detection needed
- Clear import paths signal dependency cost

### Go (Build Tags or Separate Packages)

Go typically has all-light modules. If heavy modules are needed:

- Separate packages: `gofulmen/schema/lite`
- Build tags: `//go:build !lite`

## Decision Guidelines

### When to Mark as Heavy

- Module pulls dependencies beyond serde-class exempt list
- Any language implementation has native extensions
- Total transitive dependency count > 5 (Python heuristic)

### When to Mark default_inclusion: false

- Module is `heavy` AND not universally needed
- Module is `specialized` tier
- Module is opt-in by nature (e.g., telemetry export)

**Exception**: Some heavy modules are `default_inclusion: true` because they're fundamental (e.g., `fulhash` for content hashing).

## Per-Language Notes

The `notes` field in language implementations provides build-specific guidance:

```yaml
languages:
  rust:
    status: available
    package: rsfulmen::schema_validation
    version: "0.1.0"
    implementation: "Wraps jsonschema crate for JSON Schema validation"
    notes: "Pulls jsonschema + url + icu-* tree; must be feature-gated"
```

Use notes for:

- Dependency tree warnings
- Feature gate requirements
- Installation considerations
- Platform-specific caveats

## Schema Versions

| Schema        | Version | Path                                                                         |
| ------------- | ------- | ---------------------------------------------------------------------------- |
| Module Entry  | v1.1.0  | `schemas/taxonomy/library/modules/v1.1.0/module-entry.schema.json`           |
| Catalog Entry | v1.1.0  | `schemas/taxonomy/library/foundry-catalogs/v1.1.0/catalog-entry.schema.json` |

## References

- Feature Brief: `.plans/active/v0.4.0/module-registry-weight-and-feature-gates-feature-brief.md`
- Library Memo: `.plans/memos/libraries/20260105-feature-flag-and-library-update.md`
- Platform Modules: `config/taxonomy/library/platform-modules/v1.1.0/modules.yaml`
- Foundry Catalogs: `config/taxonomy/library/foundry-catalogs/v1.1.0/catalogs.yaml`
- Helper Library Standard: `docs/architecture/fulmen-helper-library-standard.md`
