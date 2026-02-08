# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2026-02-08

### Added

- **App Identity** (`appidentity`) – Discovery from `.fulmen/app.yaml` with upward directory search
  - `load()`, `load_from()`, `load_file()`, `validate()`
  - `FULMEN_APP_IDENTITY_FILE` environment variable override
- **Structured Logging** (`logging`) – Leveled logging with SIMPLE and STRUCTURED profiles
  - `Logger` with `Severity` levels (Trace through Fatal)
  - Component context and structured key-value fields
  - Configurable output format via `Config` and `Profile`
- **Canonical Hashing** (`fulhash`) – Content-addressable digests in `algo:hex` format
  - `hash()`, `hash_string()`, `hash_reader()`, `hash_file()`
  - `verify()`, `verify_file()`, `format_digest()`, `parse_digest()`
  - xxHash3-128 (fast, default) and SHA-256 (cryptographic)
  - Strict canonical validation (lowercase hex, correct byte length)
- **Filesystem Discovery** (`pathfinder`) – Safe file search with glob patterns
  - `find_files()` with Crucible schema-aligned query/result types
  - `find_repository_root()`, `find_config_files()`, `validate_path()`
  - Symlink boundary enforcement, hidden file pruning, warning collection
  - SHA-256 checksums in `sha256:<hex>` canonical format
- **Terminal Utilities** (`ascii`) – Unicode-aware string handling
  - `string_width()` with grapheme-cluster-aware width calculation
  - `analyze()`, `truncate_to_width()`, `pad_to_width()`, `draw_box()`

### Changed

- **MSRV** – Bumped from 1.83 to 1.88
- **Cargo.toml** – Added feature flags and dependencies for all new modules
- **CI** – Added new features to matrix, updated MSRV job to 1.88

### Infrastructure

- **Test Coverage** – 286 unit tests, 70 doc tests
- **Feature Flags** – `appidentity`, `logging`, `fulhash`, `pathfinder`, `ascii` added to default and full sets

## [0.1.2] - 2026-01-08

### Added

- **Signal Resolution Helpers** – Ergonomic signal name lookup for CLI/API use (`foundry::signals`)
  - `resolve_signal(name)` – Resolves common name variants (SIGTERM/TERM/term/15/-15)
  - `list_signal_names()` – Returns all signal names for CLI completion
  - `match_signal_names(pattern)` – Glob matching with `*` and `?` wildcards
- **Resolution Algorithm** – 7-step normalization supporting exact match, numeric, SIG-prefix, and ID fallback
- **Kill-Style Negatives** – `resolve_signal("-15")` works like `kill -15`

### Changed

- **Crucible** – Updated to v0.4.4 (adds signal resolution interface spec and test fixtures)
- **Cargo.toml** – Updated version to 0.1.2

### Infrastructure

- **Test Coverage** – 175 unit tests, 41 doc tests
- **Crucible Fixtures** – Implementation validated against 39 cross-language test vectors

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
