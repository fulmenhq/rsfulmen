# Meta-Schema Cache

This directory stores the curated JSON Schema meta-schemas that FulmenHQ tools need for
offline validation. Directory names mirror the signature tags defined in
schema signatures (e.g., `draft-07`, `draft-2020-12`) so downstream automation can reason about available drafts.

## Contents

- `draft-07/` – canonical Draft-07 meta-schema (`schema.json`).
- `draft-2020-12/` – canonical Draft 2020-12 meta-schema plus offline helpers.
  - `schema.json` – raw meta-schema fetched from json-schema.org.
  - `offline.schema.json` – reduced subset used when offline validation is needed.
  - `meta/` – reserved for additional vocabularies (`core.json`, `validation.json`, etc.).

## Refresh Workflow

Meta-schemas are treated as curated assets. Refresh them from upstream when network access is available.

> **Note**: Do not hand-edit these files unless you are updating the offline subset. Always regenerate from upstream to guarantee canonical content and ensure the json-schema.org terms of service are honored. The upstream license permits redistribution, so keeping copies in-repo is allowed.

## Offline Subset

`draft-2020-12/offline.schema.json` is a minimal schema that covers the portions of the spec needed for validation. It avoids `$ref` chains to `meta/*` so the validator can operate without network access. Keep it in sync with upstream changes as needed.
