---
title: "Schema Validation Helper Standard"
description: "Contract for discovering Crucible schemas and validating documents across Fulmen helper libraries"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2026-08-20"
status: "approved"
tags: ["standards", "library", "schema", "validation", "meta-schema"]
---

# Schema Validation Helper Standard

## Purpose

Expose consistent utilities for locating Crucible schemas, validating documents, and reporting errors across
languages. Ensures applications can enforce SSOT contracts at runtime and during CI checks.

Every language foundation (`gofulmen`, `tsfulmen`, `pyfulmen`, `rsfulmen`, and later foundations) MUST
implement this module. Applications MUST call the helper; they MUST NOT wrap `jsonschema` / AJV independently
and MUST NOT subprocess goneat at runtime for instance checks.

## Responsibilities

1. Provide schema lookup APIs keyed by logical identifier (`schemas/protocol/http/v1.0.0/health-response`).
2. Support JSON Schema draft 2020-12 validation, delegating to goneat binaries where applicable (CI / optional
   CLI wrappers only — not a runtime requirement for instance checks).
3. Offer convenience helpers for common operations (validate file, validate string payload, compare schemas).
4. Emit structured validation errors with location data (pointer, message, severity).
5. Validate instances against **caller-supplied on-disk schema trees** that the helper does not embed, with
   offline `$ref` resolution (no network).

## Validation modes

Helpers ship two instance-validation modes. Both MUST return the same diagnostic taxonomy.

### Embedded catalog

`ValidateData(id, data)` / `ValidateFile(id, path)` resolve `id` against schemas **shipped in the helper**
(Crucible embeds). `$ref` stays inside that embed. This is the Fulmen-contract path (app identity, logging,
config, foundry, …).

### File-backed catalog

Some consumers validate payloads against a schema family that lives **beside the binary** (application or
sibling-repo trees). Those files are not Fulmen Crucible embeds and MUST NOT be vendored into the helper.

| Function                                                 | Description                                                                         |
| -------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| `ValidateInstance(schema, data, resolver)`               | Validate an in-memory instance against an in-memory schema with a catalog resolver. |
| `ValidateInstanceWithSchemaFile(schemaPath, data, opts)` | Load the root schema from disk; resolve `$ref` via `opts`.                          |
| `ValidateInstanceFile(schemaPath, dataPath, opts)`       | Same, with the instance loaded from a JSON/YAML file.                               |

Names MAY follow language conventions. **Do not** overload `ValidateFile(id, dataPath)` so that `id` sometimes
means an embed key and sometimes a filesystem path.

`FileSchemaOptions`:

| Field        | Description                                                                                          |
| ------------ | ---------------------------------------------------------------------------------------------------- |
| `RefDirs`    | Repeatable catalog roots (directories). Relative `$ref` also resolves against the schema file’s dir. |
| `Resolution` | `PreferId` (default) or `PathOnly`.                                                                  |

`$ref` rules:

- Relative refs resolve against the schema file’s directory and/or `RefDirs`.
- Absolute `$id` URLs (`https://…`, `http://…`) are **catalog keys**, never network fetches. They resolve inside
  `RefDirs` by `$id` (`PreferId`) or by path suffix (`PathOnly`). Missing key → compile failure.
- `#/…` pointers stay with the language JSON Schema engine.
- `file://` MUST work in this mode for **local paths only** (embedded mode MAY still reject `file://`).
- Compile failure is distinct from instance issues.
- Draft comes from the schema’s `$schema` (2020-12 default; draft-07 when declared) — same as `select_draft` /
  equivalent today.
- No network. No goneat subprocess at runtime.

#### Containment (file-backed only)

Allowed roots are the **canonical** (realpath) directory of the root schema file and the canonical path of each
`RefDirs` entry. After resolving a `file://` or relative `$ref` to a filesystem path, the helper MUST
canonicalize (including `..`) and accept the target only if it is equal to or strictly inside an allowed root.

Reject as **compile failure** (not instance issues):

- Path traversal that leaves every allowed root (`../` and equivalent).
- Symlink escape: any symlink component or final target whose realpath is outside every allowed root
  (`lstat` / no-follow on components; same posture as fulpack destination bounds).
- Unsupported URI schemes (anything other than relative, in-document `#`, `file:`, or `$id` catalog keys
  `http:`/`https:` as above). `file:` with a non-local host is unsupported.
- Absolute filesystem paths outside every allowed root.

Applications MUST NOT supply an open “read any path” resolver that bypasses these rules.

Constructs that MUST work: `additionalProperties: false`, `const` `$id` pins, `oneOf`, `allOf`, `if`/`then`,
sibling `$ref` across files.

Meta-schema checks (`ValidateSchema` / goneat `validate-schema`) remain a separate gate. File-backed **instance**
validation is the required addition; optional `MetaValidateSchemaFile` using already-embedded meta schemas is
allowed.

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

See [Validation modes](#validation-modes) for the file-backed functions. Existing embed IDs MUST keep working.

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
- Cross-language parity tests (Go vs Python vs TypeScript vs Rust) using shared sample payloads.
- File-backed tests MUST use a catalog that exists **only on disk** (not in the embed): conforming instance →
  empty issues; extra property → `additionalProperties`; sibling `$ref` across two files via `RefDirs`.
- File-backed containment tests: `../` escape, symlink-outside-root, unsupported scheme, and `file:` with a
  non-local host MUST compile-fail; a sibling file inside an allowed root MUST succeed.

## Related Documents

- `schemas/` directory for authoritative schema files
- `schemas/meta/README.md` - Meta-schema cache and draft selection guidance
- `docs/standards/schema-normalization.md` - Schema formatting standards
- `schemas/upstream/3leaps/crucible/` - Vendored classification schemas from 3leaps
- [Helper Library Standard](../../../architecture/fulmen-helper-library-standard.md) — item 6
- [Workhorse](../../../architecture/fulmen-forge-workhorse-standard.md) / [Microtool](../../../architecture/fulmen-forge-microtool-standard.md) / [Codex](../../../architecture/fulmen-forge-codex-standard.md) forge module tables
- [Consuming Crucible Assets](../../../guides/consuming-crucible-assets.md) — embed vs application catalogs
