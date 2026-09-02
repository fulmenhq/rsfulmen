# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-09-02

Stamp-only host identity plus goneat v0.6.0. **Breaking:** `resolve()` no
longer reads process `FULMEN_HOST_*`. MSRV remains 1.88. Crucible embed is
v0.4.19.

### Added

- **Host-identity producer** (`host-identity-producer` feature) — application
  `build.rs` can stamp all six `FULMEN_HOST_*` values from `CARGO_MANIFEST_DIR`.
  Git locator/index/object/config overrides are stripped (`GIT_DIR`,
  `GIT_WORK_TREE`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`, `GIT_CONFIG*`, and the
  rest of the strip set).
- **`resolve_from_process_env()`** — explicit runtime-environment resolver.
  Use this when a caller wants process `FULMEN_HOST_*`.

### Changed

- **`host_identity!()` and `resolve_from_stamps`** are compile-stamp-only.
  Missing stamps use documented defaults (`dev` / `unknown` / omitted dirty).
  They never read process `FULMEN_HOST_*` and do not derive platform from the
  running host.
- **`resolve()`** is stamp-only defaults (same as empty stamps). Callers that
  relied on process-env fallback must switch to `resolve_from_process_env()`.
- **goneat** pin `v0.5.16` → `v0.6.0`.
- Selected crate refreshes: flate2 1.1.10, uuid 1.26.0. jsonschema 0.17,
  serde_yaml_ng 0.10, zip 2.x, and direct thiserror 1.x unchanged.

### Requirements

- **Rust**: 1.88+ (MSRV, unchanged)
- **Crucible**: v0.4.19 (embedded)

## [0.1.6] - 2026-08-20

File-backed schema catalogs, fulpack create honesty, committed lockfile, and
host `version --extended`. Additive; MSRV remains 1.88. Crucible embed is
v0.4.19.

### Added

- **File-backed JSON Schema instance validation** — check payloads against
  on-disk schema trees with offline `$ref`. Existing embed-id APIs are
  unchanged. Catalog roots reject traversal, symlink components, non-local
  `file:` hosts, unsupported URI schemes, and duplicate `$id`s at compile
  time. Unix opens each path component with `openat(2)` and `O_NOFOLLOW`.
- **Host binary identity** (`rsfulmen::buildinfo`) — resolve and format
  `version` / `version --extended` from app-injected `FULMEN_HOST_*` stamps.
  Pins (rsfulmen + Crucible) are a separate block and are never the host
  commit. Invalid dirty stamps stay unknown (never false-clean).
- **Crucible v0.4.19** embed (README badge, ecosystem-guide, schema-validation
  contract). Unix `libc` for catalog containment.

### Changed

- **Fulpack create** — requesting sha512/sha1/md5 returns `INVALID_OPTIONS`
  instead of substituting sha256. Gzip create always uses compression level 6;
  `compression_level` is ignored for the gzip format (tar.gz/zip still honor it).
- **goneat** pin `v0.5.13` → `v0.5.16`.
- **`Cargo.lock` committed**; CI Test, Feature Matrix, and MSRV use `--locked`.
- **Clippy toolchain** pinned to **1.98.0** (`rust-toolchain.toml` + CI) so
  local hooks and GitHub deny the same lints. MSRV job remains 1.88.

### Fixed

- UTF-16 decode uses `as_chunks::<2>()` (clippy `chunks_exact_to_as_chunks`).
- Generated asset-index loops in `build.rs` iterate map keys (clippy
  `for_kv_map`).

### Requirements

- **Rust**: 1.88+ (MSRV, unchanged)
- **Crucible**: v0.4.19 (embedded)

## [0.1.5] - 2026-06-05

Process and maintenance release: adopts a branch/PR development workflow, syncs
Crucible v0.4.13, and clears dependency and tooling tech debt. No new modules,
no public API changes, no breaking changes.

### Changed

- **Development workflow** — moved from direct-push (micro-team) to a branch/PR
  model. Branch protection to be enabled after this release.
- **Git hooks** — removed the guardian browser-intercept on commit and push
  (obsolete under branch-based review); hooks regenerated guardian-free via
  `goneat hooks generate`.
- **goneat** — pin bumped `v0.5.1` → `v0.5.13`.
- **Crucible SSOT** — synced `v0.4.12` → `v0.4.13`: `devlead`/`devrev`/`qa`
  role-prompt enrichment (contract-conformance checklists, cross-role notes;
  role `v1.0.0` → `v1.0.1`), upstream schema slimming, and mechanical YAML
  normalization.
- **Dependencies** — migrated off the deprecated/archived `serde_yaml` to the
  maintained, API-compatible `serde_yaml_ng` (aliased; no source changes).

### Added

- **`.goneatignore`** — excludes the deliberately-malformed negative-test
  fixtures synced from Crucible from goneat assess (standard galaxy protocol).

### Fixed

- **Flaky `appidentity` test isolation** — `TestDir` temp dirs could collide
  under parallel test threads (shared `{pid}-{nanos}`), causing one test to read
  another's `.fulmen/app.yaml`. Added an `AtomicU64` sequence counter to the
  temp-dir name (mirrors the v0.1.4 pathfinder fix).

### Requirements

- **Rust**: 1.88+ (MSRV, unchanged)
- **Crucible**: v0.4.13 (embedded)

## [0.1.4] - 2026-02-21

### Added

- **Crucible v0.4.12 Sync** — Major SSOT sync jump from v0.4.4, adding 6 new agentic roles
  (cxotech, deliverylead, infraeng, qa, releng, uxdev), updated role-prompt schema with new
  fields (domains, pre_push_checklist, required_reading, cross_role_note), fulencode schemas
  and fixtures, design token schemas, and expanded upstream standards
- **Typed Role Catalog** (`crucible::roles`) — Full-fidelity `RolePrompt` deserialization from
  embedded role YAMLs with `load_role()`, `list_role_slugs()`, `load_role_catalog()`
  - 22 fields covering entire `role-prompt.schema.json` specification
  - Forward-compatible enums (`RoleCategory`, `ExampleType`) with `#[serde(other)]`
  - Invariant-based tests (no brittle role counts)
- **Fulencode** (`fulencode`) — Encoding, decoding, detection, normalization, and BOM handling
  - `encode()` / `decode()` for Base64, Base64URL, Hex, UTF-8, UTF-16LE/BE
  - `detect()` with BOM, UTF-8 validation, and NULL-byte heuristics
  - `normalize()` with NFC/NFD/NFKC/NFKD + text-safe profile (zero-width/bidi rejection)
  - `detect_bom()` / `remove_bom()` / `add_bom()` for BOM management
  - Cross-language fixture-driven tests from Crucible SSOT
- **Runtime Signal Handling** (`signals`) — Upgraded from catalog re-export to full runtime manager
  - `SignalManager` with thread-safe handler dispatch
  - Shutdown chains (LIFO) and reload chains (FIFO)
  - SIGINT double-tap force-quit with catalog-driven defaults
  - Cross-platform: `signal-hook` on Unix, `ctrlc` on Windows
  - `SignalInjector` test helper for deterministic signal delivery
- **Correlation IDs** (`foundry::correlation`) — UUIDv7 generation, parsing, and validation
  - `generate()`, `parse()`, `is_valid()` module-level functions
  - `CorrelationId` newtype with serde, `Display`, `FromStr` (strictly v7, rejects v4)
  - Uppercase input normalized to lowercase canonical form
- **Config Env Overrides** (`config::env`) — Environment variable to config key mapping
  - `load_env_overrides()` / `load_env_overrides_with_report()` with diagnostics
  - Type-safe parsing (String, Int, Float, Bool with flexible true/false variants)
  - Alias support with conflict detection and sensitive value masking
  - Output feeds directly into three-layer config as `runtime_overrides`

### Fixed

- **Pathfinder** — Eliminated temp-dir race condition in parallel tests by adding
  `AtomicU64` sequence counter to `TestDir::new()`
- **CI** — Fixed yamllint warning (missing space before inline comment in ci.yml)

### Changed

- **Crucible** — Updated from v0.4.4 to v0.4.12
- **Feature flags** — Added `fulencode`, `foundry-correlation`; `foundry` convenience
  now includes `foundry-correlation`

### Infrastructure

- **Test Coverage** — 362 unit tests, 74 doc tests
- **Feature Flags** — `fulencode`, `foundry-correlation` added to default set
- **Dependencies** — Added `base64`, `hex`, `unicode-normalization`, `uuid`,
  `signal-hook` (Unix), `ctrlc` (Windows)

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
