# Release Notes

This document tracks release notes for rsfulmen releases.

> **Convention**: Keep only the latest 3 releases here to prevent file bloat. Older releases are archived in `docs/releases/`.

## [0.1.4] - 2026-02-21

### Crucible v0.4.12 integration, five new modules, runtime signal handling

**Release Type**: Feature Release

#### Overview

Major feature release adding typed role catalog, fulencode (encoding/decoding/normalization), runtime signal handling, UUIDv7 correlation IDs, and config env overrides. Syncs Crucible from v0.4.4 to v0.4.12, the largest SSOT update in rsfulmen's history. Brings rsfulmen significantly closer to gofulmen parity.

#### Highlights

- **Typed Role Catalog** (`crucible::roles`) — Load agentic role definitions as full-fidelity `RolePrompt` structs. 22 fields covering the entire `role-prompt.schema.json` spec, including three fields discovered during the v0.4.12 cross-team review (`pre_push_checklist`, `required_reading`, `cross_role_note`). Forward-compatible enums for `RoleCategory` and `ExampleType`. 14 roles embedded (8 approved + 6 including 3 draft).
- **Fulencode** (`fulencode`) — Binary-to-text encoding/decoding (Base64, Base64URL, Hex), character encoding (UTF-8, UTF-16LE/BE), encoding detection with BOM/heuristic confidence, Unicode normalization (NFC/NFD/NFKC/NFKD + text-safe profile rejecting zero-width and bidi attacks), BOM management. Cross-language fixture tests from Crucible SSOT.
- **Runtime Signal Handling** (`signals`) — `SignalManager` with thread-safe handler dispatch, LIFO shutdown chains, FIFO reload chains, SIGINT double-tap force-quit, and cross-platform support (`signal-hook` on Unix, `ctrlc` on Windows). Includes `SignalInjector` for deterministic test injection.
- **Correlation IDs** (`foundry::correlation`) — UUIDv7 generation, parsing, validation. `CorrelationId` newtype with strict v7 enforcement, serde support, and lowercase canonical form.
- **Config Env Overrides** (`config::env`) — Map environment variables to config key paths with type parsing, alias support, conflict detection, and sensitive value masking. Feeds directly into three-layer config.
- **Crucible v0.4.12** — 6 new agentic roles, updated role-prompt schema, fulencode schemas/fixtures, design tokens, expanded upstream standards.

#### New Feature Flags

| Feature               | Dependencies                       | Description                      |
| --------------------- | ---------------------------------- | -------------------------------- |
| `fulencode`           | base64, hex, unicode-normalization | Encoding/decoding/normalization  |
| `foundry-correlation` | uuid                               | UUIDv7 correlation ID generation |

#### New Modules

| Module                   | Feature               | Key Functions                                                     |
| ------------------------ | --------------------- | ----------------------------------------------------------------- |
| `crucible::roles`        | `crucible`            | `load_role()`, `list_role_slugs()`, `load_role_catalog()`         |
| `fulencode`              | `fulencode`           | `encode()`, `decode()`, `detect()`, `normalize()`, `detect_bom()` |
| `signals::SignalManager` | `foundry-core`        | `handle()`, `on_shutdown()`, `on_reload()`, `listen()`            |
| `foundry::correlation`   | `foundry-correlation` | `generate()`, `parse()`, `is_valid()`, `CorrelationId`            |
| `config::env`            | `config`              | `load_env_overrides()`, `load_env_overrides_with_report()`        |

#### Bug Fixes

- **Pathfinder temp-dir race** — Parallel tests could collide when `SystemTime::now()` returned the same nanosecond. Fixed with `AtomicU64` sequence counter.
- **CI yamllint warning** — Fixed missing space before inline comment in `ci.yml`.

#### Breaking Changes

None. All new modules are additive. Existing APIs unchanged.

#### Testing

- `make check-all` — 362 unit tests, 74 doc tests
- Role catalog: invariant-based tests (core slugs present, sorted, README excluded)
- Fulencode: cross-language fixture tests from Crucible SSOT
- Signal handling: injector-based tests with deterministic dispatch
- Correlation IDs: uniqueness, version validation, serde roundtrip

#### Requirements

- **Rust**: 1.88+ (MSRV, unchanged)
- **Crucible**: v0.4.12 (embedded)

---

## [0.1.3] - 2026-02-08

### Five new modules — closing gofulmen parity

**Release Type**: Feature Release

#### Overview

Adds five feature-gated modules to close parity with gofulmen: app identity discovery, structured logging, canonical hashing, safe filesystem discovery, and terminal utilities. All modules follow Crucible schema conventions and include comprehensive test coverage.

#### Highlights

- **appidentity** (P0) – App identity discovery from `.fulmen/app.yaml` with upward directory search and `FULMEN_APP_IDENTITY_FILE` override
- **logging** (P0) – Structured logging with SIMPLE/STRUCTURED profiles, leveled severity (Trace–Fatal), component context, and structured fields
- **fulhash** (P1) – Canonical hashing with xxh3-128 (default) and SHA-256, streaming support, `algo:hex` digest format, strict canonical validation
- **pathfinder** (P1) – Safe filesystem discovery with glob patterns, Crucible schema alignment, symlink boundary checks, hidden file pruning, and warning collection
- **ascii** (P1) – Terminal utilities with grapheme-cluster-aware width calculation, box drawing, and safe truncation/padding

#### New Feature Flags

| Feature       | Dependencies                        | Description                          |
| ------------- | ----------------------------------- | ------------------------------------ |
| `appidentity` | serde_json                          | App identity from `.fulmen/app.yaml` |
| `logging`     | serde_json                          | SIMPLE/STRUCTURED logging profiles   |
| `fulhash`     | xxhash-rust, sha2, hex              | xxh3-128 + SHA-256 hashing           |
| `pathfinder`  | walkdir, sha2, hex, glob            | Filesystem discovery with checksums  |
| `ascii`       | unicode-width, unicode-segmentation | Terminal + Unicode utilities         |

All features are included in `default` and `full` feature sets.

#### Breaking Changes

- **MSRV** – Bumped from 1.83 to 1.88

#### Changes

- `src/appidentity/mod.rs` – AppIdentity struct, load/load_from/load_file/validate
- `src/logging/mod.rs` – Logger, Severity, Config, SIMPLE/STRUCTURED profiles
- `src/fulhash/mod.rs` – hash/verify/parse_digest/format_digest with canonical validation
- `src/pathfinder/mod.rs` – FindQuery/FindResult/FindResults with Crucible schema parity
- `src/ascii/mod.rs` – string_width/analyze/truncate/pad/draw_box with grapheme awareness
- `Cargo.toml` – New deps and feature flags for all modules
- `src/lib.rs` – Module declarations with feature gates
- `.github/workflows/ci.yml` – New features in matrix, MSRV 1.88

#### Testing

- `make check-all` – 286 unit tests, 70 doc tests
- All modules reviewed via devrev (four-eyes audit)
- Pathfinder validated against Crucible find-query/path-result schemas
- Fulhash digest format validated for cross-language interop with gofulmen

#### Requirements

- **Rust**: 1.88+ (MSRV)
- **Crucible**: v0.4.4 (embedded)

---

## [0.1.2] - 2026-01-08

### Signal name resolution helpers

**Release Type**: Feature Release

#### Overview

Adds ergonomic signal name resolution to `foundry::signals` for CLI and API use. Users can now resolve signals from common name variants without requiring exact catalog names.

#### Highlights

- **resolve_signal()** – Flexible lookup accepting SIGTERM, TERM, term, sigterm, 15, or -15
- **list_signal_names()** – Returns all signal names for CLI completion
- **match_signal_names()** – Glob pattern matching with `*` and `?` wildcards

#### Resolution Algorithm

1. Trim whitespace
2. Empty check → None
3. Exact catalog name match (SIGTERM)
4. Numeric match with kill-style negatives (15, -15)
5. Uppercase with SIG prefix normalization (term → SIGTERM)
6. Lowercase ID fallback (hup → SIGHUP)
7. Return None if no match

#### New API

```rust
use rsfulmen::foundry::signals::{resolve_signal, list_signal_names, match_signal_names};

// All resolve to SIGTERM
assert_eq!(resolve_signal("SIGTERM").unwrap().name, "SIGTERM");
assert_eq!(resolve_signal("term").unwrap().name, "SIGTERM");
assert_eq!(resolve_signal("15").unwrap().name, "SIGTERM");
assert_eq!(resolve_signal("-15").unwrap().name, "SIGTERM");

// CLI completion
let names = list_signal_names();
assert!(names.contains(&"SIGTERM"));

// Glob matching
let usr_signals = match_signal_names("*USR*");
assert!(usr_signals.contains(&"SIGUSR1"));
```

#### Changes

- `src/foundry/signals.rs` – Added resolve_signal(), list_signal_names(), match_signal_names()
- `Crucible` – Updated to v0.4.4 (signal resolution interface spec)
- `Cargo.toml` – Updated version to 0.1.2

#### Testing

- `make check-all` – 175 unit tests, 41 doc tests
- Validated against Crucible signal-resolution-fixtures.yaml (39 test vectors)

#### Requirements

- **Rust**: 1.83+ (MSRV)
- **Crucible**: v0.4.4 (embedded)

---

## Archived Releases

Older release notes are archived under `docs/releases/`.
