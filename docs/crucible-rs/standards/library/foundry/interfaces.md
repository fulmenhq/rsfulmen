---
title: "Foundry Interfaces"
description: "Required helper interfaces for consuming Foundry catalogs"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-25"
status: "stable"
tags: ["standards", "library", "foundry", "interfaces", "2025.10.3"]
---

# Foundry Interfaces

This standard specifies the minimum helper interfaces that each Fulmen helper library MUST expose to consume data
from the Foundry catalogs. Implementations remain language-idiomatic while conforming to the contracts below.

## MIME Type Detection

Each library MUST expose functions that allow consumers to identify MIME types using the curated catalog
(`config/library/foundry/mime-types.yaml`).

### Required API Surface

| Function                 | Signature                        | Description                                 |
| ------------------------ | -------------------------------- | ------------------------------------------- | -------------------------- | ------------------------------------------------------------------------------ |
| `detectMimeType`         | `(input: bytes                   | stream) -> MimeType                         | null`                      | Inspect raw bytes (or stream) and return catalog entry or `null` when unknown. |
| `isSupportedMimeType`    | `(mime: string) -> bool`         | Return `true` if MIME string is in catalog. |
| `getMimeTypeByExtension` | `(extension: string) -> MimeType | null`                                       | Resolve by file extension. |
| `listMimeTypes`          | `() -> Iterable<MimeType>`       | Enumerate curated MIME entries.             |

### Implementation Notes

- `MimeType` is a language-specific structure exposing at least:
  - canonical MIME string
  - identifier (`id`)
  - associated file extensions
  - friendly name / description
- Helper libraries SHOULD provide streaming detection where feasible (e.g., magic number checking) but may fall back to extension matching when content sniffing is unavailable.
- When detection fails, return `null` (not throw) to allow callers to apply fallback logic.

### Testing

- Unit tests covering positive/negative detection scenarios for each curated type.
- Verify fallback behavior (unknown input returns `null`).
- Ensure cache/lookup structures stay in sync with Foundry catalog updates.

## Pattern Accessors

- Provide helper functions to retrieve raw/compiled patterns (`getPattern(id)`, `match(id, value)` convenience wrappers).
- Validate that all Foundry pattern IDs are exposed (fail tests if catalog entries are missing).
- Provide positive/negative tests per pattern ID.

## Status & Country Catalogs

- Expose helper utilities (`isClientError(code)`, `lookupStatus(code)`, `lookupCountry(alpha2)`, etc.) with parity tests.
- Normalize ISO country tokens (upper-case alpha variants; left-pad numeric strings to three digits) before indexing so lookups are case-insensitive across alpha-2, alpha-3, and numeric codes.
- Precompute secondary indexes (alpha-3 → country, numeric → country) when loading the catalog to avoid repeated linear scans during validation.
- Implement mappings using the curated data rather than hardcoding values.

## Future Interfaces

Additional interfaces (e.g., version payload generators, health check templates) will be added as the Foundry catalog
expands. Library maintainers should structure code to load catalogs generically so new datasets can be adopted quickly.

---

**Conformance:** Helper libraries MUST document how these interfaces are exposed in their README/API docs and provide
unit tests demonstrating compliance.
