---
id: "ADR-0012"
title: "Use Absolute $id URLs for Cross-Schema $ref References"
status: "approved"
date: "2025-11-16"
last_updated: "2025-11-16"
deciders:
  - "@3leapsdave"
  - "@schema-cartographer"
scope: "schemas"
tags: ["json-schema", "references", "logging", "cross-repo"]
---

# ADR-0012: Use Absolute $id URLs for Cross-Schema $ref References

## Context

Some schemas in Crucible (e.g., `schemas/observability/logging/v1.0.0/logger-config.schema.json`) referenced sibling schemas using relative paths (e.g., `"$ref": "middleware-config.schema.json"`). When validators load schemas into memory (no filesystem context), these relative references fail to resolve. gofulmen v0.1.15 encountered this with the logging middleware reference, leading to schema compilation failures.

## Decision

For cross-file references in Crucible schemas, **use absolute URLs pointing to the target schema’s `$id`**. This ensures references resolve consistently in memory-based validators and aligns with JSON Schema best practices.

**Example:**

```json
"$ref": "https://schemas.fulmenhq.dev/crucible/observability/logging/middleware-config-v1.0.0.json"
```

## Status

- **Implemented**: The decision has been fully implemented. All cross-file schema references now use absolute `$id` URLs, and the gofulmen validator has been updated to correctly handle these references.

## Consequences

- Validators that preload schemas by `$id` or support HTTP resolution will succeed without filesystem context.
- Authors should prefer absolute `$id` references for cross-file links; relative refs remain acceptable for internal `$defs`.

## Action Items

1. Update gofulmen validators to preload schemas by `$id` or allow HTTP resolution if needed.
2. When adding or modifying schemas, use absolute `$id` references for cross-schema `$ref`.
3. Optionally add a lint check to catch relative cross-file `$ref` uses in future changes.
