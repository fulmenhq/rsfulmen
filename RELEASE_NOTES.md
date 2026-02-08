# Release Notes

This document tracks release notes for rsfulmen releases.

> **Convention**: Keep only the latest 3 releases here to prevent file bloat. Older releases are archived in `docs/releases/`.

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

## [0.1.1] - 2026-01-08

### Documentation scaffolding and compliance

**Release Type**: Documentation Release

#### Overview

This release adds the documentation structure required by the Fulmen Helper Library Standard. No new code features.

#### Highlights

- **Documentation Scaffolding** – Added `docs/development/` directory with README, operations runbook, and ADR structure.
- **Crucible Version Section** – README now documents how to query embedded Crucible metadata via the shim API.
- **Release Documentation** – Established CHANGELOG.md, RELEASE_NOTES.md, and `docs/releases/` archive pattern.

#### Changes

- `docs/development/README.md` – Local development guide index
- `docs/development/operations.md` – Build, test, release, and tooling runbook
- `docs/development/adr/README.md` – ADR index with ecosystem adoption tracking
- `docs/development/adr/0001-template.md` – Template for rsfulmen-specific ADRs
- `README.md` – Added Crucible Version section with API example
- `CHANGELOG.md` – New file tracking all notable changes
- `RELEASE_NOTES.md` – This file
- `docs/releases/v0.1.0.md` – Archive of v0.1.0 release notes

#### Testing

- `make check-all` – All quality gates pass

---

## Archived Releases

Older release notes are archived under `docs/releases/`.
