# Release Notes

This document tracks release notes for rsfulmen releases.

> **Convention**: Keep only the latest 3 releases here to prevent file bloat. Older releases are archived in `docs/releases/`.

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
