---
title: "Doc-Host Category Standards"
description: "Standards for doc-host repositories - path-addressed static asset hosting"
author: "Fulmen Enterprise Architect (@fulmen-ea-steward)"
date: "2026-01-06"
last_updated: "2026-01-06"
status: "active"
tags: ["standards", "repository-category", "doc-host", "publishing", "v0.4.1"]
---

# Doc-Host Category Standards

## Purpose

This document defines the requirements and standards for `doc-host` category repositories. Doc-host repositories provide machine-first static hosting for documentation, configuration, and other assets where canonical URLs are **path-derived** rather than **embedded** in the assets themselves.

## Category Definition

**Key**: `doc-host`

**Summary**: Path-addressed static hosting for documentation, configuration, and reference assets.

**Primary invariant**: Published file paths directly determine canonical URLs. No embedded identifiers required.

## The Spec-Host vs Doc-Host Distinction

The key distinction is **how canonical identity is established**:

| Aspect                    | spec-host                                         | doc-host                         |
| ------------------------- | ------------------------------------------------- | -------------------------------- |
| **Identity source**       | Embedded in asset (`$id`, `x-fulmen-id`)          | Derived from file path           |
| **Meta-validation**       | Required (JSON Schema, OpenAPI, AsyncAPI specs)   | Not required                     |
| **Asset types**           | Self-describing specifications                    | Any static file                  |
| **Publishing complexity** | Higher (extract ID, validate, map to path)        | Lower (path = URL)               |
| **Example assets**        | `.schema.json`, `.openapi.json`, `.asyncapi.json` | `.md`, `.yaml`, `.json` (config) |

### Conceptual Framing

**spec-host**: Hosts _self-describing specifications_ — assets that contain their own canonical identity and can be validated against industry-standard meta-schemas.

**doc-host**: Hosts _path-addressed assets_ — files whose identity is determined by their location, not by embedded metadata. No assumption of meta-validation capability.

### Why Two Categories?

1. **Different publishing contracts**: spec-host requires ID extraction and validation; doc-host uses simpler path-based publishing
2. **Different consumer expectations**: spec-host consumers expect resolvable `$id`; doc-host consumers just fetch by path
3. **Different validation requirements**: spec-host implies assets are specifications; doc-host makes no such assumption

## Requirements

### MUST

1. **Serve files at canonical paths**: Files MUST be accessible at URLs matching their path in the publish tree.

2. **Use versioned paths where appropriate**: Assets with versioned content SHOULD be organized under version prefixes.

3. **Be static files only**: No runtime server required. The publish tree is deployable to any static hosting provider.

4. **Use module namespace**: Follow the [Canonical URI Resolution Standard](../../publishing/canonical-uri-resolution.md) for module namespacing.

### SHOULD

1. **Provide index for discovery**: A machine-readable index (e.g., `index.json`, `manifest.yaml`) listing available assets.

2. **Use consistent file extensions**: Match content type to extension (`.md` for Markdown, `.yaml` for YAML, etc.).

3. **Preserve directory structure**: Published paths should mirror source directory structure for predictability.

### MAY

1. **Provide human-readable index page**: A simple static HTML page listing available assets.

2. **Support content negotiation**: Serve appropriate `Content-Type` headers based on file extension.

## Canonical URL Structure

Doc-host URLs follow the same module-based structure as spec-host:

```
https://docs.fulmenhq.dev/<module>/<path>
https://config.fulmenhq.dev/<module>/<path>
```

| Segment  | Required | Description                  | Examples                             |
| -------- | -------- | ---------------------------- | ------------------------------------ |
| `module` | **Yes**  | Repository/project namespace | `crucible`, `gofulmen`               |
| `path`   | **Yes**  | File path with extension     | `standards/observability/logging.md` |

### Example URLs

**Documentation (docs.fulmenhq.dev)**:

```
https://docs.fulmenhq.dev/crucible/standards/observability/logging.md
https://docs.fulmenhq.dev/crucible/guides/bootstrap-goneat.md
https://docs.fulmenhq.dev/crucible/architecture/fulmen-ecosystem-guide.md
```

**Configuration (config.fulmenhq.dev)**:

```
https://config.fulmenhq.dev/crucible/taxonomy/library/modules.yaml
https://config.fulmenhq.dev/crucible/terminal/v1.0.0/defaults.yaml
https://config.fulmenhq.dev/crucible/agentic/roles/entarch.yaml
```

## Repository Structure

Recommended layout:

```
doc-host-repo/
├── crucible/                    # Module namespace
│   ├── standards/
│   │   ├── observability/
│   │   │   └── logging.md
│   │   └── coding/
│   │       └── go.md
│   ├── guides/
│   │   └── bootstrap-goneat.md
│   └── config/
│       └── terminal/
│           └── v1.0.0/
│               └── defaults.yaml
├── index.json                   # Optional: asset catalog
├── index.html                   # Optional: human-readable index
└── .github/
    └── workflows/
        └── publish.yml          # CI pipeline
```

## Publishing Workflow

Doc-host publishing is simpler than spec-host:

1. **Corpus discovery**: Publisher tool discovers assets in source directories
2. **Path mapping**: Derive canonical URL from source path (no ID extraction needed)
3. **Validation** (optional): Verify file integrity, check for broken links
4. **Materialization**: Copy files to publish tree at path-derived locations
5. **Deployment**: Deploy static files to hosting provider

### No Meta-Validation Required

Unlike spec-host, doc-host does NOT require:

- Extracting embedded canonical IDs
- Validating assets against meta-schemas
- Verifying ID-to-path consistency

This makes doc-host suitable for:

- Markdown documentation
- YAML/JSON configuration files
- Reference data files
- Any static content without self-describing identity

## Relationship to Crucible Shim

Helper libraries resolve doc-host URLs to local embedded paths the same way as spec-host:

```python
# Resolving a doc-host URL
https://docs.fulmenhq.dev/crucible/standards/observability/logging.md
→ crucible-py/docs/standards/observability/logging.md
```

The [Canonical URI Resolution Standard](../../publishing/canonical-uri-resolution.md) applies equally to both categories.

## Differentiation from Other Categories

| Aspect           | doc-host                | spec-host                             | codex                       |
| ---------------- | ----------------------- | ------------------------------------- | --------------------------- |
| Primary consumer | Machines (shims, tools) | Machines (validators, IDEs)           | Humans (browsers)           |
| Content          | Docs, config, reference | Specifications (JSON Schema, OpenAPI) | Rich documentation          |
| Build            | None or minimal         | Publisher with ID validation          | Full SSG (Astro/Starlight)  |
| UI               | Optional index page     | Optional index page                   | Required navigation/search  |
| Meta-validation  | Not required            | Required                              | N/A                         |
| Core invariant   | Path = URL              | Embedded ID = URL                     | Human-readable presentation |

**Relationship**:

- A **codex** site MAY layer browsable UI over a **doc-host** corpus
- A **spec-host** MAY coexist with **doc-host** at different subdomains
- Both **spec-host** and **doc-host** serve the [Crucible Shim](../../library/modules/crucible-shim.md) resolution contract

## Template Naming

Forge templates for doc-host follow the pattern:

```
forge-doc-host-{name}
```

Where `{name}` reflects the content being hosted (naming pattern TBD).

## Examples

### Crucible Documentation

Publishing Crucible's `docs/` directory to `docs.fulmenhq.dev`:

- **Source**: `crucible/docs/`
- **Host domain**: `docs.fulmenhq.dev`
- **Module**: `crucible`
- **Example URL**: `https://docs.fulmenhq.dev/crucible/standards/observability/logging.md`

### Crucible Configuration

Publishing Crucible's `config/` directory to `config.fulmenhq.dev`:

- **Source**: `crucible/config/`
- **Host domain**: `config.fulmenhq.dev`
- **Module**: `crucible`
- **Example URL**: `https://config.fulmenhq.dev/crucible/agentic/roles/entarch.yaml`

## Related Documentation

- [Canonical URI Resolution Standard](../../publishing/canonical-uri-resolution.md) — URI structure and resolver contract
- [Spec-Host Category Standards](../spec-host/README.md) — For self-describing specifications
- [Crucible Shim Standard](../../library/modules/crucible-shim.md) — Asset access contract
- [Repository Category Standards](../README.md) — Overview of all categories

---

**Status**: Active (v0.4.1+)

**Maintainers**: Crucible Team
