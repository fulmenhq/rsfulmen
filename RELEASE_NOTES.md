# Release Notes

This document tracks release notes for rsfulmen releases.

> **Convention**: Keep only the latest 3 releases here to prevent file bloat. Older releases are archived in `docs/releases/`.

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

## [0.1.0] - 2026-01-08

### Initial release

**Release Type**: Initial Release

#### Overview

First release of rsfulmen, the Rust implementation of the Fulmen helper library ecosystem. Provides idiomatic Rust APIs for Crucible SSOT assets with compile-time embedding.

#### Highlights

- **Config Module** – XDG Base Directory support with cross-platform paths
- **Three-Layer Config** – Embedded defaults → user overrides → runtime injection
- **Foundry Catalogs** – Country codes, HTTP statuses, exit codes, signals, MIME types, patterns
- **Schema Validation** – Offline validation with embedded meta-schemas (Draft 2020-12)
- **Error Handling** – Canonical error envelope with severity, correlation, and exit codes
- **Telemetry Metrics** – Counters, gauges, histograms with schema-valid JSON export
- **Crucible Shim** – Embedded access to docs, schemas, and config assets
- **Docscribe** – Documentation access with frontmatter parsing
- **Similarity** – Text similarity scoring with multiple algorithms

#### Feature Flags

Supports minimal installs for lightweight consumers:

| Feature | Description |
|---------|-------------|
| `foundry-core` | Signals, exit codes, countries, HTTP statuses |
| `foundry-mime-types` | MIME type detection |
| `foundry-patterns` | Regex/glob pattern matching |
| `similarity` | Text similarity algorithms |
| `schema-validation` | JSON Schema validation |
| `error-handling` | Canonical error envelope |
| `telemetry-metrics` | Metrics export |
| `crucible` | Embedded SSOT access |
| `docscribe` | Doc access + frontmatter |

#### Requirements

- **Rust**: 1.83+ (MSRV)
- **Crucible**: v0.4.2 (embedded)

#### Testing

- `make check-all` – 156 unit tests, 37 doc tests
- Published to crates.io

---

## Archived Releases

Older release notes are archived under `docs/releases/`.
