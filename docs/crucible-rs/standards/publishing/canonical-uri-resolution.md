---
title: "Canonical URI Resolution Standard"
description: "Standard for mapping canonical specification URIs to local file paths across Fulmen ecosystem"
author: "Fulmen Enterprise Architect (@fulmen-ea-steward)"
date: "2026-01-06"
last_updated: "2026-01-06"
status: "draft"
tags:
  [
    "standards",
    "publishing",
    "resolution",
    "schemas",
    "offline-access",
    "v0.4.1",
  ]
---

# Canonical URI Resolution Standard

## Summary

This standard defines the canonical URI structure for FulmenHQ specification assets and the resolver contract that enables offline access through helper libraries. It establishes the foundation for spechost (schema registry) and dochost (documentation hosting) while ensuring libraries can resolve canonical URIs to local embedded assets.

## Scope

This standard applies to:

- **Schemas**: JSON Schema, OpenAPI, AsyncAPI published to `schemas.fulmenhq.dev`
- **Documentation**: Standards, guides, SOPs (future: `docs.fulmenhq.dev`)
- **Configuration**: Config defaults, catalogs (future: `config.fulmenhq.dev`)

It defines requirements for:

1. **Canonical URI authors**: How to construct `$id` and `x-fulmen-id` values
2. **Spechost/dochost publishers**: How URIs map to published paths
3. **Library implementers**: How to resolve URIs to local embedded assets

## Goals

- **Canonical identification**: Every asset has exactly one authoritative URI
- **Offline resolution**: Libraries resolve URIs without network access
- **Module namespacing**: Clear distinction between assets from different repos
- **Predictable mapping**: Deterministic URI-to-path transformation

## Non-Goals

- Defining the physical hosting infrastructure (Cloudflare, S3, etc.)
- Replacing the [Spec Publishing Standard](spec-publishing.md) (this standard complements it)
- Mandating specific library implementations (contract only)

## Definitions

| Term              | Definition                                                                                   |
| ----------------- | -------------------------------------------------------------------------------------------- |
| **Canonical URI** | The authoritative HTTPS URL identity embedded in an asset (`$id`, `x-fulmen-id`)             |
| **Module**        | The repository/project namespace in a canonical URI (e.g., `crucible`, `gofulmen`, `goneat`) |
| **Topic**         | The subject-area path segment(s) after module (e.g., `observability/logging`, `pathfinder`)  |
| **Resolver**      | A library component that maps canonical URIs to local file paths                             |
| **Spechost**      | The static host serving specification assets at canonical URIs                               |
| **Dochost**       | The static host serving documentation assets at canonical URIs                               |

## Canonical URI Structure

### Schema URIs (schemas.fulmenhq.dev)

```
https://schemas.fulmenhq.dev/<module>/<topic>/<version>/<filename>
```

| Segment    | Required | Description                       | Examples                                                  |
| ---------- | -------- | --------------------------------- | --------------------------------------------------------- |
| `module`   | **Yes**  | Repository/project namespace      | `crucible`, `gofulmen`, `rsfulmen`, `goneat`, `groningen` |
| `topic`    | **Yes**  | Subject-area path (may be nested) | `observability/logging`, `pathfinder`, `library/foundry`  |
| `version`  | **Yes**  | SemVer version directory          | `v1.0.0`, `v2.0.0`                                        |
| `filename` | **Yes**  | Schema filename with extension    | `logger-config.schema.json`, `recipe.schema.json`         |

**Examples**:

```
# Crucible logging schema
https://schemas.fulmenhq.dev/crucible/observability/logging/v1.0.0/logger-config.schema.json

# Crucible foundry catalog
https://schemas.fulmenhq.dev/crucible/library/foundry/v1.0.0/country-codes.schema.json

# Gofulmen pathfinder schema (if gofulmen defined its own)
https://schemas.fulmenhq.dev/gofulmen/pathfinder/v2.0.0/finder-result.schema.json

# Goneat config schema
https://schemas.fulmenhq.dev/goneat/config/v1.0.0/release-phase.schema.json
```

### Documentation URIs (docs.fulmenhq.dev) - Future

```
https://docs.fulmenhq.dev/<module>/<path>
```

| Segment  | Required | Description                  | Examples                             |
| -------- | -------- | ---------------------------- | ------------------------------------ |
| `module` | **Yes**  | Repository/project namespace | `crucible`, `gofulmen`               |
| `path`   | **Yes**  | Document path with extension | `standards/observability/logging.md` |

### Configuration URIs (config.fulmenhq.dev) - Future

```
https://config.fulmenhq.dev/<module>/<category>/<version>/<filename>
```

## Module Namespace Rules

### Rule 1: Module MUST Match Source Repository

The `module` segment MUST be the repository name where the asset is **authored and maintained**.

| Repository           | Module      | Assets                            |
| -------------------- | ----------- | --------------------------------- |
| `fulmenhq/crucible`  | `crucible`  | SSOT schemas, standards, catalogs |
| `fulmenhq/gofulmen`  | `gofulmen`  | Go-specific schemas (if any)      |
| `fulmenhq/rsfulmen`  | `rsfulmen`  | Rust-specific schemas (if any)    |
| `fulmenhq/goneat`    | `goneat`    | Goneat config schemas             |
| `fulmenhq/groningen` | `groningen` | Template-specific schemas         |

**Rationale**: This enables libraries to distinguish between assets they embed from Crucible vs. assets they define locally.

### Rule 2: Helper Libraries Embed Crucible Module

Helper libraries (gofulmen, rsfulmen, tsfulmen, pyfulmen) embed Crucible assets and MUST preserve the `crucible` module namespace for those assets.

```
# In rsfulmen, resolving a Crucible schema:
https://schemas.fulmenhq.dev/crucible/observability/logging/v1.0.0/logger-config.schema.json
→ crucible-rs/schemas/observability/logging/v1.0.0/logger-config.schema.json

# In rsfulmen, resolving a library-native schema (if any):
https://schemas.fulmenhq.dev/rsfulmen/async/v1.0.0/task-result.schema.json
→ schemas/async/v1.0.0/task-result.schema.json
```

### Rule 3: Module-Qualified Local Paths

When a library embeds assets from multiple modules, local paths SHOULD include the module as a path prefix:

```
<library-root>/
├── crucible-<lang>/           # Synced from crucible (module: crucible)
│   ├── schemas/
│   ├── docs/
│   └── config/
└── schemas/                   # Library-native (module: <library>)
    └── ...
```

## Resolver Contract

### Overview

A **Resolver** is a library component that maps canonical URIs to local file paths. Every helper library implementing the [Crucible Shim Standard](../library/modules/crucible-shim.md) SHOULD provide resolution capabilities.

### Required Capabilities

| Capability                | Description                                         |
| ------------------------- | --------------------------------------------------- |
| `ResolveSchema(uri)`      | Maps schema URI to local path and returns content   |
| `CanResolve(uri)`         | Returns true if URI can be resolved locally         |
| `ListResolvableModules()` | Returns list of module namespaces available locally |

### Resolution Algorithm

Given canonical URI `https://schemas.fulmenhq.dev/<module>/<topic>/<version>/<filename>`:

1. **Extract components**: Parse URI into `module`, `topic`, `version`, `filename`
2. **Module lookup**: Determine local base path for module
   - If `module == "crucible"` → use embedded Crucible path (e.g., `crucible-rs/schemas/`)
   - If `module == self` → use library-native schema path
   - Otherwise → return `ModuleNotAvailable` error
3. **Construct path**: `<base>/<topic>/<version>/<filename>`
4. **Verify existence**: Check file exists, return content or `SchemaNotFound`

### Cross-Module Resolution

Libraries MAY support resolution of multiple modules:

```rust
// rsfulmen resolver pseudocode
fn resolve_schema(uri: &str) -> Result<Schema, ResolveError> {
    let (module, topic, version, filename) = parse_schema_uri(uri)?;

    let base_path = match module.as_str() {
        "crucible" => "crucible-rs/schemas",
        "rsfulmen" => "schemas",
        _ => return Err(ResolveError::ModuleNotAvailable(module)),
    };

    let path = format!("{}/{}/{}/{}", base_path, topic, version, filename);
    read_and_parse_schema(&path)
}
```

### Fallback Behavior

When local resolution fails, libraries MAY:

1. **Fail fast** (recommended for offline-first): Return error immediately
2. **Network fallback** (optional): Attempt HTTPS fetch from canonical URI
3. **Cache result**: If network fallback succeeds, cache for future offline use

Libraries MUST document which behavior they implement.

## $id Construction Requirements

### For Crucible Schemas

All schemas in the Crucible repository MUST use the `crucible` module:

```json
{
  "$id": "https://schemas.fulmenhq.dev/crucible/<topic>/<version>/<filename>"
}
```

### Filename Conventions

| Asset Type  | Suffix           | Example                     |
| ----------- | ---------------- | --------------------------- |
| JSON Schema | `.schema.json`   | `logger-config.schema.json` |
| OpenAPI     | `.openapi.json`  | `api.openapi.json`          |
| AsyncAPI    | `.asyncapi.json` | `events.asyncapi.json`      |
| Taxonomy    | `.schema.json`   | `language-key.schema.json`  |

### Version in Path vs. Filename

**Canonical pattern**: Version in path directory, not filename.

```
# CORRECT: Version in path
https://schemas.fulmenhq.dev/crucible/observability/logging/v1.0.0/logger-config.schema.json

# DEPRECATED: Version in filename (legacy pattern)
https://schemas.fulmenhq.dev/crucible/observability/logging/logger-config-v1.0.0.json
```

New schemas MUST use version-in-path. Existing schemas MAY retain version-in-filename for backward compatibility but SHOULD migrate during major version bumps.

## Local Path Mapping

### Crucible Repository Layout

```
crucible/
└── schemas/
    └── <topic>/
        └── <version>/
            └── <filename>
```

Maps to URI:

```
https://schemas.fulmenhq.dev/crucible/<topic>/<version>/<filename>
```

### Helper Library Layout

```
<lang>fulmen/
├── crucible-<lang>/           # Synced Crucible assets
│   └── schemas/
│       └── <topic>/
│           └── <version>/
│               └── <filename>
└── schemas/                   # Library-native assets (if any)
    └── <topic>/
        └── <version>/
            └── <filename>
```

Maps to URIs:

```
# Crucible assets
https://schemas.fulmenhq.dev/crucible/<topic>/<version>/<filename>
→ crucible-<lang>/schemas/<topic>/<version>/<filename>

# Library-native assets
https://schemas.fulmenhq.dev/<lang>fulmen/<topic>/<version>/<filename>
→ schemas/<topic>/<version>/<filename>
```

## Migration Guidance

### Fixing Inconsistent $id Values

Existing schemas with non-conforming `$id` values SHOULD be migrated:

| Current Pattern                                        | Correct Pattern                                          |
| ------------------------------------------------------ | -------------------------------------------------------- |
| `https://schemas.fulmenhq.dev/enact/v1.0.0/...`        | `https://schemas.fulmenhq.dev/crucible/enact/v1.0.0/...` |
| `https://schemas.fulmenhq.dev/gofulmen/pathfinder/...` | `https://schemas.fulmenhq.dev/crucible/pathfinder/...`   |
| `https://schemas.3leaps.net/goneat/...`                | `https://schemas.fulmenhq.dev/goneat/...`                |
| `https://github.com/fulmenhq/...`                      | `https://schemas.fulmenhq.dev/crucible/...`              |

Migration should occur during:

- Major version bumps (breaking change acceptable)
- New schema versions (preserve old URIs in deprecated versions)

### Backward Compatibility

Spechost deployments SHOULD maintain redirects from legacy URIs to canonical URIs during transition periods.

## Validation Requirements

### Schema Authors

Before committing schemas:

1. Verify `$id` follows canonical structure
2. Verify `module` matches source repository
3. Verify path segments align with file location

### CI/CD

Spec publishing tools MUST validate:

1. All `$id` values use `https://schemas.fulmenhq.dev/`
2. Module segment is valid (known repository)
3. No duplicate canonical URIs
4. File path matches URI path structure

## Examples

### Resolver Implementation (Conceptual)

**Go**:

```go
func (r *Resolver) ResolveSchema(uri string) (*Schema, error) {
    parsed, err := ParseSchemaURI(uri)
    if err != nil {
        return nil, fmt.Errorf("invalid schema URI: %w", err)
    }

    basePath, ok := r.modulePaths[parsed.Module]
    if !ok {
        return nil, &ModuleNotAvailableError{Module: parsed.Module}
    }

    path := filepath.Join(basePath, parsed.Topic, parsed.Version, parsed.Filename)
    return r.loadSchema(path)
}
```

**Python**:

```python
def resolve_schema(self, uri: str) -> Schema:
    parsed = parse_schema_uri(uri)

    base_path = self.module_paths.get(parsed.module)
    if base_path is None:
        raise ModuleNotAvailableError(parsed.module)

    path = base_path / parsed.topic / parsed.version / parsed.filename
    return self.load_schema(path)
```

### Library Registration

```yaml
# rsfulmen resolver configuration
modules:
  crucible:
    base_path: "crucible-rs/schemas"
    source: "synced from fulmenhq/crucible"
  rsfulmen:
    base_path: "schemas"
    source: "library-native"
```

## Related Documentation

- [Spec Publishing Standard](spec-publishing.md) - Publishing workflow and canonical ID rules
- [Spec-Host Category Standards](../repository-category/spec-host/README.md) - For self-describing specifications with embedded IDs
- [Doc-Host Category Standards](../repository-category/doc-host/README.md) - For path-addressed assets without embedded IDs
- [Crucible Shim Standard](../library/modules/crucible-shim.md) - Asset access contract
- [ADR-0012: Absolute $id URLs](../../architecture/decisions/ADR-0012-schema-ref-ids.md) - Cross-schema reference decision
- [Schema Normalization Standard](../schema-normalization.md) - Format canonicalization

---

**Status**: Draft (v0.4.1+)

**Maintainers**: Crucible Team, Enterprise Architect
