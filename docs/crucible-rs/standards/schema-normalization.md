---
title: "Schema Normalization Standard"
description: "Cross-language contract for canonicalizing and comparing Crucible schemas"
author: "@3leapsdave"
date: "2025-10-02"
last_updated: "2025-10-02"
status: "approved"
tags: ["schemas", "standards", "normalization", "comparison"]
---

# Schema Normalization Standard

## Purpose

Provide a shared contract for normalizing Fulmen schemas so language teams can compare JSON and YAML sources without losing semantic fidelity (including comment-preserving authoring workflows).

## Background

- Authoring teams prefer **YAML** for rich comments and readability.
- Consumers (generators, validators, CI) require **canonical JSON** for deterministic behavior.
- Downstream libraries (`gofulmen`, `tsfulmen`, future `{lang}fulmen`) need a consistent API surface to normalize and diff schemas.

## Requirements

1. Accept both YAML and JSON inputs transparently.
2. Strip comments while preserving semantic structure.
3. Produce deterministic, pretty-printed JSON with stable key ordering.
4. Support a semantic equality check that exposes normalized payloads for debugging.
5. Operate without network access and without mutating source files.

## Standardized API

All language wrappers **MUST** expose the following signatures:

### Normalize Schema

```text
normalizeSchema(input, options?) -> string | []byte
```

- **input**: string, byte buffer, or stream containing YAML or JSON.
- **options.compact** (optional): when true, emit minified JSON.
- **returns**: canonical JSON string/bytes with:
  - Sorted object keys (lexicographical)
  - Two-space indentation (default)
  - HTML characters unescaped

### Compare Schemas

```text
compareSchemas(schemaA, schemaB, options?) -> { equal, normalizedA, normalizedB }
```

- Normalizes each input via `normalizeSchema`.
- **equal**: boolean flag
- **normalizedA/normalizedB**: canonical JSON payloads for diagnostics
- Shares the same `options` shape as `normalizeSchema`.

### Error Semantics

- Empty input ⇒ error "schema content is empty"
- Parse failure ⇒ error prefixed with "failed to parse schema"
- Encoding failure ⇒ error prefixed with "failed to encode schema"

## Reference Implementations

- **Go** (`lang/go/schema_utils.go`)
  - Uses `gopkg.in/yaml.v3` for parsing
  - Implements deterministic ordering via custom `orderedMap`
- **TypeScript** (`lang/typescript/src/schema-utils.ts`)
  - Uses `js-yaml` for parsing
  - Provides canonicalization helper reused by tests

## Testing Expectations

Language wrappers **MUST** include tests that:

- Assert YAML with comments normalizes to expected JSON
- Verify YAML and JSON equivalents compare as equal
- Validate empty inputs raise explicit errors

## Adoption Notes

- Downstream packages (`gofulmen`, `tsfulmen`) should re-export these helpers.
- CLI tooling (e.g., `goneat`) can use `compareSchemas` to detect drift between vendored assets and Crucible SSOT.
- Future enhancements may add structured diff output; maintain backward-compatible result shapes.

## Related Docs

- [Frontmatter Standard](frontmatter-standard.md)
- [Crucible Sync Strategy](../guides/sync-strategy.md)
- [Gofulmen Schema Integration Summary](../../GOFULMEN_INTEGRATION_SUMMARY.md)
