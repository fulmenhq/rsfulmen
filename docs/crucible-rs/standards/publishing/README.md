---
title: "Publishing Standards"
description: "Standards for publishing and distributing Fulmen artifacts"
author: "Fulmen Enterprise Architect (@fulmen-ea-steward)"
date: "2025-12-20"
last_updated: "2026-01-06"
status: "active"
tags: ["standards", "publishing", "distribution", "v0.4.1"]
---

# Publishing Standards

## Purpose

This directory contains standards for publishing and distributing various Fulmen artifacts. These standards define the contracts that publishing tools must implement to ensure consistent, reliable artifact distribution across the ecosystem.

## Standards

### Spec Publishing Standard

**File**: [spec-publishing.md](spec-publishing.md)

Defines the workflow and tooling contract for publishing specification artifacts (JSON Schema, OpenAPI, AsyncAPI) so their canonical identifiers resolve over HTTPS.

**Key concepts**:

- Canonical ID rules (`$id`, `x-fulmen-id`)
- Publishing tool contract (discovery, validation, materialization)
- Deployment requirements (URL resolution, content types)

**Applies to**:

- `spec-host` category repositories
- Any corpus requiring resolvable canonical URLs

### Canonical URI Resolution Standard

**File**: [canonical-uri-resolution.md](canonical-uri-resolution.md)

Defines the canonical URI structure for FulmenHQ specification assets and the resolver contract that enables offline access through helper libraries.

**Key concepts**:

- Canonical URI structure: `https://schemas.fulmenhq.dev/<module>/<topic>/<version>/<filename>`
- Module namespace semantics (repo tag identification)
- Resolver contract for URI-to-local-path mapping
- Cross-module resolution patterns

**Applies to**:

- Helper libraries (gofulmen, rsfulmen, tsfulmen, pyfulmen) implementing resolvers
- Any library or app consuming Crucible schemas offline
- Spechost and dochost publishers

## Future Standards (Planned)

### Release Publishing Standard

Publishing GitHub release artifacts with signing and verification.

### Artifact Publishing Standard

General artifact distribution (binary releases, container images, etc.).

## Relationship to Repository Categories

| Category    | Publishing Standard                    |
| ----------- | -------------------------------------- |
| `spec-host` | Spec Publishing Standard (required)    |
| `codex`     | May layer UI over spec-host corpus     |
| `library`   | Release Publishing Standard (planned)  |
| `workhorse` | Artifact Publishing Standard (planned) |

## Related Documentation

- [Repository Category Standards](../repository-category/README.md) — Category-specific requirements
- [Schema Normalization](../schema-normalization.md) — `$id` requirements for JSON Schema
- [Crucible Shim Standard](../library/modules/crucible-shim.md) — Asset access contract for helper libraries
- [Release Checklist Standard](../release-checklist-standard.md) — Release process requirements

---

**Status**: Active (v0.4.1+)

**Maintainers**: Crucible Team
