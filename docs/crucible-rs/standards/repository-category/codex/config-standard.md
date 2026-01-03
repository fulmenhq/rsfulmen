---
title: "Codex Configuration Standard"
description: "Configuration schema and governance for Fulmen codex templates"
author: "Schema Cartographer"
date: "2025-11-03"
last_updated: "2025-11-03"
status: "active"
tags: ["standards", "codex", "configuration", "v0.2.3"]
---

# Codex Configuration Standard

## Purpose

Defines the standard configuration format for all Fulmen codex templates (documentation sites, schema registries, knowledge hubs). This schema enables:

- **Ecosystem consistency**: All codex templates share the same config format
- **Extension system**: Pluggable features for specialized functionality
- **CRDL standardization**: Predictable configuration for Clone → Degit → Refit → Launch workflows
- **Build-time validation**: Templates validate user configurations against Crucible schema

## Schema Location

**Schema**: `schemas/config/repository-category/codex/v1.0.0/codex-config.schema.json`

**$id**: `https://schemas.fulmenhq.dev/config/repository-category/codex/v1.0.0/codex-config.schema.json`

**Example**: `examples/config/repository-category/codex/v1.0.0/codex-config.example.json`

**Specification**: JSON Schema 2020-12

## Configuration Structure

### Required: Project Metadata

All codex templates MUST define project metadata:

```json
{
  "project": {
    "name": "Forge Codex Pulsar",
    "description": "Production-grade documentation and knowledge hub",
    "type": "codex",
    "baseUrl": "https://docs.example.dev"
  }
}
```

**Fields**:

- `name` (required): Full formal name of the codex
- `type` (required): Codex site type (`codex`, `registry`, `docs`)
- `description` (optional): Brief description of purpose
- `baseUrl` (optional): Base URL for deployed site (used for $id generation, sitemaps, canonical URLs)

### Optional: Extensions

Extensions enable specialized functionality via opt-in activation:

```json
{
  "extensions": ["schema-registry"]
}
```

**Governance**: Extension names are controlled via schema `enum`. New extensions added through Crucible governance process (see Extension Governance below).

**Current Extensions**:

- `schema-registry`: JSON Schema catalog with validation and browsing UI

**Planned Extensions**:

- `api-reference`: OpenAPI/AsyncAPI documentation generator
- `interactive-playground`: Live code examples and sandboxes
- `multi-repo-aggregator`: Aggregate documentation from multiple repositories

### Optional: Features

Core features available to all codex templates:

```json
{
  "features": {
    "search": {
      "provider": "starlight-default"
    },
    "i18n": {
      "enabled": false,
      "defaultLocale": "en"
    },
    "analytics": {
      "enabled": false
    }
  }
}
```

**Available Features**:

- **search**: Search functionality (providers: `starlight-default`, `algolia`, `pagefind`)
- **i18n**: Internationalization (multi-language support)
- **analytics**: Analytics integration (providers: `plausible`, `umami`, `google-analytics`)

### Extension Configuration

Each extension MAY define a top-level configuration object. Extension config is only active when extension is enabled in `extensions` array.

**Example: schema-registry Extension**:

```json
{
  "extensions": ["schema-registry"],
  "schemaRegistry": {
    "baseUrl": "https://schemas.example.dev",
    "formats": ["json-schema"],
    "sourcePaths": ["schemas/"],
    "catalogOutput": "catalog/index.json",
    "validation": {
      "enforceIdMatch": true,
      "strictMode": true
    },
    "smokeTests": {
      "enabled": false,
      "sampleSize": 10
    }
  }
}
```

**Extension Config Requirements**:

1. **Conditional**: Only processed when extension enabled
2. **Validated**: Schema defines structure for each extension
3. **Documented**: Extension documentation explains all options
4. **Tested**: Templates test extension configurations

## Extension System Design

### Rationale

Codex templates serve diverse use cases (docs sites, schema registries, API references). Not all users need all features. Extensions:

- Avoid bundle bloat (opt-in features)
- Enable independent evolution (extensions update separately)
- Support community contributions (ecosystem growth)
- Specialize for use cases (schema validation vs API docs vs knowledge hubs)

### Extension Contract

Each extension MUST provide:

1. **Name**: Added to schema `extensions.items.enum`
2. **Config Block**: Top-level object in schema (e.g., `schemaRegistry`)
3. **Implementation**: Code in template (loaded conditionally when extension enabled)
4. **Documentation**: Usage guide in Crucible or template README
5. **Tests**: Validation that extension configuration works

### Extension Activation

Templates SHOULD:

1. Read `extensions` array from user config
2. Validate extension config blocks (using schema validation)
3. Load extension implementations only when enabled
4. Fail build if enabled extension has invalid configuration

**Example Activation Logic** (pseudocode):

```typescript
const config = loadConfig("codex.config.json");
validateConfig(config, crucibleSchema);

if (config.extensions.includes("schema-registry")) {
  const registry = await import("./extensions/schema-registry");
  await registry.setup(config.schemaRegistry);
}
```

## Extension Governance

### Adding New Extensions

**Process**:

1. **Proposal**: Extension author proposes in Crucible (GitHub issue or PR)
2. **Review**: Crucible maintainers review extension contract and governance fit
3. **Schema Update**: Add extension name to `extensions.items.enum`
4. **Config Block**: Add extension config block schema to main schema
5. **Documentation**: Document in this file or extension-specific guide
6. **Release**: Include in next Crucible version

**Example PR**: "feat(codex): add 'api-reference' extension"

### Extension Criteria

Extensions SHOULD:

- Serve a clear use case not covered by core features
- Be generally useful across multiple codex templates
- Have a maintainer committed to supporting it
- Follow Fulmen coding and documentation standards
- Include tests and examples

Extensions SHOULD NOT:

- Duplicate existing core features
- Be overly specific to a single organization
- Introduce breaking changes to existing configs
- Bypass schema validation

### Extension Registry

**Current Extensions** (v1.0.0):

| Extension         | Status | Purpose                                          | Config Block     |
| ----------------- | ------ | ------------------------------------------------ | ---------------- |
| `schema-registry` | Active | JSON Schema catalog with validation and browsing | `schemaRegistry` |

**Planned Extensions** (proposed):

| Extension                | Status   | Purpose                                  | Config Block            |
| ------------------------ | -------- | ---------------------------------------- | ----------------------- |
| `api-reference`          | Proposed | OpenAPI/AsyncAPI documentation generator | `apiReference`          |
| `interactive-playground` | Proposed | Live code examples and sandboxes         | `interactivePlayground` |
| `multi-repo-aggregator`  | Proposed | Aggregate docs from multiple repos       | `multiRepoAggregator`   |

## Versioning Strategy

### v1.0.0 (Current - Bootstrap)

- Core project metadata
- Extension system foundation
- Common features (search, i18n, analytics)
- `schema-registry` extension

### v1.1.0 (Future - Additive)

- New extensions (e.g., `api-reference`)
- New core features (e.g., `themes`)
- Backward compatible with v1.0.0

### v2.0.0 (Future - Breaking)

- Remove deprecated fields (e.g., `schemaRegistry.enabled`)
- Change extension activation mechanism (if needed)
- Requires migration guide

## Template Integration

### Sync from Crucible

Codex templates SHOULD sync schema from Crucible SSOT:

**Sync Target**: `{template}/schemas/crucible/config/repository-category/codex/v1.0.0/`

**Example**: `forge-codex-pulsar/schemas/crucible/config/repository-category/codex/v1.0.0/codex-config.schema.json`

**Sync Process**:

1. Template includes Crucible sync script (similar to tsfulmen)
2. Pre-commit or CI validates schemas are up-to-date
3. Users reference synced schema in their config files

### User Configuration

Users create `codex.config.json` in template root:

```json
{
  "$schema": "./schemas/crucible/config/repository-category/codex/v1.0.0/codex-config.schema.json",
  "project": {
    "name": "My Documentation Site",
    "type": "docs",
    "baseUrl": "https://docs.myproject.dev"
  },
  "extensions": [],
  "features": {
    "search": {
      "provider": "starlight-default"
    }
  }
}
```

**IDE Support**: `$schema` reference enables:

- Autocomplete in VS Code, JetBrains IDEs
- Inline validation and error messages
- Hover documentation for fields

### Build-Time Validation

Templates SHOULD validate user configuration at build time:

**Validation Flow**:

1. Read user's `codex.config.json`
2. Validate against synced Crucible schema (using Ajv or similar)
3. Fail build with helpful error messages if invalid
4. Load enabled extensions based on `extensions` array

**Example Validation** (TypeScript):

```typescript
import Ajv from "ajv";
import schema from "./schemas/crucible/config/repository-category/codex/v1.0.0/codex-config.schema.json";

const ajv = new Ajv();
const validate = ajv.compile(schema);

const userConfig = JSON.parse(fs.readFileSync("codex.config.json", "utf-8"));

if (!validate(userConfig)) {
  console.error("Invalid codex configuration:");
  console.error(validate.errors);
  process.exit(1);
}
```

## CRDL Workflow

**CRDL** (Clone → Degit → Refit → Launch) users benefit from standardized config:

### Clone

```bash
git clone https://github.com/fulmenhq/forge-codex-pulsar.git my-docs
cd my-docs
```

### Degit

```bash
rm -rf .git
git init
```

### Refit

```bash
# Copy example config
cp config/codex.config.example.json codex.config.json

# Edit configuration
# - Change project.name to "My Documentation Site"
# - Change project.baseUrl to your URL
# - Enable extensions if needed
```

### Launch

```bash
bun install
bun dev

# Build validates config against Crucible schema
# Invalid config fails build with helpful errors
```

## Maintenance Responsibility

**Primary**: Crucible maintainers (SSOT)

**Secondary**: Codex template maintainers (consumers, feedback loop)

**Review Cadence**: Quarterly or when new extensions proposed

## Related Documentation

- [Fulmen Forge Codex Standard](../../architecture/fulmen-forge-codex-standard.md) - High-level codex architecture
- [Repository Category Taxonomy](../../../schemas/taxonomy/repository-category/v1.0.0/category-key.schema.json) - Category definitions
- [Example Configuration](../../../examples/config/repository-category/codex/v1.0.0/codex-config.example.json) - Reference implementation

## Compliance

### Schema Naming Pattern

**Pattern**: `schemas/config/repository-category/{category}/{version}/{name}.schema.json`

**This Schema**: `schemas/config/repository-category/codex/v1.0.0/codex-config.schema.json`

**Aligns With**:

- Repository category taxonomy (`schemas/taxonomy/repository-category/v1.0.0/`)
- Existing config patterns (`schemas/config/fulmen-ecosystem/v1.0.0/`)

### $id Pattern

**Pattern**: `https://schemas.fulmenhq.dev/{path}/{name}.schema.json`

**This $id**: `https://schemas.fulmenhq.dev/config/repository-category/codex/v1.0.0/codex-config.schema.json`

### Specification

**Standard**: JSON Schema 2020-12 (draft 2020-12)

**Validation**: All schemas use `"$schema": "https://json-schema.org/draft/2020-12/schema"`

---

**Status**: Active (v1.0.0)

**Last Updated**: 2025-11-03

**Maintainers**: Crucible Team, Forge Codex Pulsar Team
