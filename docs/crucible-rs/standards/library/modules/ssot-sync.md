---
title: "SSOT Sync Standard"
description: "Helper module contract for synchronizing Crucible assets via FulDX"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["standards", "library", "ssot", "sync", "2025.10.2"]
---

# SSOT Sync Standard

## Purpose

Ensure helper libraries provide a reliable way to synchronize Crucible (and future SSOT) assets using FulDX.

## Workflow Requirements

1. Read `.fuldx/sync-consumer.yaml` (validated against the sync consumer schema) and optional
   `.fuldx/sync-consumer.local.yaml`.
2. Invoke `fuldx ssot sync` with appropriate arguments (respecting overrides).
3. Expose helper commands (`make sync`, `library.syncAssets()`) that orchestrate sync + post-processing (index
   generation, embedding assets).
4. Fail fast on checksum mismatch, missing sources, or conflicting output directories.

## Helper API

| Function / Command       | Description                                              |
| ------------------------ | -------------------------------------------------------- |
| `SyncAssets(options)`    | Run the sync workflow, returning updated asset metadata. |
| `ValidateManifest(path)` | Validate manifest before invocation (uses goneat).       |
| `ListSources()`          | Return configured sources and include/exclude globs.     |
| `GenerateIndex()`        | Refresh language-specific index used by Crucible shim.   |

## Local Overrides

- `.fuldx/sync-consumer.local.yaml` MAY specify `localPath` to use a local clone of Crucible. Helper functions
  must merge overrides (local > env vars > primary manifest) mirroring FulDX precedence rules.
- Provide explicit logging when overrides are used to aid debugging.

## Post-Sync Steps

- Regenerate language-specific embeddings (Go `schemas.go`, TypeScript `schemas.ts`, etc.).
- Update metadata files capturing sync timestamp and commit hash.
- Optionally run `goneat schema validate-schema` on embedded assets to catch corruption.

## Testing Requirements

- Integration test executing sync against a fixture repository (use git bundle or local path) to ensure code
  handles file writes and index regeneration.
- Unit tests covering manifest validation failures and override precedence.

## Related Documents

- `docs/architecture/pseudo-monorepo.md`
- `docs/architecture/sync-model.md`
- `.plans/active/2025.10.2/standards-docs-migration.md`
