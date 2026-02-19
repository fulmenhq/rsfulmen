# Meta-Schema Cache

This directory stores the curated JSON Schema meta-schemas that FulmenHQ tools need for
offline validation. Directory names mirror the signature tags defined in
schema signatures (e.g., `draft-04`, `draft-06`, `draft-07`, `draft-2019-09`, `draft-2020-12`) so downstream automation can reason about available drafts.

## Contents

- `draft-04/` – canonical Draft-04 meta-schema (`schema.json`).
  - Single-file, self-contained meta-schema.
  - Uses `id` instead of `$id` (pre-Draft-06 convention).
  - Common in SchemaStore and legacy tooling.

- `draft-06/` – canonical Draft-06 meta-schema (`schema.json`).
  - Single-file, self-contained meta-schema.
  - Introduced `$id`, `const`, `contains`, `propertyNames`.
  - Boolean schemas allowed (`true`/`false` as schemas).

- `draft-07/` – canonical Draft-07 meta-schema (`schema.json`).
  - Single-file, self-contained meta-schema.
  - Added `if`/`then`/`else`, `readOnly`, `writeOnly`, `$comment`.

- `draft-2019-09/` – canonical Draft 2019-09 meta-schema plus offline helpers.
  - `schema.json` – raw meta-schema with `$ref` to modular vocabularies.
  - `offline.schema.json` – reduced subset for offline validation (no external refs).
  - `meta/` – modular vocabularies (`core.json`, `validation.json`, `applicator.json`, etc.).
  - Uses `$recursiveAnchor`/`$recursiveRef` (replaced by `$dynamicAnchor` in 2020-12).
  - Introduced `$vocabulary`, `$anchor`, `unevaluatedProperties`, `unevaluatedItems`.

- `draft-2020-12/` – canonical Draft 2020-12 meta-schema plus offline helpers.
  - `schema.json` – raw meta-schema fetched from json-schema.org.
  - `offline.schema.json` – reduced subset used when offline validation is needed.
  - `meta/` – modular vocabularies (`core.json`, `validation.json`, `applicator.json`, etc.).
  - Uses `$dynamicAnchor`/`$dynamicRef` (replacing recursive anchor from 2019-09).
  - Recommended default for new schemas.

## Draft Selection Guidance

| Draft         | When to Use                                                     |
| ------------- | --------------------------------------------------------------- |
| Draft-04      | Legacy schema compatibility (SchemaStore, older tools)          |
| Draft-06      | Transitional schemas needing `const` but not `if`/`then`/`else` |
| Draft-07      | Stable choice with conditional keywords, wide tool support      |
| Draft 2019-09 | Advanced features without 2020-12's dynamic refs                |
| Draft 2020-12 | **Recommended** for new schemas, latest features                |

## Refresh Workflow

Meta-schemas are treated as curated assets. Refresh them from upstream when network access is available.

> **Note**: Do not hand-edit these files unless you are updating the offline subset. Always regenerate from upstream to guarantee canonical content and ensure the json-schema.org terms of service are honored. The upstream license permits redistribution, so keeping copies in-repo is allowed.

## Offline Subset

`draft-2020-12/offline.schema.json` is a minimal schema that covers the portions of the spec needed for validation. It avoids `$ref` chains to `meta/*` so the validator can operate without network access. Keep it in sync with upstream changes as needed.

## Fixtures

`fixtures/` contains minimal sample schemas for each draft. These are intended for cross-language parity tests and for validating that the embedded meta-schemas load correctly.
