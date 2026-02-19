---
title: "Schema Validation Helper Standard"
description: "Contract for discovering Crucible schemas and validating documents across Fulmen helper libraries"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2026-01-22"
status: "approved"
tags: ["standards", "library", "schema", "validation", "meta-schema"]
---

# Schema Validation Helper Standard

## Purpose

Expose consistent utilities for locating Crucible schemas, validating documents, and reporting errors across
languages. Ensures applications can enforce SSOT contracts at runtime and during CI checks.

## Responsibilities

1. Provide schema lookup APIs keyed by logical identifier (`schemas/protocol/http/v1.0.0/health-response`).
2. Support JSON Schema draft 2020-12 validation, delegating to goneat binaries where applicable.
3. Offer convenience helpers for common operations (validate file, validate string payload, compare schemas).
4. Emit structured validation errors with location data (pointer, message, severity).

## API Surface

| Function                    | Description                                                  |
| --------------------------- | ------------------------------------------------------------ |
| `ListSchemas(prefix?)`      | Enumerate available schemas (optionally filtered by prefix). |
| `GetSchema(id)`             | Return schema metadata (path, version, description).         |
| `ValidateData(id, data)`    | Validate an in-memory object and return diagnostics.         |
| `ValidateFile(id, path)`    | Validate a file on disk (YAML/JSON).                         |
| `CompareSchemas(id, other)` | Detect drift between expected schema and runtime copy.       |
| `NormalizeSchema(data)`     | Produce canonical form (ordering, formatting).               |

Helper libraries MAY expose streaming variants for large payloads.

## Integration with Goneat

- Provide wrapper functions invoking `goneat schema validate-data` and `goneat schema validate-schema`.
- Cache goneat binary path from FulDX bootstrap or allow consumers to supply custom path.
- Normalize JSON Schema drafts by defaulting to 2020-12 (`$schema` field).

## Meta-Schema Registry

Helper libraries SHOULD implement a `MetaSchemaRegistry` API for offline schema validation. Crucible ships curated meta-schemas in `schemas/meta/` covering all major JSON Schema drafts:

| Draft         | Path                                | Key Features                                                 |
| ------------- | ----------------------------------- | ------------------------------------------------------------ |
| Draft-04      | `schemas/meta/draft-04/schema.json` | Uses `id` (not `$id`), SchemaStore compatibility             |
| Draft-06      | `schemas/meta/draft-06/schema.json` | Introduced `$id`, `const`, boolean schemas                   |
| Draft-07      | `schemas/meta/draft-07/schema.json` | `if`/`then`/`else`, `readOnly`, wide tool support            |
| Draft 2019-09 | `schemas/meta/draft-2019-09/`       | Modular vocabularies, `$vocabulary`, `unevaluatedProperties` |
| Draft 2020-12 | `schemas/meta/draft-2020-12/`       | **Recommended default**, `$dynamicAnchor`, latest features   |

### MetaSchemaRegistry API

| Function                         | Description                                                             |
| -------------------------------- | ----------------------------------------------------------------------- |
| `ListDrafts()`                   | Return available draft identifiers (e.g., `draft-04`, `draft-2020-12`). |
| `GetMetaSchema(draft)`           | Return the meta-schema for a specific draft.                            |
| `DetectDraft(schema)`            | Infer draft from `$schema` field or heuristics.                         |
| `ValidateSchema(schema, draft?)` | Validate a schema against its meta-schema.                              |

### Offline Validation

For network-isolated environments, use the `offline.schema.json` variants in `draft-2019-09/` and `draft-2020-12/`. These self-contained subsets avoid external `$ref` chains to vocabulary files.

See `schemas/meta/README.md` for draft selection guidance.

## Error Reporting

Validation errors MUST include:

- `pointer` – JSON Pointer to the offending location.
- `message` – Human-readable description.
- `keyword` – JSON Schema keyword triggering the error.
- `severity` – `ERROR` or `WARN` (warnings optional).
- `source` – `goneat` or language-native validator.

Expose helper to render results as table, JSON, or human text.

## Testing Requirements

- Unit tests covering successful validation and error cases (missing required field, type mismatch).
- Snapshot tests ensuring error formatting stable.
- Cross-language parity tests (Go vs Python vs TypeScript) using shared sample payloads.

## Related Documents

- `schemas/` directory for authoritative schema files
- `schemas/meta/README.md` - Meta-schema cache and draft selection guidance
- `docs/standards/schema-normalization.md` - Schema formatting standards
- `schemas/upstream/3leaps/crucible/` - Vendored classification schemas from 3leaps
