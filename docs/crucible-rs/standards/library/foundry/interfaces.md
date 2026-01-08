---
title: "Foundry Interfaces"
description: "Required helper interfaces for consuming Foundry catalogs"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2026-01-08"
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

## Signal Resolution

Each library MUST expose functions for ergonomic signal name resolution using the signals catalog
(`config/library/foundry/signals.yaml`). This eliminates ad-hoc normalization in downstream CLIs
and ensures consistent behavior across the ecosystem.

### Required API Surface

| Function           | Signature (Language-Idiomatic)     | Description                                  |
| ------------------ | ---------------------------------- | -------------------------------------------- |
| `resolveSignal`    | `(name: string) -> Signal \| null` | Resolve signal from common name variants.    |
| `listSignalNames`  | `() -> string[]`                   | Return all signal names for CLI completion.  |
| `matchSignalNames` | `(pattern: string) -> string[]`    | Return signal names matching a glob pattern. |

### Language-Idiomatic Signatures

| Language   | `resolveSignal`                                             | `listSignalNames`                             | `matchSignalNames`                                          |
| ---------- | ----------------------------------------------------------- | --------------------------------------------- | ----------------------------------------------------------- |
| Rust       | `fn resolve_signal(name: &str) -> Option<&'static Signal>`  | `fn list_signal_names() -> Vec<&'static str>` | `fn match_signal_names(pattern: &str) -> Vec<&'static str>` |
| Go         | `func ResolveSignal(name string) *Signal`                   | `func ListSignalNames() []string`             | `func MatchSignalNames(pattern string) []string`            |
| Python     | `def resolve_signal(name: str) -> Signal \| None`           | `def list_signal_names() -> list[str]`        | `def match_signal_names(pattern: str) -> list[str]`         |
| TypeScript | `function resolveSignal(name: string): Signal \| undefined` | `function listSignalNames(): string[]`        | `function matchSignalNames(pattern: string): string[]`      |

### Resolution Algorithm

`resolveSignal(name)` MUST implement the following resolution order:

1. **Trim** leading/trailing whitespace
2. **Empty check**: Return `null` if empty after trim
3. **Exact match**: Try exact catalog name match (e.g., `"SIGTERM"`)
4. **Numeric match**: If input (after trim) is a positive numeric string, lookup by `unix_number`
   - Example: `"15"` → SIGTERM, `"  15  "` → SIGTERM (whitespace trimmed first)
   - Negative numbers are NOT supported: `"-15"` → `null`
5. **Uppercase normalization**:
   - Uppercase the input
   - If starts with `"SIG"`: lookup by name
   - Else: prepend `"SIG"` and lookup by name (e.g., `"term"` → `"SIGTERM"`)
6. **ID fallback**: Lowercase input and try `id` field lookup (e.g., `"hup"` via catalog `id: hup`)
7. **Not found**: Return `null` if all attempts fail

**Rationale**: This order preserves strict lookup semantics for exact matches while enabling
ergonomic input from CLI users (`kill -s TERM`, `kill -s term`, `kill -15`). Numeric resolution
supports the common `kill` CLI pattern. The ID fallback leverages existing catalog structure.

**Note**: Numeric detection (step 4) operates on the trimmed input from step 1.

### Glob Matching Rules

`matchSignalNames(pattern)` MUST support simple glob patterns:

- `*` matches zero or more characters
- `?` matches exactly one character
- Matching is case-insensitive
- No regex dependencies required (simple char-by-char implementation)
- No escape sequences needed (signal names don't contain `*` or `?`)

### Implementation Notes

- `Signal` is the language-specific structure from the signals catalog exposing at least:
  - `name` (e.g., `"SIGTERM"`)
  - `id` (e.g., `"term"`)
  - `unix_number` (e.g., `15`)
  - `exit_code` (e.g., `143`)
- Resolution returns the canonical catalog entry; platform-specific numbers are accessed via
  existing `getSignalNumberForPlatform()` helpers (separation is intentional).
- New functions complement existing strict lookups (`lookupSignal()`, `getSignalNumber()`);
  do NOT modify existing functions.
- **Index recommendation**: Build a `signalsByNumber` index (map unix_number → Signal) during
  catalog load for efficient numeric resolution (step 4). Without this index, numeric lookup
  requires linear scan of the catalog. Most implementations already build `signalsByName` and
  `signalsByID` indexes.

### Testing

- Use cross-language test vectors from `config/library/foundry/signal-resolution-fixtures.yaml`
- All implementations MUST pass the fixture test cases
- Verify resolution handles all documented variants (exact, lowercase, no-prefix, numeric, whitespace)
- Verify glob matching with `*` and `?` wildcards
- Verify unknown signals return `null` (not throw)

### Future Enhancement (Deferred)

- **Alias handling**: Some signals have historical aliases (e.g., `SIGIOT` = `SIGABRT`).
  Catalog schema changes are out of scope for v1.0. When aliases are added to the catalog,
  resolution should check aliases after ID fallback.

## Future Interfaces

Additional interfaces (e.g., version payload generators, health check templates) will be added as the Foundry catalog
expands. Library maintainers should structure code to load catalogs generically so new datasets can be adopted quickly.

---

**Conformance:** Helper libraries MUST document how these interfaces are exposed in their README/API docs and provide
unit tests demonstrating compliance.
