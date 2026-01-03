---
title: "Schema Validation Helper Standard"
description: "Contract for discovering Crucible schemas and validating documents across Fulmen helper libraries"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-09"
status: "draft"
tags: ["standards", "library", "schema", "validation", "2025.10.2"]
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

- `schemas/` directory for authoritative schema files.
- `docs/standards/schema-normalization.md`
- `.plans/active/2025.10.2/library-module-specification-architecture-v2.md`
