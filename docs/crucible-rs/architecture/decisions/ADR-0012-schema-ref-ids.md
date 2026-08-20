---
id: "ADR-0012"
title: "Use Absolute $id URLs for Cross-Schema $ref References"
status: "approved"
date: "2025-11-16"
last_updated: "2026-06-22"
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
"$ref": "https://schemas.fulmenhq.dev/crucible/observability/logging/v1.0.0/middleware-config.schema.json"
```

## Status

- **Implemented (phased)**:
  - **2025-11-16** — Initial rollout (commit `70fa5be`): converted `logger-config.schema.json` (the case gofulmen v0.1.15 hit) to absolute `$id` references; gofulmen validator updated to handle these references.
  - **2026-06-22** — Completion sweep (PR #8): the initial rollout missed the remaining sibling logging schemas (`log-event`, `severity-filter`, `middleware-config`) and an off-by-one relative `$ref` in `library/module-manifest`. These retained relative cross-file `$ref`s and failed to resolve in memory-based validators (reproduced from tsfulmen and gofulmen). All cross-file `$ref`s in those schemas are now absolute `$id` URLs. In the same change, the `observability/logging/v1.0.0/*` `$id`s were migrated from the deprecated version-in-filename form to the canonical version-in-path form per the [Canonical URI Resolution Standard](../../standards/publishing/canonical-uri-resolution.md).

> **Note**: Other schemas in the corpus still use the deprecated version-in-filename `$id` form (e.g. `pathfinder/*`, `ascii/*`, `schema-validation/*`, `taxonomy/library/fulencode/*`). These resolve correctly today (their cross-file refs, where present, are already absolute) and should migrate to version-in-path at their next version bump, not eagerly.

## Consequences

- Validators that preload schemas by `$id` or support HTTP resolution will succeed without filesystem context.
- Authors should prefer absolute `$id` references for cross-file links; relative refs remain acceptable for internal `$defs`.

## Action Items

1. Update gofulmen validators to preload schemas by `$id` or allow HTTP resolution if needed.
2. When adding or modifying schemas, use absolute `$id` references for cross-schema `$ref`.
3. Add a lint check to catch relative cross-file `$ref` uses (and version-in-filename `$id`s) in future changes. **Recommended** — the 2026-06-22 completion sweep showed the 2025-11-16 rollout silently missed several schemas; a CI check would have caught the gap. (`make validate-schemas` should fail on relative cross-file `$ref`s.)
