# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-01-08

### Added

- **Documentation Scaffolding** – Added `docs/development/` directory structure per Fulmen Helper Library Standard
  - `docs/development/README.md` – Local development guide index
  - `docs/development/operations.md` – Operations runbook (build, test, release, tooling)
  - `docs/development/adr/README.md` – ADR index with ecosystem adoption tracking
  - `docs/development/adr/0001-template.md` – ADR template for rsfulmen-specific decisions
- **Crucible Version Section** – Added to README with API example and metadata field documentation
- **Release Documentation** – Added CHANGELOG.md, RELEASE_NOTES.md, and `docs/releases/` archive structure

### Changed

- **README Version** – Updated version badge to 0.1.1
- **Cargo.toml** – Updated version to 0.1.1, fixed authors email

## [0.1.0] - 2026-01-08

### Added

- **Config Module** – XDG Base Directory support and Fulmen configuration paths
  - `get_fulmen_config_dir()`, `get_app_config_dir()`, `get_xdg_base_dirs()`
  - Cross-platform support (Linux, macOS, Windows)
  - Legacy config path fallback support
- **Three-Layer Config** – Configuration loader with embedded defaults, user overrides, and runtime injection
  - Schema validation integration
  - Null-deletes-key semantics for layer precedence

- **Foundry Module** – Enterprise-grade foundation utilities from Crucible catalogs
  - **Country Codes** – ISO 3166-1 lookups with triple-index support (alpha2, alpha3, numeric)
  - **HTTP Statuses** – Status code registry with grouping helpers
  - **Exit Codes** – Standardized exit codes with categories and signal handling
  - **Signals** – Cross-platform signal catalog with platform support detection
  - **MIME Types** – Content-based detection and extension lookup
  - **Patterns** – Regex, glob, and literal pattern matching from Crucible catalogs

- **Schema Validation** – Offline validation using embedded meta-schemas
  - Draft 2020-12 default, Draft-07 supported
  - YAML schema support (comments allowed)
  - Resolver for Crucible module schemas

- **Error Handling** – Canonical error envelope extending Pathfinder schema
  - Severity levels, correlation IDs, exit codes
  - Schema-valid JSON serialization
  - `did_you_mean` helper integration with similarity module

- **Telemetry Metrics** – Taxonomy-backed counters, gauges, and histograms
  - Schema-valid JSON event export
  - Thread-safe metric recording

- **Crucible Shim** – Embedded SSOT access
  - `metadata()` and `version()` APIs
  - Asset listing and retrieval (docs, schemas, config)
  - Generated asset index at build time

- **Docscribe** – Documentation access with frontmatter parsing
  - Embedded doc retrieval via Crucible shim
  - YAML frontmatter extraction

- **Similarity** – Text similarity scoring and suggestions
  - Multiple algorithms: Levenshtein, Damerau-Levenshtein, Jaro-Winkler
  - Normalized scoring (0.0 to 1.0)
  - `did_you_mean` formatted suggestions

- **Module Registry** – Runtime module introspection
  - Feature flag detection
  - Module metadata from Crucible taxonomy

### Infrastructure

- **MSRV 1.83** – Minimum supported Rust version
- **Feature Flags** – Minimal installs for lightweight consumers
  - `foundry-core`, `foundry-mime-types`, `foundry-patterns`
  - `similarity`, `schema-validation`, `error-handling`, `telemetry-metrics`
  - `crucible`, `docscribe`
- **Crucible v0.4.2** – Embedded SSOT assets
- **Goneat Integration** – `make sync` for SSOT synchronization
- **Quality Gates** – `make check-all` (fmt, lint, test)
- **CI Ready** – GitHub Actions compatible (MSRV 1.83)
