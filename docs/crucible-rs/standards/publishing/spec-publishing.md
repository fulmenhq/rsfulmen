---
title: "Spec Publishing Standard"
description: "Standard for publishing specification artifacts with resolvable canonical identifiers"
author: "Fulmen Enterprise Architect (@fulmen-ea-steward)"
date: "2025-12-20"
last_updated: "2025-12-20"
status: "active"
tags: ["standards", "publishing", "schemas", "openapi", "asyncapi", "v0.2.26"]
---

# Spec Publishing Standard

## Summary

This standard defines a stable, low-friction, **machine-first** way to publish canonical URLs for structured specification assets so that identifiers embedded in those assets resolve over HTTPS.

## Scope

This standard applies to publishing:

- **JSON Schema** (`$id` is canonical)
- **OpenAPI** (canonical ID via `x-fulmen-id` extension)
- **AsyncAPI** (canonical ID via `x-fulmen-id` extension and/or native `id`)
- (Future) other structured corpora (e.g., dbt models, typed catalogs) that benefit from canonical URLs

## Goals

- **Canonical URL resolution**: Every published asset's canonical identifier resolves over HTTPS.
- **Deterministic publishing**: Identical inputs produce identical outputs (byte-for-byte where possible).
- **Cross-spec consistency**: A unified policy across JSON Schema / OpenAPI / AsyncAPI.
- **Low friction**: Authors keep files in repos; publishing is a build step, not manual hand-editing.
- **M2M-first**: No UI/styling requirements; correctness and content-type behavior matter.

## Non-Goals

- Building a human-facing docs portal (Codex) for browsing specs (can layer later).
- Defining deployment tooling (Cloudflare/S3/Netlify/etc. are implementation choices).
- Solving auth/ACL for private corpora (policy can exist, but this standard is about layout + correctness).

## Definitions

| Term             | Definition                                                                         |
| ---------------- | ---------------------------------------------------------------------------------- |
| **Spec asset**   | A file that represents a formal specification (JSON Schema/OpenAPI/AsyncAPI/etc.). |
| **Canonical ID** | The authoritative URL identity embedded in the asset.                              |
| **Host domain**  | The HTTPS host under which canonical IDs resolve (e.g., `schemas.fulmenhq.dev`).   |
| **Publish tree** | The static directory tree that is deployed to the host domain.                     |

## Canonical ID Rules

### Rule 1: Canonical ID MUST Be an Absolute HTTPS URL

- Scheme MUST be `https`.
- Host MUST match the configured host domain for that publish operation.

### Rule 2: Canonical ID MUST Map to a Published Path

Given canonical ID URL:

```
https://<host-domain>/<path>
```

The publish tree MUST contain the file at:

```
/<path>
```

The publishing tool MUST NOT invent alternate paths. The canonical ID drives the path.

### Rule 3: Single Source of Truth per Asset Type

The publishing tool MUST derive the canonical ID deterministically:

| Asset Type  | Canonical ID Source                                |
| ----------- | -------------------------------------------------- |
| JSON Schema | `$id` (required)                                   |
| OpenAPI     | `x-fulmen-id` (required)                           |
| AsyncAPI    | `x-fulmen-id` (required), MAY also set native `id` |

**Rationale**: OpenAPI does not define a universal canonical-ID field; AsyncAPI's `id` is optional and inconsistently used. We standardize a Fulmen extension to make this reliable.

## Fulmen Extension: `x-fulmen-id`

For non-JSON-Schema spec formats, Fulmen defines:

```yaml
x-fulmen-id: "https://schemas.fulmenhq.dev/<path>"
```

**Requirements**:

- MUST be present for OpenAPI and AsyncAPI assets that are intended to be published.
- MUST be unique within the publish corpus.
- MUST match the asset's published path (Rule 2).

**Optional**:

- AsyncAPI MAY also set native `id` equal to `x-fulmen-id` for ecosystem compatibility.

## File Format and Naming Conventions

Canonical URLs should be predictable and content-type friendly.

### Recommended Filename Suffixes

| Asset Type  | Suffix                               |
| ----------- | ------------------------------------ |
| JSON Schema | `*.schema.json`                      |
| OpenAPI     | `*.openapi.json` or `openapi.json`   |
| AsyncAPI    | `*.asyncapi.json` or `asyncapi.json` |

### Format Recommendations

- Prefer JSON for canonical URLs when feasible (avoid YAML parsing ambiguity for M2M consumers).
- If YAML is used, the canonical URL MUST end in `.yaml`/`.yml` and the publisher MUST preserve the file bytes exactly.

## Versioning Requirements

- Version segment is **mandatory** in canonical IDs for immutability.
- Version scheme is **corpus-specific** (SemVer, CalVer, or custom) — this standard does not prescribe a specific scheme.
- Examples: `v1.0.0/`, `v2025.12.0/`, `v3/`

### `latest/` Convention

- `latest/` is **optional** and MUST NOT use filesystem symlinks (incompatible with most static hosts).
- If provided, implement via:
  - CI-produced copy, or
  - Host-specific redirect rules (e.g., Cloudflare `_redirects`)
- **Warning**: Production consumers MUST NOT reference `latest/`. Use explicit versions.

## Publishing Tool Contract

A compliant publishing tool MUST implement:

### 1. Corpus Discovery

- Input is one or more repo directories (e.g., `schemas/`, `specs/`, `docs/specs/`).
- Discovery patterns MUST be explicit (no "scan everything" default).
- Tool MUST support mixed corpora (JSON Schema + OpenAPI + AsyncAPI).

### 2. Canonical ID Extraction

For each discovered asset:

- Parse minimal metadata needed to extract canonical ID:
  - JSON: parse object, read `$id` or `x-fulmen-id`
  - YAML: parse YAML to object, read `x-fulmen-id` (and/or AsyncAPI `id`)
- Fail if canonical ID is missing for publish-eligible assets.

### 3. Validation (Hard Fail)

Tool MUST fail the build if:

- Any canonical ID host does not match configured host domain
- Any two assets share the same canonical ID
- Any canonical ID path is unsafe (e.g., contains `..`, empty path, or disallowed characters)
- Any canonical ID does not map to a single output path

Tool SHOULD fail if:

- File extension and parsed content type mismatch (e.g., `.json` that is not valid JSON)
- Canonical ID does not end with a recognized suffix (policy-driven)

### 4. Materialization (Publish Tree)

The tool MUST produce a deterministic output directory containing:

- Each asset at the canonical path derived from its canonical ID
- Any required static hosting glue for HTTPS (optional, provider-specific; can be separate recipe layer)

### 5. Optional Catalog Index

If emitted, `index.json` SHOULD include:

- `canonical_url`: The canonical URL of the asset
- `spec_type`: One of `jsonschema`, `openapi`, `asyncapi`, `other`
- `media_type`: Best-effort media type
- `sha256`: SHA-256 digest of published bytes
- `source_repo`: Provenance pointer (non-secret)
- `source_commit`: Git commit SHA (optional)

See [Spec Catalog Schema](../../../schemas/standards/publishing/v1.0.0/spec-catalog.schema.json) for the formal definition.

## Deployment Requirements

Deployment mechanisms may vary:

- `ppgate` recipes/manifests
- Enact workflows
- S3+CloudFront, Cloudflare Pages, Netlify, GitHub Pages, etc.

This standard only requires that the deployed host serves the publish tree at the canonical URLs with:

- HTTPS (HTTP redirects acceptable)
- Appropriate `Content-Type` headers based on file extension

## Acceptance Criteria

For any corpus:

- [ ] The publishing tool can run in CI and locally to produce a publish tree.
- [ ] For every published JSON Schema, `$id` resolves to the correct file over HTTPS.
- [ ] For OpenAPI and AsyncAPI, `x-fulmen-id` resolves to the correct file over HTTPS.
- [ ] The build fails on mismatched domains/paths/duplicate IDs.

## Open Questions

These questions are deferred for follow-on specification:

1. ~~**Scope boundaries**: Do we standardize a single root prefix (e.g., `/crucible/...`) or allow multiple suites at the domain root?~~ **Resolved**: See [Canonical URI Resolution Standard](canonical-uri-resolution.md) — module namespace (repo tag) is required as first path segment.
2. **Media type policy**: Do we require `application/schema+json` vs `application/json` vs extension-based defaults?
3. **Index requirements**: Should `index.json` be required for spec-host repos?
4. **Tooling owner**: Do we standardize on a specific publisher tool, or allow multiple implementations?

## Related Documentation

- [Canonical URI Resolution Standard](canonical-uri-resolution.md) — URI structure and resolver contract
- [Repository Category: spec-host](../repository-category/spec-host/README.md) — Category for spec hosting repositories
- [Schema Normalization](../schema-normalization.md) — `$id` requirements for JSON Schema
- [Spec Catalog Schema](../../../schemas/standards/publishing/v1.0.0/spec-catalog.schema.json) — Index catalog format

---

**Status**: Active (v0.2.26+)

**Maintainers**: Crucible Team
