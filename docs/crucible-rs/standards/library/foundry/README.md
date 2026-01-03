---
title: "Foundry Module Standards"
description: "Standards for common reference data, patterns, and text utilities in Fulmen helper libraries"
author: "Schema Cartographer"
date: "2025-10-09"
last_updated: "2025-10-30"
status: "stable"
tags:
  [
    "standards",
    "library",
    "foundry",
    "patterns",
    "similarity",
    "exit-codes",
    "2025.10.3",
  ]
---

# Foundry Module Standards

## Overview

The Foundry module provides common reference data, reusable patterns, and text utilities that every Fulmen helper library must expose idiomatically. This reduces duplication, ensures consistency, and provides battle-tested implementations for common tasks like pattern matching, text comparison, and normalization.

## Capabilities

| Capability         | Description                                                                                      | Standard Document                |
| ------------------ | ------------------------------------------------------------------------------------------------ | -------------------------------- |
| Patterns           | Regex, glob, and literal pattern catalogs                                                        | This document (Pattern Catalog)  |
| HTTP Status Groups | HTTP status code reference and grouping                                                          | This document (HTTP Statuses)    |
| Country Codes      | ISO 3166-1 country code lookup (alpha-2/3, numeric)                                              | This document (Country Codes)    |
| MIME Types         | MIME type reference with extension mapping                                                       | This document (MIME Types)       |
| Exit Codes         | Standardized process exit codes for consistent error signaling (60+ codes, 9 categories)         | This document (Exit Codes)       |
| Text Similarity    | Multiple distance metrics (Levenshtein, Damerau, Jaro-Winkler, Substring), suggestion API (v2.0) | [similarity.md](./similarity.md) |
| Text Normalization | Unicode case folding, accent stripping, normalization presets (v2.0)                             | [similarity.md](./similarity.md) |

## Pattern Catalog

Shared catalog of reusable patterns (regex, glob, literal) that reduce duplication of complex expressions (e.g., email validation, slugs, IP addresses) across languages.

## Data Sources

- Pattern catalog: `config/library/foundry/patterns.yaml` (schema: `schemas/library/foundry/v1.0.0/patterns.schema.json`)
- HTTP status groups: `config/library/foundry/http-statuses.yaml` (schema: `schemas/library/foundry/v1.0.0/http-status-groups.schema.json`)
- ISO country codes (sample seed): `config/library/foundry/country-codes.yaml` (schema: `schemas/library/foundry/v1.0.0/country-codes.schema.json`)
- MIME types: `config/library/foundry/mime-types.yaml` (schema: `schemas/library/foundry/v1.0.0/mime-types.schema.json`)
- **Exit codes**: `config/library/foundry/exit-codes.yaml` (schema: `schemas/library/foundry/v1.0.0/exit-codes.schema.json`)
- **Exit codes snapshots**: `config/library/foundry/exit-codes.snapshot.json`, `config/library/foundry/simplified-modes.snapshot.json` (for cross-language parity tests)
- **Similarity fixtures (v2.0)**: `config/library/foundry/similarity-fixtures.yaml` (schema: `schemas/library/foundry/v2.0.0/similarity.schema.json`)

Helper libraries MUST treat these catalogs as read-only SSOT data. Updates originate in Crucible.

## Library Responsibilities

1. Load the catalog at bootstrap and expose accessor APIs:
   - Return raw pattern strings.
   - Provide compiled/typed representations (e.g., `regex.Pattern`, `RegExp`, precompiled Go regex).
2. Enforce immutability and avoid runtime mutation of the shared catalog.
3. Surface documentation (e.g., `describe(patternId)`) pulling from the catalog’s metadata.
4. Integrate patterns into validation tooling where applicable (Pydantic validators, Go custom types, Zod schemas).
5. Normalize ISO country tokens (upper-case alpha codes, three-digit numeric strings) before indexing and maintain secondary indexes for alpha-3 and numeric codes so lookups stay case-insensitive across all ISO forms.

## Language Notes

- **Python:** Provide a `FoundryPatterns` helper that returns compiled `regex` instances and raw strings.
- **Go:** Expose typed wrappers returning `*regexp.Regexp` with lazy compilation.
- **TypeScript:** Export both the raw string and a `RegExp` instance per pattern (respecting flags).
- **Rust/C#:** Provide modules/namespaces with constants and precompiled helpers.

## Exit Codes

Foundry provides standardized process exit codes for consistent error signaling across the Fulmen ecosystem. This enables reliable automation, cross-tool error detection, and enterprise observability patterns.

### Categories

Nine semantic categories organize 60+ exit codes:

- **Standard (0-1)**: SUCCESS, FAILURE
- **Networking (10-19)**: Port conflicts, connectivity errors (PORT_IN_USE, NETWORK_UNREACHABLE, CONNECTION_TIMEOUT, etc.)
- **Configuration (20-29)**: Config validation, dependencies, SSOT mismatches (CONFIG_INVALID, MISSING_DEPENDENCY, SSOT_VERSION_MISMATCH, etc.)
- **Runtime (30-39)**: Health checks, database, resource exhaustion (HEALTH_CHECK_FAILED, DATABASE_UNAVAILABLE, RESOURCE_EXHAUSTED, etc.)
- **Usage (40-49)**: CLI argument errors, BSD EX_USAGE (INVALID_ARGUMENT, MISSING_REQUIRED_ARGUMENT, USAGE)
- **Permissions (50-59)**: File access, permission denied (PERMISSION_DENIED, FILE_NOT_FOUND, FILE_WRITE_ERROR, etc.)
- **Data (60-69)**: Validation, parsing, corruption (DATA_INVALID, PARSE_ERROR, DATA_CORRUPT, etc.)
- **Security (70-79)**: Authentication, authorization, certificate errors (AUTHENTICATION_FAILED, AUTHORIZATION_FAILED, CERTIFICATE_INVALID, etc.)
- **Observability (80-89)**: Metrics, tracing, logging failures (METRICS_UNAVAILABLE, TRACING_FAILED, LOGGING_FAILED, etc.)
- **Testing (90-99)**: Test failures, coverage thresholds (TEST_FAILURE, TEST_ERROR, COVERAGE_THRESHOLD_NOT_MET, etc.)
- **Signals (128-165)**: Unix signal termination (SIGTERM=143, SIGINT=130, SIGHUP=129, SIGKILL=137, SIGUSR1=159, SIGUSR2=160, etc.)

### Files

- **Schema**: `schemas/library/foundry/v1.0.0/exit-codes.schema.json`
- **Catalog**: `config/library/foundry/exit-codes.yaml`
- **Snapshots**: `config/library/foundry/exit-codes.snapshot.json`, `simplified-modes.snapshot.json`

### Helper Library Integration

**📖 Application Developer Guide**: See [`docs/standards/fulmen/exit-codes/README.md`](../../fulmen/exit-codes/README.md) for usage patterns, examples, and best practices.

**Implementation Requirements** (for helper library maintainers):

Helper libraries MUST:

1. **Load catalog** from synced `config/library/foundry/exit-codes.yaml`
   - Cache parsed results lazily (avoid disk I/O during module import for short-lived CLIs)
   - Fail fast with structured error if catalog missing/invalid (include remediation in message)
   - Expose `getExitCodesVersion()` accessor returning catalog `version` field for telemetry

2. **Expose language-native constants**:
   - **Go**: `const` in `pkg/foundry` (e.g., `ExitPortInUse`)
   - **Python**: `IntEnum` in `pyfulmen.foundry.exit_codes` (e.g., `ExitCode.PORT_IN_USE`)
   - **TypeScript**: `as const` object + type unions (e.g., `exitCodes.PORT_IN_USE`)
   - **Rust**: `const i32` in `rsfulmen::foundry::exit_codes` (e.g., `exit_codes::PORT_IN_USE`)
   - **C#**: Static class constants + enum (e.g., `ExitCodes.PortInUse`)
   - Generate via code generation templates (maintained in Crucible) to prevent drift

3. **Provide metadata accessor**:
   - `getExitCodeInfo(code: int)` returns: code, name, description, context, category, retry_hint, bsd_equivalent
   - Enable structured logging with exit metadata for observability

4. **Implement simplified mode mapping**:
   - `mapToSimplified(code: int, mode: SimplifiedMode)` for novice-friendly tools
   - Support `basic` (0/1/2) and `severity` (0-7) modes
   - Workhorses SHOULD expose `exit_codes.simplified_mode` configuration toggle

5. **Add snapshot parity tests**:
   - Export computed catalog and diff against canonical snapshots
   - CI/CD fails if codes drift between languages
   - Verify simplified-mode mappings and metadata fields

### Simplified Modes

Two simplified exit code modes collapse detailed codes for novice users:

- **Basic**: 0 (success), 1 (error), 2 (usage error)
  - Use for: non-technical CLI users, health-check scripts
- **Severity**: 0 (success), 1-7 (user/config/runtime/system/security/test/observability errors)
  - Use for: observability dashboards summarising failure severity

**Default posture**: Helper libraries expose opt-in mapping helpers; they MUST NOT remap codes implicitly. Workhorses configure via `exit_codes.simplified_mode=basic|severity|disabled` (default: disabled for infrastructure automation).

### Extension Ranges

Applications can define custom codes in reserved ranges:

- **100-127**: Application-specific codes (28 codes)
  - Simple workhorse: 5-10 codes
  - Moderate workhorse: 15-20 codes
  - Complex workhorse: 20-25 codes
- **166-191**: Reserved for future signals
- **192-254**: User-defined codes

### BSD Compatibility

Pragmatic mappings align with `sysexits.h`:

- **EX_USAGE (64)** → EXIT_USAGE (64) - preserved for compatibility (breaks usage range 40-49)
- **EX_OSERR (71)** → EXIT_RESOURCE_EXHAUSTED (33) - conflicts with EXIT_AUTHORIZATION_FAILED (71)
- **EX_IOERR (74)** → EXIT_FILE_WRITE_ERROR (54) - I/O errors typically file operations
- **EX_TEMPFAIL (75)** → EXIT_OPERATION_TIMEOUT (34) - temporary failures often timeouts

See catalog for complete BSD mapping table.

### Platform Considerations

- **POSIX signal exits (128+N)** not emitted by native Windows termination APIs
- Helper libraries MUST detect non-POSIX runtimes and fall back with structured warning
- Provide capability probe (e.g., `supportsSignalExitCodes()`) so workhorses can branch
- WSL/POSIX layers honor Unix mapping but surface telemetry indicating underlying kernel

## Testing Requirements

- Validate catalog loading and ensure all entries are present (patterns, status groups, countries, exit codes).
- Ensure accessors return compiled/typed representations without error.
- Provide positive/negative unit tests for each pattern (sample matches + mismatches).
- Verify HTTP status group helpers cover the documented codes; ensure language implementations expose convenience sets (e.g., `IsClientError(code)`).
- Verify country code lookups support alpha-2, alpha-3, and numeric forms case-insensitively (including numeric normalization) and reuse the precomputed indexes built at load time.
- Verify MIME type helpers handle lookup by id, MIME string, and extensions; add tests for unknown types.
- **Verify exit code catalog loading** and all 60+ codes accessible via language-native constants.
- **Verify exit code metadata** via `getExitCodeInfo()` returns complete metadata (retry_hint, category, etc.).
- **Verify simplified mode mappings** correctly collapse detailed codes to basic/severity modes.
- **Add snapshot parity tests** comparing against canonical snapshots to detect cross-language drift.
- Maintain snapshot/parity tests to detect catalog drift across languages.
- Verify per-language flag handling (Unicode, ignore-case, etc.) in unit tests.

## Standards

### Patterns, HTTP Statuses, Country Codes, MIME Types

See sections above for pattern catalog, HTTP status groups, country code lookup, and MIME type reference standards.

### Text Similarity & Normalization (v2.0)

See [similarity.md](./similarity.md) for comprehensive v2.0 standard covering:

- **Multiple distance metrics**: Levenshtein, Damerau-Levenshtein (OSA variant), Jaro-Winkler, Substring scoring
- **Suggestion API**: Fuzzy matching with configurable metrics and normalization presets
- **Normalization presets**: `none`, `minimal`, `default`, `aggressive` (replaces boolean flags)
- **Unicode-aware processing**: NFC/NFKD normalization, case folding, accent stripping, punctuation removal
- **Enhanced results**: Optional matched ranges, reason codes, normalized values
- **Shared fixtures**: Cross-language validation with v2.0.0 schema

**Breaking Changes**: v2.0 introduces versioned schema directory (`schemas/library/foundry/v2.0.0/`) and new fixture format. Libraries must explicitly opt into v2.0.0 by updating schema paths in test code.

## Related Documents

- [Text Similarity & Normalization Standard](./similarity.md)
- [Foundry Interfaces](./interfaces.md)
- `docs/architecture/modules/README.md`
- `config/library/v1.0.0/module-manifest.yaml`
- `config/library/foundry/patterns.yaml`
- `config/library/foundry/http-statuses.yaml`
- `config/library/foundry/country-codes.yaml`
- `config/library/foundry/exit-codes.yaml`
- `config/library/foundry/exit-codes.snapshot.json`
- `config/library/foundry/simplified-modes.snapshot.json`
- `config/library/foundry/similarity-fixtures.yaml` (v2.0 schema)
- `schemas/library/foundry/v1.0.0/exit-codes.schema.json`
- `schemas/library/foundry/v2.0.0/similarity.schema.json` (v2.0 schema - first versioned schema)
